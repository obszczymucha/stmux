use std::{collections::HashMap, fs};

use crate::{
    model::{TmuxSessions, TmuxWindows},
    tmux::Tmux,
    utils,
};

pub(crate) trait Sessions {
    fn save(&self, filename: &str);
    fn restore(&self, filename: &str);
}

pub(crate) struct SessionsImpl<'a, T: Tmux> {
    tmux: &'a T,
}

impl<'a, T: Tmux> SessionsImpl<'a, T> {
    pub(crate) fn new(tmux: &'a T) -> Self {
        Self { tmux }
    }

    fn process_session(&self, session_name: &str, windows: TmuxWindows) {
        if windows.is_empty() {
            return;
        }

        for (i, tmux_window) in windows.into_iter().enumerate() {
            if i == 0 {
                self.tmux.new_session(session_name, &tmux_window)
            } else {
                self.tmux.new_window(session_name, &tmux_window, i)
            }
        }

        self.tmux.select_window(session_name, 1)
    }
}

impl<'a, T: Tmux> Sessions for SessionsImpl<'a, T> {
    fn save(&self, filename: &str) {
        let stored_sessions = load_from_file(filename);
        let current_sessions = self.tmux.list_sessions();
        let sessions = merge(stored_sessions, current_sessions);
        let toml_string =
            toml::to_string(&sessions).expect("Failed to serialize sessions into TOML.");

        fs::write(filename, toml_string)
            .unwrap_or_else(|_| panic!("Failed to write to {}.", filename));
        eprintln!("TMUX sessions saved to {} file.", filename);
    }

    fn restore(&self, filename: &str) {
        eprintln!("Restoring TMUX sessions from {} file...", filename);
        let sessions = load_from_file(filename);

        for (name, windows) in sessions {
            let non_numeric = !utils::is_numeric(name.as_str());

            if !windows.is_empty() && non_numeric && !self.tmux.has_session(name.as_str()) {
                self.process_session(name.as_str(), windows);
            }
        }
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

fn load_from_file(filename: &str) -> TmuxSessions {
    let file_content = fs::read_to_string(filename);

    match file_content {
        Ok(content) => {
            toml::from_str(&content).unwrap_or_else(|_| panic!("Failed to parse {}.", filename))
        }
        Err(_) => HashMap::new(),
    }
}
