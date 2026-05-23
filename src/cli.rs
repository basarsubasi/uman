use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "uman", about = "Universal Man Page Reader")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    pub backend: Option<String>,
    pub section: Option<String>,
    pub topic: Option<String>,
}

#[derive(Subcommand, Debug)]
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
        #[arg(short, long)]
        keyword: bool,
        topic: String,
    },
    Backend {
        #[command(subcommand)]
        action: BackendAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum BackendAction {
    List,
}