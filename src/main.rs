use uman::backend;
use uman::cli;
use uman::config::Config;
use uman::paths;
use uman::render;
use uman::search;

use clap::{CommandFactory, Parser};
use clap_complete::generate;

fn is_numeric(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty()
}

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    if let Some(command) = cli.command {
        match command {
            cli::Commands::List { backend } => match backend {
                Some(b) => backend::list_topics(&b)?,
                None => backend::list()?,
            },
            cli::Commands::Config => {
                println!("{}", paths::config_path().display());
            }
            cli::Commands::Completion { shell } => {
                let mut cmd = cli::Cli::command();
                generate(shell, &mut cmd, "uman", &mut std::io::stdout());
            }
            cli::Commands::Default { name } => match name {
                Some(n) => backend::set_default(&n)?,
                None => backend::show_default()?,
            },
            cli::Commands::Install { backend } => {
                backend::install(&backend)?;
            }
            cli::Commands::Remove { backend } => {
                backend::remove(&backend)?;
            }
            cli::Commands::Update { backend } => {
                backend::update(backend.as_deref())?;
            }
            cli::Commands::Search { keyword, topic } => {
                if keyword {
                    search::run_keyword(&topic)?;
                } else {
                    search::run_filename(&topic)?;
                }
            }
        }
    } else if let Some(first) = cli.backend {
        dispatch_read(first, cli.section, cli.topic)?;
    } else {
        print_usage();
    }

    Ok(())
}

fn dispatch_read(
    first: String,
    second: Option<String>,
    third: Option<String>,
) -> anyhow::Result<()> {
    let config = Config::load()?;
    let backend_def = config.resolve(&first);
    let is_backend = backend_def.is_ok();
    let resolved = backend_def.ok();

    match (resolved, second, third, is_backend, is_numeric(&first)) {
        // uman <backend> <section> <topic>
        (Some(def), Some(sec), Some(top), _, _) => {
            render::read(&def.name, Some(&sec), &top)?;
        }

        // uman <backend> <topic>  (2 args, first resolves as backend)
        (Some(def), Some(top), None, _, _) => {
            render::read(&def.name, None, &top)?;
        }

        // Just a backend name with nothing else — incomplete, show usage
        (Some(_), None, None, _, _) => {
            print_usage();
            std::process::exit(1);
        }

        // uman <section> <topic>  (2 args, first is numeric, not a backend)
        (None, Some(top), None, false, true) => {
            let default_def = config.get_default_backend()?;
            render::read(&default_def.name, Some(&first), &top)?;
        }

        // uman <topic>  (1 arg, not a backend, not numeric)
        (None, None, None, false, false) => {
            let default_def = config.get_default_backend()?;
            render::read(&default_def.name, None, &first)?;
        }

        // Unresolvable
        _ => {
            eprintln!("error: '{}' is not a known backend or alias", first);
            eprintln!();
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    eprintln!("usage: uman <backend> [<section>] <topic>");
    eprintln!("       uman <topic>                          (uses default backend)");
    eprintln!("       uman <section> <topic>                 (uses default backend)");
    eprintln!("       uman install <backend>");
    eprintln!("       uman remove <backend>");
    eprintln!("       uman update [<backend>]");
    eprintln!("       uman search [-k] <topic>");
    eprintln!("       uman list");
    eprintln!("       uman list <backend>");
    eprintln!("       uman config");
    eprintln!("       uman completion <shell>");
    eprintln!("       uman default [<name>]");
}