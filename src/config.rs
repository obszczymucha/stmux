use dirs_next::home_dir;
use mockall::automock;
use std::fs;

const CONFIG_LOCATION: &str = ".config/stmux";
const DEFAULT_FILENAME: &str = "sessions.toml";
const RECENT_SESSIONS_FILENAME: &str = ".tmux_recent";

#[automock]
pub(crate) trait Config {
    fn create_dir(&self);
    fn default_sessions_filename(&self) -> String;
    fn recent_sessions_filename(&self) -> String;
}

pub(crate) struct ConfigImpl;

impl Config for ConfigImpl {
    fn create_dir(&self) {
        let mut config_dir = home_dir().expect("Failed to get home directory.");
        config_dir.push(CONFIG_LOCATION);

        if !config_dir.is_dir() {
            fs::create_dir_all(&config_dir).expect("Failed to create config directory.");
        }
    }

    fn default_sessions_filename(&self) -> String {
        let mut filename = home_dir().expect("Failed to get home directory.");
        filename.push(CONFIG_LOCATION);
        filename.push(DEFAULT_FILENAME);
        filename
            .to_str()
            .expect("Failed to convert to string.")
            .to_string()
    }

    fn recent_sessions_filename(&self) -> String {
        let mut filename = home_dir().expect("Failed to get home directory.");
        filename.push(RECENT_SESSIONS_FILENAME);
        filename
            .to_str()
            .expect("Failed to convert to string.")
            .to_string()
    }
}
