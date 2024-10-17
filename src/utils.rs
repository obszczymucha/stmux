use std::collections::HashMap;

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
