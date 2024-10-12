use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TmuxSession {
    pub(crate) background: Option<bool>,         // tmux -d
    pub(crate) no_recent_tracking: Option<bool>, // Won't be included in stmux recent next/previous.
    pub(crate) windows: Vec<TmuxWindow>,
}

pub(crate) type SessionName = String;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TmuxWindow {
    pub(crate) index: usize,
    pub(crate) name: WindowName,
    pub(crate) layout: String,
    pub(crate) panes: Vec<TmuxPane>,
    pub(crate) active: Option<bool>
}

pub(crate) type WindowName = String;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TmuxPane {
    pub(crate) index: usize,
    pub(crate) path: String,
    pub(crate) active: bool,
    pub(crate) startup_command: Option<String>,
    pub(crate) shell_command: Option<String>,
    pub(crate) environment: Option<Vec<EnvironmentVariable>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct EnvironmentVariable {
    pub(crate) name: String,
    pub(crate) value: String,
}

pub(crate) type TmuxSessions = HashMap<SessionName, TmuxSession>;
pub(crate) type TmuxWindows = Vec<TmuxWindow>;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct WindowDimension {
    pub(crate) width: usize,
    pub(crate) height: usize,
}

pub(crate) struct Layout {
    pub(crate) session_name: SessionName,
    pub(crate) window_name: WindowName,
    pub(crate) layout: String,
}

pub(crate) struct WindowNameAndStatus {
    pub(crate) name: WindowName,
    pub(crate) active: bool,
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
