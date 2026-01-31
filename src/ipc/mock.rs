use crate::ipc::traits::IpcClient;
use crate::types::errors::IpcError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock IPC client for testing.
/// Configure responses per endpoint, then verify calls were made.
#[derive(Clone, Default)]
pub struct MockIpcClient {
    /// Predefined responses for each endpoint
    responses: Arc<Mutex<HashMap<String, String>>>,
    /// Record of all requests made (endpoint -> request bodies)
    requests: Arc<Mutex<Vec<(String, String)>>>,
    /// If true, all requests fail with NotConnected
    disconnected: Arc<Mutex<bool>>,
}

impl MockIpcClient {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure a response for a specific endpoint.
    pub fn when_called(&self, endpoint: &str, response: impl serde::Serialize) {
        let json = serde_json::to_string(&response).unwrap();
        self.responses
            .lock()
            .unwrap()
            .insert(endpoint.to_string(), json);
    }

    /// Simulate backend being unavailable.
    pub fn set_disconnected(&self, disconnected: bool) {
        *self.disconnected.lock().unwrap() = disconnected;
    }

    /// Get all requests made to a specific endpoint.
    pub fn requests_to(&self, endpoint: &str) -> Vec<String> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|(e, _)| e == endpoint)
            .map(|(_, body)| body.clone())
            .collect()
    }

    /// Verify a request was made to an endpoint.
    pub fn assert_called(&self, endpoint: &str) {
        let calls = self.requests_to(endpoint);
        assert!(
            !calls.is_empty(),
            "Expected call to {} but none found",
            endpoint
        );
    }

    /// Verify no requests were made.
    pub fn assert_no_calls(&self) {
        let requests = self.requests.lock().unwrap();
        assert!(
            requests.is_empty(),
            "Expected no calls but found: {:?}",
            requests
        );
    }
}

#[async_trait]
impl IpcClient for MockIpcClient {
    async fn request<Req, Res>(&self, endpoint: &str, request: &Req) -> Result<Res, IpcError>
    where
        Req: serde::Serialize + Send + Sync,
        Res: serde::de::DeserializeOwned,
    {
        // Check if simulating disconnection
        if *self.disconnected.lock().unwrap() {
            return Err(IpcError::NotConnected);
        }

        // Record the request
        let request_json =
            serde_json::to_string(request).map_err(|e| IpcError::RequestFailed(e.to_string()))?;
        self.requests
            .lock()
            .unwrap()
            .push((endpoint.to_string(), request_json));

        // Return configured response
        let responses = self.responses.lock().unwrap();
        let response_json = responses
            .get(endpoint)
            .ok_or_else(|| IpcError::RequestFailed(format!("No mock response for {}", endpoint)))?;

        serde_json::from_str(response_json).map_err(|e| IpcError::InvalidResponse(e.to_string()))
    }

    async fn health_check(&self) -> Result<(), IpcError> {
        if *self.disconnected.lock().unwrap() {
            Err(IpcError::NotConnected)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestRequest {
        id: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestResponse {
        status: String,
    }

    #[tokio::test]
    async fn test_mock_returns_configured_response() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/test",
            TestResponse {
                status: "ok".into(),
            },
        );

        let req = TestRequest { id: "123".into() };
        let res: TestResponse = mock.request("/test", &req).await.unwrap();

        assert_eq!(res.status, "ok");
    }

    #[tokio::test]
    async fn test_mock_records_requests() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/test",
            TestResponse {
                status: "ok".into(),
            },
        );

        let req = TestRequest { id: "abc".into() };
        let _: TestResponse = mock.request("/test", &req).await.unwrap();

        mock.assert_called("/test");
        let requests = mock.requests_to("/test");
        assert!(requests[0].contains("abc"));
    }

    #[tokio::test]
    async fn test_mock_disconnected_returns_error() {
        let mock = MockIpcClient::new();
        mock.set_disconnected(true);

        let req = TestRequest { id: "123".into() };
        let result: Result<TestResponse, _> = mock.request("/test", &req).await;

        assert!(matches!(result, Err(IpcError::NotConnected)));
    }

    #[tokio::test]
    async fn test_health_check_succeeds_when_connected() {
        let mock = MockIpcClient::new();
        assert!(mock.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_health_check_fails_when_disconnected() {
        let mock = MockIpcClient::new();
        mock.set_disconnected(true);
        assert!(matches!(
            mock.health_check().await,
            Err(IpcError::NotConnected)
        ));
    }
}
