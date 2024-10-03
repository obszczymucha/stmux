mod args;
mod bookmarks;
mod config;
mod model;
mod recent;
mod session_name_file;
mod sessions;
mod tmux;
mod utils;
use args::{
    Action, BookmarkAction, ConfigAction, ConfigPrintFilename, RecentSessionAction, SessionsAction,
};
use bookmarks::{Bookmarks, BookmarksImpl};
use clap::Parser;
use config::Config;
use recent::{Recent, RecentImpl};
use session_name_file::SessionNameFileImpl;
use sessions::{Sessions, SessionsImpl};
use tmux::{Tmux, TmuxImpl};

fn main() {
    let args = args::Args::parse();
    let config = config::ConfigImpl;
    config.create_dir();

    match args.action {
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
        Action::Sessions { action } => match action {
            SessionsAction::Save { filename } => {
                let sessions = SessionsImpl::new(&TmuxImpl);
                sessions.save(filename.unwrap_or(config.sessions_filename()).as_str())
            }
            SessionsAction::Restore { filename } => {
                let sessions = SessionsImpl::new(&TmuxImpl);
                sessions.restore(filename.unwrap_or(config.sessions_filename()).as_str())
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
                    tmux.select_session(&name);
                }
            }
            RecentSessionAction::Previous => {
                let tmux = TmuxImpl;
                let file = SessionNameFileImpl::new(config.recent_sessions_filename().as_str());
                let recent = RecentImpl::new(&tmux, &file);

                if let Some(name) = recent.previous(&tmux.current_session_name()) {
                    tmux.select_session(&name);
                }
            }
            RecentSessionAction::Edit => {
                let file = SessionNameFileImpl::new(config.recent_sessions_filename().as_str());
                let recent = RecentImpl::new(&TmuxImpl, &file);

                recent.edit(&config);
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

                bookmarks.set(&tmux);
            }
            BookmarkAction::Select { index } => {
                let file = SessionNameFileImpl::new(config.bookmarks_filename().as_str());
                let bookmarks = BookmarksImpl::new(&file);

                if let Some(name) = bookmarks.select(index) {
                    TmuxImpl.select_session(&name);
                }
            }
            BookmarkAction::Edit => {
                let file = SessionNameFileImpl::new(config.bookmarks_filename().as_str());
                let bookmarks = BookmarksImpl::new(&file);

                bookmarks.edit(&config, &TmuxImpl);
            }
        },
    }
}
