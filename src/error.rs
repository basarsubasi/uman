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