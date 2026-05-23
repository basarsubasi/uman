use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "uman", about = "Universal Man Page Reader")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub read: Option<ReadArgs>,
}

#[derive(clap::Args)]
pub struct ReadArgs {
    pub backend: Option<String>,
    pub section: Option<String>,
    pub topic: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
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