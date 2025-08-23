use std::{collections::HashMap, fs, thread::sleep, time::Duration};

use crate::{
    model::{Layout, SessionAndWindowName, SessionName, TmuxSession, TmuxSessions, TmuxWindows},
    tmux::Tmux,
    utils,
};

pub(crate) trait SessionStorage {
    fn save(&self, sessions: TmuxSessions);
    fn restore_all(&self);
    fn restore(&self, session_name: &str) -> Option<bool>;
    fn load(&self) -> TmuxSessions;
    fn list(&self) -> Vec<SessionName>;
    fn convert(&self, output: &str);
}

pub(crate) struct SessionStorageImpl<'t, T: Tmux> {
    filename: String,
    tmux: &'t T,
}

impl<'t, T: Tmux> SessionStorageImpl<'t, T> {
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
                let command = tmux_window.startup_command_for_pane(1);
                self.tmux.new_session(session_name, tmux_window, &command);

                if command.is_none() {
                    if let Some(shell_command) = tmux_window.shell_command_for_pane(i + 1) {
                        self.tmux.send_keys(
                            Some(SessionAndWindowName {
                                session_name: session_name.to_string(),
                                window_name: tmux_window.name.clone(),
                            }),
                            1,
                            &shell_command,
                        );
                    }
                }

                if tmux_window.panes.len() > 1 {
                    for (i, pane) in tmux_window.panes.iter().enumerate().skip(1) {
                        let command = tmux_window.startup_command_for_pane(i + 1);

                        self.tmux.split_window(
                            Some(SessionAndWindowName {
                                session_name: session_name.to_string(),
                                window_name: tmux_window.name.clone(),
                            }),
                            true,
                            &pane.path,
                            &command,
                        );

                        if command.is_none() {
                            if let Some(shell_command) = &pane.shell_command {
                                self.tmux.send_keys(
                                    Some(SessionAndWindowName {
                                        session_name: session_name.to_string(),
                                        window_name: tmux_window.name.clone(),
                                    }),
                                    i,
                                    shell_command,
                                );
                            }
                        }
                    }

                    windows_to_layout.push(Layout {
                        session_name: session_name.to_string(),
                        window_name: tmux_window.name.clone(),
                        layout: tmux_window.layout.clone(),
                    });
                }
            } else {
                let command = tmux_window.startup_command_for_pane(1);
                self.tmux.new_window(session_name, tmux_window, i, &command);

                if command.is_none() {
                    if let Some(shell_command) = tmux_window.shell_command_for_pane(1) {
                        self.tmux.send_keys(
                            Some(SessionAndWindowName {
                                session_name: session_name.to_string(),
                                window_name: tmux_window.name.clone(),
                            }),
                            1,
                            &shell_command,
                        );
                    }
                }

                if tmux_window.panes.len() > 1 {
                    for pane in tmux_window.panes.iter().skip(1) {
                        let command = tmux_window.startup_command_for_pane(i + 1);
                        self.tmux.split_window(
                            Some(SessionAndWindowName {
                                session_name: session_name.to_string(),
                                window_name: tmux_window.name.clone(),
                            }),
                            true,
                            &pane.path,
                            &command,
                        );

                        if command.is_none() {
                            if let Some(shell_command) = &pane.shell_command {
                                self.tmux.send_keys(
                                    Some(SessionAndWindowName {
                                        session_name: session_name.to_string(),
                                        window_name: tmux_window.name.clone(),
                                    }),
                                    i,
                                    shell_command,
                                );
                            }
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

impl<'t, T: Tmux> SessionStorage for SessionStorageImpl<'t, T> {
    fn save(&self, sessions: TmuxSessions) {
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

        for (name, session) in sessions {
            let non_numeric = !utils::is_numeric(name.as_str());
            let windows = session.windows;

            if !windows.is_empty() && non_numeric && !self.tmux.has_session(name.as_str()) {
                self.process_session(name.as_str(), &windows, &mut windows_to_layout);
            }
        }

        self.restore_layouts(&windows_to_layout, 300);
    }

    fn load(&self) -> HashMap<SessionName, TmuxSession> {
        let file_content = fs::read_to_string(&self.filename);

        match file_content {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|error| {
                panic!("Failed to parse {}: {}.", &self.filename, error.message())
            }),
            Err(_) => HashMap::new(),
        }
    }

    // The return bool value indicates whether the session was spawned in the background. Yeah, I
    // know returning optional bool doesn't say shit. I'm lazy.
    fn restore(&self, session_name: &str) -> Option<bool> {
        let sessions = self.load();

        sessions.get(session_name).map(|session| {
            let mut windows_to_layout = Vec::new();
            let windows = &session.windows;
            self.process_session(session_name, windows, &mut windows_to_layout);
            self.restore_layouts(&windows_to_layout, 300);

            if !windows.is_empty() {
                if let Some(background) = session.background {
                    return background;
                }
            }

            false
        })
    }

    fn convert(&self, output: &str) {
        let file_content = fs::read_to_string(&self.filename);

        if let Ok(content) = file_content {
            let old: HashMap<SessionName, TmuxWindows> =
                toml::from_str(&content).unwrap_or_else(|error| {
                    panic!("Failed to parse {}: {}.", &self.filename, error.message())
                });

            let new: HashMap<SessionName, TmuxSession> = old
                .into_iter()
                .map(|(name, windows)| {
                    (
                        name.clone(),
                        TmuxSession {
                            background: None,
                            no_recent_tracking: None,
                            windows,
                        },
                    )
                })
                .collect();

            let toml_string =
                toml::to_string(&new).expect("Failed to serialize sessions into TOML.");

            fs::write(output, toml_string)
                .unwrap_or_else(|_| panic!("Failed to write to {}.", &output));
            eprintln!("TMUX sessions saved to {} file.", &output);
        }
    }

    fn list(&self) -> Vec<SessionName> {
        let file_content = fs::read_to_string(&self.filename);

        match file_content {
            Ok(content) => {
                let sessions: HashMap<SessionName, TmuxSession> = toml::from_str(&content)
                    .unwrap_or_else(|error| {
                        panic!("Failed to parse {}: {}.", &self.filename, error.message())
                    });

                sessions.keys().cloned().collect()
            }
            Err(_) => Vec::new(),
        }
    }
}
