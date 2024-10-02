use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(long_about = None, disable_help_flag = true, disable_help_subcommand = true)]
pub struct Args {
    #[command(subcommand)]
    pub(crate) action: Action,
}

#[derive(Subcommand, Debug)]
#[warn(clippy::enum_variant_names)]
pub(crate) enum ConfigPrintFilename {
    Sessions,
    RecentSessions,
    Bookmarks,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigAction {
    Print {
        #[command(subcommand)]
        action: ConfigPrintFilename,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum SessionAction {
    Save {
        /// Optional filename to store sessions
        filename: Option<String>,
    },
    Restore {
        /// Filename to restore sessions from
        filename: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum RecentSessionAction {
    Print,
    Next,
    Previous,
}

#[derive(Subcommand, Debug)]
pub(crate) enum BookmarkAction {
    Print,
    Set,
    Select {
        /// Index of bookmarked session to select (1-based)
        index: usize,
    },
    Edit
}

#[derive(Subcommand, Debug)]
pub(crate) enum Action {
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
    RecentSession {
        #[command(subcommand)]
        action: RecentSessionAction,
    },
    Bookmark {
        #[command(subcommand)]
        action: BookmarkAction,
    },
}
