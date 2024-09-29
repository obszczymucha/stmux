use std::collections::HashMap;
use std::process::Command;
use std::process::Stdio;

use crate::model::TmuxSessions;
use crate::model::TmuxWindow;
use crate::utils;

pub(crate) fn list_sessions() -> TmuxSessions {
    let output = Command::new("tmux")
        .arg("list-sessions")
        .output()
        .expect("Failed to list tmux sessions.");

    let sessions_output = String::from_utf8_lossy(&output.stdout);
    let mut sessions = HashMap::new();

    for session in sessions_output.lines() {
        if let Some((name, _)) = session
            .split_once(':')
            .filter(|(name, _)| !utils::is_numeric(name))
        {
            let windows = list_windows(name);
            sessions.insert(name.to_string(), windows);
        }
    }

    sessions
}

pub(crate) fn list_windows(session_name: &str) -> Vec<TmuxWindow> {
    let output = Command::new("tmux")
        .arg("list-windows")
        .arg("-t")
        .arg(session_name)
        .arg("-F")
        .arg("#{window_name} #{pane_current_path}")
        .output()
        .expect("Failed to list tmux windows.");

    let windows_output = String::from_utf8_lossy(&output.stdout);
    let mut windows = Vec::new();

    for window in windows_output.lines() {
        let parts: Vec<&str> = window.split_whitespace().collect();
        if parts.len() == 2 {
            let window_name = parts[0].to_string();
            let path = parts[1].to_string();
            windows.push(TmuxWindow {
                name: window_name,
                path,
            });
        }
    }

    windows
}

pub(crate) fn new_session(session_name: &str, tmux_window: &TmuxWindow) {
    let name = tmux_window.name.as_str();
    let path = tmux_window.path.as_str();

    // eprintln!(
    //     "Creating new session '{}' with window '{}' ({}).",
    //     session_name, name, path
    // );

    Command::new("tmux")
        .arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(session_name)
        .arg("-n")
        .arg(name)
        .arg("-c")
        .arg(path)
        .arg("NO_CD=1 zsh")
        .status()
        .expect("Failed to create new session.");
}

pub(crate) fn new_window(session_name: &str, tmux_window: &TmuxWindow, i: usize) {
    let name = tmux_window.name.as_str();
    let path = tmux_window.path.as_str();

    // eprintln!(
    //     "Creating new window '{}' ({}) in session '{}'.",
    //     name, path, session_name
    // );

    Command::new("tmux")
        .arg("new-window")
        .arg("-t")
        .arg(format!("{}:{}", session_name, i + 1))
        .arg("-n")
        .arg(name)
        .arg("-c")
        .arg(path)
        .arg("NO_CD=1 zsh")
        .status()
        .expect("Failed to create new session.");
}

pub(crate) fn has_session(session_name: &str) -> bool {
    let output = Command::new("tmux")
        .arg("has-session")
        .arg("-t")
        .arg(session_name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to check if session exists.");

    output.success()
}

pub(crate) fn select_window(session_name: &str, index: usize) {
    Command::new("tmux")
        .arg("select-window")
        .arg("-t")
        .arg(format!("{}:{}", session_name, index))
        .status()
        .expect("Failed to select window.");
}
