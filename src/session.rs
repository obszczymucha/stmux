use std::collections::HashSet;
use std::io::Write;
use std::{
    cmp::{max, min},
    fs::{remove_file, File},
    path::Path,
    process::Command,
    thread,
};

use crate::model::SessionNames;
use crate::sessions::Sessions;
use crate::tmux::Tmux;

const FZF_DEFAULT_OPTS: &str = "--bind=alt-q:close,alt-j:down,alt-k:up,tab:accept --color=fg:#cdd6f4,header:#f38ba8,info:#cba6f7,pointer:#f5e0dc --color=marker:#b4befe,fg+:#cdd6f4,prompt:#cba6f7,hl+:#f38ba8 --color=selected-bg:#45475a";

pub(crate) trait Session {
    fn find(&self, saved_session_names: &SessionNames);
    fn select(&self, name: &str, sessions: &dyn Sessions);
    fn save(&self);
    fn reset(&self);
    fn set_startup(&self, command: &str);
    fn delete_startup(&self);
}

pub(crate) struct SessionImpl<'t, T: Tmux> {
    tmux: &'t T,
}

impl<'t, T: Tmux> SessionImpl<'t, T> {
    pub(crate) fn new(tmux: &'t T) -> Self {
        Self { tmux }
    }
}

impl<'t, T: Tmux> Session for SessionImpl<'t, T> {
    fn find(&self, saved_session_names: &SessionNames) {
        let current_session_name = self.tmux.current_session_name();
        let session_names: HashSet<String> = self
            .tmux
            .list_session_names()
            .into_iter()
            .filter(|s| s != &current_session_name)
            .chain(saved_session_names.clone()) // TODO: check this
            .collect();

        let input_fifo_path = "/tmp/stmux_fzf_input.fifo";
        let title = " Sessions ";
        let title_len = title.len() + 4;
        let width = session_names
            .iter()
            .map(|item| item.len())
            .max()
            .unwrap_or(title_len)
            + 6;
        let max_height = 10;
        let mut popup_width = max(width, title_len);

        if session_names.len() > max_height && width >= title_len {
            popup_width += 1;
        }

        let height = min(session_names.len(), max_height) + 4;

        if !Path::new(input_fifo_path).exists() {
            let _ = nix::unistd::mkfifo(input_fifo_path, nix::sys::stat::Mode::S_IRWXU);
        }

        let write_thread = thread::spawn(move || -> Result<(), std::io::Error> {
            let mut fifo = File::create(input_fifo_path).unwrap();

            for item in session_names.into_iter() {
                let _ = writeln!(fifo, "{}", item);
            }

            Ok(())
        });

        // TODO: Extract into fzf-popup module.
        let cursor_color = "#a08afa";
        let colors_table = [
            "--color=border:#806aba",
            "--color=scrollbar:#404040",
            "--color=separator:#404040",
            "--color=label:italic:#9f7fff",
            "--color=gutter:#1a1323",
            "--color=current-bg:#3a2943",
            "--color=marker:#FF0000",
        ];

        let colors = colors_table.join(" ");
        let fzf_opts = format!(
            "--no-multi --border --border-label \"{}\" {}",
            title, colors
        );
        let fzf_command = format!(
            "echo -ne \"\\e]12;{}\\a\"; cat {} | fzf {} | xargs -I {{}} stmux session select '{{}}'",
            cursor_color, input_fifo_path, fzf_opts
        );

        let tmux_command = format!(
            "tmux display-popup -E -B -e 'FZF_DEFAULT_OPTS={}' -w {} -h {} '{}'",
            std::env::var("FZF_DEFAULT_OPTS").unwrap_or(FZF_DEFAULT_OPTS.to_string()),
            popup_width,
            height,
            fzf_command
        );

        let _ = Command::new("sh").arg("-c").arg(&tmux_command).output();

        let _ = write_thread.join().expect("Failed to join write thread");
        let _ = remove_file(input_fifo_path);
    }

    fn select(&self, name: &str, sessions: &dyn Sessions) {
        if !self.tmux.has_session(name) {
            sessions.restore(name);
        }

        self.tmux.select_session(name);
    }

    fn save(&self) {
        unimplemented!()
    }

    fn reset(&self) {
        unimplemented!()
    }

    fn set_startup(&self, command: &str) {
        eprintln!("TODO: set_startup({})", command);
    }

    fn delete_startup(&self) {
        eprintln!("TODO: delete_startup()");
    }
}
