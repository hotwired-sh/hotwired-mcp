use crate::types::errors::IpcError;
use async_trait::async_trait;

/// Trait for communicating with the Hotwired backend.
/// Implementations: UnixSocketClient (production), MockIpcClient (testing)
#[async_trait]
pub trait IpcClient: Send + Sync {
    /// Send a request and receive a response.
    async fn request<Req, Res>(&self, endpoint: &str, request: &Req) -> Result<Res, IpcError>
    where
        Req: serde::Serialize + Send + Sync,
        Res: serde::de::DeserializeOwned;

    /// Check if the backend is available.
    async fn health_check(&self) -> Result<(), IpcError>;
}
