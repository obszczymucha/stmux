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
    #[clap(name = "find-all")]
    FindAll,
    Find,
    Select {
        /// Session name to select or create (if configured).
        session_name: String,
    },
    Save,
    Reset,
    #[group(required = true, multiple = false)]
    Startup {
        /// Command to run on session startup in the current pane.
        #[arg(group = "startup")]
        command: Option<String>,
        #[arg(long, group = "startup")]
        delete: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum SessionsAction {
    Save {
        /// Optional filename to store sessions.
        filename: Option<String>,
    },
    Restore {
        /// Filename to restore sessions from.
        filename: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum RecentSessionAction {
    Print,
    Next,
    Previous,
    Edit,
    Add {
        session_name: Option<String>,
    }
}

#[derive(Subcommand, Debug)]
pub(crate) enum BookmarkAction {
    Print,
    Set,
    Select {
        /// Index of bookmarked session to select.
        index: usize,
        #[arg(long = "smart-focus")]
        smart_focus: Option<usize>,
    },
    Edit,
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
    Sessions {
        #[command(subcommand)]
        action: SessionsAction,
    },
    RecentSession {
        #[command(subcommand)]
        action: RecentSessionAction,
    },
    Bookmark {
        #[command(subcommand)]
        action: BookmarkAction,
    },
    Status
}
