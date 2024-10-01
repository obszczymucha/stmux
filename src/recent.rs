use crate::{config::Config, recent_session_file::RecentSessionFile, tmux::Tmux};

pub(crate) trait Recent {
    fn next(&self, session_name: &str) -> Option<String>;
    fn previous(&self);
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
        let recent_session_names: &Vec<String> = &self
            .recent_session_file
            .read_session_names_from_file(filename);

        let names: Vec<String> = recent_session_names
            .iter()
            .filter(|name| session_names.contains(name))
            .cloned()
            .collect();

        if !names.iter().any(|name| name == session_name) {
            return None;
        }

        names
            .iter()
            .cycle()
            .skip_while(|name| name != &session_name)
            .nth(1)
            .cloned()
    }

    fn previous(&self) {
        eprintln!("Not implemented yet: recent::previous()");
    }
}

// mod tests {
//     use super::*;
//     #[test]
//     fn test_next() {
//         let session_name = "session1";
//         let session_names = vec!["session1", "session2", "session3"];
//         let recent_session_names = vec!["session2", "session1", "session3"];
//         let expected = Some("session3".to_string());
//         let actual = next(session_name, session_names, recent_session_names);
//         assert_eq!(expected, actual);
//     }
// }
