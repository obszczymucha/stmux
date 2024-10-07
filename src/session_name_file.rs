use std::io::BufRead;
use std::io::Write;
use std::path::Path;

use mockall::automock;

#[automock]
pub(crate) trait SessionNameFile {
    fn read(&self) -> Vec<String>;
    fn read_into(&self, vec: &mut Vec<String>, skip: &str);
    fn append(&self, session_name: &str);
    fn write(&self, session_names: &[String]);
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

    fn read_into(&self, vec: &mut Vec<String>, skip: &str) {
        let path = Path::new(&self.filename);

        if !path.exists() {
            return;
        }

        let file = std::fs::File::open(path).expect("Failed to open session name file.");
        let reader = std::io::BufReader::new(file);
        let lines = reader.lines();

        let result: Vec<String> = lines
            .map_while(Result::ok)
            .filter(|line| !line.trim().is_empty())
            .filter(|line| line != skip)
            .collect();

        for name in result {
            vec.push(name);
        }
    }

    fn append(&self, session_name: &str) {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.filename.clone())
            .expect("Failed to open bookmarks file.");

        writeln!(file, "{}", session_name).expect("Failed to write to session name file.");
    }

    fn write(&self, session_names: &[String]) {
        let mut file = std::fs::File::create(self.filename.clone())
            .expect("Failed to create session name file.");

        for session_name in session_names {
            writeln!(file, "{}", session_name).expect("Failed to write to session name file.");
        }
    }
}
