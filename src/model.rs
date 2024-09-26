use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TmuxWindow {
    pub(crate) name: String,
    pub(crate) path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TmuxSession {
    pub(crate) windows: Vec<TmuxWindow>,
}
