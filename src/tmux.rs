use std::collections::HashMap;
use std::process::Command;
use std::process::Stdio;

use mockall::automock;

use crate::model::SessionName;
use crate::model::TmuxSessions;
use crate::model::TmuxWindow;
use crate::utils;

#[automock]
pub(crate) trait Tmux {
    fn list_session_names(&self) -> Vec<SessionName>;
    fn list_sessions(&self) -> TmuxSessions;
    fn list_windows(&self, session_name: &str) -> Vec<TmuxWindow>;
    fn new_session(&self, session_name: &str, tmux_window: &TmuxWindow);
    fn new_window(&self, session_name: &str, tmux_window: &TmuxWindow, i: usize);
    fn has_session(&self, session_name: &str) -> bool;
    fn select_window(&self, session_name: &str, index: usize);
}

pub(crate) struct TmuxImpl;

impl Tmux for TmuxImpl {
    fn list_session_names(&self) -> Vec<SessionName> {
        let output = Command::new("tmux")
            .arg("list-sessions")
            .output()
            .expect("Failed to list tmux sessions.");

        let sessions_output = String::from_utf8_lossy(&output.stdout);
        let mut session_names = Vec::new();
        for session in sessions_output.lines() {
            if let Some((name, _)) = session
                .split_once(':')
                .filter(|(name, _)| !utils::is_numeric(name))
            {
                session_names.push(name.to_string());
            }
        }

        session_names
    }

    fn list_sessions(&self) -> TmuxSessions {
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
                let windows = self.list_windows(name);
                sessions.insert(name.to_string(), windows);
            }
        }

        sessions
    }

    fn list_windows(&self, session_name: &str) -> Vec<TmuxWindow> {
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

    fn new_session(&self, session_name: &str, tmux_window: &TmuxWindow) {
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

    fn new_window(&self, session_name: &str, tmux_window: &TmuxWindow, i: usize) {
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

    fn has_session(&self, session_name: &str) -> bool {
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

    fn select_window(&self, session_name: &str, index: usize) {
        Command::new("tmux")
            .arg("select-window")
            .arg("-t")
            .arg(format!("{}:{}", session_name, index))
            .status()
            .expect("Failed to select window.");
    }
}

// pub(crate) fn get_current_session_name() -> String {
//     Command::new("tmux")
//         .arg("display-message")
//         .arg("-p")
//         .arg("#S")
//         .output()
//         .expect("Failed to get current session name.")
//         .stdout
//         .first()
//         .map(|s| s.to_string())
//         .unwrap_or("".to_string())
//         .to_string()
// }
