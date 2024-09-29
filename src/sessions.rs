use std::{collections::HashMap, fs};

use crate::{
    model::{TmuxSessions, TmuxWindows},
    tmux, utils,
};

pub(crate) fn save(filename: &str) {
    let stored_sessions = load_from_file(filename);
    let current_sessions = tmux::list_sessions();
    let sessions = merge(stored_sessions, current_sessions);
    let toml_string = toml::to_string(&sessions).expect("Failed to serialize sessions into TOML.");

    fs::write(filename, toml_string).unwrap_or_else(|_| panic!("Failed to write to {}.", filename));
    eprintln!("TMUX sessions saved to {} file.", filename);
}

fn load_from_file(filename: &str) -> TmuxSessions {
    let file_content = fs::read_to_string(filename);

    match file_content {
        Ok(content) => {
            toml::from_str(&content).unwrap_or_else(|_| panic!("Failed to parse {}.", filename))
        }
        Err(_) => HashMap::new(),
    }
}

fn merge(config_sessions: TmuxSessions, current_sessions: TmuxSessions) -> TmuxSessions {
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

pub(crate) fn restore(filename: &str) {
    eprintln!("Restoring TMUX sessions from {} file...", filename);
    let sessions = load_from_file(filename);

    for (name, windows) in sessions {
        let non_numeric = !utils::is_numeric(name.as_str());

        if !windows.is_empty() && non_numeric && !tmux::has_session(name.as_str()) {
            process_session(name.as_str(), windows);
        }
    }
}

fn process_session(session_name: &str, windows: TmuxWindows) {
    if windows.is_empty() {
        return;
    }

    for (i, tmux_window) in windows.into_iter().enumerate() {
        if i == 0 {
            tmux::new_session(session_name, &tmux_window)
        } else {
            tmux::new_window(session_name, &tmux_window, i)
        }
    }

    tmux::select_window(session_name, 1)
}
