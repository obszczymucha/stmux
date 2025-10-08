use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct StatusConfig {
    pub(crate) colors: Colors,
    pub(crate) style: Style,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Colors {
    pub(crate) inactive: InactiveColors,
    pub(crate) active: ActiveColors,
    pub(crate) selected: SelectedColors,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct InactiveColors {
    pub(crate) session_number: String,
    pub(crate) number_separator: String,
    pub(crate) session_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct ActiveColors {
    pub(crate) session_number: String,
    pub(crate) number_separator: String,
    pub(crate) session_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct SelectedColors {
    pub(crate) session_number: String,
    pub(crate) number_separator: String,
    pub(crate) session_name: String,
    pub(crate) window_before: String,
    pub(crate) active_pane: String,
    pub(crate) pane_separator: String,
    pub(crate) inactive_pane: String,
    pub(crate) window_after: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Style {
    pub(crate) number_separator: String,
    pub(crate) window_before: String,
    pub(crate) pane_separator: String,
    pub(crate) window_after: String,
    pub(crate) bookmark_separator: String,
    pub(crate) selected_bookmark_separator: String,
}

pub(crate) trait StatusConfigFile {
    fn load(&self) -> StatusConfig;
}

pub(crate) struct StatusConfigFileImpl {
    filename: String,
}

impl StatusConfigFileImpl {
    pub(crate) fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }
}

impl StatusConfigFile for StatusConfigFileImpl {
    fn load(&self) -> StatusConfig {
        let file_content = fs::read_to_string(&self.filename);

        match file_content {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|error| {
                panic!("Failed to parse {}: {}.", &self.filename, error.message())
            }),
            Err(e) => panic!("Failed to read status config file: {}", e),
        }
    }
}
