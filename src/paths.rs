use std::path::PathBuf;

fn home_dir() -> PathBuf {
    std::env::var("UMAN_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(PathBuf::from))
        .unwrap_or_else(|_| dirs::home_dir().expect("Could not determine home directory"))
}

pub fn config_dir() -> PathBuf {
    home_dir().join(".config").join("uman")
}

pub fn data_dir() -> PathBuf {
    home_dir().join(".uman")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn backends_dir() -> PathBuf {
    data_dir().join("backends")
}

pub fn index_dir() -> PathBuf {
    data_dir().join("index")
}

pub fn db_path() -> PathBuf {
    index_dir().join("uman.db")
}

pub fn backend_dir(name: &str) -> PathBuf {
    backends_dir().join(name)
}

pub fn ensure_dirs() -> anyhow::Result<()> {
    std::fs::create_dir_all(config_dir())?;
    std::fs::create_dir_all(backends_dir())?;
    std::fs::create_dir_all(index_dir())?;
    Ok(())
}

static BACKEND_NAME_PATTERN: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap());

pub fn validate_backend_name(name: &str) -> anyhow::Result<()> {
    if !BACKEND_NAME_PATTERN.is_match(name) {
        anyhow::bail!(
            "invalid backend name '{}': must contain only letters, numbers, hyphens, and underscores",
            name
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_valid_backend_names() {
        assert!(validate_backend_name("linux-upstream").is_ok());
        assert!(validate_backend_name("freebsd").is_ok());
        assert!(validate_backend_name("my_backend").is_ok());
        assert!(validate_backend_name("backend123").is_ok());
        assert!(validate_backend_name("a").is_ok());
        assert!(validate_backend_name("A-B_C-0").is_ok());
    }

    #[test]
    fn reject_invalid_backend_names() {
        // Path traversal
        assert!(validate_backend_name("../../etc").is_err());
        assert!(validate_backend_name("../etc").is_err());
        assert!(validate_backend_name(".").is_err());

        // Spaces
        assert!(validate_backend_name("my backend").is_err());

        // Special characters
        assert!(validate_backend_name("backend!").is_err());
        assert!(validate_backend_name("backend@host").is_err());
        assert!(validate_backend_name("backend#1").is_err());
        assert!(validate_backend_name("back\\end").is_err());

        // Empty
        assert!(validate_backend_name("").is_err());

        // Unicode
        assert!(validate_backend_name("bäckend").is_err());

        // Slash
        assert!(validate_backend_name("back/end").is_err());

        // Null byte concept — name with control chars
        assert!(validate_backend_name("back\tend").is_err());
    }

    #[test]
    fn path_functions_return_expected_structure() {
        // These tests verify that the path functions produce
        // paths relative to the home directory structure
        let dir = data_dir();
        assert!(dir.to_string_lossy().ends_with(".uman"));
        assert!(backends_dir().starts_with(&dir));
        assert!(index_dir().starts_with(&dir));

        let cdir = config_dir();
        assert!(cdir.to_string_lossy().ends_with("uman"));
        assert!(config_path().starts_with(&cdir));

        assert!(db_path().to_string_lossy().ends_with("uman.db"));
        assert_eq!(backend_dir("test"), backends_dir().join("test"));
    }
}