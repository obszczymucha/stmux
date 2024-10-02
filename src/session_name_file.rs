use std::io::BufRead;
use std::io::Write;
use std::path::Path;

use mockall::automock;

#[automock]
pub(crate) trait SessionNameFile {
    fn read(&self) -> Vec<String>;
    fn append(&self, session_name: &str);
}

pub(crate) struct SessionNameFileImpl {
    filename: String,
}

impl SessionNameFileImpl {
    pub(crate) fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }
}

impl SessionNameFile for SessionNameFileImpl {
    fn read(&self) -> Vec<String> {
        let path = Path::new(&self.filename);

        if !path.exists() {
            return Vec::new();
        }

        let file = std::fs::File::open(path).expect("Failed to open session name file.");
        let reader = std::io::BufReader::new(file);
        let lines = reader.lines();

        lines
            .map_while(Result::ok)
            .filter(|line| !line.trim().is_empty())
            .collect()
    }

    fn append(&self, session_name: &str) {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.filename.clone())
            .expect("Failed to open bookmarks file.");

        writeln!(file, "{}", session_name).expect("Failed to write to session name file.");
    }
}
