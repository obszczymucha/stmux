use crate::{model::WindowNameAndStatus, session_name_file::SessionNameFile, tmux::Tmux};

pub(crate) trait Status {
    fn get(&self) -> String;
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
        fn current(session_name: &str, windows: &[WindowNameAndStatus]) -> String {
            format!(
                "{}{} {}",
                "#[fg=#8a60ab]",
                session_name,
                windows
                    .iter()
                    .map(|w| {
                        if w.active {
                            format!("#[fg=#e0e0e0]{}", w.name)
                        } else {
                            format!("#[fg=#9797aa]{}", w.name)
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            )
        }

        let session_name = self.tmux.current_session_name();
        let windows = self.tmux.list_windows_names_with_status();
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
}
