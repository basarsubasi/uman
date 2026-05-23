use std::process::Command;

use crate::error::UmanError;

pub fn find_renderer() -> Result<String, UmanError> {
    for cmd in &["man", "mandoc"] {
        if Command::new(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .env("MANPATH", "")
            .arg("-w")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Ok(cmd.to_string());
        }
    }
    Err(UmanError::NoRenderer)
}

pub fn read(backend_name: &str, section: &str, topic: &str) -> anyhow::Result<()> {
    crate::paths::validate_backend_name(backend_name)?;

    let renderer = find_renderer()?;
    let backend_path = crate::paths::backend_dir(backend_name);

    if !backend_path.exists() {
        return Err(UmanError::BackendNotInstalled(backend_name.to_string()).into());
    }

    let manpath = format!("{}:", backend_path.display());

    let status = Command::new(&renderer)
        .args([section, topic])
        .env("MANPATH", &manpath)
        .status()?;

    if !status.success() {
        anyhow::bail!("man page not found: {backend_name} {section} {topic}");
    }
    Ok(())
}