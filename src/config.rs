use dirs_next::home_dir;
use mockall::automock;
use std::{fs, path::PathBuf};

const CONFIG_LOCATION: &str = ".config/stmux";
const SESSIONS_FILENAME: &str = "sessions.toml";
const RECENT_SESSIONS_FILENAME: &str = "recent_sessions";
const BOOKMARKS_FILENAME: &str = "bookmarks";

#[automock]
pub(crate) trait Config {
    fn create_dir(&self);
    fn sessions_filename(&self) -> String;
    fn recent_sessions_filename(&self) -> String;
    fn bookmarks_filename(&self) -> String;
}

pub(crate) struct ConfigImpl;

impl ConfigImpl {
    fn config_location() -> PathBuf {
        let mut config_dir = home_dir().expect("Failed to get home directory.");
        config_dir.push(CONFIG_LOCATION);
        config_dir
    }

    fn filename_at_config(filename: &str) -> String {
        let mut result = ConfigImpl::config_location();
        result.push(filename);
        result
            .to_str()
            .expect("Failed to convert to string.")
            .to_string()
    }
}

impl Config for ConfigImpl {
    fn create_dir(&self) {
        let config_dir = ConfigImpl::config_location();

        if !config_dir.is_dir() {
            fs::create_dir_all(&config_dir).expect("Failed to create config directory.");
        }
    }

    fn sessions_filename(&self) -> String {
        ConfigImpl::filename_at_config(SESSIONS_FILENAME)
    }

    fn recent_sessions_filename(&self) -> String {
        ConfigImpl::filename_at_config(RECENT_SESSIONS_FILENAME)
    }

    fn bookmarks_filename(&self) -> String {
        ConfigImpl::filename_at_config(BOOKMARKS_FILENAME)
    }
}
