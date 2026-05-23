#[derive(Debug, thiserror::Error)]
pub enum UmanError {
    #[error("backend '{0}' not found in config")]
    BackendNotFound(String),

    #[error("backend '{0}' is already installed")]
    BackendAlreadyInstalled(String),

    #[error("backend '{0}' is not installed")]
    BackendNotInstalled(String),

    #[error("no default backend set; use 'uman backend default <name>' to set one")]
    NoDefaultBackend,

    #[error("default backend '{0}' is not installed; install it or change the default")]
    DefaultNotInstalled(String),

    #[error("no man page renderer found (install man-db or mandoc)")]
    NoRenderer,

    #[error("git is not installed or not on PATH")]
    GitNotFound,

    #[error("curl is not installed or not on PATH")]
    CurlNotFound,

    #[error("command '{cmd}' failed: {stderr}")]
    CommandFailed { cmd: String, stderr: String },

    #[error("no backends installed")]
    NoBackendsInstalled,
}