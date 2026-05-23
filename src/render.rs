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

    let output = Command::new(&renderer)
        .args([section, topic])
        .env("MANPATH", &manpath)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let mut detail = stderr;
        if !stdout.is_empty() {
            if !detail.is_empty() {
                detail.push('\n');
            }
            detail.push_str(&stdout);
        }
        if detail.is_empty() {
            detail = format!("exit code {}", output.status.code().unwrap_or(-1));
        }
        anyhow::bail!("man page not found: {backend_name} {section} {topic}: {detail}");
    }

    // Print the man page output to stdout
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_backend_name_rejects_traversal_in_read() {
        let result = crate::paths::validate_backend_name("../../etc");
        assert!(result.is_err());
    }

    #[test]
    fn find_renderer_returns_result() {
        let _ = find_renderer();
    }

    #[test]
    fn manpath_format_includes_trailing_colon() {
        let backend_path = "/some/path";
        let manpath = format!("{}:", backend_path);
        assert!(manpath.ends_with(':'));
        assert_eq!(manpath, "/some/path:");
    }
}