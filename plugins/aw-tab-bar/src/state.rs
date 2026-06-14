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
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenameState {
    pub old_name: String,
    pub input: String,
}

/// One tab to draw on the bar. Styling (theme colors, active highlight) is
/// applied by the caller; `label` is the literal visible text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderTab {
    pub label: String,
    pub active: bool,
    pub renaming: bool,
}

#[derive(Debug, Default)]
pub struct TabBarState {
    tabs: Vec<TabItem>,
    spans: Vec<TabSpan>,
    pending_click: Option<PendingClick>,
    // Tab under the most recent press; survives motion/timer so a drag (or a
    // press-then-release on a different tab) reorders the originally pressed tab.
    drag_origin: Option<PendingClick>,
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

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Lay out the tabs that fit within `cols`, recording click spans. Each
    /// label is the literal visible text; the caller styles the active tab.
    pub fn render(&mut self, cols: usize) -> Vec<RenderTab> {
        self.spans.clear();
        let mut out = Vec::new();
        let mut vis = 0usize;
        for (index, tab) in self.tabs.iter().enumerate() {
            let label = self.tab_visible_label(tab);
            let width = label.chars().count();
            if vis + width > cols {
                break;
            }
            let start = vis;
            vis += width;
            let renaming = self
                .rename
                .as_ref()
                .is_some_and(|rename| rename.old_name == tab.name);
            self.spans.push(TabSpan {
                tab_id: tab.id,
                name: tab.name.clone(),
                index,
                start,
                end: vis,
            });
            out.push(RenderTab {
                label,
                active: tab.active,
                renaming,
            });
        }
        out
    }

    pub fn click(&mut self, col: usize) {
        let Some(span) = self.span_at(col).cloned() else {
            self.drag_origin = None;
            return;
        };
        self.drag_origin = Some(PendingClick {
            tab_id: span.tab_id,
            index: span.index,
            name: span.name.clone(),
        });
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
            name: span.name,
        });
    }

    pub fn click_timeout(&mut self) -> Option<TabBarCommand> {
        let pending = self.pending_click.take()?;
        Some(TabBarCommand::Focus {
            index: pending.index,
        })
    }

    pub fn hold(&mut self, _col: usize) {
        // A drag is underway. Cancel the pending click so the release reorders
        // instead of focusing/renaming. zellij sends a Hold per motion step;
        // the tab being moved is the one captured on press (drag_origin), so we
        // deliberately ignore the column here and don't track the hover tab.
        self.pending_click = None;
    }

    pub fn release(&mut self, col: usize) -> Option<TabBarCommand> {
        // Move the tab that was pressed (drag_origin) to the tab under the
        // release column. Works whether the drag arrived as Hold/Release motion
        // events or as a plain press-then-release on another tab.
        let origin = self.drag_origin.take()?;
        self.span_at(col)?; // ignore releases that aren't over a tab
        let target = self.target_index_for_col(col);
        if target == origin.index {
            return None; // released on its own slot: a click, not a move
        }
        self.pending_click = None; // a drag, not a (double-)click
        Some(TabBarCommand::Move {
            name: origin.name,
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

    fn tab_visible_label(&self, tab: &TabItem) -> String {
        if let Some(rename) = &self.rename {
            if rename.old_name == tab.name {
                return format!(" {}| ", rename.input);
            }
        }

        let sync = if tab.sync_panes { "~" } else { "" };
        let bell = if tab.has_bell { "!" } else { "" };
        format!(" {}{}{} ", tab.name, sync, bell)
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

        assert!(state.render(80).is_empty());
        assert_eq!(state.status(), Some("aw-tab-bar: loading tabs"));
        assert!(state.spans().is_empty());
    }

    #[test]
    fn render_tracks_tab_spans() {
        let mut state = TabBarState::default();
        state.replace_tabs(vec![item(2, 1, "server"), item(1, 0, "app")]);

        let tabs = state.render(80);
        assert_eq!(
            tabs.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(),
            vec![" app ", " server "]
        );
        assert_eq!(
            state.spans(),
            &[
                TabSpan {
                    tab_id: 1,
                    name: "app".to_string(),
                    index: 0,
                    start: 0,
                    end: 5,
                },
                TabSpan {
                    tab_id: 2,
                    name: "server".to_string(),
                    index: 1,
                    start: 5,
                    end: 13,
                },
            ]
        );
    }

    #[test]
    fn click_timeout_focuses_single_clicked_tab() {
        let mut state = TabBarState::default();
        state.replace_tabs(vec![item(1, 0, "app"), item(2, 1, "server")]);
        state.render(80);

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
        state.render(80);

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
    fn drag_moves_pressed_tab_not_hover_tab() {
        // zellij delivers a Hold per motion step; the move must target the
        // pressed tab (drag_origin), not whichever tab the cursor ended over.
        let mut state = TabBarState::default();
        state.replace_tabs(vec![
            item(1, 0, "app"),
            item(2, 1, "server"),
            item(3, 2, "git"),
        ]);
        state.render(80);

        state.click(2); // press on "app"
        state.hold(8); // motion over "server"
        state.hold(15); // motion over "git"

        assert_eq!(
            state.release(15), // release over "git"
            Some(TabBarCommand::Move {
                name: "app".to_string(),
                index: 2,
            })
        );
    }

    #[test]
    fn press_then_release_on_other_tab_reorders() {
        let mut state = TabBarState::default();
        state.replace_tabs(vec![
            item(1, 0, "app"),
            item(2, 1, "server"),
            item(3, 2, "git"),
        ]);
        state.render(80);

        state.click(2); // press on "app"; no Hold/motion events delivered
        assert_eq!(
            state.release(15), // release over "git" (span [13,18))
            Some(TabBarCommand::Move {
                name: "app".to_string(),
                index: 2,
            })
        );
    }

    #[test]
    fn press_and_release_same_tab_is_not_a_reorder() {
        let mut state = TabBarState::default();
        state.replace_tabs(vec![item(1, 0, "app"), item(2, 1, "server")]);
        state.render(80);

        state.click(2); // press on "app"
        assert_eq!(state.release(3), None); // release still on "app"
        assert_eq!(
            state.click_timeout(),
            Some(TabBarCommand::Focus { index: 0 })
        );
    }

    #[test]
    fn rename_accepts_safe_names_only() {
        let mut state = TabBarState::default();
        state.replace_tabs(vec![item(1, 0, "app")]);
        state.render(80);
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
