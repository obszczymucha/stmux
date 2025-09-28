use std::process::Command;
use std::process::Stdio;

use mockall::automock;

use crate::command_builder::CommandBuilder;
use crate::model::EnvironmentVariable;
use crate::model::Position;
use crate::model::TmuxOption;
use crate::model::TmuxWindow;
use crate::model::WindowDimension;

pub(crate) struct SplitWindowOptions {
    pub(crate) horizontally: bool,
    pub(crate) path: String,
    pub(crate) startup_command: Option<String>,
    pub(crate) at_index: Option<usize>,
    pub(crate) before: bool,
}

#[automock]
pub(crate) trait Tmux {
    fn list_sessions(&self, format: &str) -> Result<Vec<String>, Vec<String>>;
    fn list_current_session_panes(&self, format: &str) -> Vec<String>;
    fn list_session_panes(&self, session_name: &str, format: &str) -> Vec<String>;
    fn list_current_window_panes(&self, format: &str) -> Vec<String>;
    fn list_windows_for_current_session(&self, format: &str) -> Vec<String>;
    // fn list_windows_names_with_status(&self) -> Vec<WindowDetails>;
    fn new_session(
        &self,
        session_name: &str,
        tmux_window: &TmuxWindow,
        startup_command: &Option<String>,
    );
    fn new_window_in_current_session(
        &self,
        window_name: &str,
        path: &str,
        environment: &[EnvironmentVariable],
        startup_command: &Option<String>,
        background: bool,
    );
    fn new_window(
        &self,
        session_name: &str,
        window_name: &str,
        path: &str,
        environment: &[EnvironmentVariable],
        startup_command: &Option<String>,
        background: bool,
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
    fn split_current_window(&self, options: &SplitWindowOptions);
    fn split_window(&self, session_name: &str, window_name: &str, options: &SplitWindowOptions);
    fn select_layout(&self, session_name: &str, window_name: &str, layout: &str);
    fn send_keys_to_current_window(&self, pane_index: usize, keys: &str);
    fn send_keys(&self, session_name: &str, window_name: &str, pane_index: usize, keys: &str);
    fn window_dimension(&self) -> Option<WindowDimension>;
    fn set_global(&self, option_name: &str, value: &str);
    fn current_window_index(&self) -> usize;
    // fn get_pane_option(&self, pane_index: usize, option_name: &str) -> Option<String>;
    fn count_panes(&self) -> usize;
    fn set_pane_option_for_current_window(&self, pane_index: usize, name: &str, value: &str);
    fn set_pane_option(&self, window_name: &str, pane_index: usize, name: &str, value: &str);
    fn set_session_option(&self, session_name: &str, option: &TmuxOption);

    fn swap_panes(
        &self,
        current_window_pane_index: usize,
        source_window_name: &str,
        source_pane_index: usize,
    );
    fn rename_window_in_current_session(&self, old_name: &str, new_name: &str);
    fn join_pane_to_current_window(
        &self,
        window_name: &str,
        pane_index: usize,
        at_index: Option<usize>,
        before: bool,
    );
    fn select_pane(&self, index: usize);
    fn get_cursor_position(&self) -> Option<Position>;
}

pub(crate) struct TmuxImpl<'cb, CB: CommandBuilder> {
    pub command_builder: &'cb CB,
}

impl<'cb, CB: CommandBuilder> TmuxImpl<'cb, CB> {
    pub fn new(command_builder: &'cb CB) -> Self {
        Self { command_builder }
    }

    fn new_window<F>(
        &self,
        window_name: &str,
        path: &str,
        environment: &[EnvironmentVariable],
        startup_command: &Option<String>,
        decorator_fn: F,
    ) where
        F: FnOnce(&mut Command),
    {
        let command = &mut self.command_builder.new_command();
        command.arg("new-window");

        decorator_fn(command);

        command.arg("-n").arg(window_name).arg("-e").arg("NO_CD=1");
        command.arg("-c").arg(path);

        for env in environment.iter() {
            command.arg("-e").arg(format!("{}={}", env.name, env.value));
        }

        if let Some(program) = startup_command {
            command.arg(program);
        }

        command.status().expect("Failed to create new session.");
    }

    fn split_window<F>(&self, options: &SplitWindowOptions, decorator_fn: F)
    where
        F: FnOnce(&mut Command),
    {
        let command = &mut self.command_builder.new_command();
        command.arg("split-window");

        if options.horizontally {
            command.arg("-h");
        } else {
            command.arg("-v");
        }

        decorator_fn(command);

        if let Some(at_index) = options.at_index {
            command.arg("-t").arg(format!(".{}", at_index));
        }

        if options.before {
            command.arg("-b");
        }

        // WTF is this NO_CD=1 doing here?
        command
            .arg("-e")
            .arg("NO_CD=1")
            .arg("-c")
            .arg(&options.path);

        if let Some(program) = &options.startup_command {
            command.arg(program);
        }

        command.status().expect("Failed to split a window.");
    }

    fn send_keys<F>(&self, keys: &str, decorator_fn: F)
    where
        F: FnOnce(&mut Command),
    {
        let command = &mut self.command_builder.new_command();
        command.arg("send-keys");
        decorator_fn(command);

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
        let command = &mut self.command_builder.new_command();
        command.arg("set").arg("-p");

        decorator_fn(command);

        command
            .arg(name)
            .arg(value)
            .status()
            .expect("Failed to get the count of window panes.");
    }
}

impl<'cb, CB: CommandBuilder> Tmux for TmuxImpl<'cb, CB> {
    fn list_sessions(&self, format: &str) -> Result<Vec<String>, Vec<String>> {
        let output = &self
            .command_builder
            .new_command()
            .arg("list-sessions")
            .arg("-F")
            .arg(format)
            .output()
            .expect("Failed to list tmux sessions.");

        let s = if output.status.success() {
            &output.stdout
        } else {
            &output.stderr
        };

        let result = String::from_utf8_lossy(s)
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        if output.status.success() {
            Ok(result)
        } else {
            Err(result)
        }
    }

    fn list_session_panes(&self, session_name: &str, format: &str) -> Vec<String> {
        let output = &self
            .command_builder
            .new_command()
            .arg("list-panes")
            .arg("-s")
            .arg("-t")
            .arg(session_name)
            .arg("-F")
            .arg(format)
            .output()
            .expect("Failed to list session panes.");

        let result = String::from_utf8_lossy(&output.stdout);
        result.lines().map(|s| s.to_string()).collect()
    }

    fn list_current_session_panes(&self, format: &str) -> Vec<String> {
        let output = &self
            .command_builder
            .new_command()
            .arg("list-panes")
            .arg("-s")
            .arg("-F")
            .arg(format)
            .output()
            .expect("Failed to list current session panes.");

        let result = String::from_utf8_lossy(&output.stdout);
        result.lines().map(|x| x.to_string()).collect()
    }

    fn list_current_window_panes(&self, format: &str) -> Vec<String> {
        let output = &self
            .command_builder
            .new_command()
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

        let command = &mut self.command_builder.new_command();
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

            for env in pane.environment.iter() {
                command.arg("-e").arg(format!("{}={}", env.name, env.value));
            }

            command.arg("-c").arg(pane.path.as_str());
        }

        if let Some(program) = startup_command {
            command.arg(program);
        }

        command.status().expect("Failed to create new session.");
    }

    fn new_window_in_current_session(
        &self,
        window_name: &str,
        path: &str,
        environment: &[EnvironmentVariable],
        startup_command: &Option<String>,
        background: bool,
    ) {
        let decorator = |command: &mut Command| {
            if background {
                command.arg("-d");
            }
        };

        self.new_window(window_name, path, environment, startup_command, decorator);
    }

    fn new_window(
        &self,
        session_name: &str,
        window_name: &str,
        path: &str,
        environment: &[EnvironmentVariable],
        startup_command: &Option<String>,
        background: bool,
    ) {
        let decorator = |command: &mut Command| {
            command.arg("-t").arg(session_name);

            if background {
                command.arg("-d");
            }
        };

        self.new_window(window_name, path, environment, startup_command, decorator);
    }

    fn has_session(&self, session_name: &str) -> bool {
        let output = &self
            .command_builder
            .new_command()
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
        self.command_builder
            .new_command()
            .arg("select-window")
            .arg("-t")
            .arg(format!("{}:{}", session_name, index))
            .status()
            .expect("Failed to select window.");
    }

    fn current_session_name(&self) -> String {
        let stdout = &self
            .command_builder
            .new_command()
            .arg("display-message")
            .arg("-p")
            .arg("#S")
            .output()
            .expect("Failed to get current session name.")
            .stdout;

        String::from_utf8_lossy(stdout).trim().to_string()
    }

    fn select_session(&self, session_name: &str) {
        self.command_builder
            .new_command()
            .arg("switch-client")
            .arg("-t")
            .arg(session_name)
            .status()
            .expect("Failed to select session.");
    }

    fn display_message(&self, message: &str) {
        self.command_builder
            .new_command()
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
        let cmd = &mut self.command_builder.new_command();
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

    fn split_current_window(&self, options: &SplitWindowOptions) {
        self.split_window(options, |_| {});
    }

    fn split_window(&self, session_name: &str, window_name: &str, options: &SplitWindowOptions) {
        let decorator = |command: &mut Command| {
            command
                .arg("-t")
                .arg(format!("{}:{}", session_name, window_name));
        };

        self.split_window(options, decorator);
    }

    fn select_layout(&self, session_name: &str, window_name: &str, layout: &str) {
        // eprintln!(
        //     "Selecting layout '{}' for window '{}' in session '{}'.",
        //     layout, window_name, session_name
        // );
        self.command_builder
            .new_command()
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
        let output = &self
            .command_builder
            .new_command()
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
        self.command_builder
            .new_command()
            .arg("set")
            .arg("-g")
            .arg(option_name)
            .arg(value)
            .status()
            .expect("Failed to set global option.");
    }

    fn list_windows_for_current_session(&self, format: &str) -> Vec<String> {
        let output = &self
            .command_builder
            .new_command()
            .arg("list-windows")
            .arg("-F")
            .arg(format)
            .output()
            .expect("Failed to list tmux windows.");

        let result = String::from_utf8_lossy(&output.stdout);
        result.lines().map(|x| x.to_string()).collect()
    }

    fn current_window_index(&self) -> usize {
        let output = &self
            .command_builder
            .new_command()
            .arg("display-message")
            .arg("-p")
            .arg("#I")
            .output()
            .expect("Failed to get current window index.");

        let id = String::from_utf8_lossy(&output.stdout);
        id.trim().parse().expect("Failed to parse window index.")
    }

    // fn get_pane_option(&self, pane_index: usize, option_name: &str) -> Option<String> {
    //     let window_name = &self.command_builder.new_command()
    //         .arg("show-option")
    //         .arg("-t")
    //         .arg(pane_index.to_string())
    //         .arg("-p")
    //         .arg("-v")
    //         .arg(option_name)
    //         .output()
    //         .expect("Failed to get @window-name");
    //
    //     let result = String::from_utf8_lossy(&window_name.stdout);
    //     let trimmed = result.trim();
    //
    //     if trimmed.is_empty() {
    //         None
    //     } else {
    //         Some(trimmed.to_string())
    //     }
    // }

    fn count_panes(&self) -> usize {
        let output = &self
            .command_builder
            .new_command()
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

    fn swap_panes(
        &self,
        current_window_pane_index: usize,
        source_window_name: &str,
        source_pane_index: usize,
    ) {
        let current_window_index = self.current_window_index();
        self.command_builder
            .new_command()
            .arg("swap-pane")
            .arg("-s")
            .arg(format!("{}.{}", source_window_name, source_pane_index))
            .arg("-t")
            .arg(format!(
                "{}.{}",
                current_window_index, current_window_pane_index
            ))
            .status()
            .expect("Failed to swap panes.");
    }

    fn rename_window_in_current_session(&self, old_name: &str, new_name: &str) {
        self.command_builder
            .new_command()
            .arg("rename-window")
            .arg("-t")
            .arg(old_name)
            .arg(new_name)
            .status()
            .expect("Failed to rename window.");
    }

    fn join_pane_to_current_window(
        &self,
        window_name: &str,
        pane_index: usize,
        at_index: Option<usize>,
        before: bool,
    ) {
        let command = &mut self.command_builder.new_command();
        command
            .arg("join-pane")
            .arg("-d")
            .arg("-h")
            .arg("-s")
            .arg(format!("{}.{}", window_name, pane_index));

        if let Some(at_index) = at_index {
            command.arg("-t").arg(format!(".{}", at_index));
        }

        if before {
            command.arg("-b");
        }

        command
            .status()
            .expect("Failed to join pane to current window.");
    }

    fn select_pane(&self, index: usize) {
        self.command_builder
            .new_command()
            .arg("select-pane")
            .arg("-t")
            .arg(format!("{}", index))
            .status()
            .expect("Failed to select pane.");
    }

    fn set_pane_option(&self, window_name: &str, pane_index: usize, name: &str, value: &str) {
        let decorator = |command: &mut Command| {
            command
                .arg("-t")
                .arg(format!("{}.{}", window_name, pane_index));
        };

        self.set_pane_option(name, value, decorator);
    }

    fn set_session_option(&self, session_name: &str, option: &TmuxOption) {
        let command = &mut self.command_builder.new_command();
        command
            .arg("set")
            .arg("-t")
            .arg(session_name)
            .arg(&option.name)
            .arg(&option.value)
            .status()
            .expect("Failed to set session option.");
    }

    fn get_cursor_position(&self) -> Option<Position> {
        let output = &self
            .command_builder
            .new_command()
            .arg("display-message")
            .arg("-p")
            .arg("#{cursor_x},#{cursor_y}")
            .output()
            .expect("Failed to get cursor position.");

        let position_str = String::from_utf8_lossy(&output.stdout);
        position_str
            .trim()
            .split_once(',')
            .and_then(|(x_str, y_str)| {
                let x = x_str.parse::<i32>().ok()?;
                let y = y_str.parse::<i32>().ok()?;
                Some(Position { x, y })
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::command_builder::TestCommandBuilderImpl;

    use super::*;

    #[test]
    fn test_should_error_while_listing_sessions_on_a_non_running_server() {
        let cb = TestCommandBuilderImpl::new("princesskenny");
        let tmux = TmuxImpl::new(&cb);
        let result = tmux.list_sessions("#W");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err[0].contains("error connecting"), "Was: {}", err[0]);
        assert!(
            err[0].contains("No such file or directory"),
            "Was: {}",
            err[0]
        );
    }
}
