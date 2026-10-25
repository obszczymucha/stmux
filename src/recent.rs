use std::cmp::max;

use crate::{
    config::Config,
    model::{TmuxSession, WindowDimension},
    session::{Session, SessionImpl},
    session_name_file::SessionNameFile,
    tmux::Tmux,
    utils,
};

pub(crate) trait Recent {
    fn add(&self, session: Option<&TmuxSession>, session_name: &str);
    fn next(&self, session_name: &str, all: bool) -> Option<String>;
    fn previous(&self, session_name: &str, all: bool) -> Option<String>;
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

    /// Running recent sessions, followed by (if `all`) the remaining running sessions sorted by name.
    fn session_names(&self, all: bool) -> Vec<String> {
        let mut current_session_names = SessionImpl::new(self.tmux).list_names();
        let mut session_names = self
            .recent_session_file
            .read()
            .into_iter()
            .filter(|s| current_session_names.contains(s))
            .collect::<Vec<String>>();

        if all {
            current_session_names.retain(|s| !session_names.contains(s));
            current_session_names.sort_by_key(|s| s.to_lowercase());
            session_names.extend(current_session_names);
        }

        session_names
    }
}

impl<'t, 's, T: Tmux, S: SessionNameFile> Recent for RecentImpl<'t, 's, T, S> {
    fn add(&self, session: Option<&TmuxSession>, session_name: &str) {
        if utils::is_numeric(session_name) {
            return;
        }

        if let Some(session) = session
            && let Some(no_recent_tracking) = session.no_recent_tracking
            && no_recent_tracking
        {
            return;
        }

        let mut names: Vec<String> = vec![session_name.to_string()];
        self.recent_session_file.read_into(&mut names, session_name);
        self.recent_session_file.write(&names);
    }

    fn next(&self, session_name: &str, all: bool) -> Option<String> {
        let session_names = self.session_names(all);

        if session_names.is_empty() {
            return None;
        }

        if !session_names.contains(&session_name.to_string()) {
            return Some(session_names[0].clone());
        }

        session_names
            .iter()
            .skip_while(|&name| name != session_name)
            .nth(1)
            .cloned()
    }

    fn previous(&self, session_name: &str, all: bool) -> Option<String> {
        let session_names = self.session_names(all);

        if session_names.is_empty() {
            return None;
        }

        let mut previous_name = None;

        for name in session_names.iter() {
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
            &WindowDimension {
                width: popup_width,
                height: popup_height,
            },
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
        tmux.expect_list_sessions()
            .returning(|_| Ok(vec!["a".into(), "b".into()].clone()));

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.next("a", false);

        // Then
        assert_eq!(result.as_deref(), Some("b"));
    }

    #[test]
    fn should_return_the_next_available_session() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_sessions()
            .returning(|_| Ok(vec!["a".into(), "c".into()].clone()));

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.next("a", false);

        // Then
        assert_eq!(result.as_deref(), Some("c"));
    }

    #[test]
    fn should_return_none_if_the_session_is_the_last_one() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_sessions()
            .returning(|_| Ok(vec!["a".into(), "b".into(), "c".into()].clone()));

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.next("c", false);

        // Then
        assert_eq!(result.as_deref(), None);
    }

    #[test]
    fn should_continue_into_unknown_sessions_with_all() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_sessions()
            .returning(|_| Ok(vec!["z".into(), "a".into(), "b".into(), "C".into()].clone()));

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["b".into(), "z".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // Then
        assert_eq!(recent.next("z", false).as_deref(), None);
        assert_eq!(recent.next("z", true).as_deref(), Some("a"));
        assert_eq!(recent.next("a", true).as_deref(), Some("C"));
        assert_eq!(recent.previous("a", true).as_deref(), Some("z"));
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
        tmux.expect_list_sessions()
            .returning(|_| Ok(vec!["a".into(), "b".into(), "c".into()].clone()));

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.previous("c", false);

        // Then
        assert_eq!(result.as_deref(), Some("b"));
    }

    #[test]
    fn should_return_the_previous_available_session() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_sessions()
            .returning(|_| Ok(vec!["a".into(), "c".into()].clone()));

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.previous("c", false);

        // Then
        assert_eq!(result.as_deref(), Some("a"));
    }

    #[test]
    fn should_return_none_if_the_session_is_the_first_one() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_sessions()
            .returning(|_| Ok(vec!["a".into(), "b".into(), "c".into()].clone()));

        let mut recent_session_file = MockSessionNameFile::new();
        recent_session_file
            .expect_read()
            .returning(|| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&tmux, &recent_session_file);

        // When
        let result = recent.previous("a", false);

        // Then
        assert_eq!(result.as_deref(), None);
    }
}
