use std::collections::HashMap;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::TmuxSessions;

pub(crate) fn is_numeric(as_str: &str) -> bool {
    as_str.chars().all(char::is_numeric)
}

pub(crate) fn merge(config_sessions: TmuxSessions, current_sessions: TmuxSessions) -> TmuxSessions {
    let mut sessions = HashMap::new();

    for (name, windows) in config_sessions {
        sessions.insert(name, windows);
    }

    for (name, windows) in current_sessions {
        if !sessions.contains_key(name.as_str()) {
            sessions.insert(name, windows);
        }
    }

    sessions
}

pub(crate) fn random_window_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let random_number = (nanos % 98) + 1;
    format!("win_{}", random_number)
}

// A temporary workaround for when hooks are not implemented in tmux yet.
pub(crate) fn refresh_status() {
    Command::new("stmux")
        .arg("status")
        .status()
        .expect("Couldn't refresh status.");
}
