use std::{collections::HashMap, fs, thread::sleep, time::Duration};

use crate::{
    model::{Layout, TmuxSessions, TmuxWindows},
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

    fn process_session(
        &self,
        session_name: &str,
        windows: TmuxWindows,
        windows_to_layout: &mut Vec<Layout>,
    ) {
        if windows.is_empty() {
            return;
        }

        // eprintln!("Session name: {}. Windows: {}", session_name, windows.len());
        for (i, tmux_window) in windows.into_iter().enumerate() {
            if i == 0 {
                self.tmux.new_session(session_name, &tmux_window);

                if tmux_window.panes.len() > 1 {
                    for pane in tmux_window.panes.iter().skip(1) {
                        self.tmux
                            .split_window(session_name, &tmux_window.name, &pane.path);
                    }

                    windows_to_layout.push(Layout {
                        session_name: session_name.to_string(),
                        window_name: tmux_window.name,
                        layout: tmux_window.layout,
                    });
                }
            } else {
                self.tmux.new_window(session_name, &tmux_window, i);

                if tmux_window.panes.len() > 1 {
                    for pane in tmux_window.panes.iter().skip(1) {
                        self.tmux
                            .split_window(session_name, &tmux_window.name, &pane.path);
                    }

                    windows_to_layout.push(Layout {
                        session_name: session_name.to_string(),
                        window_name: tmux_window.name,
                        layout: tmux_window.layout,
                    });
                }
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
        // eprintln!("sessions: {:?}", sessions);

        let mut windows_to_layout = Vec::new();

        for (name, windows) in sessions {
            let non_numeric = !utils::is_numeric(name.as_str());

            if !windows.is_empty() && non_numeric && !self.tmux.has_session(name.as_str()) {
                self.process_session(name.as_str(), windows, &mut windows_to_layout);
            }
        }

        if !windows_to_layout.is_empty() {
            sleep(Duration::from_millis(250)); // To remove WSL quirks.

            for layout in windows_to_layout {
                self.tmux
                    .select_layout(&layout.session_name, &layout.window_name, &layout.layout);
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
