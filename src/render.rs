use std::process::{Command, Stdio};

use crate::error::UmanError;

pub fn find_renderer() -> Result<String, UmanError> {
    for cmd in &["man", "mandoc"] {
        if Command::new(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
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

fn build_command(
    renderer: &str,
    manpath: &str,
    section: Option<&str>,
    topic: &str,
) -> Command {
    let mut cmd = Command::new(renderer);
    cmd.env("MANPATH", manpath);

    match section {
        Some(sec) => {
            cmd.arg(sec);
        }
        None => {}
    }
    cmd.arg(topic);

    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::piped());

    cmd
}

pub fn read(backend_name: &str, section: Option<&str>, topic: &str) -> anyhow::Result<()> {
    crate::paths::validate_backend_name(backend_name)?;

    let renderer = find_renderer()?;
    let backend_path = crate::paths::backend_dir(backend_name);

    if !backend_path.exists() {
        return Err(UmanError::BackendNotInstalled(backend_name.to_string()).into());
    }

    let resolved_section = match section {
        Some(s) => Some(s.to_string()),
        None => crate::db::find_page(backend_name, topic)?
            .map(|(sec, _)| sec.to_string()),
    };

    let manpath = format!("{}:", backend_path.display());

    let mut cmd = build_command(
        &renderer,
        &manpath,
        resolved_section.as_deref(),
        topic,
    );

    let child = cmd.spawn()?;
    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() {
            format!("exit code {}", output.status.code().unwrap_or(-1))
        } else {
            stderr
        };

        match &resolved_section {
            Some(sec) => anyhow::bail!(
                "man page not found: {backend_name} {sec} {topic}: {detail}"
            ),
            None => anyhow::bail!(
                "man page not found: {backend_name} {topic}: {detail}"
            ),
        }
    }

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
    fn read_with_section_some() {
        fn _type_check() {
            let _: fn(&str, Option<&str>, &str) -> anyhow::Result<()> = read;
        }
    }

    #[test]
    fn read_with_section_none() {
        fn _type_check() {
            let _: fn(&str, Option<&str>, &str) -> anyhow::Result<()> = read;
        }
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

    #[test]
    fn build_command_with_section_sets_manpath() {
        let cmd = build_command("man", "/man:", Some("2"), "execve");
        let manpath_val = cmd.get_envs()
            .find(|(k, _)| k.to_str() == Some("MANPATH"))
            .and_then(|(_, v)| v);
        assert_eq!(manpath_val, Some(std::ffi::OsStr::new("/man:")));
    }

    #[test]
    fn build_command_without_section_sets_manpath() {
        let cmd = build_command("man", "/man:", None, "execve");
        let manpath_val = cmd.get_envs()
            .find(|(k, _)| k.to_str() == Some("MANPATH"))
            .and_then(|(_, v)| v);
        assert_eq!(manpath_val, Some(std::ffi::OsStr::new("/man:")));
    }

    #[test]
    fn read_rejects_invalid_backend_name() {
        let result = read("..", Some("2"), "open");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid backend name"));
    }

    #[test]
    fn read_rejects_nonexistent_backend() {
        let result = read("nonexistent_backend_xyz", Some("2"), "open");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not installed") || err_msg.contains("not found"));
    }
}