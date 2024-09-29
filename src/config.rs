use dirs_next::home_dir;
use std::fs;

const CONFIG_LOCATION: &str = ".config/stmux";
const DEFAULT_FILENAME: &str = "sessions.toml";

pub(crate) fn create_dir() {
    let mut config_dir = home_dir().expect("Failed to get home directory.");
    config_dir.push(CONFIG_LOCATION);

    fs::create_dir_all(&config_dir).expect("Failed to create config directory.");
}

pub(crate) fn default_sessions_filename() -> String {
    let mut filename = home_dir().expect("Failed to get home directory.");
    filename.push(CONFIG_LOCATION);
    filename.push(DEFAULT_FILENAME);
    filename
        .to_str()
        .expect("Failed to convert to string.")
        .to_string()
}
