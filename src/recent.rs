use crate::{config::Config, recent_session_file::RecentSessionFile, tmux::Tmux};

pub(crate) trait Recent {
    fn next(&self, session_name: &str) -> Option<String>;
    fn previous(&self, session_name: &str) -> Option<String>;
}

pub(crate) struct RecentImpl<'a, 'b, 'c, C: Config, T: Tmux, R: RecentSessionFile> {
    config: &'a C,
    tmux: &'b T,
    recent_session_file: &'c R,
}

impl<'a, 'b, 'c, C: Config, T: Tmux, R: RecentSessionFile> RecentImpl<'a, 'b, 'c, C, T, R> {
    pub(crate) fn new(config: &'a C, tmux: &'b T, recent_session_file: &'c R) -> Self {
        Self {
            config,
            tmux,
            recent_session_file,
        }
    }
}

impl<'a, 'b, 'c, C: Config, T: Tmux, R: RecentSessionFile> Recent
    for RecentImpl<'a, 'b, 'c, C, T, R>
{
    fn next(&self, session_name: &str) -> Option<String> {
        let session_names = self.tmux.list_session_names();
        let filename = &self.config.recent_sessions_filename();
        let recent_session_names = &self
            .recent_session_file
            .read_session_names_from_file(filename);

        recent_session_names
            .iter()
            .filter(|name| session_names.contains(name))
            .skip_while(|&name| name != session_name)
            .nth(1)
            .cloned()
    }

    fn previous(&self, session_name: &str) -> Option<String> {
        let session_names = self.tmux.list_session_names();
        let filename = &self.config.recent_sessions_filename();
        let recent_session_names = &self
            .recent_session_file
            .read_session_names_from_file(filename);

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
}

#[cfg(test)]
mod next_tests {
    use super::*;
    use crate::{config, recent_session_file, tmux};
    use config::MockConfig;
    use mockall::predicate::*;
    use recent_session_file::MockRecentSessionFile;
    use tmux::MockTmux;

    #[test]
    fn should_return_the_next_session() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_session_names()
            .returning(|| vec!["a".into(), "b".into()].clone());

        let mut config = MockConfig::new();
        config
            .expect_recent_sessions_filename()
            .returning(|| ".tmux_recent".into());

        let mut recent_session_file = MockRecentSessionFile::new();
        recent_session_file
            .expect_read_session_names_from_file()
            .with(eq(".tmux_recent"))
            .returning(|_| vec!["a".into(), "b".into()]);

        let recent = RecentImpl::new(&config, &tmux, &recent_session_file);

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

        let mut config = MockConfig::new();
        config
            .expect_recent_sessions_filename()
            .returning(|| ".tmux_recent".into());

        let mut recent_session_file = MockRecentSessionFile::new();
        recent_session_file
            .expect_read_session_names_from_file()
            .with(eq(".tmux_recent"))
            .returning(|_| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&config, &tmux, &recent_session_file);

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

        let mut config = MockConfig::new();
        config
            .expect_recent_sessions_filename()
            .returning(|| ".tmux_recent".into());

        let mut recent_session_file = MockRecentSessionFile::new();
        recent_session_file
            .expect_read_session_names_from_file()
            .with(eq(".tmux_recent"))
            .returning(|_| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&config, &tmux, &recent_session_file);

        // When
        let result = recent.next("c");

        // Then
        assert_eq!(result.as_deref(), None);
    }
}

#[cfg(test)]
mod previous_tests {
    use super::*;
    use crate::{config, recent_session_file, tmux};
    use config::MockConfig;
    use mockall::predicate::*;
    use recent_session_file::MockRecentSessionFile;
    use tmux::MockTmux;

    #[test]
    fn should_return_the_previous_session() {
        // Given
        let mut tmux = MockTmux::new();
        tmux.expect_list_session_names()
            .returning(|| vec!["a".into(), "b".into(), "c".into()].clone());

        let mut config = MockConfig::new();
        config
            .expect_recent_sessions_filename()
            .returning(|| ".tmux_recent".into());

        let mut recent_session_file = MockRecentSessionFile::new();
        recent_session_file
            .expect_read_session_names_from_file()
            .with(eq(".tmux_recent"))
            .returning(|_| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&config, &tmux, &recent_session_file);

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

        let mut config = MockConfig::new();
        config
            .expect_recent_sessions_filename()
            .returning(|| ".tmux_recent".into());

        let mut recent_session_file = MockRecentSessionFile::new();
        recent_session_file
            .expect_read_session_names_from_file()
            .with(eq(".tmux_recent"))
            .returning(|_| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&config, &tmux, &recent_session_file);

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

        let mut config = MockConfig::new();
        config
            .expect_recent_sessions_filename()
            .returning(|| ".tmux_recent".into());

        let mut recent_session_file = MockRecentSessionFile::new();
        recent_session_file
            .expect_read_session_names_from_file()
            .with(eq(".tmux_recent"))
            .returning(|_| vec!["a".into(), "b".into(), "c".into()]);

        let recent = RecentImpl::new(&config, &tmux, &recent_session_file);

        // When
        let result = recent.previous("a");

        // Then
        assert_eq!(result.as_deref(), None);
    }
}
