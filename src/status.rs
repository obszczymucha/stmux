use crate::{
    model::WindowDetails,
    session_name_file::SessionNameFile,
    tmux::Tmux,
    window::{Window, WindowImpl},
};

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
        fn current(session_name: &str, windows: &[WindowDetails]) -> String {
            format!(
                "{}{} {}",
                "#[fg=#8a60ab]",
                session_name,
                windows
                    .iter()
                    .map(|w| {
                        if w.active {
                            if let Some(pane_window_name) = &w.pane_window_name {
                                format!("#[fg=#9797aa][#[fg=#e0e0e0]{}#[fg=#9797aa]|#[fg=#d0d0d0]{}#[fg=#9797aa]]", w.name, pane_window_name)
                            } else {
                                format!("#[fg=#9797aa][#[fg=#e0e0e0]{}#[fg=#9797aa]]", w.name)
                            }
                        } else {
                            format!("#[fg=#9797aa]{}", w.name)
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            )
        }

        let session_name = self.tmux.current_session_name();
        let window = WindowImpl::new(self.tmux);
        let windows = window.list_names_with_status_for_current_session();
        let bookmarks = self.bookmarks.read();
        let bookmark_names = bookmarks
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if v == &session_name {
                    let index = if i == 0 {
                        format!("{}", i + 1)
                    } else {
                        format!(" {}", i + 1)
                    };

                    format!(
                        "{}{}{}:{}{} ",
                        "#[fg=#8a60ba]",
                        index,
                        "#[fg=#af9fbf]",
                        "#[fg=#e0e0e0]",
                        current(v, &windows)
                    )
                } else {
                    format!(
                        "{}{}{}:{}{}",
                        "#[fg=#8a60ba]",
                        i + 1,
                        "#[fg=#af9fbf]",
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
