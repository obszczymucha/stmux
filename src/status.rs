use crate::{
    model::{StatusPane, StatusWindow},
    session_name_file::SessionNameFile,
    status_config::StatusConfig,
    tmux::Tmux,
    window::{Window, WindowImpl},
};

pub(crate) trait Status {
    fn get(&self) -> String;
    fn set(&self);
}

pub(crate) struct StatusImpl<'t, 'b, 'c, T: Tmux, B: SessionNameFile> {
    tmux: &'t T,
    bookmarks: &'b B,
    config: &'c StatusConfig,
}

impl<'t, 'b, 'c, T: Tmux, B: SessionNameFile> StatusImpl<'t, 'b, 'c, T, B> {
    pub(crate) fn new(tmux: &'t T, bookmarks: &'b B, config: &'c StatusConfig) -> Self {
        Self {
            tmux,
            bookmarks,
            config,
        }
    }
}

impl<'t, 'b, 'c, T: Tmux, B: SessionNameFile> Status for StatusImpl<'t, 'b, 'c, T, B> {
    fn get(&self) -> String {
        fn format_pane(w: &StatusWindow, p: &StatusPane, c: &StatusConfig) -> String {
            let name = if w.panes.len() == 1 {
                w.name.clone()
            } else {
                p.window_name.clone().unwrap_or(p.index.to_string())
            };

            if w.active && p.active {
                format!("{}{}", c.colors.selected.active_pane, name)
            } else {
                format!("{}{}", c.colors.selected.inactive_pane, name)
            }
        }

        fn format_window(w: &StatusWindow, c: &StatusConfig) -> String {
            w.panes
                .iter()
                .map(|p| format_pane(w, p, c))
                .collect::<Vec<String>>()
                .join(
                    format!(
                        "{}{}",
                        c.colors.selected.pane_separator, c.style.pane_separator
                    )
                    .as_str(),
                )
        }

        fn current(session_name: &str, windows: &[StatusWindow], c: &StatusConfig) -> String {
            format!(
                "{}{} {}",
                c.colors.selected.session_name,
                session_name,
                windows
                    .iter()
                    .map(|w| {
                        if w.active {
                            format!(
                                "{}{}{}{}{}",
                                c.colors.selected.window_before,
                                c.style.window_before,
                                format_window(w, c),
                                c.colors.selected.window_after,
                                c.style.window_after
                            )
                        } else {
                            format_window(w, c)
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            )
        }

        let active_session_names = self.tmux.list_sessions("#S").unwrap();
        let session_name = self.tmux.current_session_name();
        let window = WindowImpl::new(self.tmux);
        let windows = window.list_names_for_status();
        let bookmarks = self.bookmarks.read();
        let bookmark_names = bookmarks
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let (color, colon_color) = if active_session_names.contains(v) {
                    (
                        self.config.colors.active.session_number.as_str(),
                        self.config.colors.active.number_separator.as_str(),
                    )
                } else {
                    (
                        self.config.colors.inactive.session_number.as_str(),
                        self.config.colors.inactive.number_separator.as_str(),
                    )
                };

                if v == &session_name {
                    let index = if i == 0 {
                        format!("{}", i + 1)
                    } else {
                        format!(" {}", i + 1)
                    };

                    format!(
                        "{}{}{}{}{} ",
                        self.config.colors.selected.session_number,
                        index,
                        self.config.colors.selected.number_separator,
                        self.config.style.number_separator,
                        current(v, &windows, self.config)
                    )
                } else {
                    format!(
                        "{}{}{}:{}{}",
                        color,
                        i + 1,
                        colon_color,
                        self.config.colors.inactive.session_name,
                        v
                    )
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        format!(
            "{}{}",
            if !bookmarks.contains(&session_name) {
                format!("{}  ", current(&session_name, &windows, self.config))
            } else {
                "".to_string()
            },
            bookmark_names
        )
    }

    fn set(&self) {
        self.tmux.set_global("status-left", &self.get());
    }
}
