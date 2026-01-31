use crate::ipc::messages::{
    GetProtocolResponse, GetProtocolSocketRequest, GetRunStatusRequest, GetRunStatusResponse,
};
use crate::ipc::traits::IpcClient;
use crate::types::errors::IpcError;

/// Fetches the protocol for a specific run and role.
/// This is the PRIMARY way agents receive their instructions.
pub async fn get_protocol<C: IpcClient>(
    client: &C,
    run_id: &str,
    agent_role: &str,
) -> Result<GetProtocolResponse, IpcError> {
    // Use socket request struct which sends "role" (not "agentRole") to match hotwired-core
    let request = GetProtocolSocketRequest {
        run_id: run_id.to_string(),
        role: agent_role.to_string(),
    };

    let endpoint = format!("/api/runs/{}/protocol", run_id);
    client.request(&endpoint, &request).await
}

/// Formats the protocol response for display to the agent.
pub fn format_protocol_response(response: &GetProtocolResponse) -> String {
    let role_section = match &response.role_protocol {
        Some(rp) if !rp.is_empty() => format!("\n## Your Role Instructions\n\n{}\n", rp),
        _ => String::new(),
    };

    let protocol_instructions = response
        .playbook_protocol
        .as_deref()
        .unwrap_or("(No protocol instructions available)");

    let init_condition = response
        .initialization_condition
        .as_deref()
        .unwrap_or("(No initialization condition set)");

    // Format capabilities section
    let capabilities_section = match &response.capabilities {
        Some(caps) => {
            let mut caps_lines = vec![];
            if caps.can_resolve_impediments {
                caps_lines.push("- **Can resolve impediments**: You can use `resolve_impediment` to resolve blockers raised by other agents");
            }
            if caps_lines.is_empty() {
                String::new()
            } else {
                format!("\n## Your Capabilities\n\n{}\n", caps_lines.join("\n"))
            }
        }
        None => String::new(),
    };

    format!(
        r#"# Hotwired Workflow Protocol

**Run ID:** {}
**Playbook:** {}

## Protocol Instructions

{}
{}{}
## Initialization Condition

{}
"#,
        response.run_id,
        response.template_name,
        protocol_instructions,
        role_section,
        capabilities_section,
        init_condition,
    )
}

/// Fetches the current status of a run.
pub async fn get_run_status<C: IpcClient>(
    client: &C,
    run_id: &str,
) -> Result<GetRunStatusResponse, IpcError> {
    let request = GetRunStatusRequest {
        run_id: run_id.to_string(),
    };

    let endpoint = format!("/api/runs/{}/status", run_id);
    client.request(&endpoint, &request).await
}

/// Formats the run status for display.
///
/// IMPORTANT: Uses plain text format (not markdown bold) so that the mock agent's
/// status parser can detect the status. The parser looks for "Status: blocked"
/// (plain text), not "**Status:** blocked" (markdown bold).
pub fn format_run_status_response(response: &GetRunStatusResponse) -> String {
    // Build the connected agents section
    let agents_section = if response.connected_agents.is_empty() {
        "  (no agents connected)".to_string()
    } else {
        response
            .connected_agents
            .iter()
            .map(|agent| {
                format!(
                    "  - {}: {} ({})",
                    agent.role_id, agent.session_name, agent.agent_type
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"═══════════════════════════════════════════════════════════════
RUN STATUS
═══════════════════════════════════════════════════════════════

Run ID: {}
Status: {}
Phase: {}
Template: {}
Has Protocol: {}

Connected Agents:
{}

═══════════════════════════════════════════════════════════════
"#,
        response.run_id,
        response.status,
        response.phase,
        response.template_name,
        response.has_protocol,
        agents_section,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::mock::MockIpcClient;

    fn sample_protocol_response() -> GetProtocolResponse {
        use crate::ipc::messages::RoleCapabilities;
        GetProtocolResponse {
            run_id: "test-run-123".into(),
            template_name: "Plan → Build".into(),
            playbook_protocol: Some("Follow these steps...".into()),
            role_protocol: Some("As the strategist, you should...".into()),
            initialization_condition: Some("Build a login page".into()),
            project_name: Some("test-project".into()),
            capabilities: Some(RoleCapabilities {
                can_resolve_impediments: true,
            }),
        }
    }

    #[tokio::test]
    async fn test_get_protocol_sends_correct_request() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/runs/run-123/protocol", sample_protocol_response());

        let _ = get_protocol(&mock, "run-123", "strategist").await;

        mock.assert_called("/api/runs/run-123/protocol");
        let requests = mock.requests_to("/api/runs/run-123/protocol");
        assert!(requests[0].contains("run-123"));
        assert!(requests[0].contains("strategist"));
    }

    #[tokio::test]
    async fn test_get_protocol_returns_response() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/runs/run-123/protocol", sample_protocol_response());

        let result = get_protocol(&mock, "run-123", "strategist").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.run_id, "test-run-123");
        assert_eq!(response.template_name, "Plan → Build");
    }

    #[tokio::test]
    async fn test_get_protocol_returns_error_when_disconnected() {
        let mock = MockIpcClient::new();
        mock.set_disconnected(true);

        let result = get_protocol(&mock, "run-123", "strategist").await;

        assert!(matches!(result, Err(IpcError::NotConnected)));
    }

    #[test]
    fn test_format_protocol_includes_all_sections() {
        let response = sample_protocol_response();
        let formatted = format_protocol_response(&response);

        assert!(formatted.contains("test-run-123"), "Should include run ID");
        assert!(
            formatted.contains("Plan → Build"),
            "Should include template name"
        );
        assert!(
            formatted.contains("Follow these steps"),
            "Should include protocol"
        );
        assert!(
            formatted.contains("As the strategist"),
            "Should include role instructions"
        );
        assert!(
            formatted.contains("Build a login page"),
            "Should include init condition"
        );
    }

    #[test]
    fn test_format_protocol_uses_markdown() {
        let response = sample_protocol_response();
        let formatted = format_protocol_response(&response);

        assert!(formatted.contains("# "), "Should use markdown headers");
        assert!(
            formatted.contains("**Run ID:**"),
            "Should use bold formatting"
        );
    }

    #[test]
    fn test_format_protocol_includes_capabilities() {
        let response = sample_protocol_response();
        let formatted = format_protocol_response(&response);

        assert!(
            formatted.contains("## Your Capabilities"),
            "Should include capabilities section"
        );
        assert!(
            formatted.contains("Can resolve impediments"),
            "Should mention resolve impediments capability"
        );
    }

    #[test]
    fn test_format_protocol_no_capabilities_section_when_none() {
        use crate::ipc::messages::RoleCapabilities;
        let response = GetProtocolResponse {
            run_id: "test-run".into(),
            template_name: "Test".into(),
            playbook_protocol: Some("Protocol".into()),
            role_protocol: None,
            initialization_condition: None,
            project_name: None,
            capabilities: Some(RoleCapabilities {
                can_resolve_impediments: false,
            }),
        };
        let formatted = format_protocol_response(&response);

        assert!(
            !formatted.contains("## Your Capabilities"),
            "Should NOT include capabilities section when no capabilities"
        );
    }
}

#[cfg(test)]
mod status_tests {
    use super::*;
    use crate::ipc::messages::ConnectedAgent;
    use crate::ipc::mock::MockIpcClient;

    fn sample_status_response() -> GetRunStatusResponse {
        GetRunStatusResponse {
            run_id: "run-456".into(),
            status: "active".into(),
            phase: "executing".into(),
            template_name: "Plan → Build".into(),
            has_protocol: true,
            connected_agents: vec![
                ConnectedAgent {
                    role_id: "strategist".into(),
                    session_name: "claude-1".into(),
                    agent_type: "claude".into(),
                },
                ConnectedAgent {
                    role_id: "builder".into(),
                    session_name: "claude-2".into(),
                    agent_type: "claude".into(),
                },
            ],
        }
    }

    #[tokio::test]
    async fn test_get_run_status_sends_correct_request() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/runs/run-456/status", sample_status_response());

        let _ = get_run_status(&mock, "run-456").await;

        mock.assert_called("/api/runs/run-456/status");
    }

    #[tokio::test]
    async fn test_get_run_status_parses_connected_agents() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/runs/run-456/status", sample_status_response());

        let result = get_run_status(&mock, "run-456").await.unwrap();

        assert_eq!(result.connected_agents.len(), 2);
        assert_eq!(result.connected_agents[0].role_id, "strategist");
        assert_eq!(result.connected_agents[0].session_name, "claude-1");
        assert_eq!(result.connected_agents[1].role_id, "builder");
        assert_eq!(result.connected_agents[1].session_name, "claude-2");
    }

    #[test]
    fn test_format_status_shows_connected_agents() {
        let response = sample_status_response();
        let formatted = format_run_status_response(&response);

        assert!(formatted.contains("claude-1"));
        assert!(formatted.contains("claude-2"));
        assert!(formatted.contains("strategist"));
        assert!(formatted.contains("builder"));
        assert!(formatted.contains("Connected Agents"));
    }

    #[test]
    fn test_format_status_handles_no_agents() {
        let response = GetRunStatusResponse {
            run_id: "run-456".into(),
            status: "active".into(),
            phase: "executing".into(),
            template_name: "Plan → Build".into(),
            has_protocol: false,
            connected_agents: vec![],
        };

        let formatted = format_run_status_response(&response);

        assert!(formatted.contains("(no agents connected)"));
    }
}
