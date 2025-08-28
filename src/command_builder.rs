use std::process::Command;

pub(crate) trait CommandBuilder {
    fn new_command(&self) -> Command;
}

pub(crate) struct CommandBuilderImpl;

impl CommandBuilder for CommandBuilderImpl {
    fn new_command(&self) -> Command {
        Command::new("tmux")
    }
}

#[cfg(test)]
pub(crate) struct TestCommandBuilderImpl {
    socket: String,
}

#[cfg(test)]
impl TestCommandBuilderImpl {
    #[cfg(test)]
    pub(crate) fn new(socket: &str) -> Self {
        Self {
            socket: socket.to_string(),
        }
    }

    #[cfg(test)]
    pub(crate) fn kill_server(&mut self) {
        let cmd = &mut self.new_command();
        cmd.arg("kill-server")
            .status()
            .expect("Couldn't kill tmux server.");
    }
}

#[cfg(test)]
impl CommandBuilder for TestCommandBuilderImpl {
    fn new_command(&self) -> Command {
        let mut cmd = Command::new("tmux");
        cmd.arg("-L").arg(&self.socket);
        cmd
    }
}
