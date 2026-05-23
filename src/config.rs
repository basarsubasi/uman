use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::UmanError;
use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendDef {
    pub name: String,
    pub source: String,
    pub format: String,
    pub fetching: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub backends: BTreeMap<String, BackendDef>,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = paths::config_path();
        if !path.exists() {
            let config = Self::defaults();
            config.save()?;
            return Ok(config);
        }
        let content = std::fs::read_to_string(&path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = paths::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn get_backend(&self, name: &str) -> Result<&BackendDef, UmanError> {
        self.backends
            .get(name)
            .ok_or_else(|| UmanError::BackendNotFound(name.to_string()))
    }

    fn defaults() -> Self {
        let mut backends = BTreeMap::new();
        backends.insert(
            "linux-upstream".to_string(),
            BackendDef {
                name: "linux-upstream".to_string(),
                source: "https://github.com/mkerrisk/man-pages".to_string(),
                format: "roff".to_string(),
                fetching: "git".to_string(),
            },
        );
        backends.insert(
            "freebsd".to_string(),
            BackendDef {
                name: "freebsd".to_string(),
                source: "https://gitlab.freebsd.org/freebsd/doc-manual.git".to_string(),
                format: "roff".to_string(),
                fetching: "git".to_string(),
            },
        );
        Self { backends }
    }
}