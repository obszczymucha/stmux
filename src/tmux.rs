use std::collections::HashMap;
use std::process::Command;
use std::process::Stdio;

use mockall::automock;

use crate::model::SessionName;
use crate::model::TmuxPane;
use crate::model::TmuxSession;
use crate::model::TmuxSessions;
use crate::model::TmuxWindow;
use crate::model::WindowDimension;
use crate::model::WindowName;
use crate::model::WindowNameAndStatus;
use crate::utils;

#[automock]
pub(crate) trait Tmux {
    fn list_session_names(&self) -> Vec<SessionName>;
    fn list_sessions(&self) -> TmuxSessions;
    fn list_windows(&self, session_name: &str) -> Vec<TmuxWindow>;
    fn list_windows_names_with_status(&self) -> Vec<WindowNameAndStatus>;
    fn new_session(
        &self,
        session_name: &str,
        tmux_window: &TmuxWindow,
        startup_command: &Option<String>,
    );
    fn new_window(
        &self,
        session_name: &str,
        tmux_window: &TmuxWindow,
        i: usize,
        startup_command: &Option<String>,
    );
    fn has_session(&self, session_name: &str) -> bool;
    fn select_window(&self, session_name: &str, index: usize);
    fn current_session_name(&self) -> String;
    fn select_session(&self, session_name: &str);
    fn display_message(&self, message: &str);
    fn display_popup(
        &self,
        title: &str,
        title_style: &Option<String>,
        border_color: &str,
        dimension: &WindowDimension,
        y: &Option<usize>,
        command: &str,
    );
    fn split_window(
        &self,
        session_name: &str,
        window_name: &str,
        path: &str,
        startup_command: &Option<String>,
    );
    fn select_layout(&self, session_name: &str, window_name: &str, layout: &str);
    fn send_keys(
        &self,
        session_name: &str,
        window_name: &str,
        pane_index: Option<usize>,
        keys: &str,
    );
    fn window_dimension(&self) -> Option<WindowDimension>;
    fn set_global(&self, option_name: &str, value: &str);
    fn current_window_index(&self) -> usize;
    fn current_pane_index(&self) -> usize;
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
            .arg("-F")
            .arg("#{session_name}")
            .output()
            .expect("Failed to list tmux sessions.");

        let sessions_output = String::from_utf8_lossy(&output.stdout);
        let mut sessions = HashMap::new();

        for name in sessions_output.lines().filter(|s| !utils::is_numeric(s)) {
            let windows = self.list_windows(name);
            let session = TmuxSession {
                background: None,
                no_recent_tracking: None,
                windows,
            };

            sessions.insert(name.to_string(), session);
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
            .arg("#{window_index}:#{window_name}:#{window_layout}:#{pane_index}:#{pane_active}:#{pane_current_path}")
            .output()
            .expect("Failed to list tmux windows.");

        let windows_output = String::from_utf8_lossy(&output.stdout);
        let mut windows: Vec<TmuxWindow> = Vec::new();
        let mut map: HashMap<WindowName, usize> = HashMap::new();
        let mut index: usize = 0;

        for window in windows_output.lines() {
            let tokens = window.split(':').collect::<Vec<&str>>();
            let window_index = tokens[0].parse::<usize>().unwrap();
            let window_name = tokens[1];
            let layout = tokens[2];
            let pindex = tokens[3].parse::<usize>().unwrap();
            let active = tokens[4] == "1";
            let path = tokens[5].to_string();
            let pane = TmuxPane {
                index: pindex,
                path,
                active,
                startup_command: None,
                shell_command: None,
                environment: None,
            };

            if let Some(i) = map.get(window_name) {
                let window = &mut windows[*i];
                window.panes.push(pane);
            } else {
                let window = TmuxWindow {
                    index: window_index,
                    name: window_name.to_string(),
                    layout: layout.to_string(),
                    panes: vec![pane],
                    active: None,
                };

                windows.push(window);
                map.insert(window_name.to_string(), index);
                index += 1;
            }
        }

        windows
    }

    fn new_session(
        &self,
        session_name: &str,
        tmux_window: &TmuxWindow,
        startup_command: &Option<String>,
    ) {
        let name = tmux_window.name.as_str();
        let panes = &tmux_window.panes;

        // eprintln!(
        //     "Creating new session '{}' with window '{}'.",
        //     session_name, name
        // );

        let mut command = Command::new("tmux");
        command
            .arg("new-session")
            .arg("-d")
            .arg("-s")
            .arg(session_name)
            .arg("-n")
            .arg(name)
            .arg("-e") // TODO: Add global config first and add global env variables from there.
            .arg("NO_CD=1");

        if !panes.is_empty() {
            let pane = &panes[0];

            if let Some(environment) = &pane.environment {
                for env in environment.iter() {
                    command.arg("-e").arg(format!("{}={}", env.name, env.value));
                }
            }

            command.arg("-c").arg(pane.path.as_str());
        }

        if let Some(program) = startup_command {
            command.arg(program);
        }

        command.status().expect("Failed to create new session.");
    }

    fn new_window(
        &self,
        session_name: &str,
        tmux_window: &TmuxWindow,
        i: usize,
        startup_command: &Option<String>,
    ) {
        let name = tmux_window.name.as_str();
        let panes = &tmux_window.panes;

        // eprintln!(
        //     "Creating new window '{}' in session '{}'.",
        //     name, session_name
        // );

        let mut command = Command::new("tmux");
        command
            .arg("new-window")
            .arg("-t")
            .arg(format!("{}:{}", session_name, i + 1))
            .arg("-n")
            .arg(name)
            .arg("-e")
            .arg("NO_CD=1");

        if !panes.is_empty() {
            let pane = &panes[0];

            if let Some(environment) = &pane.environment {
                for env in environment.iter() {
                    command.arg("-e").arg(format!("{}={}", env.name, env.value));
                }
            }

            command.arg("-c").arg(pane.path.as_str());
        }

        if let Some(program) = startup_command {
            command.arg(program);
        }

        command.status().expect("Failed to create new session.");
    }

    fn has_session(&self, session_name: &str) -> bool {
        let output = Command::new("tmux")
            .arg("has-session")
            .arg("-t")
            .arg(format!("={}", session_name))
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

    fn display_popup(
        &self,
        title: &str,
        title_style: &Option<String>,
        style: &str,
        dimension: &WindowDimension,
        y: &Option<usize>,
        command: &str,
    ) {
        let mut cmd = Command::new("tmux");
        cmd.arg("display-popup")
            .arg("-E")
            .arg("-b")
            .arg("rounded")
            .arg("-T")
            .arg(
                title_style
                    .as_ref()
                    .map(|s| format!("#[{}] {} ", s, title))
                    .unwrap_or(title.to_string()),
            )
            .arg("-S")
            .arg(style)
            .arg("-w")
            .arg(dimension.width.to_string())
            .arg("-h")
            .arg(dimension.height.to_string());

        if let Some(y) = y {
            cmd.arg("-y").arg(y.to_string());
        }

        cmd.arg(command).status().expect("Failed to display popup.");
    }

    fn split_window(
        &self,
        session_name: &str,
        window_name: &str,
        path: &str,
        startup_command: &Option<String>,
    ) {
        // eprintln!(
        //     "Splitting window '{}' in session '{}'.",
        //     window_name, session_name
        // );
        let mut command = Command::new("tmux");
        command
            .arg("split-window")
            .arg("-t")
            .arg(format!("{}:{}", session_name, window_name))
            .arg("-e")
            .arg("NO_CD=1")
            .arg("-c")
            .arg(path);

        if let Some(program) = startup_command {
            command.arg(program);
        }

        command.status().expect("Failed to split a window.");
    }

    fn select_layout(&self, session_name: &str, window_name: &str, layout: &str) {
        // eprintln!(
        //     "Selecting layout '{}' for window '{}' in session '{}'.",
        //     layout, window_name, session_name
        // );
        Command::new("tmux")
            .arg("select-layout")
            .arg("-t")
            .arg(format!("{}:{}", session_name, window_name))
            .arg(layout)
            .status()
            .expect("Failed to select window layout.");
    }

    fn send_keys(
        &self,
        session_name: &str,
        window_name: &str,
        pane_index: Option<usize>,
        keys: &str,
    ) {
        Command::new("tmux")
            .arg("send-keys")
            .arg("-t")
            .arg(format!(
                "{}:{}{}",
                session_name,
                window_name,
                pane_index
                    .map(|i| format!(".{}", i))
                    .unwrap_or("".to_string())
            ))
            .arg(keys)
            .arg("C-m")
            .status()
            .expect("Failed to send keys.");
    }

    fn window_dimension(&self) -> Option<WindowDimension> {
        let output = Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#{window_width}x#{window_height}")
            .output()
            .expect("Failed to get window dimension.");

        let dimension_str = String::from_utf8_lossy(&output.stdout);
        dimension_str
            .trim()
            .split_once('x')
            .into_iter()
            .flat_map(|(width, height)| {
                let width = width.parse::<usize>().ok()?;
                let height = height.parse::<usize>().ok()?;
                Some(WindowDimension { width, height })
            })
            .next()
    }

    fn set_global(&self, option_name: &str, value: &str) {
        Command::new("tmux")
            .arg("set")
            .arg("-g")
            .arg(option_name)
            .arg(value)
            .status()
            .expect("Failed to set global option.");
    }

    fn list_windows_names_with_status(&self) -> Vec<WindowNameAndStatus> {
        let output = Command::new("tmux")
            .arg("list-windows")
            .arg("-F")
            .arg("#{window_name}:#{window_active}")
            .output()
            .expect("Failed to list tmux windows.");

        let windows_output = String::from_utf8_lossy(&output.stdout);

        windows_output
            .lines()
            .map(|line| {
                let mut parts = line.split(':');
                let name = parts.next().unwrap().to_string();
                let active = parts.next().unwrap() == "1";
                WindowNameAndStatus { name, active }
            })
            .collect()
    }

    fn current_window_index(&self) -> usize {
        let output = Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#I")
            .output()
            .expect("Failed to get current window index.");

        let id = String::from_utf8_lossy(&output.stdout);
        id.trim().parse().expect("Failed to parse window index.")
    }

    fn current_pane_index(&self) -> usize {
        let output = Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#P")
            .output()
            .expect("Failed to get current pane index.");

        let id = String::from_utf8_lossy(&output.stdout);
        id.trim().parse().expect("Failed to parse pane index.")
    }
}
