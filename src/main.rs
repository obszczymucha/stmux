mod args;
mod bookmarks;
mod config;
mod model;
mod recent;
mod session;
mod session_name_file;
mod sessions;
mod status;
mod tmux;
mod utils;
use std::collections::HashSet;

use args::{
    Action, BookmarkAction, ConfigAction, ConfigPrintFilename, RecentSessionAction, SessionAction,
    SessionsAction,
};
use bookmarks::{Bookmarks, BookmarksImpl};
use clap::Parser;
use config::Config;
use recent::{Recent, RecentImpl};
use session::{Session, SessionImpl};
use session_name_file::{SessionNameFile, SessionNameFileImpl};
use sessions::{Sessions, SessionsImpl};
use status::{Status, StatusImpl};
use tmux::{Tmux, TmuxImpl};

fn run(config: &dyn Config, action: Action) {
    match action {
        Action::Config { action } => match action {
            ConfigAction::Print { action } => match action {
                ConfigPrintFilename::Sessions => {
                    eprintln!("{}", config.sessions_filename());
                }
                ConfigPrintFilename::RecentSessions => {
                    eprintln!("{}", config.recent_sessions_filename());
                }
                ConfigPrintFilename::Bookmarks => {
                    eprintln!("{}", config.bookmarks_filename());
                }
            },
        },
        Action::Session { action } => match action {
            SessionAction::FindAll => {
                let tmux = TmuxImpl;
                let sessions = SessionsImpl::new(config.sessions_filename().as_str(), &tmux).load();
                let recent_sessions: &dyn SessionNameFile =
                    &SessionNameFileImpl::new(config.recent_sessions_filename().as_str());
                let saved_session_names: Vec<String> = sessions.into_keys().collect();
                let recent_session_names: Vec<String> = recent_sessions.read();
                let unique_session_names: HashSet<String> = tmux
                    .list_session_names()
                    .into_iter()
                    .chain(saved_session_names) // TODO: check this
                    .collect();
                let mut stored_names: Vec<String> = unique_session_names.into_iter().collect();
                let compare = |a: &String, b: &String| a.to_lowercase().cmp(&b.to_lowercase());
                stored_names.sort_by(compare);
                let current_session_name = tmux.current_session_name();

                let session_names: Vec<String> = recent_session_names
                    .iter()
                    .chain(
                        stored_names
                            .iter()
                            .filter(|s| !recent_session_names.contains(s)),
                    )
                    .map(|name| name.to_string())
                    .filter(|s| s != &current_session_name)
                    .collect();

                if session_names.is_empty() {
                    tmux.display_message("No other sessions found.");
                    return;
                }

                let session = SessionImpl::new(&tmux);
                session.find(session_names, "All Sessions");
            }
            SessionAction::Find => {
                let tmux = TmuxImpl;
                let current_session_name = tmux.current_session_name();
                let session_names: Vec<String> = tmux
                    .list_session_names()
                    .into_iter()
                    .filter(|s| s != &current_session_name)
                    .collect();

                if session_names.is_empty() {
                    run(
                        config,
                        Action::Session {
                            action: SessionAction::FindAll,
                        },
                    );
                    return;
                }

                let session = SessionImpl::new(&tmux);
                session.find(session_names, "Sessions");
            }
            SessionAction::Select { session_name } => {
                let tmux = TmuxImpl;
                let session = SessionImpl::new(&tmux);
                let sessions = SessionsImpl::new(config.sessions_filename().as_str(), &tmux);

                session.select(session_name.as_str(), &sessions);
            }
            SessionAction::Save => {
                let session = SessionImpl::new(&TmuxImpl);
                session.save();
            }
            SessionAction::Reset => {
                let session = SessionImpl::new(&TmuxImpl);
                session.reset();
            }
            SessionAction::Startup { command, delete } => {
                let session = SessionImpl::new(&TmuxImpl);

                if delete {
                    session.delete_startup();
                } else if let Some(command) = command {
                    session.set_startup(&command);
                }
            }
        },
        Action::Sessions { action } => match action {
            SessionsAction::Save { filename } => {
                let file = filename.unwrap_or(config.sessions_filename());
                let sessions = SessionsImpl::new(&file, &TmuxImpl);
                sessions.save();
            }
            SessionsAction::Restore { filename } => {
                let file = filename.unwrap_or(config.sessions_filename());
                let sessions = SessionsImpl::new(&file, &TmuxImpl);
                sessions.restore_all();
            }
        },
        Action::RecentSession { action } => match action {
            RecentSessionAction::Print => {
                let file = SessionNameFileImpl::new(config.recent_sessions_filename().as_str());
                let recent = RecentImpl::new(&TmuxImpl, &file);

                recent.print()
            }
            RecentSessionAction::Next => {
                let tmux = TmuxImpl;
                let file = SessionNameFileImpl::new(config.recent_sessions_filename().as_str());
                let recent = RecentImpl::new(&tmux, &file);

                if let Some(name) = recent.next(&tmux.current_session_name()) {
                    let session = SessionImpl::new(&tmux);
                    session.select(
                        &name,
                        &SessionsImpl::new(config.sessions_filename().as_str(), &tmux),
                    );
                }
            }
            RecentSessionAction::Previous => {
                let tmux = TmuxImpl;
                let file = SessionNameFileImpl::new(config.recent_sessions_filename().as_str());
                let recent = RecentImpl::new(&tmux, &file);

                if let Some(name) = recent.previous(&tmux.current_session_name()) {
                    let session = SessionImpl::new(&tmux);
                    session.select(
                        &name,
                        &SessionsImpl::new(config.sessions_filename().as_str(), &tmux),
                    );
                }
            }
            RecentSessionAction::Edit => {
                let file = SessionNameFileImpl::new(config.recent_sessions_filename().as_str());
                let recent = RecentImpl::new(&TmuxImpl, &file);

                recent.edit(config);
            }
            RecentSessionAction::Add { session_name } => {
                let tmux = &TmuxImpl;
                let name = session_name.unwrap_or(tmux.current_session_name());
                let file = SessionNameFileImpl::new(config.recent_sessions_filename().as_str());
                let recent = RecentImpl::new(tmux, &file);

                recent.add(&name);
            }
        },
        Action::Bookmark { action } => match action {
            BookmarkAction::Print => {
                let file = SessionNameFileImpl::new(config.bookmarks_filename().as_str());
                let bookmarks = BookmarksImpl::new(&file);

                bookmarks.print();
            }
            BookmarkAction::Set => {
                let tmux = TmuxImpl;
                let file = SessionNameFileImpl::new(config.bookmarks_filename().as_str());
                let bookmarks = BookmarksImpl::new(&file);

                if bookmarks.set(&tmux) {
                    run(config, Action::Status);
                }
            }
            BookmarkAction::Select { index, smart_focus } => {
                let file = SessionNameFileImpl::new(config.bookmarks_filename().as_str());
                let bookmarks = BookmarksImpl::new(&file);
                let tmux = TmuxImpl;
                let current_session_name = tmux.current_session_name();

                if let Some(name) = bookmarks.select(index) {
                    if name == current_session_name {
                        if let Some(smart_focus) = smart_focus {
                            tmux.select_window(&name, smart_focus);
                            return;
                        }

                        return;
                    }

                    let session = SessionImpl::new(&tmux);
                    let sessions = SessionsImpl::new(config.sessions_filename().as_str(), &tmux);

                    session.select(&name, &sessions);
                }
            }
            BookmarkAction::Edit => {
                let file = SessionNameFileImpl::new(config.bookmarks_filename().as_str());
                let bookmarks = BookmarksImpl::new(&file);

                bookmarks.edit(config, &TmuxImpl);
                run(config, Action::Status);
            }
        },
        Action::Status => {
            let tmux = &TmuxImpl;
            let file = SessionNameFileImpl::new(config.bookmarks_filename().as_str());
            let status = StatusImpl::new(tmux, &file);
            tmux.set_global("status-left", &status.get());
        }
    }
}

fn main() {
    let args = args::Args::parse();
    let config = config::ConfigImpl;
    config.create_dir();

    run(&config, args.action);
}
