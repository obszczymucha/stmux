use std::{collections::HashMap, process::Command};

use crate::{
    model::{StatusPane, StatusWindow, TmuxPane, TmuxSession, TmuxWindow, WindowName},
    tmux::Tmux,
};

pub(crate) trait Window {
    /// Splits the current window into two panes, one for the current session and another for the
    /// first window of given (stored) session.
    fn smart_split(&self, session_name: &str, session: &TmuxSession);
    fn list_with_pane_details(&self, session_name: &str) -> Vec<TmuxWindow>;
    fn list_names_for_current_session(&self) -> Vec<WindowName>;
    fn list_names_for_status(&self) -> Vec<StatusWindow>;
}

struct PaneWindowName {
    pub(crate) index: usize,
    pub(crate) window_name: Option<String>,
}

pub(crate) struct WindowImpl<'t, T: Tmux> {
    tmux: &'t T,
}

impl<'t, T: Tmux> WindowImpl<'t, T> {
    pub(crate) fn new(tmux: &'t T) -> Self {
        Self { tmux }
    }

    fn refresh_status(&self) {
        Command::new("tmux")
            .arg("refresh-client")
            .arg("-S")
            .status()
            .expect("Couldn't refresh status.");
    }

    fn split_window(&self, pane: &TmuxPane) -> usize {
        self.tmux
            .split_current_window(true, &pane.path, &pane.startup_command);

        let pane_count = self.tmux.count_panes();

        if pane.startup_command.is_none()
            && let Some(shell_command) = &pane.shell_command
        {
            self.tmux
                .send_keys_to_current_window(pane_count, shell_command);
        }

        pane_count
    }

    fn replace_pane(
        &self,
        session_name: &str,
        pane_to_swap: &TmuxPane,
        current_window_last_pane: &PaneWindowName,
    ) {
        let window_name = current_window_last_pane
            .window_name
            .clone()
            .unwrap_or("princess_kenny".to_string());

        let current_session_window_names = self.list_names_for_current_session();

        if !current_session_window_names
            .into_iter()
            .any(|w| w == session_name)
        {
            self.tmux.new_window_in_current_session(
                window_name.as_str(),
                &pane_to_swap.path,
                &pane_to_swap.environment,
                &pane_to_swap.startup_command,
                true,
            );

            if pane_to_swap.startup_command.is_none()
                && let Some(shell_command) = &pane_to_swap.shell_command
            {
                let current_session_name = self.tmux.current_session_name();
                self.tmux.send_keys(
                    &current_session_name,
                    &window_name,
                    1,
                    shell_command.as_str(),
                );
            }

            self.tmux
                .swap_panes(current_window_last_pane.index, window_name.as_str(), 1);
        } else {
            self.tmux
                .swap_panes(current_window_last_pane.index, session_name, 1);
            self.tmux
                .rename_window_in_current_session(session_name, window_name.as_str());
        }

        self.tmux.set_pane_option_for_current_window(
            current_window_last_pane.index,
            "@window-name",
            session_name,
        );
    }

    fn get_pane_window_names(&self) -> Vec<PaneWindowName> {
        self.tmux
            .list_current_window_panes("#{pane_index}:#{@window-name}")
            .iter()
            .map(|pane_info| {
                let parts = pane_info
                    .split(':')
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                PaneWindowName {
                    index: parts[0].parse().unwrap_or(0),
                    window_name: {
                        let name = &parts[1];
                        if name.is_empty() {
                            None
                        } else {
                            Some(name.clone())
                        }
                    },
                }
            })
            .collect()
    }
}

impl<'t, T: Tmux> Window for WindowImpl<'t, T> {
    /// Performs a smart split based on the current window state.
    /// - Single pane: Creates a split within the current window.
    /// - Multiple panes: Creates a new window using the first pane from the `session`'s first window,
    ///   then swaps it with the current window's last pane.
    fn smart_split(&self, session_name: &str, session: &TmuxSession) {
        if let Some(pane) = session
            .windows
            .first()
            .and_then(|window| window.panes.first())
        {
            let pane_window_names = self.get_pane_window_names();
            let window_exists = {
                let window_names = self.list_names_for_current_session();
                window_names.into_iter().any(|w| w == session_name)
            };

            if window_exists && pane_window_names.len() == 1 {
                self.tmux.join_pane_to_current_window(session_name, 1);
                self.refresh_status();

                return;
            }

            if pane_window_names.len() == 1 {
                let pane_index = self.split_window(pane);
                self.tmux.set_pane_option_for_current_window(
                    pane_index,
                    "@window-name",
                    session_name,
                );
                return;
            }

            if let Some(pane) = pane_window_names
                .iter()
                .filter(|p| p.window_name.is_some())
                .find(|p| matches!(&p.window_name, Some(name) if name == session_name))
            {
                self.tmux.display_message(
                    format!(
                    "#[fg=#e0e0e0,align=centre]Pane #[fg=#8a60ab]{}#[fg=#e0e0e0] is #[fg=#8a60ab]{}",
                    pane.index,
                    session_name
                )
                    .as_str(),
                );
                return;
            }

            if let Some(current_window_last_pane) = pane_window_names.last() {
                self.replace_pane(session_name, pane, current_window_last_pane);
            }
        }
    }

    fn list_with_pane_details(&self, session_name: &str) -> Vec<TmuxWindow> {
        let output = self.tmux.list_session_panes(session_name,
            "#{window_index}:#{window_name}:#{window_layout}:#{pane_index}:#{pane_active}:#{pane_current_path}");

        let mut windows: Vec<TmuxWindow> = Vec::new();
        let mut map: HashMap<WindowName, usize> = HashMap::new();
        let mut index: usize = 0;

        for window in output {
            let tokens = window.split(':').collect::<Vec<&str>>();
            let window_index = tokens[0].parse::<usize>().unwrap();
            let window_name = tokens[1];
            let layout = tokens[2];
            let pindex = tokens[3].parse::<usize>().unwrap();
            let active = tokens[4] == "1";
            let path = tokens[5].to_string();
            let pane = TmuxPane {
                index: pindex,
                path,
                active,
                startup_command: None,
                shell_command: None,
                environment: vec![],
            };

            if let Some(i) = map.get(window_name) {
                let window = &mut windows[*i];
                window.panes.push(pane);
            } else {
                let window = TmuxWindow {
                    index: window_index,
                    name: window_name.to_string(),
                    layout: layout.to_string(),
                    panes: vec![pane],
                    active: None,
                };

                windows.push(window);
                map.insert(window_name.to_string(), index);
                index += 1;
            }
        }

        windows
    }

    fn list_names_for_current_session(&self) -> Vec<WindowName> {
        self.tmux.list_windows_for_current_session("#W")
    }

    fn list_names_for_status(&self) -> Vec<StatusWindow> {
        // We need to list panes, not windows, to get all panes in each window
        let lines = self.tmux.list_current_session_panes(
            "#{window_index}:#{window_active}:#W:#{pane_index}:#{@window-name}:#{pane_active}",
        );

        let mut windows: HashMap<usize, StatusWindow> = HashMap::new();

        for line in &lines {
            let parts: Vec<&str> = line.split(':').collect();

            if parts.len() < 6 {
                continue;
            }

            let window_index = parts[0].parse::<usize>().unwrap_or(0);
            let window_active = parts[1] == "1";
            let window_name = parts[2].to_string();
            let pane_index = parts[3].parse::<usize>().unwrap_or(0);
            let pane_window_name = if parts[4].is_empty() {
                None
            } else {
                Some(parts[4].to_string())
            };

            let pane_active = parts[5] == "1";

            let status_pane = StatusPane {
                index: pane_index,
                window_name: pane_window_name,
                active: pane_active,
            };

            windows
                .entry(window_index)
                .and_modify(|w| w.panes.push(status_pane.clone()))
                .or_insert_with(|| StatusWindow {
                    name: window_name.clone(),
                    index: window_index,
                    active: window_active,
                    panes: vec![status_pane],
                });
        }

        let mut result: Vec<StatusWindow> = windows.into_values().collect();
        result.sort_by_key(|w| w.index);
        result
    }
}
