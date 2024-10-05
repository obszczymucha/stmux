use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub(crate) type SessionName = String;
pub(crate) type SessionNames = Vec<SessionName>;
pub(crate) type WindowName = String;
pub(crate) type TmuxWindows = Vec<TmuxWindow>;
pub(crate) type TmuxSessions = HashMap<SessionName, TmuxWindows>;

pub(crate) struct Layout {
    pub(crate) session_name: SessionName,
    pub(crate) window_name: WindowName,
    pub(crate) layout: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TmuxWindow {
    pub(crate) index: usize,
    pub(crate) name: WindowName,
    pub(crate) layout: String,
    pub(crate) panes: Vec<TmuxPane>,
}

impl TmuxWindow {
    pub(crate) fn startup_command_for_pane(&self, index: usize) -> Option<String> {
        if self.panes.is_empty() {
            None
        } else {
            self.panes[index - 1].startup_command.clone()
        }
    }

    pub(crate) fn shell_command_for_pane(&self, index: usize) -> Option<String> {
        if self.panes.is_empty() {
            None
        } else {
            self.panes[index - 1].shell_command.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TmuxPane {
    pub(crate) index: usize,
    pub(crate) path: String,
    pub(crate) active: bool,
    pub(crate) startup_command: Option<String>,
    pub(crate) shell_command: Option<String>,
}
