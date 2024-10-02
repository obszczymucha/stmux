use crate::{session_name_file::SessionNameFile, tmux::Tmux};

pub(crate) trait Bookmarks {
    fn print(&self);
    fn bookmark(&self, tmux: &dyn Tmux);
    fn select(&self, index: usize) -> Option<String>;
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
            println!("{:3}: {}", i + 1, bookmark);
        }
    }

    fn bookmark(&self, tmux: &dyn Tmux) {
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
}
