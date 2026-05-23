use uniman::backend;
use uniman::cli;
use uniman::config::Config;
use uniman::paths;
use uniman::render;
use uniman::search;

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
                generate(shell, &mut cmd, "uniman", &mut std::io::stdout());
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
                if let Some(t) = topic {
                    if keyword {
                        search::run_keyword(&t)?;
                    } else {
                        search::run_filename(&t)?;
                    }
                } else {
                    search::run_all()?;
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
        // uniman <backend> <section> <topic>
        (Some(def), Some(sec), Some(top), _, _) => {
            render::read(&def.name, Some(&sec), &top)?;
        }

        // uniman <backend> <topic>  (2 args, first resolves as backend)
        (Some(def), Some(top), None, _, _) => {
            render::read(&def.name, None, &top)?;
        }

        // Just a backend name with nothing else — incomplete, show usage
        (Some(_), None, None, _, _) => {
            print_usage();
            std::process::exit(1);
        }

        // uniman <section> <topic>  (2 args, first is numeric, not a backend)
        (None, Some(top), None, false, true) => {
            let default_def = config.get_default_backend()?;
            render::read(&default_def.name, Some(&first), &top)?;
        }

        // uniman <topic>  (1 arg, not a backend, not numeric)
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
    eprintln!("usage: uniman <backend> [<section>] <topic>");
    eprintln!("       uniman <topic>                          (uses default backend)");
    eprintln!("       uniman <section> <topic>                 (uses default backend)");
    eprintln!("       uniman install <backend>");
    eprintln!("       uniman remove <backend>");
    eprintln!("       uniman update [<backend>]");
    eprintln!("       uniman search [-k] <topic>");
    eprintln!("       uniman list");
    eprintln!("       uniman list <backend>");
    eprintln!("       uniman config");
    eprintln!("       uniman completion <shell>");
    eprintln!("       uniman default [<name>]");
}