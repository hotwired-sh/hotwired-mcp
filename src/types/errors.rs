use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("Backend not running. Please start the Hotwired app.")]
    NotConnected,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Timeout waiting for response")]
    Timeout,
}
