use std::cmp::max;

use crate::{config::Config, session_name_file::SessionNameFile, tmux::Tmux};

pub(crate) trait Recent {
    fn next(&self, session_name: &str) -> Option<String>;
    fn previous(&self, session_name: &str) -> Option<String>;
    fn print(&self);
    fn edit(&self, config: &dyn Config);
}

pub(crate) struct RecentImpl<'t, 's, T: Tmux, S: SessionNameFile> {
    tmux: &'t T,
    recent_session_file: &'s S,
}

impl<'t, 's, T: Tmux, S: SessionNameFile> RecentImpl<'t, 's, T, S> {
    pub(crate) fn new(tmux: &'t T, recent_session_file: &'s S) -> Self {
        Self {
            tmux,
            recent_session_file,
        }
    }
}

impl<'t, 's, T: Tmux, S: SessionNameFile> Recent for RecentImpl<'t, 's, T, S> {
    fn next(&self, session_name: &str) -> Option<String> {
        let session_names = self.tmux.list_session_names();
        let recent_session_names = &self.recent_session_file.read();

        recent_session_names
            .iter()
            .filter(|name| session_names.contains(name))
            .skip_while(|&name| name != session_name)
            .nth(1)
            .cloned()
    }

    fn previous(&self, session_name: &str) -> Option<String> {
        let session_names = self.tmux.list_session_names();
        let recent_session_names = &self.recent_session_file.read();
        let mut previous_name = None;

        for name in recent_session_names
            .iter()
            .filter(|name| session_names.contains(name))
        {
            if name == session_name {
                return previous_name.clone();
            }

            previous_name = Some(name.clone());
        }

        None
    }

    fn print(&self) {
        let recent_session_names = &self.recent_session_file.read();

        for name in recent_session_names {
            eprintln!("{}", name.trim());
        }
    }

    fn edit(&self, config: &dyn Config) {
        let width = self
            .recent_session_file
            .read()
            .iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(17);

        let popup_width = max(width + 6, 21);
        let popup_height = 7;
        let y = self.tmux.window_dimension().map(|d| d.height / 2 - 1);

        self.tmux.display_popup(
            "Recent sessions",
            &Some("fg=#9f7fff italics align=centre".to_string()),
            "fg=#806aba", // TODO: put in the config
            popup_width,
            popup_height,
            &y,
            &format!(
                "nvim --clean -u {} {}",
                config.neovim_config_filename(),
                config.recent_sessions_filename()
            ),
        );
    }
}

#[cfg(test)]
mod next_tests {
    use super::*;
    use crate::{session_name_file, tmux};
    use session_name_file::MockSessionNameFile;
    use tmux::MockTmux;

    #[test]
    fn should_return_the_next_session() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_session_names()
            .returning(|| vec!["a".into(), "b".into()].clone());

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.next("a");

        // Then
        assert_eq!(result.as_deref(), Some("b"));
    }

    #[test]
    fn should_return_the_next_available_session() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_session_names()
            .returning(|| vec!["a".into(), "c".into()].clone());

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.next("a");

        // Then
        assert_eq!(result.as_deref(), Some("c"));
    }

    #[test]
    fn should_return_none_if_the_session_is_the_last_one() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_session_names()
            .returning(|| vec!["a".into(), "b".into(), "c".into()].clone());

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.next("c");

        // Then
        assert_eq!(result.as_deref(), None);
    }
}

#[cfg(test)]
mod previous_tests {
    use super::*;
    use crate::{session_name_file, tmux};
    use session_name_file::MockSessionNameFile;
    use tmux::MockTmux;

    #[test]
    fn should_return_the_previous_session() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_session_names()
            .returning(|| vec!["a".into(), "b".into(), "c".into()].clone());

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.previous("c");

        // Then
        assert_eq!(result.as_deref(), Some("b"));
    }

    #[test]
    fn should_return_the_previous_available_session() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_session_names()
            .returning(|| vec!["a".into(), "c".into()].clone());

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.previous("c");

        // Then
        assert_eq!(result.as_deref(), Some("a"));
    }

    #[test]
    fn should_return_none_if_the_session_is_the_first_one() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_session_names()
            .returning(|| vec!["a".into(), "b".into(), "c".into()].clone());

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.previous("a");

        // Then
        assert_eq!(result.as_deref(), None);
    }
}
