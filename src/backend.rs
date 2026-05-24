use std::io::{self, Write};
use std::process::Command;

use crate::config::{Config, FetchMethod};
use crate::error::UnimanError;
use crate::paths;

fn check_command_exists(cmd: &str) -> Result<(), UnimanError> {
    if which_exists(cmd) {
        Ok(())
    } else {
        match cmd {
            "git" => Err(UnimanError::GitNotFound),
            "curl" => Err(UnimanError::CurlNotFound),
            other => Err(UnimanError::CommandFailed {
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
        return Err(UnimanError::CommandFailed {
            cmd: error_context.to_string(),
            stderr: detail,
        }
        .into());
    }
    Ok(())
}

pub fn install(name: Option<&str>) -> anyhow::Result<()> {
    let config = Config::load()?;

    if let Some(n) = name {
        install_single(&config, n)?;
        
        let mut config = Config::load()?;
        if config.default_backend.is_none() {
            let backend_name = config.resolve(n)?.name.clone();
            config.default_backend = Some(backend_name.clone());
            config.save()?;
            println!("Default backend set to '{}'.", backend_name);
        }
    } else {
        let mut any_installed = false;
        let mut any = false;

        for (backend_name, _) in &config.backends {
            any = true;
            match install_single(&config, backend_name) {
                Ok(()) => {
                    any_installed = true;
                }
                Err(e) => {
                    if e.to_string().contains("already installed") {
                        println!("Backend '{backend_name}' is already installed.");
                    } else {
                        eprintln!("warning: failed to install '{backend_name}': {e}");
                    }
                }
            }
        }

        if !any {
            println!("No backends configured in config.");
        } else if any_installed {
            let mut config = Config::load()?;
            if config.default_backend.is_none() {
                if let Some((name, _)) = config.backends.first_key_value() {
                    config.default_backend = Some(name.clone());
                    config.save()?;
                    println!("Default backend set to '{name}'.");
                }
            }
        }
    }

    Ok(())
}

fn install_single(config: &Config, name: &str) -> anyhow::Result<()> {
    paths::validate_backend_name(name)?;

    let backend = config.resolve(name)?;
    let canonical = &backend.name;
    let dest = paths::backend_dir(canonical);

    if dest.exists() {
        return Err(UnimanError::BackendAlreadyInstalled(canonical.to_string()).into());
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
        return Err(UnimanError::BackendNotInstalled(canonical.to_string()).into());
    }

    if config.default_backend.as_deref() == Some(canonical) {
        if !confirm_default_remove(canonical)? {
            println!("Aborted.");
            return Ok(());
        }
    }

    std::fs::remove_dir_all(&dest)?;

    crate::db::remove_backend_entries(canonical)?;

    if config.default_backend.as_deref() == Some(canonical) {
        eprintln!("warning: '{canonical}' was the default backend. Set a new default with 'uniman default <name>'.");
    }

    println!("Backend '{canonical}' removed.");
    Ok(())
}

fn confirm_default_remove(name: &str) -> anyhow::Result<bool> {
    let mut stdout = io::stdout();
    write!(
        stdout,
        "{} is the default man page backend, do you want to remove it?(Y/n) ",
        name
    )?;
    stdout.flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(parse_confirm_response(&input))
}

fn parse_confirm_response(input: &str) -> bool {
    let trimmed = input.trim().to_lowercase();

    if trimmed.is_empty() || trimmed == "y" || trimmed == "yes" {
        true
    } else if trimmed == "n" || trimmed == "no" {
        false
    } else {
        true
    }
}

#[cfg(test)]
mod confirm_tests {
    use super::parse_confirm_response;

    #[test]
    fn confirm_response_accepts_empty_as_yes() {
        assert!(parse_confirm_response(""));
        assert!(parse_confirm_response("\n"));
        assert!(parse_confirm_response("   \n"));
    }

    #[test]
    fn confirm_response_accepts_yes() {
        assert!(parse_confirm_response("y"));
        assert!(parse_confirm_response("Y"));
        assert!(parse_confirm_response("yes"));
        assert!(parse_confirm_response("YES"));
    }

    #[test]
    fn confirm_response_rejects_no() {
        assert!(!parse_confirm_response("n"));
        assert!(!parse_confirm_response("N"));
        assert!(!parse_confirm_response("no"));
        assert!(!parse_confirm_response("NO"));
    }

    #[test]
    fn confirm_response_defaults_to_yes_for_other_input() {
        assert!(parse_confirm_response("maybe"));
        assert!(parse_confirm_response("lol"));
        assert!(parse_confirm_response("yep"));
    }
}

pub fn update(name: Option<&str>) -> anyhow::Result<()> {
    let config = Config::load()?;

    if let Some(name) = name {
        paths::validate_backend_name(name)?;
        let def = config.resolve(name)?;
        let canonical = &def.name;
        let dir = paths::backend_dir(canonical);
        if !dir.exists() {
            return Err(UnimanError::BackendNotInstalled(canonical.to_string()).into());
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
            println!("No backends installed. Use 'uniman install <backend>' first.");
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
                return Err(UnimanError::CommandFailed {
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
        let a_is_default = is_default_backend(default_name, a.0, &a.1.aliases);
        let b_is_default = is_default_backend(default_name, b.0, &b.1.aliases);
        b_is_default.cmp(&a_is_default).then(a.0.cmp(b.0))
    });

    let mut found_default = false;
    let mut default_canonical: Option<&str> = None;
    println!("{:<20} {:<12} {:<10} {} {}", "NAME", "DEFAULT", "STATUS", "FORMAT", "SOURCE");
    for (name, def) in &sorted {
        let is_default = is_default_backend(default_name, name, &def.aliases);
        let default_marker = if is_default { "*" } else { "" };
        if is_default {
            found_default = true;
            default_canonical = Some(name.as_str());
        }
        let installed = paths::backend_dir(name).exists();
        let status = if installed { "installed" } else { "available" };
        println!(
            "{:<20} {:<12} {:<10} {} {}",
            name, default_marker, status, def.format, def.source
        );
    }

    if let Some(default_name) = default_name {
        if !found_default {
            println!("\nwarning: default backend '{}' is not in config", default_name);
        } else if let Some(canonical) = default_canonical {
            let canonical_dir = paths::backend_dir(canonical);
            if !canonical_dir.exists() {
                println!("\nwarning: default backend '{}' is not installed", canonical);
            }
        }
    }

    Ok(())
}

pub fn list_topics(name: &str, plain_text: bool) -> anyhow::Result<()> {
    paths::validate_backend_name(name)?;

    let config = Config::load()?;
    let backend = config.resolve(name)?;
    let canonical = &backend.name;

    if !paths::backend_dir(canonical).exists() {
        return Err(UnimanError::BackendNotInstalled(canonical.to_string()).into());
    }

    let topics = crate::db::list_topics_for_backend(canonical)?;

    if topics.is_empty() {
        println!("No topics indexed for backend '{canonical}'. Try 'uniman update {canonical}'.");
        return Ok(());
    }

    let lines: Vec<String> = topics
        .iter()
        .map(|(section, topic_name, description)| {
            let display_name = format!("{}({})", topic_name, section);
            // 1=section, 2=name, 3=display_name, 4=description
            format!("{}\t{}\t{:<40}\t{}", section, topic_name, display_name, description)
        })
        .collect();

    let header = format!("{:<40}\t{}", "NAME", "DESCRIPTION");

    if plain_text {
        println!("{}", header);
        for line in &lines {
            println!("{}", line);
        }
        return Ok(());
    }

    crate::fzf::require_fzf()?;

    // Use the running binary's path so the execute command works regardless
    // of how uniman is installed (PATH, full path, cargo run, etc.).
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "uniman".to_string());

    // {1} = section, {2} = name
    let execute_template = format!("{} {} {{1}} {{2}}", exe, canonical);

    crate::fzf::browse(&header, &execute_template, Some("3,4"), &lines)?;

    Ok(())
}

fn is_default_backend(default_name: Option<&str>, backend_key: &str, aliases: &[String]) -> bool {
    match default_name {
        Some(dn) => dn == backend_key || aliases.iter().any(|a| a == dn),
        None => false,
    }
}

pub fn set_default(name: &str) -> anyhow::Result<()> {
    paths::validate_backend_name(name)?;

    let config = Config::load()?;
    let backend = config.resolve(name)?;
    let canonical = &backend.name;

    if !paths::backend_dir(canonical).exists() {
        return Err(UnimanError::DefaultNotInstalled(canonical.clone()).into());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_default_backend_matches_canonical_name() {
        assert!(is_default_backend(Some("linux-upstream"), "linux-upstream", &[]));
        assert!(!is_default_backend(Some("linux-upstream"), "freebsd", &[]));
        assert!(!is_default_backend(None, "linux-upstream", &[]));
    }

    #[test]
    fn is_default_backend_matches_alias() {
        assert!(is_default_backend(Some("linux"), "linux-upstream", &["linux".to_string()]));
        assert!(is_default_backend(Some("bsd"), "freebsd", &["bsd".to_string()]));
        assert!(!is_default_backend(Some("linux"), "freebsd", &[]));
    }

    #[test]
    fn is_default_backend_none_means_no_default() {
        assert!(!is_default_backend(None, "linux-upstream", &["linux".to_string()]));
    }
}