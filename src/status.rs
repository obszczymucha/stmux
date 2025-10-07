use crate::{
    model::{StatusPane, StatusWindow},
    session_name_file::SessionNameFile,
    tmux::Tmux,
    window::{Window, WindowImpl},
};

const ACTIVE_SESSION_COLOR: &str = "#[fg=#8a60ba]";
const INACTIVE_SESSION_COLOR: &str = "#[fg=#574d62]";
const ACTIVE_SESSION_COLON_COLOR: &str = "#[fg=#8e78a5]";
const INACTIVE_SESSION_COLON_COLOR: &str = "#[fg=#8e78a5]";

pub(crate) trait Status {
    fn get(&self) -> String;
    fn set(&self);
}

pub(crate) struct StatusImpl<'t, 'b, T: Tmux, B: SessionNameFile> {
    tmux: &'t T,
    bookmarks: &'b B,
}

impl<'t, 'b, T: Tmux, B: SessionNameFile> StatusImpl<'t, 'b, T, B> {
    pub(crate) fn new(tmux: &'t T, bookmarks: &'b B) -> Self {
        Self { tmux, bookmarks }
    }
}

impl<'t, 'b, T: Tmux, B: SessionNameFile> Status for StatusImpl<'t, 'b, T, B> {
    fn get(&self) -> String {
        fn format_pane(w: &StatusWindow, p: &StatusPane) -> String {
            let name = if w.panes.len() == 1 {
                w.name.clone()
            } else {
                p.window_name.clone().unwrap_or(p.index.to_string())
            };

            if w.active && p.active {
                format!("#[fg=#e0e0e0]{}#[fg=#9797aa]", name)
            } else {
                format!("#[fg=#9797aa]{}", name)
            }
        }

        fn format_window(w: &StatusWindow) -> String {
            w.panes
                .iter()
                .map(|p| format_pane(w, p))
                .collect::<Vec<String>>()
                .join("#[fg=#9797aa]|")
        }

        fn current(session_name: &str, windows: &[StatusWindow]) -> String {
            format!(
                "{}{} {}",
                ACTIVE_SESSION_COLOR,
                session_name,
                windows
                    .iter()
                    .map(|w| {
                        if w.active {
                            format!("#[fg=#9797aa][{}#[fg=#9797aa]]", format_window(w))
                        } else {
                            format_window(w)
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
                    (ACTIVE_SESSION_COLOR, ACTIVE_SESSION_COLON_COLOR)
                } else {
                    (INACTIVE_SESSION_COLOR, INACTIVE_SESSION_COLON_COLOR)
                };

                if v == &session_name {
                    let index = if i == 0 {
                        format!("{}", i + 1)
                    } else {
                        format!(" {}", i + 1)
                    };

                    format!(
                        "{}{}{}:{}{} ",
                        ACTIVE_SESSION_COLOR,
                        index,
                        "#[fg=#af9fbf]",
                        "#[fg=#e0e0e0]",
                        current(v, &windows)
                    )
                } else {
                    format!(
                        "{}{}{}:{}{}",
                        color,
                        i + 1,
                        colon_color,
                        "#[fg=#75707a]",
                        v
                    )
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        format!(
            "{}{}",
            if !bookmarks.contains(&session_name) {
                format!("{}  ", current(&session_name, &windows))
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
