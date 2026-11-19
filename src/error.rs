use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid HTP-1 settings or command")]
    Invalid,
    #[error("WebSocket transport error: {0}")]
    Transport(String),
    #[error("timed out waiting for HTP-1 response")]
    Timeout,
    #[error("invalid or unexpected HTP-1 protocol message")]
    Protocol,
    #[error("HTP-1 connection closed")]
    Disconnected,
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
