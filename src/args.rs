use clap::{Parser, Subcommand, ValueEnum};

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
    /// Show a session picker for all managed sessions.
    #[clap(name = "find-all")]
    FindAll,
    /// Show a session picker for active sessions only.
    Find,
    /// Switch to or create a session.
    Select {
        /// Session name to select or create (if configured).
        session_name: String,
    },
    /// Save the current session.
    Save,
    /// Delete a session.
    Delete {
        /// Session name to delete. session_name: String,
        session_name: String,
    },
    /// Update session settings.
    Update {
        /// Session name to update.
        session_name: String,
        background: bool,
        no_recent_tracking: bool,
        window_active: bool,
        pane_active: bool,
        startup_command: Option<String>,
        shell_command: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum SessionsAction {
    /// Save all active sessions.
    Save {
        /// Optional filename to store sessions.
        filename: Option<String>,
    },
    /// Restore all sessions managed by stmux.
    Restore {
        /// Filename to restore sessions from.
        filename: Option<String>,
    },
    /// List sessions managed by stmux.
    List,
    /// Convert old toml config to a new format (to be removed).
    Convert { filename: String },
}

#[derive(Subcommand, Debug)]
pub(crate) enum RecentSessionAction {
    List,
    Next,
    Previous,
    Edit,
    Add { session_name: Option<String> },
}

#[derive(Subcommand, Debug)]
pub(crate) enum BookmarkAction {
    /// List all bookmarked sessions.
    List,
    /// Bookmark the current session.
    Set,
    /// Switch to or create a bookmarked session.
    Select {
        /// Index of bookmarked session to select.
        index: usize,
        #[arg(long = "smart-focus")]
        smart_focus: Option<usize>,
    },
    /// Edit bookmarks.
    Edit,
}

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum SplitType {
    Right,
    Left,
}

#[derive(Subcommand, Debug)]
pub(crate) enum WindowAction {
    SmartSplit {
        #[arg(value_enum)]
        split_type: SplitType,
        session_name: String,
    },
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
    Window {
        #[command(subcommand)]
        action: WindowAction,
    },
    Status,
}
