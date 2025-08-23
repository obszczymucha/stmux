use std::collections::HashMap;

use crate::{
    model::{TmuxPane, TmuxSession, TmuxWindow, WindowName},
    tmux::Tmux,
};

pub(crate) trait Window {
    /// Splits the current window into two panes, one for the current session and another for the
    /// first window of given (stored) session.
    fn smart_split(&self, session_name: &str, session: &TmuxSession);
    fn list_with_pane_details(&self, session_name: &str) -> Vec<TmuxWindow>;
}

struct PaneWindowName {
    pub(crate) index: usize,
    pub(crate) window_name: String,
}

pub(crate) struct WindowImpl<'t, T: Tmux> {
    tmux: &'t T,
}

impl<'t, T: Tmux> WindowImpl<'t, T> {
    pub(crate) fn new(tmux: &'t T) -> Self {
        Self { tmux }
    }

    fn split_window(&self, pane: &TmuxPane) -> usize {
        self.tmux
            .split_current_window(true, &pane.path, &pane.startup_command);

        let pane_count = self.tmux.count_panes();

        if pane.startup_command.is_none() {
            if let Some(shell_command) = &pane.shell_command {
                self.tmux
                    .send_keys_to_current_window(pane_count, shell_command);
            }
        }

        pane_count
    }

    fn replace_pane(&self, pane: &TmuxPane) {
        self.tmux
            .display_message("#[fg=#8a60ab,align=centre]replacing #[fg=#e0e0e0]pane");
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
                    window_name: parts[1].clone(),
                }
            })
            .collect()
    }
}

impl<'t, T: Tmux> Window for WindowImpl<'t, T> {
    fn smart_split(&self, session_name: &str, session: &TmuxSession) {
        if let Some(pane) = session
            .windows
            .first()
            .and_then(|window| window.panes.first())
        {
            let pane_window_names = self.get_pane_window_names();

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
                .find(|p| p.window_name == session_name)
            {
                self.tmux.display_message(
                    format!(
                    "#[fg=#e0e0e0,align=centre]Pane #[fg=#8a60ab]{}#[fg=#e0e0e0] is #[fg=#8a60ab]{}",
                    pane.index,
                    session_name
                )
                    .as_str(),
                );
            } else {
                self.replace_pane(pane);
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
                environment: None,
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
}
