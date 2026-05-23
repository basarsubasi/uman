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

    match cli.command {
        cli::Commands::Read {
            backend,
            section,
            topic,
        } => {
            render::read(&backend, &section, &topic)?;
        }
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

    Ok(())
}