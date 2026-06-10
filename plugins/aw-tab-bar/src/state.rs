#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabItem {
    pub id: usize,
    pub position: usize,
    pub name: String,
    pub active: bool,
    pub has_bell: bool,
    pub sync_panes: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabSpan {
    pub tab_id: usize,
    pub name: String,
    pub index: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TabBarCommand {
    Focus { index: usize },
    Move { name: String, index: usize },
    Rename { old_name: String, new_name: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeyInput {
    Char(char),
    Backspace,
    Enter,
    Esc,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingClick {
    tab_id: usize,
    index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DragState {
    name: String,
    index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenameState {
    pub old_name: String,
    pub input: String,
}

#[derive(Debug, Default)]
pub struct TabBarState {
    tabs: Vec<TabItem>,
    spans: Vec<TabSpan>,
    pending_click: Option<PendingClick>,
    drag: Option<DragState>,
    rename: Option<RenameState>,
    status: Option<String>,
}

impl TabBarState {
    pub fn replace_tabs(&mut self, mut tabs: Vec<TabItem>) {
        tabs.sort_by_key(|tab| tab.position);
        self.tabs = tabs;
    }

    #[cfg(test)]
    pub fn spans(&self) -> &[TabSpan] {
        &self.spans
    }

    pub fn rename(&self) -> Option<&RenameState> {
        self.rename.as_ref()
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = Some(status.into());
    }

    pub fn clear_status(&mut self) {
        self.status = None;
    }

    pub fn render_line(&mut self, cols: usize) -> String {
        self.spans.clear();
        if cols == 0 {
            return String::new();
        }

        let mut line = String::new();
        for (index, tab) in self.tabs.iter().enumerate() {
            if !line.is_empty() {
                push_fitting(&mut line, " ", cols);
            }
            let start = line.chars().count();
            let label = self.tab_label(tab);
            push_fitting(&mut line, &label, cols);
            let end = line.chars().count();
            if end > start {
                self.spans.push(TabSpan {
                    tab_id: tab.id,
                    name: tab.name.clone(),
                    index,
                    start,
                    end,
                });
            }
            if line.chars().count() >= cols {
                break;
            }
        }

        if let Some(status) = &self.status {
            if !line.is_empty() {
                push_fitting(&mut line, " ", cols);
            }
            push_fitting(&mut line, status, cols);
        }
        line
    }

    pub fn click(&mut self, col: usize) {
        let Some(span) = self.span_at(col).cloned() else {
            return;
        };
        if self
            .pending_click
            .as_ref()
            .is_some_and(|pending| pending.tab_id == span.tab_id)
        {
            self.pending_click = None;
            self.begin_rename(span.index);
            return;
        }
        self.pending_click = Some(PendingClick {
            tab_id: span.tab_id,
            index: span.index,
        });
    }

    pub fn click_timeout(&mut self) -> Option<TabBarCommand> {
        let pending = self.pending_click.take()?;
        Some(TabBarCommand::Focus {
            index: pending.index,
        })
    }

    pub fn hold(&mut self, col: usize) {
        if let Some(span) = self.span_at(col).cloned() {
            self.drag = Some(DragState {
                name: span.name,
                index: span.index,
            });
            self.pending_click = None;
        }
    }

    pub fn release(&mut self, col: usize) -> Option<TabBarCommand> {
        let drag = self.drag.take()?;
        let target = self.target_index_for_col(col);
        if target == drag.index {
            return None;
        }
        Some(TabBarCommand::Move {
            name: drag.name,
            index: target,
        })
    }

    pub fn key(&mut self, key: KeyInput) -> Option<TabBarCommand> {
        let rename = self.rename.as_mut()?;
        match key {
            KeyInput::Char(ch) if is_valid_tab_name_char(ch) => {
                rename.input.push(ch);
                None
            }
            KeyInput::Backspace => {
                rename.input.pop();
                None
            }
            KeyInput::Esc => {
                self.rename = None;
                None
            }
            KeyInput::Enter => {
                let rename = self.rename.take()?;
                if rename.input.is_empty()
                    || rename.input == rename.old_name
                    || !is_valid_tab_name(&rename.input)
                {
                    return None;
                }
                Some(TabBarCommand::Rename {
                    old_name: rename.old_name,
                    new_name: rename.input,
                })
            }
            _ => None,
        }
    }

    fn tab_label(&self, tab: &TabItem) -> String {
        if let Some(rename) = &self.rename {
            if rename.old_name == tab.name {
                return format!("[{}|]", rename.input);
            }
        }

        let marker = if tab.active { "*" } else { " " };
        let sync = if tab.sync_panes { "~" } else { "" };
        let bell = if tab.has_bell { "!" } else { "" };
        format!("[{}{}{}{}]", marker, tab.name, sync, bell)
    }

    fn begin_rename(&mut self, index: usize) {
        if let Some(tab) = self.tabs.get(index) {
            self.rename = Some(RenameState {
                old_name: tab.name.clone(),
                input: tab.name.clone(),
            });
        }
    }

    fn span_at(&self, col: usize) -> Option<&TabSpan> {
        self.spans
            .iter()
            .find(|span| col >= span.start && col < span.end)
    }

    fn target_index_for_col(&self, col: usize) -> usize {
        if self.spans.is_empty() {
            return 0;
        }
        for span in &self.spans {
            let midpoint = span.start + (span.end.saturating_sub(span.start) / 2);
            if col < midpoint {
                return span.index;
            }
        }
        self.spans.len().saturating_sub(1)
    }
}

fn push_fitting(line: &mut String, value: &str, cols: usize) {
    for ch in value.chars() {
        if line.chars().count() >= cols {
            break;
        }
        line.push(ch);
    }
}

pub fn is_valid_tab_name(value: &str) -> bool {
    !value.is_empty() && value.chars().all(is_valid_tab_name_char)
}

fn is_valid_tab_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: usize, position: usize, name: &str) -> TabItem {
        TabItem {
            id,
            position,
            name: name.to_string(),
            active: false,
            has_bell: false,
            sync_panes: false,
        }
    }

    #[test]
    fn status_renders_without_tabs() {
        let mut state = TabBarState::default();
        state.set_status("aw-tab-bar: loading tabs");

        assert_eq!(state.render_line(80), "aw-tab-bar: loading tabs");
        assert!(state.spans().is_empty());
    }

    #[test]
    fn status_truncates_to_available_columns() {
        let mut state = TabBarState::default();
        state.set_status("aw-tab-bar: loading tabs");

        assert_eq!(state.render_line(10), "aw-tab-bar");
    }

    #[test]
    fn render_tracks_tab_spans() {
        let mut state = TabBarState::default();
        state.replace_tabs(vec![item(2, 1, "server"), item(1, 0, "app")]);

        assert_eq!(state.render_line(80), "[ app] [ server]");
        assert_eq!(
            state.spans(),
            &[
                TabSpan {
                    tab_id: 1,
                    name: "app".to_string(),
                    index: 0,
                    start: 0,
                    end: 6,
                },
                TabSpan {
                    tab_id: 2,
                    name: "server".to_string(),
                    index: 1,
                    start: 7,
                    end: 16,
                },
            ]
        );
    }

    #[test]
    fn click_timeout_focuses_single_clicked_tab() {
        let mut state = TabBarState::default();
        state.replace_tabs(vec![item(1, 0, "app"), item(2, 1, "server")]);
        state.render_line(80);

        state.click(9);
        assert_eq!(
            state.click_timeout(),
            Some(TabBarCommand::Focus { index: 1 })
        );
    }

    #[test]
    fn second_click_on_same_tab_starts_rename() {
        let mut state = TabBarState::default();
        state.replace_tabs(vec![item(1, 0, "app")]);
        state.render_line(80);

        state.click(2);
        state.click(3);

        assert_eq!(
            state.rename(),
            Some(&RenameState {
                old_name: "app".to_string(),
                input: "app".to_string(),
            })
        );
        assert_eq!(state.click_timeout(), None);
    }

    #[test]
    fn drag_release_requests_aw_move_target() {
        let mut state = TabBarState::default();
        state.replace_tabs(vec![
            item(1, 0, "app"),
            item(2, 1, "server"),
            item(3, 2, "git"),
        ]);
        state.render_line(80);

        state.hold(2);

        assert_eq!(
            state.release(20),
            Some(TabBarCommand::Move {
                name: "app".to_string(),
                index: 2,
            })
        );
    }

    #[test]
    fn rename_accepts_safe_names_only() {
        let mut state = TabBarState::default();
        state.replace_tabs(vec![item(1, 0, "app")]);
        state.render_line(80);
        state.click(2);
        state.click(2);
        state.key(KeyInput::Backspace);
        state.key(KeyInput::Backspace);
        state.key(KeyInput::Backspace);
        state.key(KeyInput::Char('a'));
        state.key(KeyInput::Char('p'));
        state.key(KeyInput::Char('i'));
        state.key(KeyInput::Char('/'));

        assert_eq!(
            state.key(KeyInput::Enter),
            Some(TabBarCommand::Rename {
                old_name: "app".to_string(),
                new_name: "api".to_string(),
            })
        );
    }
}
