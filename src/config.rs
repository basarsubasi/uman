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

    pub fn defaults() -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_contains_expected_backends() {
        let config = Config::defaults();
        assert!(config.backends.contains_key("linux-upstream"));
        assert!(config.backends.contains_key("freebsd"));
        assert_eq!(config.backends.len(), 2);
    }

    #[test]
    fn default_backend_fields() {
        let config = Config::defaults();
        let linux = config.backends.get("linux-upstream").unwrap();
        assert_eq!(linux.name, "linux-upstream");
        assert_eq!(linux.source, "https://github.com/mkerrisk/man-pages");
        assert_eq!(linux.format, "roff");
        assert_eq!(linux.fetching, "git");
    }

    #[test]
    fn get_backend_found() {
        let config = Config::defaults();
        let result = config.get_backend("linux-upstream");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "linux-upstream");
    }

    #[test]
    fn get_backend_not_found() {
        let config = Config::defaults();
        let result = config.get_backend("nonexistent");
        assert!(result.is_err());
        match result.unwrap_err() {
            UmanError::BackendNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("expected BackendNotFound, got {:?}", other),
        }
    }

    #[test]
    fn config_serialization_roundtrip() {
        let config = Config::defaults();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.backends.len(), config.backends.len());
        assert!(deserialized.backends.contains_key("linux-upstream"));
        assert!(deserialized.backends.contains_key("freebsd"));
    }

    #[test]
    fn config_pretty_json_structure() {
        let config = Config::defaults();
        let json = serde_json::to_string_pretty(&config).unwrap();
        // Verify it's valid JSON with expected structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["backends"]["linux-upstream"]["source"].is_string());
        assert!(parsed["backends"]["freebsd"]["format"].is_string());
    }

    #[test]
    fn custom_backend_def_roundtrip() {
        let def = BackendDef {
            name: "test-backend".to_string(),
            source: "https://example.com/repo".to_string(),
            format: "roff".to_string(),
            fetching: "curl".to_string(),
        };
        let json = serde_json::to_string(&def).unwrap();
        let parsed: BackendDef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test-backend");
        assert_eq!(parsed.source, "https://example.com/repo");
        assert_eq!(parsed.format, "roff");
        assert_eq!(parsed.fetching, "curl");
    }

    #[test]
    fn backends_are_sorted_in_btreemap() {
        let config = Config::defaults();
        let keys: Vec<&String> = config.backends.keys().collect();
        assert_eq!(keys, &["freebsd", "linux-upstream"]); // BTreeMap is sorted
    }
}