use std::cmp::max;

use crate::{
    config::Config, model::WindowDimension, session_name_file::SessionNameFile, tmux::Tmux,
};

pub(crate) trait Bookmarks {
    fn print(&self);
    fn set(&self, tmux: &dyn Tmux);
    fn select(&self, index: usize) -> Option<String>;
    fn edit(&self, config: &dyn Config, tmux: &dyn Tmux);
}

pub(crate) struct BookmarksImpl<'s, S: SessionNameFile> {
    bookmarks_file: &'s S,
}

impl<'s, S: SessionNameFile> BookmarksImpl<'s, S> {
    pub(crate) fn new(bookmarks_file: &'s S) -> Self {
        Self { bookmarks_file }
    }
}

impl<'s, S: SessionNameFile> Bookmarks for BookmarksImpl<'s, S> {
    fn print(&self) {
        let bookmarks = self.bookmarks_file.read();

        for (i, bookmark) in bookmarks.iter().enumerate() {
            eprintln!("{:3}: {}", i + 1, bookmark);
        }
    }

    fn set(&self, tmux: &dyn Tmux) {
        let current_session_name = tmux.current_session_name();
        let bookmarks = self.bookmarks_file.read();

        if !bookmarks.contains(&current_session_name) {
            self.bookmarks_file.append(&current_session_name);
            tmux.display_message("Session bookmarked.");
        }
    }

    fn select(&self, index: usize) -> Option<String> {
        let bookmarks = self.bookmarks_file.read();
        bookmarks.get(index - 1).map(|s| s.to_string())
    }

    fn edit(&self, config: &dyn Config, tmux: &dyn Tmux) {
        let width = self
            .bookmarks_file
            .read()
            .iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(17);

        let popup_width = max(width + 6, 18);
        let popup_height = 7;
        let y = tmux.window_dimension().map(|d| d.height / 2 - 1);

        tmux.display_popup(
            "Bookmarks",
            &Some("fg=#9f7fff,italics,align=centre".to_string()),
            "fg=#806aba",
            &WindowDimension {
                width: popup_width,
                height: popup_height,
            },
            &y,
            &format!(
                "nvim --clean -u {} {}",
                config.neovim_config_filename(),
                config.bookmarks_filename()
            ),
        );
    }
}
