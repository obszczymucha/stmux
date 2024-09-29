mod args;
mod config;
mod model;
mod sessions;
mod tmux;
mod utils;
use clap::Parser;

fn main() {
    let args = args::Args::parse();
    config::create_dir();

    match args.action {
        args::Action::Save { filename } => sessions::save(
            filename
                .unwrap_or(config::default_sessions_filename())
                .as_str(),
        ),
        args::Action::Restore { filename } => sessions::restore(
            filename
                .unwrap_or(config::default_sessions_filename())
                .as_str(),
        ),
    }
}
