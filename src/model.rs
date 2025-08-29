use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct NameValue {
    pub(crate) name: String,
    pub(crate) value: String,
}

pub(crate) type TmuxOption = NameValue;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct TmuxSession {
    pub(crate) background: Option<bool>,         // tmux -d
    pub(crate) no_recent_tracking: Option<bool>, // Won't be included in stmux recent next/previous.
    pub(crate) windows: Vec<TmuxWindow>,
    #[serde(default)]
    pub(crate) options: Vec<TmuxOption>,
}

pub(crate) type SessionName = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct TmuxWindow {
    pub(crate) index: usize,
    pub(crate) name: WindowName,
    pub(crate) layout: String,
    pub(crate) panes: Vec<TmuxPane>,
    pub(crate) active: Option<bool>,
}

pub(crate) type WindowName = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct TmuxPane {
    pub(crate) index: usize,
    pub(crate) path: String,
    pub(crate) active: bool,
    pub(crate) startup_command: Option<String>,
    pub(crate) shell_command: Option<String>,
    #[serde(default)]
    pub(crate) environment: Vec<EnvironmentVariable>,
}

pub(crate) type EnvironmentVariable = NameValue;

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

#[derive(Debug, Clone)]
pub(crate) struct StatusPane {
    pub(crate) index: usize,
    pub(crate) window_name: Option<String>,
    pub(crate) active: bool,
}

#[derive(Debug)]
pub(crate) struct StatusWindow {
    pub(crate) name: WindowName,
    pub(crate) index: usize,
    pub(crate) active: bool,
    pub(crate) panes: Vec<StatusPane>,
}

impl TmuxWindow {
    /// `index` is 1-based (tmux style)
    pub(crate) fn startup_command_for_pane(&self, index: usize) -> Option<String> {
        if self.panes.is_empty() {
            None
        } else {
            self.panes[index - 1].startup_command.clone()
        }
    }
}
