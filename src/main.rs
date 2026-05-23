mod backend;
mod cli;
mod config;
mod db;
mod error;
mod paths;
mod render;
mod search;

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
            cli::Commands::Search { topic } => {
                search::run(&topic)?;
            }
            cli::Commands::Backend { action } => match action {
                cli::BackendAction::List => {
                    backend::list()?;
                }
            },
        }
    } else if let (Some(backend), Some(section), Some(topic)) =
        (cli.backend, cli.section, cli.topic)
    {
        render::read(&backend, &section, &topic)?;
    } else {
        println!("usage: uman <backend> <section> <topic>");
        println!("       uman install <backend>");
        println!("       uman remove <backend>");
        println!("       uman update [<backend>]");
        println!("       uman search <topic>");
        println!("       uman backend list");
    }

    Ok(())
}