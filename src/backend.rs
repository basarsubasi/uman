use std::process::Command;

use crate::config::Config;
use crate::error::UmanError;
use crate::paths;

pub fn install(name: &str) -> anyhow::Result<()> {
    paths::validate_backend_name(name)?;

    let config = Config::load()?;
    let backend = config.get_backend(name)?;
    let dest = paths::backend_dir(&backend.name);

    if dest.exists() {
        return Err(UmanError::BackendAlreadyInstalled(name.to_string()).into());
    }

    paths::ensure_dirs()?;

    match backend.fetching.as_str() {
        "git" => install_git(&backend.source, &dest)?,
        "curl" => install_curl(&backend.source, &dest)?,
        other => anyhow::bail!("unknown fetching method: {other}"),
    }

    println!("Backend '{name}' installed successfully.");
    crate::db::index_backend(&backend)?;
    Ok(())
}

fn install_git(source: &str, dest: &std::path::Path) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["clone", "--depth", "1", source])
        .arg(dest)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if !status.success() {
        return Err(UmanError::CommandFailed(format!("git clone {source}")).into());
    }
    Ok(())
}

fn install_curl(source: &str, dest: &std::path::Path) -> anyhow::Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    let tmp_file = tmp_dir.path().join("archive");

    let status = Command::new("curl")
        .args(["-Lsf", "-o"])
        .arg(&tmp_file)
        .arg(source)
        .stdout(std::process::Stdio::null())
        .status()?;

    if !status.success() {
        return Err(UmanError::CommandFailed(format!("curl -L {source}")).into());
    }

    std::fs::create_dir_all(dest)?;
    let extract_status = Command::new("tar")
        .args(["-xf"])
        .arg(&tmp_file)
        .arg("-C")
        .arg(dest)
        .stdout(std::process::Stdio::null())
        .status()?;

    if !extract_status.success() {
        return Err(UmanError::CommandFailed("tar extract".to_string()).into());
    }

    Ok(())
}

pub fn remove(name: &str) -> anyhow::Result<()> {
    paths::validate_backend_name(name)?;

    let dest = paths::backend_dir(name);
    if !dest.exists() {
        return Err(UmanError::BackendNotInstalled(name.to_string()).into());
    }

    std::fs::remove_dir_all(&dest)?;

    crate::db::remove_backend_entries(name)?;

    println!("Backend '{name}' removed.");
    Ok(())
}

pub fn update(name: Option<&str>) -> anyhow::Result<()> {
    let config = Config::load()?;

    if let Some(name) = name {
        paths::validate_backend_name(name)?;
        let def = config.get_backend(name)?;
        let dir = paths::backend_dir(name);
        if !dir.exists() {
            return Err(UmanError::BackendNotInstalled(name.to_string()).into());
        }

        update_single(def, &dir)?;
        println!("Backend '{name}' updated.");
    } else {
        let mut any = false;
        for (name, def) in &config.backends {
            let dir = paths::backend_dir(name);
            if !dir.exists() {
                continue;
            }
            any = true;

            match update_single(def, &dir) {
                Ok(()) => println!("Backend '{name}' updated."),
                Err(e) => eprintln!("warning: failed to update '{name}': {e}"),
            }
        }

        if !any {
            println!("No backends installed. Use 'uman install <backend>' first.");
        }
    }

    Ok(())
}

fn update_single(def: &crate::config::BackendDef, dir: &std::path::Path) -> anyhow::Result<()> {
    match def.fetching.as_str() {
        "git" => {
            let status = Command::new("git")
                .args(["-C"])
                .arg(dir)
                .args(["pull"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()?;

            if !status.success() {
                anyhow::bail!("git pull failed for '{}'", def.name);
            }
            crate::db::index_backend(def)?;
        }
        "curl" => {
            let name = &def.name;
            let dest = paths::backend_dir(name);
            std::fs::remove_dir_all(&dest)?;
            install(name)?;
        }
        other => anyhow::bail!("unknown fetching method: {other}"),
    }
    Ok(())
}

pub fn list() -> anyhow::Result<()> {
    let config = Config::load()?;
    if config.backends.is_empty() {
        println!("No backends configured.");
        return Ok(());
    }

    println!("{:<20} {:<10} {}", "NAME", "STATUS", "SOURCE");
    for (name, def) in &config.backends {
        let installed = paths::backend_dir(name).exists();
        let status = if installed { "installed" } else { "available" };
        println!("{:<20} {:<10} {}", name, status, def.source);
    }
    Ok(())
}