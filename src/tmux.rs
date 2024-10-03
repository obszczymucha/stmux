use std::collections::HashMap;
use std::process::Command;
use std::process::Stdio;

use mockall::automock;

use crate::model::SessionName;
use crate::model::TmuxPane;
use crate::model::TmuxSessions;
use crate::model::TmuxWindow;
use crate::model::WindowName;
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
    fn current_session_name(&self) -> String;
    fn select_session(&self, session_name: &str);
    fn display_message(&self, message: &str);
    fn display_popup(
        &self,
        title: &str,
        border_color: &str,
        width: usize,
        height: usize,
        command: &str,
    );
    fn split_window(&self, session_name: &str, window_name: &str, path: &str);
    fn select_layout(&self, session_name: &str, window_name: &str, layout: &str);
}

pub(crate) struct TmuxImpl;

impl TmuxImpl {
    fn center_title(&self, title: &str, popup_width: usize) -> String {
        let title_width = title.len() + 2;

        if title_width >= popup_width - 3 {
            return format!(" {} ", &title[..(popup_width - 6)]);
        }

        let left_padding = (popup_width - 2 - title_width) / 2 - 1;
        let c = "\u{2500}";

        format!("{} {} ", c.repeat(left_padding), title)
    }
}

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
            .arg("-F")
            .arg("#{session_name}")
            .output()
            .expect("Failed to list tmux sessions.");

        let sessions_output = String::from_utf8_lossy(&output.stdout);
        let mut sessions = HashMap::new();

        for name in sessions_output.lines().filter(|s| !utils::is_numeric(s)) {
            let windows = self.list_windows(name);
            sessions.insert(name.to_string(), windows);
        }

        sessions
    }

    fn list_windows(&self, session_name: &str) -> Vec<TmuxWindow> {
        let output = Command::new("tmux")
            .arg("list-panes")
            .arg("-s")
            .arg("-t")
            .arg(session_name)
            .arg("-F")
            .arg("#{window_name}:#{window_layout}:#{pane_active}:#{pane_current_path}")
            .output()
            .expect("Failed to list tmux windows.");

        let windows_output = String::from_utf8_lossy(&output.stdout);
        let mut map: HashMap<WindowName, TmuxWindow> = HashMap::new();

        for window in windows_output.lines() {
            let tokens = window.split(':').collect::<Vec<&str>>();
            let name = tokens[0];
            let layout = tokens[1];
            let active = tokens[2] == "1";
            let path = tokens[3].to_string();
            let pane = TmuxPane { path, active };

            if let Some(window) = map.get_mut(name) {
                window.panes.push(pane);
            } else {
                let window = TmuxWindow {
                    name: name.to_string(),
                    layout: layout.to_string(),
                    panes: vec![pane],
                };

                map.insert(name.to_string(), window);
            }
        }

        map.into_values().collect()
    }

    fn new_session(&self, session_name: &str, tmux_window: &TmuxWindow) {
        let name = tmux_window.name.as_str();
        let panes = &tmux_window.panes;

        eprintln!(
            "Creating new session '{}' with window '{}'.",
            session_name, name
        );

        let mut command = Command::new("tmux");
        command
            .arg("new-session")
            .arg("-d")
            .arg("-s")
            .arg(session_name)
            .arg("-n")
            .arg(name)
            .arg("-e")
            .arg("NO_CD=1");

        if panes.len() > 1 {
            command.arg("-c").arg(panes[0].path.as_str());
        }

        command.status().expect("Failed to create new session.");
    }

    fn new_window(&self, session_name: &str, tmux_window: &TmuxWindow, i: usize) {
        let name = tmux_window.name.as_str();
        let panes = &tmux_window.panes;

        eprintln!(
            "Creating new window '{}' in session '{}'.",
            name, session_name
        );

        let mut command = Command::new("tmux");
        command
            .arg("new-window")
            .arg("-t")
            .arg(format!("{}:{}", session_name, i + 1))
            .arg("-n")
            .arg(name)
            .arg("-e")
            .arg("NO_CD=1");

        if panes.len() > 1 {
            command.arg("-c").arg(panes[0].path.as_str());
        }

        command.status().expect("Failed to create new session.");
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

    fn current_session_name(&self) -> String {
        let stdout = Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#S")
            .output()
            .expect("Failed to get current session name.")
            .stdout;

        String::from_utf8_lossy(&stdout).trim().to_string()
    }

    fn select_session(&self, session_name: &str) {
        Command::new("tmux")
            .arg("switch-client")
            .arg("-t")
            .arg(session_name)
            .status()
            .expect("Failed to select session.");
    }

    fn display_message(&self, message: &str) {
        Command::new("tmux")
            .arg("display-message")
            .arg(message)
            .status()
            .expect("Failed to display message.");
    }

    fn display_popup(&self, title: &str, style: &str, width: usize, height: usize, command: &str) {
        Command::new("tmux")
            .arg("display-popup")
            .arg("-E")
            .arg("-T")
            .arg(self.center_title(title, width))
            .arg("-S")
            .arg(style)
            .arg("-w")
            .arg(width.to_string())
            .arg("-h")
            .arg(height.to_string())
            .arg(command)
            .status()
            .expect("Failed to display popup.");
    }

    fn split_window(&self, session_name: &str, window_name: &str, path: &str) {
        eprintln!(
            "Splitting window '{}' in session '{}'.",
            window_name, session_name
        );
        Command::new("tmux")
            .arg("split-window")
            .arg("-t")
            .arg(format!("{}:{}", session_name, window_name))
            .arg("-e")
            .arg("NO_CD=1")
            .arg("-c")
            .arg(path)
            .status()
            .expect("Failed to split a window.");
    }

    fn select_layout(&self, session_name: &str, window_name: &str, layout: &str) {
        eprintln!(
            "Selecting layout '{}' for window '{}' in session '{}'.",
            layout, window_name, session_name
        );
        Command::new("tmux")
            .arg("select-layout")
            .arg("-t")
            .arg(format!("{}:{}", session_name, window_name))
            .arg(layout)
            .status()
            .expect("Failed to select window layout.");
    }
}
