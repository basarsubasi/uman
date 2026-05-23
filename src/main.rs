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
                update(backend.as_deref())?;
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
    } else if let Some(read) = cli.read {
        let backend = read
            .backend
            .ok_or_else(|| anyhow::anyhow!("usage: uman <backend> <section> <topic>"))?;
        let section = read
            .section
            .ok_or_else(|| anyhow::anyhow!("missing section"))?;
        let topic = read
            .topic
            .ok_or_else(|| anyhow::anyhow!("missing topic"))?;

        render::read(&backend, &section, &topic)?;
    } else {
        println!("usage: uman <backend> <section> <topic>");
        println!("       uman install <backend>");
        println!("       uman remove <backend>");
        println!("       uman backend list");
        println!("       uman update [<backend>]");
        println!("       uman search <topic>");
    }

    Ok(())
}

fn update(backend: Option<&str>) -> anyhow::Result<()> {
    let config = config::Config::load()?;

    if let Some(name) = backend {
        let def = config.get_backend(name)?;
        let dir = paths::backend_dir(name);
        if !dir.exists() {
            return Err(error::UmanError::BackendNotInstalled(name.to_string()).into());
        }

        match def.fetching.as_str() {
            "git" => {
                let status = std::process::Command::new("git")
                    .args(["-C"])
                    .arg(&dir)
                    .args(["pull"])
                    .status()?;
                if !status.success() {
                    anyhow::bail!("git pull failed for '{name}'");
                }
            }
            "curl" => {
                std::fs::remove_dir_all(&dir)?;
                crate::backend::install(name)?;
            }
            other => anyhow::bail!("unknown fetching method: {other}"),
        }

        db::index_backend(def)?;
        println!("Backend '{name}' updated.");
    } else {
        let mut any = false;
        for (name, def) in &config.backends {
            let dir = paths::backend_dir(name);
            if !dir.exists() {
                continue;
            }
            any = true;

            match def.fetching.as_str() {
                "git" => {
                    let status = std::process::Command::new("git")
                        .args(["-C"])
                        .arg(&dir)
                        .args(["pull"])
                        .status()?;
                    if !status.success() {
                        eprintln!("warning: git pull failed for '{name}'");
                        continue;
                    }
                }
                "curl" => {
                    std::fs::remove_dir_all(&dir)?;
                    crate::backend::install(name)?;
                    continue;
                }
                other => {
                    eprintln!("warning: unknown fetching method '{other}' for '{name}'");
                    continue;
                }
            }

            db::index_backend(def)?;
            println!("Backend '{name}' updated.");
        }

        if !any {
            println!("No backends installed. Use 'uman install <backend>' first.");
        }
    }

    Ok(())
}