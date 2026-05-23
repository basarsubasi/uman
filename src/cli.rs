use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "uman",
    about = "Universal Man Page Reader",
    long_about = "Read man pages from any OS locally, without VMs or containers.\n\n\
                  Backends are man page collections (e.g. linux-upstream, freebsd) \
                  that are cloned or downloaded and indexed locally."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(help = "Backend name (e.g. linux-upstream)")]
    pub backend: Option<String>,

    #[arg(help = "Man page section (e.g. 2, 3)")]
    pub section: Option<String>,

    #[arg(help = "Man page topic (e.g. execve, printf)")]
    pub topic: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Install a backend", long_about = "Download and index a man page backend.\n\
                 The backend name must match an entry in the config.")]
    Install {
        #[arg(help = "Name of the backend to install")]
        backend: String,
    },
    #[command(about = "Remove a backend", long_about = "Remove an installed backend and delete its data and index entries.")]
    Remove {
        #[arg(help = "Name of the backend to remove")]
        backend: String,
    },
    #[command(about = "Update backends", long_about = "Pull latest changes for one or all installed backends, then re-index.")]
    Update {
        #[arg(help = "Name of the backend to update (updates all if omitted)")]
        backend: Option<String>,
    },
    #[command(about = "Search for man pages", long_about = "Search installed backends for man pages.\n\n\
                 By default searches page names. Use -k to search page names and descriptions.")]
    Search {
        #[arg(short, long, help = "Search by keyword (name + description) instead of filename")]
        keyword: bool,
        #[arg(help = "Search term")]
        topic: String,
    },
    #[command(about = "Manage backends", long_about = "Backend management subcommands.")]
    Backend {
        #[command(subcommand)]
        action: BackendAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum BackendAction {
    #[command(about = "List configured backends and their status")]
    List,
}