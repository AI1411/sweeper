use thiserror::Error;

#[derive(Debug, Error)]
pub enum SweeperError {
    #[error("process not found: {0}")]
    ProcessNotFound(String),
    #[error("port not in use: {0}")]
    PortNotInUse(u16),
    #[error("protected process: {0} (pid {1})")]
    Protected(String, u32),
    #[error("lsof failed: {0}")]
    Lsof(String),
    #[error("kill failed for pid {0}: {1}")]
    Kill(u32, String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SweeperError>;
