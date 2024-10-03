use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub(crate) type SessionName = String;
pub(crate) type WindowName = String;
pub(crate) type TmuxWindows = Vec<TmuxWindow>;
pub(crate) type TmuxSessions = HashMap<SessionName, TmuxWindows>;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TmuxWindow {
    pub(crate) name: WindowName,
    pub(crate) layout: String,
    pub(crate) panes: Vec<TmuxPane>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TmuxPane {
    pub(crate) path: String,
    pub(crate) active: bool,
}
