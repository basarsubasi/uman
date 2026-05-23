#[derive(Debug, thiserror::Error)]
pub enum UmanError {
    #[error("backend '{0}' not found in config")]
    BackendNotFound(String),

    #[error("backend '{0}' is already installed")]
    BackendAlreadyInstalled(String),

    #[error("backend '{0}' is not installed")]
    BackendNotInstalled(String),

    #[error("no man page renderer found (install man-db or mandoc)")]
    NoRenderer,

    #[error("git is not installed or not on PATH")]
    #[allow(dead_code)]
    GitNotFound,

    #[error("curl is not installed or not on PATH")]
    #[allow(dead_code)]
    CurlNotFound,

    #[error("command failed: {0}")]
    CommandFailed(String),

    #[error("no backends installed")]
    #[allow(dead_code)]
    NoBackendsInstalled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(
            UmanError::BackendNotFound("test".to_string()).to_string(),
            "backend 'test' not found in config"
        );
        assert_eq!(
            UmanError::BackendAlreadyInstalled("my-backend".to_string()).to_string(),
            "backend 'my-backend' is already installed"
        );
        assert_eq!(
            UmanError::BackendNotInstalled("foo".to_string()).to_string(),
            "backend 'foo' is not installed"
        );
        assert_eq!(
            UmanError::NoRenderer.to_string(),
            "no man page renderer found (install man-db or mandoc)"
        );
        assert_eq!(
            UmanError::CommandFailed("git clone failed".to_string()).to_string(),
            "command failed: git clone failed"
        );
    }

    #[test]
    fn error_variants_contain_data() {
        match UmanError::BackendNotFound("xyz".to_string()) {
            UmanError::BackendNotFound(name) => assert_eq!(name, "xyz"),
            _ => panic!("wrong variant"),
        }
        match UmanError::CommandFailed("oops".to_string()) {
            UmanError::CommandFailed(msg) => assert_eq!(msg, "oops"),
            _ => panic!("wrong variant"),
        }
    }
}