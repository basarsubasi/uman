use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "uman", about = "Universal Man Page Reader")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Read {
        backend: String,
        section: String,
        topic: String,
    },
    Install {
        backend: String,
    },
    Remove {
        backend: String,
    },
    Update {
        backend: Option<String>,
    },
    Search {
        topic: String,
    },
    Backend {
        #[command(subcommand)]
        action: BackendAction,
    },
}

#[derive(Subcommand)]
pub enum BackendAction {
    List,
}