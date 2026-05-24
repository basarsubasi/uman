use uniman::backend;
use uniman::cli;
use uniman::config::Config;
use uniman::deps;
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
    let requires_fzf = !cli.plain_text
        && matches!(
            cli.command,
            Some(cli::Commands::Search { .. }) | Some(cli::Commands::List { backend: Some(_) })
        );
    let requires_renderer = matches!(cli.command, None) && cli.backend.is_some();
    deps::check_dependencies(requires_fzf, requires_renderer)?;

    if let Some(command) = cli.command {
        match command {
            cli::Commands::List { backend } => match backend {
                Some(b) => backend::list_topics(&b, cli.plain_text)?,
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
                backend::install(backend.as_deref())?;
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
                        search::run_keyword(&t, cli.plain_text)?;
                    } else {
                        search::run_filename(&t, cli.plain_text)?;
                    }
                } else {
                    search::run_all(cli.plain_text)?;
                }
            }
        }
    } else if let Some(first) = cli.backend {
        dispatch_read(first, cli.section, cli.topic, cli.plain_text)?;
    } else {
        let mut cmd = cli::Cli::command();
        cmd.print_help()?;
        println!();
    }

    Ok(())
}

fn dispatch_read(
    first: String,
    second: Option<String>,
    third: Option<String>,
    plain_text: bool,
) -> anyhow::Result<()> {
    let config = Config::load()?;
    let backend_def = config.resolve(&first);
    let is_backend = backend_def.is_ok();
    let resolved = backend_def.ok();

    match (resolved, second, third, is_backend, is_numeric(&first)) {
        // uniman <backend> <section> <topic>
        (Some(def), Some(sec), Some(top), _, _) => {
            if plain_text {
                render::read_plain(&def.name, Some(&sec), &top)?;
            } else {
                render::read(&def.name, Some(&sec), &top)?;
            }
        }

        // uniman <backend> <topic>  (2 args, first resolves as backend)
        (Some(def), Some(top), None, _, _) => {
            if plain_text {
                render::read_plain(&def.name, None, &top)?;
            } else {
                render::read(&def.name, None, &top)?;
            }
        }

        // Just a backend name with nothing else — incomplete, show usage
        (Some(_), None, None, _, _) => {
            let mut cmd = cli::Cli::command();
            cmd.print_help()?;
            println!();
            std::process::exit(1);
        }

        // uniman <section> <topic>  (2 args, first is numeric, not a backend)
        (None, Some(top), None, false, true) => {
            let default_def = get_or_prompt_default_backend(&config)?;
            if plain_text {
                render::read_plain(&default_def.name, Some(&first), &top)?;
            } else {
                render::read(&default_def.name, Some(&first), &top)?;
            }
        }

        // uniman <topic>  (1 arg, not a backend, not numeric)
        (None, None, None, false, false) => {
            let default_def = get_or_prompt_default_backend(&config)?;
            if plain_text {
                render::read_plain(&default_def.name, None, &first)?;
            } else {
                render::read(&default_def.name, None, &first)?;
            }
        }

        // Unresolvable
        _ => {
            eprintln!("error: '{}' is not a known backend or alias", first);
            eprintln!();
            let mut cmd = cli::Cli::command();
            cmd.print_help()?;
            println!();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn get_or_prompt_default_backend(config: &uniman::config::Config) -> anyhow::Result<uniman::config::BackendDef> {
    match config.get_default_backend() {
        Ok(def) => Ok(def.clone()),
        Err(uniman::error::UnimanError::DefaultNotInstalled(name)) => {
            use std::io::Write;
            eprint!("Default backend '{name}' is not installed. Install now? [Y/n] ");
            std::io::stderr().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();
            if input.is_empty() || input == "y" || input == "yes" {
                uniman::backend::install(Some(&name))?;
                config.get_default_backend().map(|d| d.clone()).map_err(|e| e.into())
            } else {
                anyhow::bail!("Aborted. Default backend must be installed to read man pages.");
            }
        }
        Err(e) => Err(e.into()),
    }
}
