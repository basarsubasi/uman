use std::process::Command;

use crate::config::Config;
use crate::error::UmanError;
use crate::paths;

pub fn install(name: &str) -> anyhow::Result<()> {
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
        .status()?;

    if !status.success() {
        return Err(UmanError::CommandFailed(format!("git clone {source}")).into());
    }
    Ok(())
}

fn install_curl(source: &str, dest: &std::path::Path) -> anyhow::Result<()> {
    let tmp_dir = std::env::temp_dir().join("uman-download");
    std::fs::create_dir_all(&tmp_dir)?;
    let tmp_file = tmp_dir.join("archive");

    let status = Command::new("curl")
        .args(["-L", "-o"])
        .arg(&tmp_file)
        .arg(source)
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
        .status()?;

    if !extract_status.success() {
        return Err(UmanError::CommandFailed("tar extract".to_string()).into());
    }

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

pub fn remove(name: &str) -> anyhow::Result<()> {
    let dest = paths::backend_dir(name);
    if !dest.exists() {
        return Err(UmanError::BackendNotInstalled(name.to_string()).into());
    }

    std::fs::remove_dir_all(&dest)?;

    crate::db::remove_backend_entries(name)?;

    println!("Backend '{name}' removed.");
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