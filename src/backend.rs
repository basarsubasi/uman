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
    let backend = config.get_backend(name)?;
    let dest = paths::backend_dir(&backend.name);

    if dest.exists() {
        return Err(UmanError::BackendAlreadyInstalled(name.to_string()).into());
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
        // Rollback: clean up partial download
        if dest.exists() {
            let _ = std::fs::remove_dir_all(&dest);
        }
        return result;
    }

    println!("Backend '{name}' installed successfully.");
    crate::db::index_backend(&backend)?;
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
        // Clean up partial extraction
        if dest.exists() {
            let _ = std::fs::remove_dir_all(dest);
        }
    }

    result
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

            // Download-first-then-swap: download to temp, verify, then swap
            let name = &def.name;
            let dest = paths::backend_dir(name);
            let staging = paths::backends_dir().join(format!("{}.staging", name));

            // Clean up any leftover staging directory
            if staging.exists() {
                std::fs::remove_dir_all(&staging)?;
            }

            // Download to staging directory
            install_curl(&def.source, &staging)?;

            // Swap: remove old, rename staging
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

    println!("{:<20} {:<10} {} {}", "NAME", "STATUS", "FORMAT", "SOURCE");
    for (name, def) in &config.backends {
        let installed = paths::backend_dir(name).exists();
        let status = if installed { "installed" } else { "available" };
        println!(
            "{:<20} {:<10} {} {}",
            name, status, def.format, def.source
        );
    }
    Ok(())
}