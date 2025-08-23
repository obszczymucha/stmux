use std::process::Command;
use std::process::Stdio;

use mockall::automock;

use crate::model::TmuxWindow;
use crate::model::WindowDetails;
use crate::model::WindowDimension;

// Note to myself:
// Should this be dum and just expose tmux cli api?
// Right now there's a mixture of logic and the above.
// Dunno yet. Going towards this being just a cli and
// other structs like Window/Session use this for more
// sophisticated logic.

#[automock]
pub(crate) trait Tmux {
    fn list_sessions(&self, format: &str) -> Vec<String>;
    fn list_session_panes(&self, session_name: &str, format: &str) -> Vec<String>;
    fn list_current_window_panes(&self, format: &str) -> Vec<String>;
    fn list_windows_names_with_status(&self) -> Vec<WindowDetails>;
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
    fn split_current_window(
        &self,
        horizontally: bool,
        path: &str,
        startup_command: &Option<String>,
    );
    fn split_window(
        &self,
        session_name: &str,
        window_name: &str,
        horizontally: bool,
        path: &str,
        startup_command: &Option<String>,
    );
    fn select_layout(&self, session_name: &str, window_name: &str, layout: &str);
    fn send_keys_to_current_window(&self, pane_index: usize, keys: &str);
    fn send_keys(&self, session_name: &str, window_name: &str, pane_index: usize, keys: &str);
    fn window_dimension(&self) -> Option<WindowDimension>;
    fn set_global(&self, option_name: &str, value: &str);
    fn current_window_index(&self) -> usize;
    fn get_pane_option(&self, pane_index: usize, option_name: &str) -> Option<String>;
    fn count_panes(&self) -> usize;
    fn set_pane_option_for_current_window(&self, pane_index: usize, name: &str, value: &str);
    // fn set_pane_option(
    //     &self,
    //     session_name: &str,
    //     window_name: &str,
    //     pane_index: usize,
    //     name: &str,
    //     value: &str,
    // );
}

pub(crate) struct TmuxImpl;

impl TmuxImpl {
    fn split_window<F>(
        &self,
        horizontally: bool,
        path: &str,
        decorator_fn: F,
        startup_command: &Option<String>,
    ) where
        F: FnOnce(&mut Command),
    {
        let mut command = Command::new("tmux");
        command.arg("split-window");

        if horizontally {
            command.arg("-h");
        } else {
            command.arg("-v");
        }

        decorator_fn(&mut command);

        // WTF is this NO_CD=1 doing here?
        command.arg("-e").arg("NO_CD=1").arg("-c").arg(path);

        if let Some(program) = startup_command {
            command.arg(program);
        }

        command.status().expect("Failed to split a window.");
    }

    fn send_keys<F>(&self, keys: &str, decorator_fn: F)
    where
        F: FnOnce(&mut Command),
    {
        let mut command = Command::new("tmux");
        command.arg("send-keys");
        decorator_fn(&mut command);

        command
            .arg(keys)
            .arg("C-m")
            .status()
            .expect("Failed to send keys.");
    }

    fn set_pane_option<F>(&self, name: &str, value: &str, decorator_fn: F)
    where
        F: FnOnce(&mut Command),
    {
        let mut command = Command::new("tmux");
        command.arg("set").arg("-p");

        decorator_fn(&mut command);

        command
            .arg(name)
            .arg(value)
            .status()
            .expect("Failed to get the count of window panes.");
    }
}

impl Tmux for TmuxImpl {
    fn list_sessions(&self, format: &str) -> Vec<String> {
        let output = Command::new("tmux")
            .arg("list-sessions")
            .arg("-F")
            .arg(format)
            .output()
            .expect("Failed to list tmux sessions.");

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    }

    fn list_session_panes(&self, session_name: &str, format: &str) -> Vec<String> {
        let output = Command::new("tmux")
            .arg("list-panes")
            .arg("-s")
            .arg("-t")
            .arg(session_name)
            .arg("-F")
            .arg(format)
            .output()
            .expect("Failed to list tmux windows.");

        let result = String::from_utf8_lossy(&output.stdout);
        result.lines().map(|s| s.to_string()).collect()
    }

    fn list_current_window_panes(&self, format: &str) -> Vec<String> {
        let output = Command::new("tmux")
            .arg("list-panes")
            .arg("-F")
            .arg(format)
            .output()
            .expect("Failed to list current window panes.");

        let result = String::from_utf8_lossy(&output.stdout);

        result.lines().map(|x| x.to_string()).collect()
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

    fn split_current_window(
        &self,
        horizontally: bool,
        path: &str,
        startup_command: &Option<String>,
    ) {
        self.split_window(horizontally, path, |_| {}, startup_command);
    }

    fn split_window(
        &self,
        session_name: &str,
        window_name: &str,
        horizontally: bool,
        path: &str,
        startup_command: &Option<String>,
    ) {
        let decorator = |command: &mut Command| {
            command
                .arg("-t")
                .arg(format!("{}:{}", session_name, window_name));
        };

        self.split_window(horizontally, path, decorator, startup_command);
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

    fn send_keys_to_current_window(&self, pane_index: usize, keys: &str) {
        let decorator = |command: &mut Command| {
            command.arg("-t").arg(format!(".{}", pane_index));
        };

        self.send_keys(keys, decorator);
    }

    fn send_keys(&self, session_name: &str, window_name: &str, pane_index: usize, keys: &str) {
        let decorator = |command: &mut Command| {
            command
                .arg("-t")
                .arg(format!("{}:{}.{}", session_name, window_name, pane_index));
        };

        self.send_keys(keys, decorator);
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

    fn list_windows_names_with_status(&self) -> Vec<WindowDetails> {
        let output = Command::new("tmux")
            .arg("list-windows")
            .arg("-F")
            .arg("#{window_name}:#{window_active}:#{window_panes}")
            .output()
            .expect("Failed to list tmux windows.");

        let windows_output = String::from_utf8_lossy(&output.stdout);

        windows_output
            .lines()
            .map(|line| {
                let mut parts = line.split(':');
                let name = parts.next().unwrap().to_string();
                let active = parts.next().unwrap() == "1";
                let pane_count = parts.next().unwrap().parse::<usize>().ok().unwrap();

                let pane_window_name = if pane_count > 1 {
                    self.get_pane_option(2, "@window-name")
                } else {
                    None
                };

                WindowDetails {
                    name,
                    active,
                    pane_window_name,
                }
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

    fn get_pane_option(&self, pane_index: usize, option_name: &str) -> Option<String> {
        let window_name = Command::new("tmux")
            .arg("show-option")
            .arg("-t")
            .arg(pane_index.to_string())
            .arg("-p")
            .arg("-v")
            .arg(option_name)
            .output()
            .expect("Failed to get @window-name");

        let result = String::from_utf8_lossy(&window_name.stdout);
        let trimmed = result.trim();

        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    fn count_panes(&self) -> usize {
        let output = Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#{window_panes}")
            .output()
            .expect("Failed to get the count of window panes.");

        let id = String::from_utf8_lossy(&output.stdout);
        id.trim().parse().expect("Failed to parse pane count.")
    }

    fn set_pane_option_for_current_window(&self, pane_index: usize, name: &str, value: &str) {
        let decorator = |command: &mut Command| {
            command.arg("-t").arg(format!(".{}", pane_index));
        };

        self.set_pane_option(name, value, decorator);
    }

    // fn set_pane_option(
    //     &self,
    //     session_name: &str,
    //     window_name: &str,
    //     pane_index: usize,
    //     name: &str,
    //     value: &str,
    // ) {
    //     let decorator = |command: &mut Command| {
    //         command
    //             .arg("-t")
    //             .arg(format!("{}:{}.{}", session_name, window_name, pane_index));
    //     };
    //
    //     self.set_pane_option(name, value, decorator);
    // }
}
