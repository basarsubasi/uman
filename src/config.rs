use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::UnimanError;
use crate::paths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FetchMethod {
    Git,
    Curl,
}

impl FromStr for FetchMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "git" => Ok(FetchMethod::Git),
            "curl" => Ok(FetchMethod::Curl),
            other => Err(format!(
                "unknown fetching method '{}': must be 'git' or 'curl'",
                other
            )),
        }
    }
}

impl fmt::Display for FetchMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchMethod::Git => write!(f, "git"),
            FetchMethod::Curl => write!(f, "curl"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ManFormat {
    Roff,
}

impl FromStr for ManFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "roff" => Ok(ManFormat::Roff),
            other => Err(format!(
                "unknown format '{}': currently only 'roff' is supported",
                other
            )),
        }
    }
}

impl fmt::Display for ManFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManFormat::Roff => write!(f, "roff"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendDef {
    pub name: String,
    pub source: String,
    #[serde(default = "default_format", deserialize_with = "deserialize_format")]
    pub format: ManFormat,
    #[serde(default = "default_fetching", deserialize_with = "deserialize_fetching")]
    pub fetching: FetchMethod,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

fn default_format() -> ManFormat {
    ManFormat::Roff
}

fn default_fetching() -> FetchMethod {
    FetchMethod::Git
}

fn deserialize_format<'de, D>(deserializer: D) -> Result<ManFormat, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    ManFormat::from_str(&s).map_err(serde::de::Error::custom)
}

fn deserialize_fetching<'de, D>(deserializer: D) -> Result<FetchMethod, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    FetchMethod::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub backends: BTreeMap<String, BackendDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_backend: Option<String>,
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

    pub fn get_backend(&self, name: &str) -> Result<&BackendDef, UnimanError> {
        self.backends
            .get(name)
            .ok_or_else(|| UnimanError::BackendNotFound(name.to_string()))
    }

    pub fn resolve(&self, name: &str) -> Result<&BackendDef, UnimanError> {
        if let Some(def) = self.backends.get(name) {
            return Ok(def);
        }
        for def in self.backends.values() {
            if def.aliases.iter().any(|a| a == name) {
                return Ok(def);
            }
        }
        // Also check defaults for well-known aliases
        let defaults = Self::defaults();
        for def in defaults.backends.values() {
            if def.aliases.iter().any(|a| a == name) {
                if let Some(our_def) = self.backends.get(&def.name) {
                    return Ok(our_def);
                }
            }
        }
        Err(UnimanError::BackendNotFound(name.to_string()))
    }

    pub fn get_default_backend(&self) -> Result<&BackendDef, UnimanError> {
        let name = self
            .default_backend
            .as_ref()
            .ok_or(UnimanError::NoDefaultBackend)?;
        let def = self.resolve(name).map_err(|_| {
            UnimanError::DefaultNotFoundInConfig(name.clone())
        })?;
        if !crate::paths::backend_dir(&def.name).exists() {
            return Err(UnimanError::DefaultNotInstalled(def.name.clone()));
        }
        Ok(def)
    }

    pub fn defaults() -> Self {
        let mut backends = BTreeMap::new();
        backends.insert(
            "netbsd".to_string(),
            BackendDef {
                name: "netbsd".to_string(),
                source: "https://github.com/basarsubasi/netbsd-man.git".to_string(),
                format: ManFormat::Roff,
                fetching: FetchMethod::Git,
                aliases: vec!["netbsd".to_string()],
            },
        );
        backends.insert(
            "openbsd".to_string(),
            BackendDef {
                name: "openbsd".to_string(),
                source: "https://github.com/basarsubasi/openbsd-man.git".to_string(),
                format: ManFormat::Roff,
                fetching: FetchMethod::Git,
                aliases: vec!["openbsd".to_string()],
            },
        );
        backends.insert(
            "freebsd".to_string(),
            BackendDef {
                name: "freebsd".to_string(),
                source: "https://github.com/basarsubasi/freebsd-man.git".to_string(),
                format: ManFormat::Roff,
                fetching: FetchMethod::Git,
                aliases: vec!["freebsd".to_string(), "bsd".to_string()],
            },
        );
        backends.insert(
            "macos".to_string(),
            BackendDef {
                name: "macos".to_string(),
                source: "https://github.com/basarsubasi/apple-man.git".to_string(),
                format: ManFormat::Roff,
                fetching: FetchMethod::Git,
                aliases: vec!["macos".to_string(), "darwin".to_string(), "apple".to_string()],
            },
        );
        backends.insert(
            "linux-upstream".to_string(),
            BackendDef {
                name: "linux-upstream".to_string(),
                source: "https://git.kernel.org/pub/scm/docs/man-pages/man-pages.git".to_string(),
                format: ManFormat::Roff,
                fetching: FetchMethod::Git,
                aliases: vec!["linux".to_string(), "goat".to_string()],
            },
        );
        let default_backend_name = match std::env::consts::OS {
            "macos" => "macos",
            "freebsd" => "freebsd",
            "netbsd" => "netbsd",
            "openbsd" => "openbsd",
            _ => "linux-upstream", // fallback to linux for "linux" and others
        };

        Self {
            backends,
            default_backend: Some(default_backend_name.to_string()),
        }
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
        assert!(config.backends.contains_key("netbsd"));
        assert!(config.backends.contains_key("openbsd"));
        assert!(config.backends.contains_key("apple"));
        assert_eq!(config.backends.len(), 5);
    }

    #[test]
    fn default_backend_fields() {
        let config = Config::defaults();
        let linux = config.backends.get("linux-upstream").unwrap();
        assert_eq!(linux.name, "linux-upstream");
        assert_eq!(linux.source, "https://git.kernel.org/pub/scm/docs/man-pages/man-pages.git");
        assert_eq!(linux.format, ManFormat::Roff);
        assert_eq!(linux.fetching, FetchMethod::Git);
        assert!(linux.aliases.contains(&"linux".to_string()));
    }

    #[test]
    fn defaults_has_default_backend() {
        let config = Config::defaults();
        assert_eq!(config.default_backend.as_deref(), Some("linux-upstream"));
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
            UnimanError::BackendNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("expected BackendNotFound, got {:?}", other),
        }
    }

    #[test]
    fn resolve_by_name() {
        let config = Config::defaults();
        let def = config.resolve("linux-upstream").unwrap();
        assert_eq!(def.name, "linux-upstream");
    }

    #[test]
    fn resolve_by_alias() {
        let config = Config::defaults();
        let def = config.resolve("linux").unwrap();
        assert_eq!(def.name, "linux-upstream");
    }

    #[test]
    fn resolve_by_alias_freebsd() {
        let config = Config::defaults();
        let def = config.resolve("freebsd").unwrap();
        assert_eq!(def.name, "freebsd");
    }

    #[test]
    fn resolve_not_found() {
        let config = Config::defaults();
        let result = config.resolve("nope");
        assert!(result.is_err());
        match result.unwrap_err() {
            UnimanError::BackendNotFound(name) => assert_eq!(name, "nope"),
            other => panic!("expected BackendNotFound, got {:?}", other),
        }
    }

    #[test]
    fn get_default_backend_none_set() {
        let mut config = Config::defaults();
        config.default_backend = None;
        let result = config.get_default_backend();
        assert!(result.is_err());
        match result.unwrap_err() {
            UnimanError::NoDefaultBackend => {}
            other => panic!("expected NoDefaultBackend, got {:?}", other),
        }
    }

    #[test]
    fn get_default_backend_set_but_not_installed() {
        let mut config = Config::defaults();
        config.backends.insert("not-installed-test".to_string(), BackendDef {
            name: "not-installed-test".to_string(),
            source: "".to_string(),
            format: ManFormat::Roff,
            fetching: FetchMethod::Git,
            aliases: vec![],
        });
        config.default_backend = Some("not-installed-test".to_string());
        let result = config.get_default_backend();
        assert!(result.is_err());
        match result.unwrap_err() {
            UnimanError::DefaultNotInstalled(name) => assert_eq!(name, "not-installed-test"),
            other => panic!("expected DefaultNotInstalled, got {:?}", other),
        }
    }

    #[test]
    fn get_default_backend_not_in_config() {
        let mut config = Config::defaults();
        config.default_backend = Some("totally-bogus".to_string());
        let result = config.get_default_backend();
        assert!(result.is_err());
        match result.unwrap_err() {
            UnimanError::DefaultNotFoundInConfig(name) => assert_eq!(name, "totally-bogus"),
            other => panic!("expected DefaultNotFoundInConfig, got {:?}", other),
        }
    }

    #[test]
    fn get_default_backend_alias_not_in_config() {
        // If we set default to an alias that resolves to something in config,
        // but the resolved name is not installed, we get DefaultNotInstalled.
        // If default is set to something that can't resolve at all, we get DefaultNotFoundInConfig.
        let mut config = Config::defaults();
        config.default_backend = Some("nonexistent-alias".to_string());
        let result = config.get_default_backend();
        assert!(result.is_err());
        match result.unwrap_err() {
            UnimanError::DefaultNotFoundInConfig(name) => assert_eq!(name, "nonexistent-alias"),
            other => panic!("expected DefaultNotFoundInConfig, got {:?}", other),
        }
    }

    #[test]
    fn get_default_backend_with_alias() {
        let mut config = Config::defaults();
        config.default_backend = Some("linux".to_string());
        let def = config.resolve("linux").unwrap();
        assert_eq!(def.name, "linux-upstream");
    }

    #[test]
    fn config_serialization_roundtrip() {
        let mut config = Config::defaults();
        config.default_backend = None;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.backends.len(), config.backends.len());
        assert!(deserialized.backends.contains_key("linux-upstream"));
        assert!(deserialized.backends.contains_key("freebsd"));
        assert!(deserialized.default_backend.is_none());
    }

    #[test]
    fn config_with_default_backend_roundtrip() {
        let mut config = Config::defaults();
        config.default_backend = Some("linux-upstream".to_string());
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.default_backend, Some("linux-upstream".to_string()));
    }

    #[test]
    fn config_pretty_json_structure() {
        let config = Config::defaults();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["backends"]["linux-upstream"]["source"].is_string());
        assert!(parsed["backends"]["freebsd"]["format"].is_string());
    }

    #[test]
    fn custom_backend_def_roundtrip() {
        let def = BackendDef {
            name: "test-backend".to_string(),
            source: "https://example.com/repo".to_string(),
            format: ManFormat::Roff,
            fetching: FetchMethod::Curl,
            aliases: vec!["tb".to_string(), "test".to_string()],
        };
        let json = serde_json::to_string(&def).unwrap();
        let parsed: BackendDef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test-backend");
        assert_eq!(parsed.source, "https://example.com/repo");
        assert_eq!(parsed.format, ManFormat::Roff);
        assert_eq!(parsed.fetching, FetchMethod::Curl);
        assert_eq!(parsed.aliases, vec!["tb", "test"]);
    }

    #[test]
    fn backend_def_without_aliases_roundtrip() {
        let json = r#"{
            "name": "minimal",
            "source": "https://example.com",
            "format": "roff",
            "fetching": "git"
        }"#;
        let parsed: BackendDef = serde_json::from_str(json).unwrap();
        assert!(parsed.aliases.is_empty());
    }

    #[test]
    fn backends_are_sorted_in_btreemap() {
        let config = Config::defaults();
        let keys: Vec<&String> = config.backends.keys().collect();
        assert_eq!(keys, &["apple", "freebsd", "linux-upstream", "netbsd", "openbsd"]);
    }

    #[test]
    fn fetch_method_from_str() {
        assert_eq!(FetchMethod::from_str("git").unwrap(), FetchMethod::Git);
        assert_eq!(FetchMethod::from_str("curl").unwrap(), FetchMethod::Curl);
        assert!(FetchMethod::from_str("svn").is_err());
        assert!(FetchMethod::from_str("http").is_err());
    }

    #[test]
    fn fetch_method_display() {
        assert_eq!(FetchMethod::Git.to_string(), "git");
        assert_eq!(FetchMethod::Curl.to_string(), "curl");
    }

    #[test]
    fn man_format_from_str() {
        assert_eq!(ManFormat::from_str("roff").unwrap(), ManFormat::Roff);
        assert!(ManFormat::from_str("html").is_err());
    }

    #[test]
    fn man_format_display() {
        assert_eq!(ManFormat::Roff.to_string(), "roff");
    }

    #[test]
    fn config_rejects_invalid_fetching() {
        let json = r#"{
            "backends": {
                "bad": {
                    "name": "bad",
                    "source": "https://example.com",
                    "format": "roff",
                    "fetching": "ftp"
                }
            }
        }"#;
        let result: Result<Config, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn config_rejects_invalid_format() {
        let json = r#"{
            "backends": {
                "bad": {
                    "name": "bad",
                    "source": "https://example.com",
                    "format": "html",
                    "fetching": "git"
                }
            }
        }"#;
        let result: Result<Config, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}