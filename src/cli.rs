use clap::{Parser, Subcommand};
use clap::builder::ValueHint;
use clap_complete::Shell;

#[derive(Parser, Debug)]
#[command(
    name = "uniman",
    about = "Universal Man Page Reader",
    long_about = "Read man pages from any OS locally, without VMs or containers.\n\n\
                  Backends are man page collections (e.g. linux-upstream, freebsd) \
                  that are cloned or downloaded and indexed locally."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(help = "Backend name or alias (e.g. linux-upstream, linux)", value_hint = ValueHint::Other)]
    pub backend: Option<String>,

    #[arg(help = "Man page section (e.g. 2, 3)", value_hint = ValueHint::Other)]
    pub section: Option<String>,

    #[arg(help = "Man page topic (e.g. execve, printf)", value_hint = ValueHint::Other)]
    pub topic: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "List configured backends and their status, or list topics for a backend",
              long_about = "Without an argument, lists all configured backends and their status.\n\
                 With a backend name or alias, lists all topics indexed for that backend.")]
    List {
        #[arg(help = "Name or alias of the backend to list topics for (lists backends if omitted)", value_hint = ValueHint::Other)]
        backend: Option<String>,
    },
    #[command(about = "Print the config file path")]
    Config,
    #[command(about = "Generate shell completion scripts")]
    Completion {
        #[arg(value_enum, help = "Shell type")]
        shell: Shell,
    },
    #[command(about = "Set or show the default backend", long_about = "Without an argument, shows the current default backend.\n\
                 With an argument, sets the default backend to the given name or alias.\n\
                 The backend must exist in config and be installed.")]
    Default {
        #[arg(help = "Name or alias of the backend to set as default", value_hint = ValueHint::Other)]
        name: Option<String>,
    },
    #[command(about = "Install a backend", long_about = "Download and index a man page backend.\n\
                 The backend name must match an entry in the config (or an alias).")]
    Install {
        #[arg(help = "Backend to install (if omitted, installs all configured backends)", value_hint = ValueHint::Other)]
        backend: Option<String>,
    },
    #[command(about = "Remove a backend", long_about = "Remove an installed backend and delete its data and index entries.")]
    Remove {
        #[arg(help = "Name or alias of the backend to remove", value_hint = ValueHint::Other)]
        backend: String,
    },
    #[command(about = "Update backends", long_about = "Pull latest changes for one or all installed backends, then re-index.")]
    Update {
        #[arg(help = "Name or alias of the backend to update (updates all if omitted)", value_hint = ValueHint::Other)]
        backend: Option<String>,
    },
    #[command(about = "Search for man pages", long_about = "Search installed backends for man pages.\n\n\
                 By default searches page names. Use -k to search page names and descriptions.\n\
                 If no search term is provided, lists all man pages interactively.")]
    Search {
        #[arg(short, long, help = "Search by keyword (name + description) instead of filename")]
        keyword: bool,
        #[arg(help = "Search term (if omitted, lists all pages)", value_hint = ValueHint::Other)]
        topic: Option<String>,
    },
}