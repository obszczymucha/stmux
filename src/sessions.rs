use std::{collections::HashMap, fs, thread::sleep, time::Duration};

use crate::{
    model::{Layout, TmuxSessions, TmuxWindows},
    tmux::Tmux,
    utils,
};

pub(crate) trait Sessions {
    fn save(&self);
    fn restore_all(&self);
    fn restore(&self, session_name: &str);
    fn load(&self) -> TmuxSessions;
}

pub(crate) struct SessionsImpl<'t, T: Tmux> {
    filename: String,
    tmux: &'t T,
}

impl<'t, T: Tmux> SessionsImpl<'t, T> {
    pub(crate) fn new(filename: &str, tmux: &'t T) -> Self {
        Self {
            filename: filename.to_string(),
            tmux,
        }
    }

    fn process_session(
        &self,
        session_name: &str,
        windows: &TmuxWindows,
        windows_to_layout: &mut Vec<Layout>,
    ) {
        if windows.is_empty() {
            return;
        }

        for (i, tmux_window) in windows.iter().enumerate() {
            if i == 0 {
                self.tmux.new_session(session_name, tmux_window);

                if let Some(startup_command) = tmux_window.startup_command_for_pane(1) {
                    self.tmux
                        .send_keys(session_name, &tmux_window.name, Some(1), &startup_command);
                }

                if tmux_window.panes.len() > 1 {
                    for (i, pane) in tmux_window.panes.iter().enumerate().skip(1) {
                        self.tmux
                            .split_window(session_name, &tmux_window.name, &pane.path);

                        if let Some(startup_command) = &pane.startup_command {
                            self.tmux.send_keys(
                                session_name,
                                &tmux_window.name,
                                Some(i),
                                startup_command,
                            );
                        }
                    }

                    windows_to_layout.push(Layout {
                        session_name: session_name.to_string(),
                        window_name: tmux_window.name.clone(),
                        layout: tmux_window.layout.clone(),
                    });
                }
            } else {
                self.tmux.new_window(session_name, tmux_window, i);

                if let Some(startup_command) = tmux_window.startup_command_for_pane(1) {
                    self.tmux
                        .send_keys(session_name, &tmux_window.name, Some(1), &startup_command);
                }

                if tmux_window.panes.len() > 1 {
                    for pane in tmux_window.panes.iter().skip(1) {
                        self.tmux
                            .split_window(session_name, &tmux_window.name, &pane.path);

                        if let Some(startup_command) = &pane.startup_command {
                            self.tmux.send_keys(
                                session_name,
                                &tmux_window.name,
                                Some(i),
                                startup_command,
                            );
                        }
                    }

                    windows_to_layout.push(Layout {
                        session_name: session_name.to_string(),
                        window_name: tmux_window.name.clone(),
                        layout: tmux_window.layout.clone(),
                    });
                }
            }
        }

        self.tmux.select_window(session_name, 1)
    }

    fn restore_layouts(&self, windows_to_layout: &Vec<Layout>, delay_in_millis: u64) {
        if !windows_to_layout.is_empty() {
            if delay_in_millis > 0 {
                sleep(Duration::from_millis(delay_in_millis)); // To remove WSL quirks.
            }

            for layout in windows_to_layout {
                self.tmux
                    .select_layout(&layout.session_name, &layout.window_name, &layout.layout);
            }
        }
    }
}

impl<'t, T: Tmux> Sessions for SessionsImpl<'t, T> {
    fn save(&self) {
        let stored_sessions = self.load();
        let current_sessions = self.tmux.list_sessions();
        let sessions = merge(stored_sessions, current_sessions);
        let toml_string =
            toml::to_string(&sessions).expect("Failed to serialize sessions into TOML.");

        fs::write(&self.filename, toml_string)
            .unwrap_or_else(|_| panic!("Failed to write to {}.", &self.filename));
        eprintln!("TMUX sessions saved to {} file.", &self.filename);
    }

    fn restore_all(&self) {
        eprintln!("Restoring TMUX sessions from {} file...", &self.filename);
        let sessions = self.load();
        // eprintln!("sessions: {:?}", sessions);

        let mut windows_to_layout = Vec::new();

        for (name, windows) in sessions {
            let non_numeric = !utils::is_numeric(name.as_str());

            if !windows.is_empty() && non_numeric && !self.tmux.has_session(name.as_str()) {
                self.process_session(name.as_str(), &windows, &mut windows_to_layout);
            }
        }

        self.restore_layouts(&windows_to_layout, 300);
    }

    fn load(&self) -> TmuxSessions {
        let file_content = fs::read_to_string(&self.filename);

        match file_content {
            Ok(content) => toml::from_str(&content)
                .unwrap_or_else(|_| panic!("Failed to parse {}.", &self.filename)),
            Err(_) => HashMap::new(),
        }
    }

    fn restore(&self, session_name: &str) {
        let sessions = self.load();

        if let Some(windows) = sessions.get(session_name) {
            let mut windows_to_layout = Vec::new();
            self.process_session(session_name, windows, &mut windows_to_layout);
            self.restore_layouts(&windows_to_layout, 300);
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
