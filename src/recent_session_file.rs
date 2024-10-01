use std::io::BufRead;

use mockall::automock;

#[automock]
pub(crate) trait RecentSessionFile {
    fn read_session_names_from_file(&self, filename: &str) -> Vec<String>;
}

pub(crate) struct RecentSessionFileImpl;

impl RecentSessionFile for RecentSessionFileImpl {
    fn read_session_names_from_file(&self, filename: &str) -> Vec<String> {
        let file = std::fs::File::open(filename).expect("Failed to open file");
        let reader = std::io::BufReader::new(file);
        let lines = reader.lines();
        let session_names: Vec<String> = lines.map(|line| line.unwrap()).collect();
        session_names
    }
}
