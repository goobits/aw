mod state;

use std::collections::BTreeMap;

use state::{KeyInput, TabBarCommand, TabBarState, TabItem};
use zellij_tile::prelude::*;

const STATUS_WAITING_FOR_PERMISSIONS: &str = "aw-tab-bar: waiting for permissions";
const STATUS_LOADING_TABS: &str = "aw-tab-bar: loading tabs";
const STATUS_PERMISSION_DENIED: &str = "aw-tab-bar: permissions denied";
const STATUS_NO_TABS: &str = "aw-tab-bar: no tabs";

#[derive(Debug)]
struct Config {
    workspace: Option<String>,
    aw_bin: String,
    double_click_seconds: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workspace: None,
            aw_bin: "aw".to_string(),
            double_click_seconds: 0.35,
        }
    }
}

#[derive(Default)]
struct PluginState {
    config: Config,
    tabs: TabBarState,
    intercepting: bool,
}

register_plugin!(PluginState);

impl ZellijPlugin for PluginState {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.config = Config::from(configuration);
        self.tabs.set_status(STATUS_WAITING_FOR_PERMISSIONS);
        // Stay non-selectable so the bar never takes keyboard focus from the
        // shell panes. Mouse events still route here by position; key input is
        // grabbed only while a rename is active (see sync_key_intercept).
        set_selectable(false);
        subscribe(&[EventType::PermissionRequestResult]);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
            // Needed for intercept_key_presses() during in-bar tab rename.
            PermissionType::InterceptInput,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.tabs.set_status(STATUS_LOADING_TABS);
                subscribe(&[
                    EventType::TabUpdate,
                    EventType::Mouse,
                    EventType::Key,
                    // Keys grabbed during a rename arrive as InterceptedKeyPress, not Key.
                    EventType::InterceptedKeyPress,
                    EventType::Timer,
                    EventType::RunCommandResult,
                ]);
                true
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                self.tabs.set_status(STATUS_PERMISSION_DENIED);
                true
            }
            Event::TabUpdate(tabs) => {
                let tabs: Vec<_> = tabs.into_iter().map(TabItem::from).collect();
                if tabs.is_empty() {
                    self.tabs.set_status(STATUS_NO_TABS);
                } else {
                    self.tabs.clear_status();
                }
                self.tabs.replace_tabs(tabs);
                true
            }
            Event::Mouse(Mouse::LeftClick(_, col)) => {
                self.tabs.clear_status();
                self.tabs.click(col);
                if self.tabs.rename().is_none() {
                    set_timeout(self.config.double_click_seconds);
                }
                self.sync_key_intercept();
                true
            }
            Event::Mouse(Mouse::Hold(_, col)) => {
                self.tabs.hold(col);
                true
            }
            Event::Mouse(Mouse::Release(_, col)) => {
                if let Some(command) = self.tabs.release(col) {
                    self.run_tab_command(command);
                }
                true
            }
            Event::Timer(_) => {
                if let Some(command) = self.tabs.click_timeout() {
                    self.run_tab_command(command);
                    return true;
                }
                false
            }
            Event::Key(key) | Event::InterceptedKeyPress(key) => {
                if let Some(command) = self.tabs.key(KeyInput::from(key)) {
                    self.run_tab_command(command);
                }
                self.sync_key_intercept();
                true
            }
            Event::RunCommandResult(exit_code, _, stderr, context) => {
                if context
                    .get("source")
                    .is_some_and(|source| source == "aw-tab-bar")
                {
                    if exit_code != Some(0) {
                        let error = String::from_utf8_lossy(&stderr);
                        self.tabs.set_status(format!("aw failed: {}", error.trim()));
                    }
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        // Render tabs as zellij ribbons (built-in tab-bar look): theme ribbon
        // colors + powerline separators, active tab via .selected(), a renaming
        // tab tinted. Each ribbon renders as `<sep> label <sep>`, so its width is
        // label + state::RIBBON_DECORATION_WIDTH; click spans account for that.
        let tabs = self.tabs.render(cols);
        let mut line = String::new();
        for tab in &tabs {
            let mut text = Text::new(&tab.label);
            if tab.active {
                text = text.selected();
            }
            if tab.renaming {
                text = text.color_range(2, ..);
            }
            line.push_str(&serialize_ribbon(&text));
        }
        if let Some(status) = self.tabs.status() {
            line.push_str(&serialize_text(&Text::new(status)));
        }
        print!("{}", line);
    }
}

impl PluginState {
    /// Grab key presses only while a rename is in progress, then release them
    /// back to the focused pane. This lets the non-selectable bar capture typed
    /// tab names without ever stealing focus from the shell.
    fn sync_key_intercept(&mut self) {
        let renaming = self.tabs.rename().is_some();
        if renaming && !self.intercepting {
            intercept_key_presses();
            self.intercepting = true;
        } else if !renaming && self.intercepting {
            clear_key_presses_intercepts();
            self.intercepting = false;
        }
    }

    fn run_tab_command(&mut self, command: TabBarCommand) {
        match command {
            TabBarCommand::Focus { index } => switch_tab_to((index + 1) as u32),
            TabBarCommand::Move { name, index } => {
                let Some(workspace) = self.config.workspace.clone() else {
                    self.tabs.set_status("missing workspace");
                    return;
                };
                self.run_aw(vec![
                    workspace,
                    "tab".to_string(),
                    "move".to_string(),
                    format!("{}@{}", name, index),
                ]);
            }
            TabBarCommand::Rename { old_name, new_name } => {
                let Some(workspace) = self.config.workspace.clone() else {
                    self.tabs.set_status("missing workspace");
                    return;
                };
                self.run_aw(vec![
                    workspace,
                    "tab".to_string(),
                    "rename".to_string(),
                    old_name,
                    new_name,
                ]);
            }
        }
    }

    fn run_aw(&mut self, args: Vec<String>) {
        let mut command = Vec::with_capacity(args.len() + 1);
        command.push(self.config.aw_bin.clone());
        command.extend(args);
        let command_refs: Vec<&str> = command.iter().map(String::as_str).collect();
        let mut context = BTreeMap::new();
        context.insert("source".to_string(), "aw-tab-bar".to_string());
        run_command(&command_refs, context);
    }
}

impl From<BTreeMap<String, String>> for Config {
    fn from(configuration: BTreeMap<String, String>) -> Self {
        let mut config = Config::default();
        if let Some(workspace) = configuration.get("workspace") {
            if !workspace.is_empty() {
                config.workspace = Some(workspace.clone());
            }
        }
        if let Some(aw_bin) = configuration.get("aw") {
            if !aw_bin.is_empty() {
                config.aw_bin = aw_bin.clone();
            }
        }
        if let Some(raw) = configuration.get("double_click_ms") {
            if let Ok(ms) = raw.parse::<u64>() {
                config.double_click_seconds = (ms as f64 / 1000.0).clamp(0.1, 2.0);
            }
        }
        config
    }
}

impl From<TabInfo> for TabItem {
    fn from(tab: TabInfo) -> Self {
        Self {
            id: tab.tab_id,
            position: tab.position,
            name: tab.name,
            active: tab.active,
            has_bell: tab.has_bell_notification || tab.is_flashing_bell,
            sync_panes: tab.is_sync_panes_active,
        }
    }
}

impl From<KeyWithModifier> for KeyInput {
    fn from(key: KeyWithModifier) -> Self {
        if !key.has_no_modifiers() {
            return KeyInput::Other;
        }
        match key.bare_key {
            BareKey::Char(ch) => KeyInput::Char(ch),
            BareKey::Backspace => KeyInput::Backspace,
            BareKey::Enter => KeyInput::Enter,
            BareKey::Esc => KeyInput::Esc,
            _ => KeyInput::Other,
        }
    }
}
