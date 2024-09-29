mod args;
mod model;
mod tmux;
mod utils;
use args::Action;
use args::Args;
use clap::Parser;
use dirs_next::home_dir;
use model::TmuxSession;
use model::TmuxWindow;
use std::collections::HashMap;
use std::fs;
use tmux::get_tmux_sessions;
use tmux::has_session;
use tmux::new_session;
use tmux::new_window;
use tmux::select_window;
use utils::is_numeric;

const CONFIG_LOCATION: &str = ".config/stmux";
const DEFAULT_FILENAME: &str = "sessions.toml";

fn create_config_dir() {
    let mut config_dir = home_dir().expect("Failed to get home directory.");
    config_dir.push(CONFIG_LOCATION);

    fs::create_dir_all(&config_dir).expect("Failed to create config directory.");
}

fn get_default_filename() -> String {
    let mut filename = home_dir().expect("Failed to get home directory.");
    filename.push(CONFIG_LOCATION);
    filename.push(DEFAULT_FILENAME);
    filename
        .to_str()
        .expect("Failed to convert to string.")
        .to_string()
}

fn main() {
    let args = Args::parse();
    create_config_dir();

    match args.action {
        Action::Save { filename } => {
            save_sessions(filename.unwrap_or(get_default_filename()).as_str())
        }
        Action::Restore { filename } => {
            restore_sessions(filename.unwrap_or(get_default_filename()).as_str())
        }
    }
}

fn load_sessions_from_config(filename: &str) -> HashMap<String, TmuxSession> {
    let file_content = fs::read_to_string(filename);
    match file_content {
        Ok(content) => {
            toml::from_str(&content).unwrap_or_else(|_| panic!("Failed to parse {}.", filename))
        }
        Err(_) => HashMap::new(),
    }
}

fn merge(
    old_sessions: HashMap<String, TmuxSession>,
    new_sessions: HashMap<String, TmuxSession>,
) -> HashMap<String, TmuxSession> {
    let mut sessions = HashMap::new();

    for (session_name, session) in old_sessions {
        sessions.insert(session_name, session);
    }

    for (session_name, session) in new_sessions {
        if !sessions.contains_key(session_name.as_str()) {
            sessions.insert(session_name, session);
        }
    }

    sessions
}

fn save_sessions(filename: &str) {
    let old_sessions = load_sessions_from_config(filename);
    let new_sessions = get_tmux_sessions();
    let sessions = merge(old_sessions, new_sessions);
    let toml_string = toml::to_string(&sessions).expect("Failed to serialize sessions into TOML.");

    fs::write(filename, toml_string).unwrap_or_else(|_| panic!("Failed to write to {}.", filename));
    eprintln!("TMUX sessions saved to {} file.", filename);
}

fn restore_sessions(filename: &str) {
    eprintln!("Restoring TMUX sessions from {} file...", filename);
    let sessions = load_sessions_from_config(filename);

    for (session_name, session) in sessions {
        let windows = session.windows;
        let non_numeric = !is_numeric(session_name.as_str());

        if !windows.is_empty() && non_numeric && !has_session(session_name.as_str()) {
            process_session(session_name.as_str(), windows);
        }
    }
}

fn process_session(session_name: &str, windows: Vec<TmuxWindow>) {
    if windows.is_empty() {
        return;
    }

    for (i, tmux_window) in windows.into_iter().enumerate() {
        if i == 0 {
            new_session(session_name, &tmux_window)
        } else {
            new_window(session_name, &tmux_window, i)
        }
    }

    select_window(session_name, 1)
}
