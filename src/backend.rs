use std::process::Command;

use crate::config::{Config, FetchMethod};
use crate::error::UmanError;
use crate::paths;

fn check_command_exists(cmd: &str) -> Result<(), UmanError> {
    if which_exists(cmd) {
        Ok(())
    } else {
        match cmd {
            "git" => Err(UmanError::GitNotFound),
            "curl" => Err(UmanError::CurlNotFound),
            other => Err(UmanError::CommandFailed {
                cmd: other.to_string(),
                stderr: format!("{other} is not installed or not on PATH"),
            }),
        }
    }
}

fn which_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_command(cmd: &str, args: &[&str], error_context: &str) -> anyhow::Result<()> {
    let output = Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let mut detail = String::new();
        if !stderr.is_empty() {
            detail.push_str(&stderr);
        }
        if !stdout.is_empty() {
            if !detail.is_empty() {
                detail.push('\n');
            }
            detail.push_str(&stdout);
        }
        return Err(UmanError::CommandFailed {
            cmd: error_context.to_string(),
            stderr: detail,
        }
        .into());
    }
    Ok(())
}

pub fn install(name: &str) -> anyhow::Result<()> {
    paths::validate_backend_name(name)?;

    let config = Config::load()?;
    let backend = config.resolve(name)?;
    let canonical = &backend.name;
    let dest = paths::backend_dir(canonical);

    if dest.exists() {
        return Err(UmanError::BackendAlreadyInstalled(canonical.to_string()).into());
    }

    paths::ensure_dirs()?;

    let result = match backend.fetching {
        FetchMethod::Git => {
            check_command_exists("git")?;
            install_git(&backend.source, &dest)
        }
        FetchMethod::Curl => {
            check_command_exists("curl")?;
            install_curl(&backend.source, &dest)
        }
    };

    if result.is_err() {
        if dest.exists() {
            let _ = std::fs::remove_dir_all(&dest);
        }
        return result;
    }

    println!("Backend '{canonical}' installed successfully.");
    crate::db::index_backend(backend)?;

    let mut config = Config::load()?;
    if config.default_backend.is_none() {
        config.default_backend = Some(canonical.clone());
        config.save()?;
        println!("Default backend set to '{canonical}'.");
    }

    Ok(())
}

fn install_git(source: &str, dest: &std::path::Path) -> anyhow::Result<()> {
    run_command(
        "git",
        &["clone", "--depth", "1", source, &dest.to_string_lossy()],
        &format!("git clone {}", source),
    )
}

fn install_curl(source: &str, dest: &std::path::Path) -> anyhow::Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    let tmp_file = tmp_dir.path().join("archive");

    run_command(
        "curl",
        &[
            "-Lsf",
            "--connect-timeout",
            "30",
            "--max-time",
            "300",
            "-o",
            &tmp_file.to_string_lossy(),
            source,
        ],
        &format!("curl download {}", source),
    )?;

    std::fs::create_dir_all(dest)?;
    let result = run_command(
        "tar",
        &["-xf", &tmp_file.to_string_lossy(), "-C", &dest.to_string_lossy()],
        "tar extract",
    );

    if result.is_err() {
        if dest.exists() {
            let _ = std::fs::remove_dir_all(dest);
        }
    }

    result
}

pub fn remove(name: &str) -> anyhow::Result<()> {
    paths::validate_backend_name(name)?;

    let config = Config::load()?;
    let backend = config.resolve(name)?;
    let canonical = &backend.name;
    let dest = paths::backend_dir(canonical);

    if !dest.exists() {
        return Err(UmanError::BackendNotInstalled(canonical.to_string()).into());
    }

    std::fs::remove_dir_all(&dest)?;

    crate::db::remove_backend_entries(canonical)?;

    if config.default_backend.as_deref() == Some(canonical) {
        eprintln!("warning: '{canonical}' was the default backend. Set a new default with 'uman backend default <name>'.");
    }

    println!("Backend '{canonical}' removed.");
    Ok(())
}

pub fn update(name: Option<&str>) -> anyhow::Result<()> {
    let config = Config::load()?;

    if let Some(name) = name {
        paths::validate_backend_name(name)?;
        let def = config.resolve(name)?;
        let canonical = &def.name;
        let dir = paths::backend_dir(canonical);
        if !dir.exists() {
            return Err(UmanError::BackendNotInstalled(canonical.to_string()).into());
        }

        update_single(def, &dir)?;
        println!("Backend '{canonical}' updated.");
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
    match def.fetching {
        FetchMethod::Git => {
            check_command_exists("git")?;

            let output = Command::new("git")
                .args(["-C"])
                .arg(dir)
                .args(["pull"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Err(UmanError::CommandFailed {
                    cmd: format!("git pull for '{}'", def.name),
                    stderr,
                }
                .into());
            }
            crate::db::index_backend(def)?;
        }
        FetchMethod::Curl => {
            check_command_exists("curl")?;

            let name = &def.name;
            let dest = paths::backend_dir(name);
            let staging = paths::backends_dir().join(format!("{}.staging", name));

            if staging.exists() {
                std::fs::remove_dir_all(&staging)?;
            }

            install_curl(&def.source, &staging)?;

            if dest.exists() {
                std::fs::remove_dir_all(&dest)?;
            }
            std::fs::rename(&staging, &dest)?;

            crate::db::index_backend(def)?;
        }
    }
    Ok(())
}

pub fn list() -> anyhow::Result<()> {
    let config = Config::load()?;
    if config.backends.is_empty() {
        println!("No backends configured.");
        return Ok(());
    }

    let default_name = config.default_backend.as_deref();
    let mut sorted: Vec<_> = config.backends.iter().collect();
    sorted.sort_by(|a, b| {
        let a_is_default = default_name == Some(a.0.as_str()) || default_name.map(|d| a.1.aliases.contains(&d.to_string())).unwrap_or(false);
        let b_is_default = default_name == Some(b.0.as_str()) || default_name.map(|d| b.1.aliases.contains(&d.to_string())).unwrap_or(false);
        b_is_default.cmp(&a_is_default).then(a.0.cmp(b.0))
    });

    let mut has_default = false;
    println!("{:<20} {:<10} {:<10} {} {}", "NAME", "DEFAULT", "STATUS", "FORMAT", "SOURCE");
    for (name, def) in &sorted {
        let is_default = default_name == Some(name.as_str());
        let default_marker = if is_default { "*" } else { "" };
        if is_default { has_default = true; }
        let installed = paths::backend_dir(name).exists();
        let status = if installed { "installed" } else { "available" };
        println!(
            "{:<20} {:<10} {:<10} {} {}",
            name, default_marker, status, def.format, def.source
        );
    }

    if !has_default && default_name.is_some() {
        println!("\nwarning: default backend '{}' is not installed", default_name.unwrap());
    }

    Ok(())
}

pub fn set_default(name: &str) -> anyhow::Result<()> {
    paths::validate_backend_name(name)?;

    let config = Config::load()?;
    let backend = config.resolve(name)?;
    let canonical = &backend.name;

    if !paths::backend_dir(canonical).exists() {
        return Err(UmanError::DefaultNotInstalled(canonical.clone()).into());
    }

    let mut config = Config::load()?;
    config.default_backend = Some(canonical.clone());
    config.save()?;

    println!("Default backend set to '{canonical}'.");
    Ok(())
}

pub fn show_default() -> anyhow::Result<()> {
    let config = Config::load()?;
    match &config.default_backend {
        Some(name) => {
            let display = match config.resolve(name) {
                Ok(def) => {
                    if def.name != name.as_str() {
                        format!("{} (alias: {})", def.name, name)
                    } else {
                        def.name.clone()
                    }
                }
                Err(_) => name.clone(),
            };
            println!("{}", display);
        }
        None => println!("No default backend set."),
    }
    Ok(())
}