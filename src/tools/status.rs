use crate::ipc::messages::{
    HandoffRequest, HandoffResponse, ReportImpedimentRequest, ReportImpedimentResponse,
    ReportStatusRequest, ReportStatusResponse, RequestEndRunRequest, RequestEndRunResponse,
    RequestInputRequest, RequestInputResponse, ResolveImpedimentRequest, ResolveImpedimentResponse,
    RespondToEndRequestRequest, RespondToEndRequestResponse, ResponseFormat, SendMessageRequest,
    SendMessageResponse, SuggestedFollowUp, TaskCompleteRequest, TaskCompleteResponse,
};
use crate::ipc::traits::IpcClient;
use crate::types::errors::IpcError;

/// Report the agent's current working status.
pub async fn report_status<C: IpcClient>(
    client: &C,
    run_id: &str,
    status: &str,
    message: &str,
    source: &str,
    agent_status: Option<&str>,
) -> Result<(), IpcError> {
    let request = ReportStatusRequest {
        run_id: run_id.to_string(),
        status: status.to_string(),
        message: message.to_string(),
        source: source.to_string(),
        agent_status: agent_status.map(String::from),
    };

    let endpoint = format!("/api/runs/{}/report-status", run_id);
    let response: ReportStatusResponse = client.request(&endpoint, &request).await?;

    if response.success {
        Ok(())
    } else {
        Err(IpcError::RequestFailed("Status update failed".into()))
    }
}

/// Validate status value.
pub fn validate_status(status: &str) -> Result<(), String> {
    const VALID_STATUSES: &[&str] = &["working", "thinking", "waiting", "idle", "complete"];
    if VALID_STATUSES.contains(&status) {
        Ok(())
    } else {
        Err(format!(
            "Invalid status '{}'. Must be one of: {}",
            status,
            VALID_STATUSES.join(", ")
        ))
    }
}

/// Validate agent status value.
pub fn validate_agent_status(status: &str) -> Result<(), String> {
    const VALID: &[&str] = &["active", "awaiting_response", "blocked", "idle"];
    if VALID.contains(&status) {
        Ok(())
    } else {
        Err(format!(
            "Invalid agent status '{}'. Must be one of: {}",
            status,
            VALID.join(", ")
        ))
    }
}

/// Send a message to other agents or the user.
pub async fn send_message<C: IpcClient>(
    client: &C,
    run_id: &str,
    content: &str,
    source: &str,
    target: Option<&str>,
    agent_status: Option<&str>,
) -> Result<String, IpcError> {
    let request = SendMessageRequest {
        run_id: run_id.to_string(),
        content: content.to_string(),
        source: source.to_string(),
        target: target.map(String::from),
        agent_status: agent_status.map(String::from),
    };

    let endpoint = format!("/api/runs/{}/message", run_id);
    let response: SendMessageResponse = client.request(&endpoint, &request).await?;

    if response.success {
        Ok(response.event_id)
    } else {
        Err(IpcError::RequestFailed("Message send failed".into()))
    }
}

/// Mark a specific task as complete.
pub async fn task_complete<C: IpcClient>(
    client: &C,
    run_id: &str,
    task_description: &str,
    source: &str,
    outcome: Option<&str>,
    agent_status: Option<&str>,
) -> Result<(), IpcError> {
    let request = TaskCompleteRequest {
        run_id: run_id.to_string(),
        task_description: task_description.to_string(),
        source: source.to_string(),
        outcome: outcome.map(String::from),
        agent_status: agent_status.map(String::from),
    };

    let endpoint = format!("/api/runs/{}/task-complete", run_id);
    let response: TaskCompleteResponse = client.request(&endpoint, &request).await?;

    if response.success {
        Ok(())
    } else {
        Err(IpcError::RequestFailed("Task complete failed".into()))
    }
}

/// Validate impediment type value.
pub fn validate_impediment_type(impediment_type: &str) -> Result<(), String> {
    const VALID: &[&str] = &[
        "missing_information",
        "permission_needed",
        "technical_error",
        "unclear_requirements",
        "dependency_blocked",
        "other",
    ];
    if VALID.contains(&impediment_type) {
        Ok(())
    } else {
        Err(format!(
            "Invalid impediment type '{}'. Must be one of: {}",
            impediment_type,
            VALID.join(", ")
        ))
    }
}

/// Report a blocker or impediment that prevents progress.
pub async fn report_impediment<C: IpcClient>(
    client: &C,
    run_id: &str,
    impediment_type: &str,
    description: &str,
    source: &str,
    context: Option<&str>,
    suggestion: Option<&str>,
    agent_status: Option<&str>,
    response_format: Option<ResponseFormat>,
) -> Result<(), IpcError> {
    let request = ReportImpedimentRequest {
        run_id: run_id.to_string(),
        impediment_type: impediment_type.to_string(),
        description: description.to_string(),
        source: source.to_string(),
        context: context.map(String::from),
        suggestion: suggestion.map(String::from),
        agent_status: agent_status.map(String::from),
        response_format,
    };

    let endpoint = format!("/api/runs/{}/impediment", run_id);
    let response: ReportImpedimentResponse = client.request(&endpoint, &request).await?;

    if response.success {
        Ok(())
    } else {
        Err(IpcError::RequestFailed("Report impediment failed".into()))
    }
}

/// Request input or clarification from the human user.
pub async fn request_input<C: IpcClient>(
    client: &C,
    run_id: &str,
    question: &str,
    source: &str,
    context: Option<&str>,
    options: Option<Vec<String>>,
) -> Result<(), IpcError> {
    let request = RequestInputRequest {
        run_id: run_id.to_string(),
        question: question.to_string(),
        source: source.to_string(),
        context: context.map(String::from),
        options,
    };

    let endpoint = format!("/api/runs/{}/input", run_id);
    let response: RequestInputResponse = client.request(&endpoint, &request).await?;

    if response.success {
        Ok(())
    } else {
        Err(IpcError::RequestFailed("Request input failed".into()))
    }
}

/// Hand off work to another agent.
pub async fn handoff<C: IpcClient>(
    client: &C,
    run_id: &str,
    to: &str,
    summary: &str,
    source: &str,
    details: Option<&str>,
    artifacts: Option<Vec<String>>,
) -> Result<(), IpcError> {
    let request = HandoffRequest {
        run_id: run_id.to_string(),
        to: to.to_string(),
        summary: summary.to_string(),
        source: source.to_string(),
        details: details.map(String::from),
        artifacts,
    };

    let endpoint = format!("/api/runs/{}/handoff", run_id);
    let response: HandoffResponse = client.request(&endpoint, &request).await?;

    if response.success {
        Ok(())
    } else {
        Err(IpcError::RequestFailed("Handoff failed".into()))
    }
}

/// Validate end run reason value.
pub fn validate_end_run_reason(reason: &str) -> Result<(), String> {
    const VALID: &[&str] = &["completed", "scope_changed", "pause_requested", "error"];
    if VALID.contains(&reason) {
        Ok(())
    } else {
        Err(format!(
            "Invalid reason '{}'. Must be one of: {}",
            reason,
            VALID.join(", ")
        ))
    }
}

/// Request to end, restart, or pause the workflow.
pub async fn request_end_run<C: IpcClient>(
    client: &C,
    run_id: &str,
    reason: &str,
    description: &str,
    source: &str,
    suggested_follow_up: Option<SuggestedFollowUp>,
) -> Result<String, IpcError> {
    let request = RequestEndRunRequest {
        run_id: run_id.to_string(),
        reason: reason.to_string(),
        description: description.to_string(),
        source: source.to_string(),
        suggested_follow_up,
    };

    let endpoint = format!("/api/runs/{}/end", run_id);
    let response: RequestEndRunResponse = client.request(&endpoint, &request).await?;

    if response.success {
        Ok(response.request_id)
    } else {
        Err(IpcError::RequestFailed("Request end run failed".into()))
    }
}

/// Validate end response value.
pub fn validate_end_response(response: &str) -> Result<(), String> {
    const VALID: &[&str] = &["agree", "disagree", "verify"];
    if VALID.contains(&response) {
        Ok(())
    } else {
        Err(format!(
            "Invalid response '{}'. Must be one of: {}",
            response,
            VALID.join(", ")
        ))
    }
}

/// Respond to another agent's request to end the workflow.
pub async fn respond_to_end_request<C: IpcClient>(
    client: &C,
    run_id: &str,
    request_id: &str,
    response: &str,
    source: &str,
    reason: Option<&str>,
) -> Result<(), IpcError> {
    let request = RespondToEndRequestRequest {
        run_id: run_id.to_string(),
        request_id: request_id.to_string(),
        response: response.to_string(),
        source: source.to_string(),
        reason: reason.map(String::from),
    };

    let endpoint = format!("/api/runs/{}/end/respond", run_id);
    let response_obj: RespondToEndRequestResponse = client.request(&endpoint, &request).await?;

    if response_obj.success {
        Ok(())
    } else {
        Err(IpcError::RequestFailed(
            "Respond to end request failed".into(),
        ))
    }
}

/// Resolve an impediment raised by another agent.
///
/// This allows agents (not just humans) to resolve impediments if:
/// 1. Their role's protocol allows it (defined in playbook capabilities)
/// 2. The response matches the impediment's responseFormat options
///
/// The backend will validate the response against the original impediment's
/// responseFormat if one was defined.
pub async fn resolve_impediment<C: IpcClient>(
    client: &C,
    run_id: &str,
    impediment_id: i64,
    response: serde_json::Value,
    source: &str,
    rationale: Option<&str>,
) -> Result<(), IpcError> {
    let request = ResolveImpedimentRequest {
        run_id: run_id.to_string(),
        impediment_id,
        response,
        source: source.to_string(),
        rationale: rationale.map(String::from),
    };

    let endpoint = format!("/api/runs/{}/impediment/resolve", run_id);
    let response_obj: ResolveImpedimentResponse = client.request(&endpoint, &request).await?;

    if response_obj.success {
        Ok(())
    } else {
        let error_msg = response_obj
            .error
            .unwrap_or_else(|| "Resolve impediment failed".into());
        Err(IpcError::RequestFailed(error_msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::mock::MockIpcClient;

    #[tokio::test]
    async fn test_report_status_sends_all_fields() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/report-status",
            ReportStatusResponse { success: true },
        );

        let _ = report_status(
            &mock,
            "run-1",
            "working",
            "Implementing feature",
            "builder",
            Some("active"),
        )
        .await;

        let requests = mock.requests_to("/api/runs/run-1/report-status");
        let req = &requests[0];
        assert!(req.contains("run-1"));
        assert!(req.contains("working"));
        assert!(req.contains("Implementing feature"));
        assert!(req.contains("builder"));
        assert!(req.contains("active"));
    }

    #[tokio::test]
    async fn test_report_status_without_agent_status() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/report-status",
            ReportStatusResponse { success: true },
        );

        let result = report_status(
            &mock,
            "run-1",
            "thinking",
            "Planning next step",
            "strategist",
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_status_accepts_valid() {
        assert!(validate_status("working").is_ok());
        assert!(validate_status("thinking").is_ok());
        assert!(validate_status("complete").is_ok());
    }

    #[test]
    fn test_validate_status_rejects_invalid() {
        let result = validate_status("running");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("running"));
    }

    #[test]
    fn test_validate_agent_status_accepts_valid() {
        assert!(validate_agent_status("active").is_ok());
        assert!(validate_agent_status("blocked").is_ok());
    }

    #[test]
    fn test_validate_agent_status_rejects_invalid() {
        assert!(validate_agent_status("busy").is_err());
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;
    use crate::ipc::mock::MockIpcClient;

    #[tokio::test]
    async fn test_send_message_with_target() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/message",
            SendMessageResponse {
                success: true,
                event_id: "msg-123".into(),
            },
        );

        let result = send_message(
            &mock,
            "run-1",
            "Here's the plan",
            "strategist",
            Some("builder"),
            None,
        )
        .await;

        assert!(result.is_ok());
        let requests = mock.requests_to("/api/runs/run-1/message");
        let req = &requests[0];
        assert!(req.contains("builder"));
    }

    #[tokio::test]
    async fn test_send_message_without_target() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/message",
            SendMessageResponse {
                success: true,
                event_id: "msg-456".into(),
            },
        );

        let result = send_message(
            &mock,
            "run-1",
            "Broadcasting update",
            "strategist",
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_message_returns_event_id() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/message",
            SendMessageResponse {
                success: true,
                event_id: "event-789".into(),
            },
        );

        let result = send_message(&mock, "run-1", "Test message", "builder", None, None).await;

        assert_eq!(result.unwrap(), "event-789");
    }
}

#[cfg(test)]
mod task_tests {
    use super::*;
    use crate::ipc::mock::MockIpcClient;

    #[tokio::test]
    async fn test_task_complete_with_outcome() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/task-complete",
            TaskCompleteResponse { success: true },
        );

        let result = task_complete(
            &mock,
            "run-1",
            "Implemented login feature",
            "builder",
            Some("All tests passing"),
            Some("active"),
        )
        .await;

        assert!(result.is_ok());
        let requests = mock.requests_to("/api/runs/run-1/task-complete");
        let req = &requests[0];
        assert!(req.contains("Implemented login feature"));
        assert!(req.contains("All tests passing"));
    }

    #[tokio::test]
    async fn test_task_complete_without_outcome() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/task-complete",
            TaskCompleteResponse { success: true },
        );

        let result = task_complete(&mock, "run-1", "Fixed bug", "builder", None, None).await;

        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod impediment_tests {
    use super::*;
    use crate::ipc::mock::MockIpcClient;

    #[test]
    fn test_validate_impediment_type_accepts_valid() {
        assert!(validate_impediment_type("missing_information").is_ok());
        assert!(validate_impediment_type("permission_needed").is_ok());
        assert!(validate_impediment_type("technical_error").is_ok());
        assert!(validate_impediment_type("unclear_requirements").is_ok());
        assert!(validate_impediment_type("dependency_blocked").is_ok());
        assert!(validate_impediment_type("other").is_ok());
    }

    #[test]
    fn test_validate_impediment_type_rejects_invalid() {
        let result = validate_impediment_type("invalid_type");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid_type"));
    }

    #[tokio::test]
    async fn test_report_impediment_sends_all_fields() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/impediment",
            ReportImpedimentResponse { success: true },
        );

        let _ = report_impediment(
            &mock,
            "run-1",
            "technical_error",
            "Database connection failed",
            "builder",
            Some("Tried 3 times"),
            Some("Check DB credentials"),
            Some("blocked"),
            None,
        )
        .await;

        let requests = mock.requests_to("/api/runs/run-1/impediment");
        let req = &requests[0];
        assert!(req.contains("run-1"));
        assert!(req.contains("technical_error"));
        assert!(req.contains("Database connection failed"));
        assert!(req.contains("builder"));
        assert!(req.contains("Tried 3 times"));
        assert!(req.contains("Check DB credentials"));
        assert!(req.contains("blocked"));
    }

    #[tokio::test]
    async fn test_report_impediment_without_optional_fields() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/impediment",
            ReportImpedimentResponse { success: true },
        );

        let result = report_impediment(
            &mock,
            "run-1",
            "missing_information",
            "Need API key",
            "strategist",
            None,
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_report_impediment_handles_backend_error() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/impediment",
            ReportImpedimentResponse { success: false },
        );

        let result = report_impediment(
            &mock,
            "run-1",
            "other",
            "Something blocked",
            "builder",
            None,
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_report_impediment_includes_suggestion() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/impediment",
            ReportImpedimentResponse { success: true },
        );

        let _ = report_impediment(
            &mock,
            "run-1",
            "permission_needed",
            "Cannot write to directory",
            "builder",
            None,
            Some("Run with sudo or change permissions"),
            None,
            None,
        )
        .await;

        let requests = mock.requests_to("/api/runs/run-1/impediment");
        let req = &requests[0];
        assert!(req.contains("Run with sudo"));
    }

    #[tokio::test]
    async fn test_report_impediment_with_response_format() {
        use crate::ipc::messages::{ResponseFormatField, ResponseFormatOption};

        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/impediment",
            ReportImpedimentResponse { success: true },
        );

        let response_format = ResponseFormat {
            fields: vec![ResponseFormatField {
                id: "database".to_string(),
                field_type: "radio".to_string(),
                label: "Select database:".to_string(),
                description: None,
                required: Some(true),
                options: Some(vec![
                    ResponseFormatOption {
                        value: "postgresql".to_string(),
                        label: "PostgreSQL".to_string(),
                    },
                    ResponseFormatOption {
                        value: "sqlite".to_string(),
                        label: "SQLite".to_string(),
                    },
                ]),
            }],
        };

        let result = report_impediment(
            &mock,
            "run-1",
            "missing_information",
            "Need database selection",
            "strategist",
            None,
            None,
            None,
            Some(response_format),
        )
        .await;

        assert!(result.is_ok());
        let requests = mock.requests_to("/api/runs/run-1/impediment");
        let req = &requests[0];
        assert!(req.contains("responseFormat"));
        assert!(req.contains("radio"));
        assert!(req.contains("postgresql"));
        assert!(req.contains("sqlite"));
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;
    use crate::ipc::mock::MockIpcClient;

    #[tokio::test]
    async fn test_request_input_with_context_and_options() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/input",
            RequestInputResponse { success: true },
        );

        let result = request_input(
            &mock,
            "run-1",
            "Which approach should we use?",
            "strategist",
            Some("We have multiple options for auth"),
            Some(vec!["OAuth".to_string(), "JWT".to_string()]),
        )
        .await;

        assert!(result.is_ok());
        let requests = mock.requests_to("/api/runs/run-1/input");
        let req = &requests[0];
        assert!(req.contains("Which approach should we use?"));
        assert!(req.contains("multiple options for auth"));
        assert!(req.contains("OAuth"));
        assert!(req.contains("JWT"));
    }

    #[tokio::test]
    async fn test_request_input_without_optional_fields() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/input",
            RequestInputResponse { success: true },
        );

        let result = request_input(
            &mock,
            "run-1",
            "What color for the button?",
            "builder",
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_request_input_sends_all_fields() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/input",
            RequestInputResponse { success: true },
        );

        let _ = request_input(
            &mock,
            "run-1",
            "Should we refactor now?",
            "strategist",
            Some("Code is working but messy"),
            None,
        )
        .await;

        let requests = mock.requests_to("/api/runs/run-1/input");
        let req = &requests[0];
        assert!(req.contains("run-1"));
        assert!(req.contains("Should we refactor now?"));
        assert!(req.contains("strategist"));
        assert!(req.contains("Code is working but messy"));
    }

    #[tokio::test]
    async fn test_request_input_handles_backend_error() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/input",
            RequestInputResponse { success: false },
        );

        let result = request_input(&mock, "run-1", "Question?", "builder", None, None).await;

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod handoff_tests {
    use super::*;
    use crate::ipc::mock::MockIpcClient;

    #[tokio::test]
    async fn test_handoff_with_details_and_artifacts() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/runs/run-1/handoff", HandoffResponse { success: true });

        let result = handoff(
            &mock,
            "run-1",
            "builder",
            "Implement login feature",
            "strategist",
            Some("Use bcrypt for password hashing. Add tests."),
            Some(vec![
                "src/auth.rs".to_string(),
                "tests/auth_test.rs".to_string(),
            ]),
        )
        .await;

        assert!(result.is_ok());
        let requests = mock.requests_to("/api/runs/run-1/handoff");
        let req = &requests[0];
        assert!(req.contains("builder"));
        assert!(req.contains("Implement login feature"));
        assert!(req.contains("Use bcrypt"));
        assert!(req.contains("src/auth.rs"));
    }

    #[tokio::test]
    async fn test_handoff_without_optional_fields() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/runs/run-1/handoff", HandoffResponse { success: true });

        let result = handoff(
            &mock,
            "run-1",
            "strategist",
            "Ready for review",
            "builder",
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handoff_sends_all_required_fields() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/runs/run-1/handoff", HandoffResponse { success: true });

        let _ = handoff(
            &mock,
            "run-1",
            "builder",
            "Fix bug in payment flow",
            "strategist",
            None,
            None,
        )
        .await;

        let requests = mock.requests_to("/api/runs/run-1/handoff");
        let req = &requests[0];
        assert!(req.contains("run-1"));
        assert!(req.contains("builder"));
        assert!(req.contains("Fix bug in payment flow"));
        assert!(req.contains("strategist"));
    }

    #[tokio::test]
    async fn test_handoff_handles_backend_error() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/handoff",
            HandoffResponse { success: false },
        );

        let result = handoff(
            &mock,
            "run-1",
            "builder",
            "Do something",
            "strategist",
            None,
            None,
        )
        .await;

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use crate::ipc::mock::MockIpcClient;

    #[test]
    fn test_validate_end_run_reason_accepts_valid() {
        assert!(validate_end_run_reason("completed").is_ok());
        assert!(validate_end_run_reason("scope_changed").is_ok());
        assert!(validate_end_run_reason("pause_requested").is_ok());
        assert!(validate_end_run_reason("error").is_ok());
    }

    #[test]
    fn test_validate_end_run_reason_rejects_invalid() {
        let result = validate_end_run_reason("cancelled");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cancelled"));
    }

    #[tokio::test]
    async fn test_request_end_run_with_follow_up() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/end",
            RequestEndRunResponse {
                success: true,
                request_id: "req-123".into(),
            },
        );

        let follow_up = SuggestedFollowUp {
            title: "Add tests".to_string(),
            description: "Write unit tests for new feature".to_string(),
        };

        let result = request_end_run(
            &mock,
            "run-1",
            "completed",
            "All tasks finished",
            "strategist",
            Some(follow_up),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "req-123");

        let requests = mock.requests_to("/api/runs/run-1/end");
        let req = &requests[0];
        assert!(req.contains("completed"));
        assert!(req.contains("All tasks finished"));
        assert!(req.contains("Add tests"));
    }

    #[tokio::test]
    async fn test_request_end_run_without_follow_up() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/end",
            RequestEndRunResponse {
                success: true,
                request_id: "req-456".into(),
            },
        );

        let result = request_end_run(
            &mock,
            "run-1",
            "error",
            "Encountered blocking issue",
            "builder",
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_end_response_accepts_valid() {
        assert!(validate_end_response("agree").is_ok());
        assert!(validate_end_response("disagree").is_ok());
        assert!(validate_end_response("verify").is_ok());
    }

    #[test]
    fn test_validate_end_response_rejects_invalid() {
        let result = validate_end_response("maybe");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("maybe"));
    }

    #[tokio::test]
    async fn test_respond_to_end_request_agree() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/end/respond",
            RespondToEndRequestResponse { success: true },
        );

        let result =
            respond_to_end_request(&mock, "run-1", "req-123", "agree", "builder", None).await;

        assert!(result.is_ok());

        let requests = mock.requests_to("/api/runs/run-1/end/respond");
        let req = &requests[0];
        assert!(req.contains("req-123"));
        assert!(req.contains("agree"));
    }

    #[tokio::test]
    async fn test_respond_to_end_request_disagree_with_reason() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/end/respond",
            RespondToEndRequestResponse { success: true },
        );

        let result = respond_to_end_request(
            &mock,
            "run-1",
            "req-123",
            "disagree",
            "builder",
            Some("Still have work to finish"),
        )
        .await;

        assert!(result.is_ok());

        let requests = mock.requests_to("/api/runs/run-1/end/respond");
        let req = &requests[0];
        assert!(req.contains("disagree"));
        assert!(req.contains("Still have work to finish"));
    }
}

#[cfg(test)]
mod resolve_impediment_tests {
    use super::*;
    use crate::ipc::mock::MockIpcClient;
    use serde_json::json;

    #[tokio::test]
    async fn test_resolve_impediment_success() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/impediment/resolve",
            ResolveImpedimentResponse {
                success: true,
                error: None,
            },
        );

        let result = resolve_impediment(
            &mock,
            "run-1",
            42,
            json!({"database": "postgresql"}),
            "strategist",
            Some("Based on the project requirements, PostgreSQL is the better choice"),
        )
        .await;

        assert!(result.is_ok());
        let requests = mock.requests_to("/api/runs/run-1/impediment/resolve");
        let req = &requests[0];
        assert!(req.contains("run-1"));
        assert!(req.contains("42"));
        assert!(req.contains("postgresql"));
        assert!(req.contains("strategist"));
        assert!(req.contains("Based on the project requirements"));
    }

    #[tokio::test]
    async fn test_resolve_impediment_without_rationale() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/impediment/resolve",
            ResolveImpedimentResponse {
                success: true,
                error: None,
            },
        );

        let result =
            resolve_impediment(&mock, "run-1", 123, json!("use_sqlite"), "builder", None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resolve_impediment_returns_error_message() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/impediment/resolve",
            ResolveImpedimentResponse {
                success: false,
                error: Some("Response does not match responseFormat options".into()),
            },
        );

        let result = resolve_impediment(
            &mock,
            "run-1",
            42,
            json!({"database": "mongodb"}), // Invalid option
            "builder",
            None,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            IpcError::RequestFailed(msg) => {
                assert!(msg.contains("responseFormat"));
            }
            _ => panic!("Expected RequestFailed error"),
        }
    }

    #[tokio::test]
    async fn test_resolve_impediment_with_complex_response() {
        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/runs/run-1/impediment/resolve",
            ResolveImpedimentResponse {
                success: true,
                error: None,
            },
        );

        // Test with a complex response object (multiple fields from a form)
        let response = json!({
            "database": "postgresql",
            "connection_pool_size": 10,
            "use_ssl": true
        });

        let result = resolve_impediment(
            &mock,
            "run-1",
            99,
            response,
            "strategist",
            Some("Configuration based on production requirements"),
        )
        .await;

        assert!(result.is_ok());
        let requests = mock.requests_to("/api/runs/run-1/impediment/resolve");
        let req = &requests[0];
        assert!(req.contains("postgresql"));
        assert!(req.contains("connection_pool_size"));
        assert!(req.contains("use_ssl"));
    }
}
