use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(long_about = None, disable_help_flag = true, disable_help_subcommand = true)]
pub struct Args {
    #[command(subcommand)]
    pub(crate) action: Action,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Action {
    Save {
        /// Optional filename to store sessions
        filename: Option<String>,
    },
    Restore {
        /// Filename to restore sessions from
        filename: Option<String>,
    },
    NextRecent {
        /// Session name to get next recent session from
        session_name: String,
    },
    PreviousRecent,
}
