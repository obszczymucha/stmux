use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub(crate) type SessionName = String;
pub(crate) type TmuxWindows = Vec<TmuxWindow>;
pub(crate) type TmuxSessions = HashMap<SessionName, TmuxWindows>;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TmuxWindow {
    pub(crate) name: String,
    pub(crate) path: String,
}
