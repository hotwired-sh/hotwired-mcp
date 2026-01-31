use crate::ipc::messages::{
    DocArtifactAcceptSuggestionRequest,
    DocArtifactAddCommentRequest,
    DocArtifactCreateRequest,
    DocArtifactEditRequest,
    DocArtifactListCommentsRequest,
    DocArtifactListRequest,
    DocArtifactListSuggestionsRequest,
    DocArtifactReadRequest,
    DocArtifactRejectSuggestionRequest,
    DocArtifactResolveCommentRequest,
    DocArtifactSearchRequest,
    // Edit suggestions (Mode 2)
    DocArtifactSuggestEditRequest,
    GetProtocolRequest,
    GetRunStatusRequest,
    HandoffRequest,
    ReportImpedimentRequest,
    ReportStatusRequest,
    RequestEndRunRequest,
    RequestInputRequest,
    ResolveImpedimentRequest,
    RespondToEndRequestRequest,
    SendMessageRequest,
    TaskCompleteRequest,
};
use crate::ipc::traits::IpcClient;
use crate::tools::{artifacts, protocol, status};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ErrorData as McpError,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct HotwiredMcp<C: IpcClient> {
    tool_router: ToolRouter<Self>,
    client: Arc<C>,
}

#[tool_router]
impl<C: IpcClient + 'static> HotwiredMcp<C> {
    pub fn new(client: C) -> Self {
        Self {
            tool_router: Self::tool_router(),
            client: Arc::new(client),
        }
    }

    #[tool(description = "Test connectivity to the Hotwired MCP server and backend API")]
    async fn ping(&self) -> Result<CallToolResult, McpError> {
        match self.client.health_check().await {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "pong - Connected to Hotwired backend",
            )])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "MCP running but backend unavailable: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "Fetch your complete protocol, role instructions, and initialization condition. Call this when a run is initialized or if you need to refresh your understanding of the task. This is the PRIMARY way to receive your workflow instructions."
    )]
    async fn get_protocol(
        &self,
        Parameters(params): Parameters<GetProtocolRequest>,
    ) -> Result<CallToolResult, McpError> {
        match protocol::get_protocol(&*self.client, &params.run_id, &params.agent_role).await {
            Ok(response) => {
                let formatted = protocol::format_protocol_response(&response);
                Ok(CallToolResult::success(vec![Content::text(formatted)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to get protocol: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "Get the current status of a run (phase, active agents, etc.). Use this to check if a run is still active before calling other tools. If you don't have the protocol yet, call get_protocol instead."
    )]
    async fn get_run_status(
        &self,
        Parameters(params): Parameters<GetRunStatusRequest>,
    ) -> Result<CallToolResult, McpError> {
        match protocol::get_run_status(&*self.client, &params.run_id).await {
            Ok(response) => {
                let formatted = protocol::format_run_status_response(&response);
                Ok(CallToolResult::success(vec![Content::text(formatted)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to get run status: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "Report your current status to the Hotwired dashboard. MUST only be invoked when inside an active run that has a `RUN_ID` and has not been ended. If unsure, call `get_run_status` first."
    )]
    async fn report_status(
        &self,
        Parameters(params): Parameters<ReportStatusRequest>,
    ) -> Result<CallToolResult, McpError> {
        match status::report_status(
            &*self.client,
            &params.run_id,
            &params.status,
            &params.message,
            &params.source,
            params.agent_status.as_deref(),
        )
        .await
        {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Status reported successfully",
            )])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to report status: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "Send a message to the conversation log. Use this for general communication that should be visible in the Hotwired dashboard. MUST only be invoked when inside an active run that has a `RUN_ID` and has not been ended."
    )]
    async fn send_message(
        &self,
        Parameters(params): Parameters<SendMessageRequest>,
    ) -> Result<CallToolResult, McpError> {
        match status::send_message(
            &*self.client,
            &params.run_id,
            &params.content,
            &params.source,
            params.target.as_deref(),
            params.agent_status.as_deref(),
        )
        .await
        {
            Ok(event_id) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Message sent (event ID: {})",
                event_id
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to send message: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "Mark a specific task as complete. Use this to signal that a piece of work is done (not the entire workflow). MUST only be invoked when inside an active run that has a `RUN_ID` and has not been ended."
    )]
    async fn task_complete(
        &self,
        Parameters(params): Parameters<TaskCompleteRequest>,
    ) -> Result<CallToolResult, McpError> {
        match status::task_complete(
            &*self.client,
            &params.run_id,
            &params.task_description,
            &params.source,
            params.outcome.as_deref(),
            params.agent_status.as_deref(),
        )
        .await
        {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Task marked as complete",
            )])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to mark task complete: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "Report a blocker or impediment that prevents progress. Use this when you need human intervention, are missing information, or encounter an error you cannot resolve. MUST only be invoked when inside an active run that has a `RUN_ID` and has not been ended."
    )]
    async fn report_impediment(
        &self,
        Parameters(params): Parameters<ReportImpedimentRequest>,
    ) -> Result<CallToolResult, McpError> {
        match status::report_impediment(
            &*self.client,
            &params.run_id,
            &params.impediment_type,
            &params.description,
            &params.source,
            params.context.as_deref(),
            params.suggestion.as_deref(),
            params.agent_status.as_deref(),
            params.response_format,
        )
        .await
        {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Impediment reported successfully",
            )])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to report impediment: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "Resolve an impediment raised by another agent. Use this when your role's protocol authorizes you to resolve certain types of impediments. The response must match the impediment's responseFormat options if one was defined. Check your role's capabilities in the protocol to know which impediment types you can resolve."
    )]
    async fn resolve_impediment(
        &self,
        Parameters(params): Parameters<ResolveImpedimentRequest>,
    ) -> Result<CallToolResult, McpError> {
        match status::resolve_impediment(
            &*self.client,
            &params.run_id,
            params.impediment_id,
            params.response,
            &params.source,
            params.rationale.as_deref(),
        )
        .await
        {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Impediment {} resolved successfully",
                params.impediment_id
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to resolve impediment: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "Request input or clarification from the human user. Use this when you need a decision, preference, or additional information to proceed. This will block the workflow until the human responds. MUST only be invoked when inside an active run that has a `RUN_ID` and has not been ended."
    )]
    async fn request_input(
        &self,
        Parameters(params): Parameters<RequestInputRequest>,
    ) -> Result<CallToolResult, McpError> {
        match status::request_input(
            &*self.client,
            &params.run_id,
            &params.question,
            &params.source,
            params.context.as_deref(),
            params.options,
        )
        .await
        {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Input requested from human. Wait for response before continuing.",
            )])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to request input: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "Hand off work to another agent. Use this when transitioning responsibility, such as when the strategist hands tasks to the builder, or vice versa. MUST only be invoked when inside an active run that has a `RUN_ID` and has not been ended."
    )]
    async fn handoff(
        &self,
        Parameters(params): Parameters<HandoffRequest>,
    ) -> Result<CallToolResult, McpError> {
        match status::handoff(
            &*self.client,
            &params.run_id,
            &params.to,
            &params.summary,
            &params.source,
            params.details.as_deref(),
            params.artifacts,
        )
        .await
        {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Handoff to {} recorded",
                params.to
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to record handoff: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "Request to end, restart, or pause the workflow. Use this when the workflow task is complete or when the scope has changed significantly. MUST only be invoked when inside an active run that has a `RUN_ID` and has not been ended."
    )]
    async fn request_end_run(
        &self,
        Parameters(params): Parameters<RequestEndRunRequest>,
    ) -> Result<CallToolResult, McpError> {
        match status::request_end_run(
            &*self.client,
            &params.run_id,
            &params.reason,
            &params.description,
            &params.source,
            params.suggested_follow_up,
        ).await {
            Ok(request_id) => Ok(CallToolResult::success(vec![
                Content::text(format!("End run request submitted (ID: {}). Other agents will be asked to confirm. Wait for consensus before proceeding.", request_id))
            ])),
            Err(e) => Ok(CallToolResult::success(vec![
                Content::text(format!("Failed to request end run: {}", e))
            ])),
        }
    }

    #[tool(
        description = "Respond to another agent's request to end the workflow. Called when you're notified that another agent wants to end the run. MUST only be invoked when inside an active run that has a `RUN_ID` and has not been ended."
    )]
    async fn respond_to_end_request(
        &self,
        Parameters(params): Parameters<RespondToEndRequestRequest>,
    ) -> Result<CallToolResult, McpError> {
        match status::respond_to_end_request(
            &*self.client,
            &params.run_id,
            &params.request_id,
            &params.response,
            &params.source,
            params.reason.as_deref(),
        )
        .await
        {
            Ok(()) => {
                let message = match params.response.as_str() {
                    "agree" => "You agreed to end the run.".to_string(),
                    "verify" => "You requested to verify before deciding. Perform your verification and call this tool again with 'agree' or 'disagree'.".to_string(),
                    "disagree" => format!("You disagreed: {}", params.reason.unwrap_or_default()),
                    _ => "Response recorded.".to_string(),
                };
                Ok(CallToolResult::success(vec![Content::text(message)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to respond to end request: {}",
                e
            ))])),
        }
    }

    // =========================================================================
    // DOC-ARTIFACT TOOLS
    // =========================================================================

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] List all document artifacts in the run. Returns artifact IDs, filenames, types, and metadata."
    )]
    async fn doc_artifact_list(
        &self,
        Parameters(params): Parameters<DocArtifactListRequest>,
    ) -> Result<CallToolResult, McpError> {
        match artifacts::list_artifacts(&*self.client, &params.run_id).await {
            Ok(response) => {
                let formatted = artifacts::format_list_response(&response);
                Ok(CallToolResult::success(vec![Content::text(formatted)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to list artifacts: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] Read content from a tracked document artifact. \
        Returns document content with metadata. For large documents, use offset/limit for pagination. \
        The response includes a contentHash for conflict detection on subsequent edits. \
        WARNING: Only use this for documents in the doc-editor. Use Read tool for source files."
    )]
    async fn doc_artifact_read(
        &self,
        Parameters(params): Parameters<DocArtifactReadRequest>,
    ) -> Result<CallToolResult, McpError> {
        match artifacts::read_artifact(
            &*self.client,
            &params.run_id,
            &params.artifact_id,
            params.offset,
            params.limit,
            params.include_comments,
        )
        .await
        {
            Ok(response) => {
                let formatted = artifacts::format_read_response(&response);
                Ok(CallToolResult::success(vec![Content::text(formatted)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to read artifact: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] Create a new document artifact in the run. \
        Returns the artifact ID and content hash for subsequent edits."
    )]
    async fn doc_artifact_create(
        &self,
        Parameters(params): Parameters<DocArtifactCreateRequest>,
    ) -> Result<CallToolResult, McpError> {
        match artifacts::create_artifact(
            &*self.client,
            &params.run_id,
            &params.filename,
            params.initial_content.as_deref(),
            params.document_type.as_deref(),
            params.created_by.as_deref(),
        )
        .await
        {
            Ok(response) => Ok(CallToolResult::success(vec![Content::text(format!(
                "✓ Artifact created successfully\n\n\
                **ID:** `{}`\n\
                **Filename:** {}\n\
                **Content Hash:** `{}`",
                response.artifact_id, response.filename, response.content_hash
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to create artifact: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] Edit content in a tracked document artifact. \
        WARNING: Only for documents in the doc-editor. Use Edit/Write tools for source files. \
        Supports edit types: replace_range, insert, append, full_replace. \
        Requires contentHash from doc_artifact_read for conflict detection."
    )]
    async fn doc_artifact_edit(
        &self,
        Parameters(params): Parameters<DocArtifactEditRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Validate edit type
        if let Err(e) = artifacts::validate_edit_type(&params.edit_type) {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Invalid edit type: {}",
                e
            ))]));
        }

        match artifacts::edit_artifact(
            &*self.client,
            &params.run_id,
            &params.artifact_id,
            &params.edit_type,
            &params.content_hash,
            &params.new_content,
            params.start_offset,
            params.end_offset,
            params.insert_offset,
            params.edit_reason.as_deref(),
            params.source.as_deref(),
        )
        .await
        {
            Ok(response) => {
                let formatted = artifacts::format_edit_response(&response);
                Ok(CallToolResult::success(vec![Content::text(formatted)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to edit artifact: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] Search within a tracked document artifact. \
        Unlike grep, this search returns character offsets for precise editing, understands markdown structure, \
        and returns surrounding context. Use this to find exact positions before making edits."
    )]
    async fn doc_artifact_search(
        &self,
        Parameters(params): Parameters<DocArtifactSearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        match artifacts::search_artifact(
            &*self.client,
            &params.run_id,
            &params.artifact_id,
            &params.query,
            params.match_type.as_deref(),
            params.scope.as_deref(),
            params.context_lines,
            params.max_results,
        )
        .await
        {
            Ok(response) => {
                let formatted = artifacts::format_search_response(&response);
                Ok(CallToolResult::success(vec![Content::text(formatted)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to search artifact: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] Add a comment to a tracked document. \
        Comment types: comment (general feedback), question (needs clarification), \
        suggestion (proposed change with replacement text), issue (problem to address). \
        Selection range is in character offsets. \
        To reply to an existing comment thread, provide the parent_comment_id."
    )]
    async fn doc_artifact_add_comment(
        &self,
        Parameters(params): Parameters<DocArtifactAddCommentRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Validate comment type
        if let Err(e) = artifacts::validate_comment_type(&params.comment_type) {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Invalid comment type: {}",
                e
            ))]));
        }

        match artifacts::add_comment(
            &*self.client,
            &params.run_id,
            &params.artifact_id,
            &params.comment_type,
            params.selection_start,
            params.selection_end,
            &params.content,
            params.suggested_text.as_deref(),
            &params.author,
            params.parent_comment_id.as_deref(),
        )
        .await
        {
            Ok(response) => Ok(CallToolResult::success(vec![Content::text(format!(
                "✓ Comment added successfully\n\n\
                **Comment ID:** `{}`\n\
                **Selected text:** \"{}\"",
                response.comment_id,
                if response.selection_text.len() > 50 {
                    format!("{}...", &response.selection_text[..50])
                } else {
                    response.selection_text
                }
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to add comment: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] Take action on a document comment. \
        Actions: accept (user approves feedback, agent should address it), \
        reject (user disagrees, closes the thread), \
        reply (continue discussion, thread stays open), \
        address (agent has addressed the feedback, closes the thread). \
        Terminal actions (reject, address) resolve the entire comment thread."
    )]
    async fn doc_artifact_resolve_comment(
        &self,
        Parameters(params): Parameters<DocArtifactResolveCommentRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Validate action
        if let Err(e) = artifacts::validate_resolve_action(&params.action) {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Invalid action: {}",
                e
            ))]));
        }

        match artifacts::resolve_comment(
            &*self.client,
            &params.run_id,
            &params.artifact_id,
            &params.comment_id,
            &params.action,
            params.response.as_deref(),
            &params.resolved_by,
        )
        .await
        {
            Ok(response) => {
                let action_verb = match params.action.as_str() {
                    "accept" => "accepted",
                    "reject" => "rejected",
                    "reply" => "replied to",
                    "address" => "addressed",
                    "resolve" => "resolved",
                    _ => "updated",
                };
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "✓ Comment {} successfully\n\n\
                    **Comment ID:** `{}`\n\
                    **New status:** {}",
                    action_verb, response.comment_id, response.new_status
                ))]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to resolve comment: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] List comments on a document artifact. \
        Filter by status (open/resolved/rejected/all) and comment type."
    )]
    async fn doc_artifact_list_comments(
        &self,
        Parameters(params): Parameters<DocArtifactListCommentsRequest>,
    ) -> Result<CallToolResult, McpError> {
        match artifacts::list_comments(
            &*self.client,
            &params.run_id,
            &params.artifact_id,
            params.status.as_deref(),
            params.comment_type.as_deref(),
            params.line_start,
            params.line_end,
        )
        .await
        {
            Ok(response) => {
                let formatted = artifacts::format_comments_response(&response);
                Ok(CallToolResult::success(vec![Content::text(formatted)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to list comments: {}",
                e
            ))])),
        }
    }

    // =========================================================================
    // EDIT SUGGESTION TOOLS (Mode 2: Suggest with diff preview)
    // =========================================================================

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] Create an edit suggestion linked to a comment. \
        Instead of directly editing, this proposes a change that the user can preview and accept/reject. \
        Edit types: replace_range (replace text between offsets), insert (insert at offset), \
        append (add to end), full_replace (replace entire content)."
    )]
    async fn doc_artifact_suggest_edit(
        &self,
        Parameters(params): Parameters<DocArtifactSuggestEditRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Validate edit type
        if let Err(e) = artifacts::validate_edit_type(&params.edit_type) {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Invalid edit type: {}",
                e
            ))]));
        }

        match artifacts::suggest_edit(
            &*self.client,
            &params.run_id,
            &params.artifact_id,
            &params.comment_id,
            &params.edit_type,
            params.start_offset,
            params.end_offset,
            &params.suggested_text,
            params.rationale.as_deref(),
            &params.source,
        )
        .await
        {
            Ok(response) => {
                let formatted = artifacts::format_suggest_edit_response(&response);
                Ok(CallToolResult::success(vec![Content::text(formatted)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to create suggestion: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] Accept an edit suggestion and apply it to the document. \
        This applies the suggested change to the document content."
    )]
    async fn doc_artifact_accept_suggestion(
        &self,
        Parameters(params): Parameters<DocArtifactAcceptSuggestionRequest>,
    ) -> Result<CallToolResult, McpError> {
        match artifacts::accept_suggestion(
            &*self.client,
            &params.run_id,
            &params.artifact_id,
            &params.suggestion_id,
            &params.source,
        )
        .await
        {
            Ok(response) => {
                let formatted = artifacts::format_accept_suggestion_response(&response);
                Ok(CallToolResult::success(vec![Content::text(formatted)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to accept suggestion: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] Reject an edit suggestion without applying it. \
        Optionally provide a reason for rejection."
    )]
    async fn doc_artifact_reject_suggestion(
        &self,
        Parameters(params): Parameters<DocArtifactRejectSuggestionRequest>,
    ) -> Result<CallToolResult, McpError> {
        match artifacts::reject_suggestion(
            &*self.client,
            &params.run_id,
            &params.artifact_id,
            &params.suggestion_id,
            params.reason.as_deref(),
            &params.source,
        )
        .await
        {
            Ok(response) => {
                let formatted = artifacts::format_reject_suggestion_response(&response);
                Ok(CallToolResult::success(vec![Content::text(formatted)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to reject suggestion: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "[[HOTWIRED DOC-EDITOR ONLY]] List edit suggestions for a document artifact. \
        Filter by status (pending/accepted/rejected/all) to see relevant suggestions."
    )]
    async fn doc_artifact_list_suggestions(
        &self,
        Parameters(params): Parameters<DocArtifactListSuggestionsRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Validate status if provided
        if let Some(ref status) = params.status {
            if let Err(e) = artifacts::validate_suggestion_status(status) {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Invalid status filter: {}",
                    e
                ))]));
            }
        }

        match artifacts::list_suggestions(
            &*self.client,
            &params.run_id,
            &params.artifact_id,
            params.status.as_deref(),
        )
        .await
        {
            Ok(response) => {
                let formatted = artifacts::format_suggestions_response(&response);
                Ok(CallToolResult::success(vec![Content::text(formatted)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to list suggestions: {}",
                e
            ))])),
        }
    }
}

// Implement the server handler
#[tool_handler]
impl<C: IpcClient + 'static> rmcp::ServerHandler for HotwiredMcp<C> {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Hotwired MCP server for multi-agent workflow coordination".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::mock::MockIpcClient;

    #[tokio::test]
    async fn test_ping_returns_connected_when_backend_available() {
        let mock = MockIpcClient::new();
        let server = HotwiredMcp::new(mock);

        let result = server.ping().await.unwrap();

        assert_eq!(result.is_error, Some(false));
        // Check that content indicates connection
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_ping_returns_helpful_error_when_backend_unavailable() {
        let mock = MockIpcClient::new();
        mock.set_disconnected(true);
        let server = HotwiredMcp::new(mock);

        let result = server.ping().await.unwrap();

        assert_eq!(
            result.is_error,
            Some(false),
            "Ping MCP call should succeed even if backend unavailable"
        );
        // The content should indicate backend is unavailable
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_get_protocol_returns_formatted_response() {
        let mock = MockIpcClient::new();
        let server = HotwiredMcp::new(mock);

        let params = GetProtocolRequest {
            run_id: "test-run".to_string(),
            agent_role: "strategist".to_string(),
        };

        let result = server.get_protocol(Parameters(params)).await.unwrap();

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_get_run_status_returns_formatted_response() {
        let mock = MockIpcClient::new();
        let server = HotwiredMcp::new(mock);

        let params = GetRunStatusRequest {
            run_id: "test-run".to_string(),
        };

        let result = server.get_run_status(Parameters(params)).await.unwrap();

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_report_status_succeeds() {
        let mock = MockIpcClient::new();
        let server = HotwiredMcp::new(mock);

        let params = ReportStatusRequest {
            run_id: "test-run".to_string(),
            status: "working".to_string(),
            message: "Implementing feature".to_string(),
            source: "strategist".to_string(),
            agent_status: None,
        };

        let result = server.report_status(Parameters(params)).await.unwrap();

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_send_message_returns_event_id() {
        let mock = MockIpcClient::new();
        let server = HotwiredMcp::new(mock);

        let params = SendMessageRequest {
            run_id: "test-run".to_string(),
            content: "Test message".to_string(),
            source: "strategist".to_string(),
            target: None,
            agent_status: None,
        };

        let result = server.send_message(Parameters(params)).await.unwrap();

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_task_complete_succeeds() {
        let mock = MockIpcClient::new();
        let server = HotwiredMcp::new(mock);

        let params = TaskCompleteRequest {
            run_id: "test-run".to_string(),
            task_description: "Implemented feature X".to_string(),
            source: "builder".to_string(),
            outcome: None,
            agent_status: None,
        };

        let result = server.task_complete(Parameters(params)).await.unwrap();

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_report_impediment_succeeds() {
        let mock = MockIpcClient::new();
        let server = HotwiredMcp::new(mock);

        let params = ReportImpedimentRequest {
            run_id: "test-run".to_string(),
            impediment_type: "missing_information".to_string(),
            description: "Need clarification on requirements".to_string(),
            source: "strategist".to_string(),
            context: None,
            suggestion: None,
            agent_status: None,
            response_format: None,
        };

        let result = server.report_impediment(Parameters(params)).await.unwrap();

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_request_input_succeeds() {
        let mock = MockIpcClient::new();
        let server = HotwiredMcp::new(mock);

        let params = RequestInputRequest {
            run_id: "test-run".to_string(),
            question: "Which approach should we use?".to_string(),
            source: "strategist".to_string(),
            context: None,
            options: None,
        };

        let result = server.request_input(Parameters(params)).await.unwrap();

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_handoff_succeeds() {
        let mock = MockIpcClient::new();
        let server = HotwiredMcp::new(mock);

        let params = HandoffRequest {
            run_id: "test-run".to_string(),
            to: "builder".to_string(),
            summary: "Ready for implementation".to_string(),
            source: "strategist".to_string(),
            details: None,
            artifacts: None,
        };

        let result = server.handoff(Parameters(params)).await.unwrap();

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_request_end_run_returns_request_id() {
        let mock = MockIpcClient::new();
        let server = HotwiredMcp::new(mock);

        let params = RequestEndRunRequest {
            run_id: "test-run".to_string(),
            reason: "completed".to_string(),
            description: "All tasks finished".to_string(),
            source: "strategist".to_string(),
            suggested_follow_up: None,
        };

        let result = server.request_end_run(Parameters(params)).await.unwrap();

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_respond_to_end_request_succeeds() {
        let mock = MockIpcClient::new();
        let server = HotwiredMcp::new(mock);

        let params = RespondToEndRequestRequest {
            run_id: "test-run".to_string(),
            request_id: "req-123".to_string(),
            response: "agree".to_string(),
            source: "builder".to_string(),
            reason: None,
        };

        let result = server
            .respond_to_end_request(Parameters(params))
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_resolve_impediment_succeeds() {
        use crate::ipc::messages::ResolveImpedimentResponse;

        let mock = MockIpcClient::new();
        mock.when_called(
            "/api/resolve-impediment",
            ResolveImpedimentResponse {
                success: true,
                error: None,
            },
        );
        let server = HotwiredMcp::new(mock);

        let params = ResolveImpedimentRequest {
            run_id: "test-run".to_string(),
            impediment_id: 42,
            response: serde_json::json!({"database": "postgresql"}),
            source: "strategist".to_string(),
            rationale: Some("PostgreSQL better suits the production requirements".to_string()),
        };

        let result = server.resolve_impediment(Parameters(params)).await.unwrap();

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }
}
