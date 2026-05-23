use uman::backend;
use uman::cli;
use uman::render;
use uman::search;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    if let Some(command) = cli.command {
        match command {
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
            cli::Commands::Backend { action } => match action {
                cli::BackendAction::List => {
                    backend::list()?;
                }
            },
        }
    } else if let Some(backend) = cli.backend {
        match (cli.section, cli.topic) {
            (Some(section), Some(topic)) => {
                render::read(&backend, Some(&section), &topic)?;
            }
            (Some(topic), None) => {
                render::read(&backend, None, &topic)?;
            }
            (None, None) => {
                println!("usage: uman <backend> [<section>] <topic>");
                println!("       uman install <backend>");
                println!("       uman remove <backend>");
                println!("       uman update [<backend>]");
                println!("       uman search [-k] <topic>");
                println!("       uman backend list");
            }
            (None, Some(_)) => unreachable!(),
        }
    } else {
        println!("usage: uman <backend> [<section>] <topic>");
        println!("       uman install <backend>");
        println!("       uman remove <backend>");
        println!("       uman update [<backend>]");
        println!("       uman search [-k] <topic>");
        println!("       uman backend list");
    }

    Ok(())
}