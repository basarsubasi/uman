use std::path::PathBuf;

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
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