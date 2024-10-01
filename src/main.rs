mod args;
mod config;
mod model;
mod recent;
mod recent_session_file;
mod sessions;
mod tmux;
mod utils;
use clap::Parser;
use config::Config;
use recent::Recent;
use sessions::Sessions;
use tmux::Tmux;

fn main() {
    let args = args::Args::parse();

    let config = config::ConfigImpl;
    config.create_dir();

    let tmux = tmux::TmuxImpl;
    let sessions = sessions::SessionsImpl::new(&tmux);
    let recent_session_file = recent_session_file::RecentSessionFileImpl;
    let recent = recent::RecentImpl::new(&config, &tmux, &recent_session_file);

    match args.action {
        args::Action::Save { filename } => sessions.save(
            filename
                .unwrap_or(config.default_sessions_filename())
                .as_str(),
        ),
        args::Action::Restore { filename } => sessions.restore(
            filename
                .unwrap_or(config.default_sessions_filename())
                .as_str(),
        ),
        args::Action::NextRecent => {
            if let Some(name) = recent.next(&tmux.current_session_name()) {
                tmux.select_session(&name);
            }
        }
        args::Action::PreviousRecent => {
            if let Some(name) = recent.previous(&tmux.current_session_name()) {
                tmux.select_session(&name);
            }
        }
    }
}
