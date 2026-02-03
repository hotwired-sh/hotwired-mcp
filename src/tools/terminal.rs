//! Terminal workflow tools for /hotwire and /pair commands.
//!
//! These tools support the terminal-first workflow where agents initiate
//! runs directly from the terminal rather than through the app wizard.

use crate::ipc::messages::{
    HotwireArtifact, HotwireRequest, HotwireResponse, ListActiveRunsRequest,
    ListActiveRunsResponse, ListPlaybooksRequest, ListPlaybooksResponse, PairRequest,
    PairResponse, PlaybookInfo,
};
use crate::ipc::traits::IpcClient;
use crate::types::errors::IpcError;

// =============================================================================
// HOTWIRE - Initiate run from terminal
// =============================================================================

/// Initiates a new workflow run from the terminal.
///
/// Returns one of:
/// - `Started`: Run started immediately, protocol included
/// - `NeedsConfirmation`: User must confirm in app, agent waits for Zellij trigger
/// - `Error`: Something went wrong
pub async fn hotwire<C: IpcClient>(
    client: &C,
    project_path: &str,
    zellij_session: &str,
    intent: Option<&str>,
    suggested_playbook: Option<&str>,
    suggested_artifacts: Option<Vec<HotwireArtifact>>,
) -> Result<HotwireResponse, IpcError> {
    let request = HotwireRequest {
        project_path: project_path.to_string(),
        zellij_session: zellij_session.to_string(),
        intent: intent.map(|s| s.to_string()),
        suggested_playbook: suggested_playbook.map(|s| s.to_string()),
        suggested_artifacts,
    };

    client.request("/api/hotwire", &request).await
}

/// Formats the hotwire response for display to the agent.
pub fn format_hotwire_response(response: &HotwireResponse) -> String {
    match response {
        HotwireResponse::Started(started) => {
            format!(
                r#"═══════════════════════════════════════════════════════════════
WORKFLOW STARTED
═══════════════════════════════════════════════════════════════

Run ID: {}
Playbook: {}
Your Role: {}

You are now the PRIMARY AGENT. Your protocol is below.

═══════════════════════════════════════════════════════════════

{}
"#,
                started.run_id, started.playbook, started.role, started.protocol
            )
        }
        HotwireResponse::NeedsConfirmation(needs_conf) => {
            let playbook_hint = needs_conf
                .suggested_playbook
                .as_ref()
                .map(|p| format!("\nSuggested playbook: {}", p))
                .unwrap_or_default();

            format!(
                r#"═══════════════════════════════════════════════════════════════
CONFIRMATION NEEDED
═══════════════════════════════════════════════════════════════

{}
{}
Pending Run ID: {}

Please confirm the workflow in the Hotwired app.
When you confirm, you will receive a trigger message.
Then call get_protocol(run_id, role) to get your instructions.

═══════════════════════════════════════════════════════════════
"#,
                needs_conf.message, playbook_hint, needs_conf.pending_run_id
            )
        }
        HotwireResponse::Error { error } => {
            format!(
                r#"═══════════════════════════════════════════════════════════════
ERROR
═══════════════════════════════════════════════════════════════

Failed to start workflow: {}

═══════════════════════════════════════════════════════════════
"#,
                error
            )
        }
    }
}

// =============================================================================
// PAIR - Join run as second agent
// =============================================================================

/// Joins an existing run as the second agent.
///
/// Returns one of:
/// - `Joined`: Successfully joined, protocol and context included
/// - `NeedsSelection`: Multiple runs available, user must select in app
/// - `NoneAvailable`: No runs waiting for a second agent
/// - `ProjectMismatch`: Run requires different project directory
/// - `Error`: Something went wrong
pub async fn pair<C: IpcClient>(
    client: &C,
    zellij_session: &str,
    project_path: &str,
) -> Result<PairResponse, IpcError> {
    let request = PairRequest {
        zellij_session: zellij_session.to_string(),
        project_path: project_path.to_string(),
    };

    client.request("/api/pair", &request).await
}

/// Formats the pair response for display to the agent.
pub fn format_pair_response(response: &PairResponse) -> String {
    match response {
        PairResponse::Joined(joined) => {
            format!(
                r#"═══════════════════════════════════════════════════════════════
JOINED WORKFLOW
═══════════════════════════════════════════════════════════════

Run ID: {}
Playbook: {}
Your Role: {} ({})

You are now the SECONDARY AGENT.

Context from primary agent:
- Primary status: {}
- Current artifact: {}
- Summary: {}

Your protocol is below.

═══════════════════════════════════════════════════════════════

{}
"#,
                joined.run_id,
                joined.playbook,
                joined.role_name,
                joined.role,
                joined.context.primary_status,
                joined
                    .context
                    .current_artifact
                    .as_deref()
                    .unwrap_or("(none)"),
                joined.context.conversation_summary,
                joined.protocol
            )
        }
        PairResponse::NeedsSelection(needs_sel) => {
            let runs_list: String = needs_sel
                .pending_runs
                .iter()
                .enumerate()
                .map(|(i, run)| {
                    format!(
                        "{}. [{}] {} - needs {} ({})",
                        i + 1,
                        run.run_id,
                        run.playbook,
                        run.role_needed,
                        run.intent
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                r#"═══════════════════════════════════════════════════════════════
SELECTION NEEDED
═══════════════════════════════════════════════════════════════

{}

Available runs needing a second agent:
{}

Please select which run to join in the Hotwired app.
When you select, you will receive a trigger message.
Then call get_protocol(run_id, role) to get your instructions.

═══════════════════════════════════════════════════════════════
"#,
                needs_sel.message, runs_list
            )
        }
        PairResponse::NoneAvailable { message } => {
            format!(
                r#"═══════════════════════════════════════════════════════════════
NO RUNS AVAILABLE
═══════════════════════════════════════════════════════════════

{}

Check that the primary agent has raised a needs_second_agent impediment.

═══════════════════════════════════════════════════════════════
"#,
                message
            )
        }
        PairResponse::ProjectMismatch(mismatch) => {
            format!(
                r#"═══════════════════════════════════════════════════════════════
PROJECT MISMATCH
═══════════════════════════════════════════════════════════════

{}

Required path: {}
Your current path: {}

Please cd to the required path and try again.

═══════════════════════════════════════════════════════════════
"#,
                mismatch.message, mismatch.required_path, mismatch.current_path
            )
        }
        PairResponse::Error { error } => {
            format!(
                r#"═══════════════════════════════════════════════════════════════
ERROR
═══════════════════════════════════════════════════════════════

Failed to join workflow: {}

═══════════════════════════════════════════════════════════════
"#,
                error
            )
        }
    }
}

// =============================================================================
// LIST ACTIVE RUNS
// =============================================================================

/// Lists active or resumable runs, optionally filtered by project or session.
pub async fn list_active_runs<C: IpcClient>(
    client: &C,
    project_path: Option<&str>,
    zellij_session: Option<&str>,
) -> Result<ListActiveRunsResponse, IpcError> {
    let request = ListActiveRunsRequest {
        project_path: project_path.map(|s| s.to_string()),
        zellij_session: zellij_session.map(|s| s.to_string()),
    };

    client.request("/api/active-runs", &request).await
}

/// Formats the active runs response for display.
pub fn format_active_runs(response: &ListActiveRunsResponse) -> String {
    if response.runs.is_empty() {
        return r#"═══════════════════════════════════════════════════════════════
ACTIVE RUNS
═══════════════════════════════════════════════════════════════

No active runs found.

═══════════════════════════════════════════════════════════════
"#
        .to_string();
    }

    let runs_list: String = response
        .runs
        .iter()
        .map(|run| {
            let role_info = run
                .my_role
                .as_ref()
                .map(|r| format!(" (your role: {})", r))
                .unwrap_or_default();

            format!(
                "- [{}] {} - {} ({}){}\n  Created: {}",
                run.run_id, run.playbook, run.intent, run.status, role_info, run.created_at
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        r#"═══════════════════════════════════════════════════════════════
ACTIVE RUNS
═══════════════════════════════════════════════════════════════

{}

═══════════════════════════════════════════════════════════════
"#,
        runs_list
    )
}

// =============================================================================
// LIST PLAYBOOKS
// =============================================================================

/// Lists available playbooks with metadata for intent matching.
pub async fn list_playbooks<C: IpcClient>(
    client: &C,
) -> Result<ListPlaybooksResponse, IpcError> {
    let request = ListPlaybooksRequest {};
    client.request("/api/playbooks", &request).await
}

/// Formats the playbooks response for display.
pub fn format_playbooks(response: &ListPlaybooksResponse) -> String {
    if response.playbooks.is_empty() {
        return r#"═══════════════════════════════════════════════════════════════
AVAILABLE PLAYBOOKS
═══════════════════════════════════════════════════════════════

No playbooks available.

═══════════════════════════════════════════════════════════════
"#
        .to_string();
    }

    let playbooks_list: String = response
        .playbooks
        .iter()
        .map(|pb| format_single_playbook(pb))
        .collect::<Vec<_>>()
        .join("\n---\n\n");

    format!(
        r#"═══════════════════════════════════════════════════════════════
AVAILABLE PLAYBOOKS
═══════════════════════════════════════════════════════════════

{}

═══════════════════════════════════════════════════════════════
"#,
        playbooks_list
    )
}

fn format_single_playbook(pb: &PlaybookInfo) -> String {
    let roles_list: String = pb
        .roles
        .iter()
        .map(|r| {
            let initiating = if r.is_initiating { " [INITIATING]" } else { "" };
            format!("  - {}: {}{}", r.name, r.description, initiating)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let best_for = if pb.best_for.is_empty() {
        "(not specified)".to_string()
    } else {
        pb.best_for.join(", ")
    };

    let keywords = if pb.keywords.is_empty() {
        "(none)".to_string()
    } else {
        pb.keywords.join(", ")
    };

    format!(
        r#"## {} ({})

{}

{}

**Roles:**
{}

**Best for:** {}
**Keywords:** {}
**Artifact mode:** {}"#,
        pb.name,
        pb.id,
        pb.tagline,
        pb.description,
        roles_list,
        best_for,
        keywords,
        pb.artifact_mode
    )
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::messages::{
        ActiveRun, HotwireNeedsConfirmation, HotwireStarted, PairJoined, PairNeedsSelection,
        PairProjectMismatch, PairingContext, PendingPairRun, PlaybookInitHints, PlaybookRoleInfo,
    };
    use crate::ipc::mock::MockIpcClient;

    // ===== HOTWIRE TESTS =====

    fn sample_hotwire_started() -> HotwireResponse {
        HotwireResponse::Started(HotwireStarted {
            run_id: "run-123".to_string(),
            role: "writer".to_string(),
            playbook: "doc-editor".to_string(),
            protocol: "# Protocol\n\nYour instructions here...".to_string(),
        })
    }

    fn sample_hotwire_needs_confirmation() -> HotwireResponse {
        HotwireResponse::NeedsConfirmation(HotwireNeedsConfirmation {
            pending_run_id: "pending-456".to_string(),
            suggested_playbook: Some("doc-editor".to_string()),
            message: "Please confirm the workflow configuration.".to_string(),
        })
    }

    #[tokio::test]
    async fn test_hotwire_sends_correct_request() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/hotwire", sample_hotwire_started());

        let _ = hotwire(
            &mock,
            "/path/to/project",
            "my-session",
            Some("write a PRD"),
            Some("doc-editor"),
            None,
        )
        .await;

        mock.assert_called("/api/hotwire");
        let requests = mock.requests_to("/api/hotwire");
        assert!(requests[0].contains("/path/to/project"));
        assert!(requests[0].contains("my-session"));
        assert!(requests[0].contains("write a PRD"));
    }

    #[tokio::test]
    async fn test_hotwire_returns_started() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/hotwire", sample_hotwire_started());

        let result = hotwire(&mock, "/path", "session", None, None, None).await;

        assert!(result.is_ok());
        match result.unwrap() {
            HotwireResponse::Started(started) => {
                assert_eq!(started.run_id, "run-123");
                assert_eq!(started.role, "writer");
            }
            _ => panic!("Expected Started response"),
        }
    }

    #[test]
    fn test_format_hotwire_started() {
        let response = sample_hotwire_started();
        let formatted = format_hotwire_response(&response);

        assert!(formatted.contains("WORKFLOW STARTED"));
        assert!(formatted.contains("run-123"));
        assert!(formatted.contains("doc-editor"));
        assert!(formatted.contains("writer"));
        assert!(formatted.contains("PRIMARY AGENT"));
    }

    #[test]
    fn test_format_hotwire_needs_confirmation() {
        let response = sample_hotwire_needs_confirmation();
        let formatted = format_hotwire_response(&response);

        assert!(formatted.contains("CONFIRMATION NEEDED"));
        assert!(formatted.contains("pending-456"));
        assert!(formatted.contains("doc-editor"));
        assert!(formatted.contains("confirm the workflow"));
    }

    // ===== PAIR TESTS =====

    fn sample_pair_joined() -> PairResponse {
        PairResponse::Joined(PairJoined {
            run_id: "run-789".to_string(),
            role: "reviewer".to_string(),
            role_name: "Reviewer".to_string(),
            playbook: "doc-editor".to_string(),
            protocol: "# Reviewer Protocol\n\nReview the document...".to_string(),
            context: PairingContext {
                primary_status: "Writing section 3".to_string(),
                current_artifact: Some("docs/prd.md".to_string()),
                conversation_summary: "Writer has completed intro and section 1-2.".to_string(),
            },
        })
    }

    fn sample_pair_needs_selection() -> PairResponse {
        PairResponse::NeedsSelection(PairNeedsSelection {
            pending_runs: vec![
                PendingPairRun {
                    run_id: "run-1".to_string(),
                    playbook: "doc-editor".to_string(),
                    intent: "Write PRD for auth".to_string(),
                    role_needed: "reviewer".to_string(),
                },
                PendingPairRun {
                    run_id: "run-2".to_string(),
                    playbook: "plan-build".to_string(),
                    intent: "Implement login".to_string(),
                    role_needed: "builder".to_string(),
                },
            ],
            message: "Multiple runs need a second agent.".to_string(),
        })
    }

    #[tokio::test]
    async fn test_pair_sends_correct_request() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/pair", sample_pair_joined());

        let _ = pair(&mock, "my-session", "/path/to/project").await;

        mock.assert_called("/api/pair");
        let requests = mock.requests_to("/api/pair");
        assert!(requests[0].contains("my-session"));
        assert!(requests[0].contains("/path/to/project"));
    }

    #[test]
    fn test_format_pair_joined() {
        let response = sample_pair_joined();
        let formatted = format_pair_response(&response);

        assert!(formatted.contains("JOINED WORKFLOW"));
        assert!(formatted.contains("run-789"));
        assert!(formatted.contains("Reviewer"));
        assert!(formatted.contains("SECONDARY AGENT"));
        assert!(formatted.contains("Writing section 3"));
        assert!(formatted.contains("docs/prd.md"));
    }

    #[test]
    fn test_format_pair_needs_selection() {
        let response = sample_pair_needs_selection();
        let formatted = format_pair_response(&response);

        assert!(formatted.contains("SELECTION NEEDED"));
        assert!(formatted.contains("run-1"));
        assert!(formatted.contains("run-2"));
        assert!(formatted.contains("reviewer"));
        assert!(formatted.contains("builder"));
    }

    #[test]
    fn test_format_pair_project_mismatch() {
        let response = PairResponse::ProjectMismatch(PairProjectMismatch {
            required_path: "/correct/path".to_string(),
            current_path: "/wrong/path".to_string(),
            message: "Project paths don't match.".to_string(),
        });
        let formatted = format_pair_response(&response);

        assert!(formatted.contains("PROJECT MISMATCH"));
        assert!(formatted.contains("/correct/path"));
        assert!(formatted.contains("/wrong/path"));
    }

    // ===== LIST ACTIVE RUNS TESTS =====

    fn sample_active_runs() -> ListActiveRunsResponse {
        ListActiveRunsResponse {
            runs: vec![
                ActiveRun {
                    run_id: "run-a".to_string(),
                    playbook: "doc-editor".to_string(),
                    intent: "Write PRD".to_string(),
                    status: "active".to_string(),
                    my_role: Some("writer".to_string()),
                    created_at: "2024-01-15T10:00:00Z".to_string(),
                },
                ActiveRun {
                    run_id: "run-b".to_string(),
                    playbook: "plan-build".to_string(),
                    intent: "Implement auth".to_string(),
                    status: "paused".to_string(),
                    my_role: None,
                    created_at: "2024-01-15T11:00:00Z".to_string(),
                },
            ],
        }
    }

    #[tokio::test]
    async fn test_list_active_runs_sends_request() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/active-runs", sample_active_runs());

        let _ = list_active_runs(&mock, Some("/path"), Some("session")).await;

        mock.assert_called("/api/active-runs");
    }

    #[test]
    fn test_format_active_runs() {
        let response = sample_active_runs();
        let formatted = format_active_runs(&response);

        assert!(formatted.contains("ACTIVE RUNS"));
        assert!(formatted.contains("run-a"));
        assert!(formatted.contains("run-b"));
        assert!(formatted.contains("your role: writer"));
        assert!(formatted.contains("doc-editor"));
    }

    #[test]
    fn test_format_active_runs_empty() {
        let response = ListActiveRunsResponse { runs: vec![] };
        let formatted = format_active_runs(&response);

        assert!(formatted.contains("No active runs found"));
    }

    // ===== LIST PLAYBOOKS TESTS =====

    fn sample_playbooks() -> ListPlaybooksResponse {
        ListPlaybooksResponse {
            playbooks: vec![PlaybookInfo {
                id: "doc-editor".to_string(),
                name: "Document Editor".to_string(),
                tagline: "Create and edit documents collaboratively".to_string(),
                description: "A workflow for writing documents with AI assistance.".to_string(),
                artifact_mode: true,
                roles: vec![
                    PlaybookRoleInfo {
                        id: "writer".to_string(),
                        name: "Writer".to_string(),
                        description: "Creates and edits the document".to_string(),
                        is_initiating: true,
                    },
                    PlaybookRoleInfo {
                        id: "reviewer".to_string(),
                        name: "Reviewer".to_string(),
                        description: "Reviews and suggests improvements".to_string(),
                        is_initiating: false,
                    },
                ],
                best_for: vec!["PRDs".to_string(), "specs".to_string()],
                keywords: vec!["write".to_string(), "document".to_string(), "prd".to_string()],
                initialization: PlaybookInitHints {
                    expects_document: true,
                    expects_goal: false,
                    suggested_paths: vec!["docs/".to_string()],
                },
            }],
        }
    }

    #[tokio::test]
    async fn test_list_playbooks_sends_request() {
        let mock = MockIpcClient::new();
        mock.when_called("/api/playbooks", sample_playbooks());

        let _ = list_playbooks(&mock).await;

        mock.assert_called("/api/playbooks");
    }

    #[test]
    fn test_format_playbooks() {
        let response = sample_playbooks();
        let formatted = format_playbooks(&response);

        assert!(formatted.contains("AVAILABLE PLAYBOOKS"));
        assert!(formatted.contains("Document Editor"));
        assert!(formatted.contains("doc-editor"));
        assert!(formatted.contains("Writer"));
        assert!(formatted.contains("[INITIATING]"));
        assert!(formatted.contains("PRDs, specs"));
        assert!(formatted.contains("write, document, prd"));
    }
}
