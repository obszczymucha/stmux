use crate::{
    model::{TmuxPane, TmuxSession},
    tmux::Tmux,
};

pub(crate) trait Window {
    /// Splits the current window into two panes, one for the current session and another for the
    /// first window of given (stored) session.
    fn smart_split(&self, session_name: &str, session: &TmuxSession);
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
            .split_window(None, true, &pane.path, &pane.startup_command);

        let pane_count = self.tmux.count_panes();

        if pane.startup_command.is_none() {
            if let Some(shell_command) = &pane.shell_command {
                self.tmux.send_keys(None, pane_count, shell_command);
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
            .list_panes_with_format("#{pane_index}:#{@window-name}")
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
                self.tmux
                    .set_pane_option(None, pane_index, "@window-name", session_name);
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
}
