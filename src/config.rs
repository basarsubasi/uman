use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::UmanError;
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
                format: ManFormat::Roff,
                fetching: FetchMethod::Git,
            },
        );
        backends.insert(
            "freebsd".to_string(),
            BackendDef {
                name: "freebsd".to_string(),
                source: "https://gitlab.freebsd.org/freebsd/doc-manual.git".to_string(),
                format: ManFormat::Roff,
                fetching: FetchMethod::Git,
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
        assert_eq!(linux.format, ManFormat::Roff);
        assert_eq!(linux.fetching, FetchMethod::Git);
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
        };
        let json = serde_json::to_string(&def).unwrap();
        let parsed: BackendDef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test-backend");
        assert_eq!(parsed.source, "https://example.com/repo");
        assert_eq!(parsed.format, ManFormat::Roff);
        assert_eq!(parsed.fetching, FetchMethod::Curl);
    }

    #[test]
    fn backends_are_sorted_in_btreemap() {
        let config = Config::defaults();
        let keys: Vec<&String> = config.backends.keys().collect();
        assert_eq!(keys, &["freebsd", "linux-upstream"]);
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