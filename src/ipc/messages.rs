use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// =============================================================================
// SESSION REGISTRATION (Claude Code Plugin Hooks)
// =============================================================================

/// Request to register an active Claude session
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegisterSessionRequest {
    pub session_name: String,
    pub project_dir: String,
}

/// Response to session registration
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegisterSessionResponse {
    pub success: bool,
}

/// Request to deregister a Claude session
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeregisterSessionRequest {
    pub session_name: String,
}

/// Response to session deregistration
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeregisterSessionResponse {
    pub success: bool,
}

/// Request to list active Claude sessions
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListActiveSessionsRequest {}

/// Information about an active Claude session
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActiveSessionInfo {
    pub session_name: String,
    pub project_dir: String,
    pub registered_at: i64,
}

/// Response with list of active sessions
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListActiveSessionsResponse {
    pub sessions: Vec<ActiveSessionInfo>,
}

// =============================================================================

/// Helper module to deserialize i64 that may come as string or integer.
/// AI agents sometimes send numbers as strings in JSON.
mod string_or_i64 {
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrInt {
            String(String),
            Int(i64),
        }

        match StringOrInt::deserialize(deserializer)? {
            StringOrInt::String(s) => s.parse::<i64>().map_err(serde::de::Error::custom),
            StringOrInt::Int(i) => Ok(i),
        }
    }
}

/// Helper module to deserialize Option<i64> that may come as string or integer.
mod option_string_or_i64 {
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrInt {
            String(String),
            Int(i64),
            Null,
        }

        match Option::<StringOrInt>::deserialize(deserializer)? {
            None => Ok(None),
            Some(StringOrInt::Null) => Ok(None),
            Some(StringOrInt::String(s)) if s.is_empty() => Ok(None),
            Some(StringOrInt::String(s)) => {
                s.parse::<i64>().map(Some).map_err(serde::de::Error::custom)
            }
            Some(StringOrInt::Int(i)) => Ok(Some(i)),
        }
    }
}

// ===== GET PROTOCOL =====

/// MCP-facing request (accepts agentRole from Claude/mock agents).
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetProtocolRequest {
    /// The run ID (UUID) for the active Hotwired workflow
    pub run_id: String,
    /// Your role in the workflow (e.g., "strategist", "builder")
    pub agent_role: String,
}

/// Socket-facing request (sends role to hotwired-core).
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetProtocolSocketRequest {
    pub run_id: String,
    pub role: String,
}

impl From<&GetProtocolRequest> for GetProtocolSocketRequest {
    fn from(req: &GetProtocolRequest) -> Self {
        Self {
            run_id: req.run_id.clone(),
            role: req.agent_role.clone(),
        }
    }
}

/// Role capabilities for an agent
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoleCapabilities {
    /// Whether this role can resolve impediments raised by other agents
    #[serde(default)]
    pub can_resolve_impediments: bool,
}

/// Response with protocol content.
/// Uses camelCase to match hotwired-core's response format.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetProtocolResponse {
    pub run_id: String,
    pub template_name: String,
    #[serde(default)]
    pub playbook_protocol: Option<String>,
    #[serde(default)]
    pub role_protocol: Option<String>,
    #[serde(default)]
    pub initialization_condition: Option<String>,
    #[serde(default)]
    pub project_name: Option<String>,
    /// Capabilities for this role (e.g., can resolve impediments)
    #[serde(default)]
    pub capabilities: Option<RoleCapabilities>,
}

// ===== GET RUN STATUS =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRunStatusRequest {
    /// The run ID (UUID) to check status for
    pub run_id: String,
}

/// A connected agent in a run (matches hotwired-core's ConnectedAgent)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAgent {
    /// The playbook role ID (e.g., "strategist", "builder")
    pub role_id: String,
    /// The Zellij session name
    pub session_name: String,
    /// The agent type (e.g., "claude", "gemini")
    pub agent_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetRunStatusResponse {
    pub run_id: String,
    pub status: String,
    pub phase: String,
    pub template_name: String,
    pub has_protocol: bool,
    /// Connected agents with their roles
    #[serde(default)]
    pub connected_agents: Vec<ConnectedAgent>,
}

// ===== REPORT STATUS =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReportStatusRequest {
    pub run_id: String,
    /// Your current working status: working, thinking, waiting, idle, or complete
    pub status: String,
    pub message: String,
    /// Your agent role (e.g., "strategist", "builder")
    pub source: String,
    /// Optional agent status for UI indicators (active, awaiting_response, blocked, idle)
    /// Note: hotwired-core may ignore this field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReportStatusResponse {
    pub success: bool,
}

// ===== SEND MESSAGE =====

/// Send a message event.
/// Note: This maps to create_event on the socket server side.
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub run_id: String,
    pub content: String,
    /// Your agent role (e.g., "strategist", "builder")
    pub source: String,
    /// Optional target agent or "human" for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse {
    pub success: bool,
    pub event_id: String,
}

// ===== TASK COMPLETE =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskCompleteRequest {
    pub run_id: String,
    /// Description of the task that was completed
    pub task_description: String,
    /// Your agent role (e.g., "strategist", "builder")
    pub source: String,
    /// Optional outcome description or result summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskCompleteResponse {
    pub success: bool,
}

// ===== REPORT IMPEDIMENT =====

/// A single option for radio/checkbox/select fields.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseFormatOption {
    pub value: String,
    pub label: String,
}

/// A field in the response format form.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseFormatField {
    pub id: String,
    /// Field type: radio, checkbox, select, text, textarea
    #[serde(rename = "type")]
    pub field_type: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ResponseFormatOption>>,
}

/// Schema defining how the human should respond to an impediment.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseFormat {
    pub fields: Vec<ResponseFormatField>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReportImpedimentRequest {
    pub run_id: String,
    /// Type of impediment: missing_information, permission_needed, technical_error, unclear_requirements, dependency_blocked, or other
    pub impediment_type: String,
    /// Clear description of what is blocking progress
    pub description: String,
    /// Your agent role (e.g., "strategist", "builder")
    pub source: String,
    /// Additional context about the impediment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Your suggested resolution if you have one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_status: Option<String>,
    /// Optional schema defining how the human should respond.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReportImpedimentResponse {
    pub success: bool,
}

// ===== RESOLVE IMPEDIMENT (Agent) =====

/// Resolve an impediment raised by another agent.
/// The response must match the impediment's responseFormat options if defined.
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolveImpedimentRequest {
    pub run_id: String,
    /// The event ID of the impediment to resolve
    #[serde(deserialize_with = "string_or_i64::deserialize")]
    pub impediment_id: i64,
    /// The resolution response - must match responseFormat options if defined
    pub response: serde_json::Value,
    /// Who is resolving (agent role)
    pub source: String,
    /// Optional rationale for why this agent is resolving the impediment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolveImpedimentResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ===== REQUEST INPUT =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestInputRequest {
    pub run_id: String,
    /// The question to ask the human user
    pub question: String,
    /// Your agent role (e.g., "strategist", "builder")
    pub source: String,
    /// Additional context to help the human understand the question
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Optional list of choices for multiple-choice questions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestInputResponse {
    pub success: bool,
}

// ===== HANDOFF =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HandoffRequest {
    pub run_id: String,
    /// The target agent role to hand off to (e.g., "builder", "strategist")
    pub to: String,
    /// Brief summary of what is being handed off
    pub summary: String,
    /// Your agent role (e.g., "strategist", "builder")
    pub source: String,
    /// Detailed information about the handoff
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// List of relevant file paths or artifact references
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HandoffResponse {
    pub success: bool,
}

// ===== REQUEST END RUN =====

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedFollowUp {
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestEndRunRequest {
    pub run_id: String,
    /// Reason for ending the run: completed, scope_changed, pause_requested, or error
    pub reason: String,
    /// Detailed description of why the run should end
    pub description: String,
    /// Your agent role (e.g., "strategist", "builder")
    pub source: String,
    /// Optional suggested follow-up task after this run ends
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_follow_up: Option<SuggestedFollowUp>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestEndRunResponse {
    pub success: bool,
    pub request_id: String,
}

// ===== RESPOND TO END REQUEST =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RespondToEndRequestRequest {
    pub run_id: String,
    /// The ID of the end request to respond to
    pub request_id: String,
    /// Your response: agree, disagree, or verify
    pub response: String,
    /// Your agent role (e.g., "strategist", "builder")
    pub source: String,
    /// Optional explanation for your response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RespondToEndRequestResponse {
    pub success: bool,
}

// =============================================================================
// DOC ARTIFACT MESSAGES
// =============================================================================

// ===== DOC ARTIFACT LIST =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactListRequest {
    pub run_id: String,
}

/// Artifact summary for listing
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSummary {
    pub id: String,
    pub filename: String,
    pub document_type: String,
    pub total_lines: i64,
    pub content_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactListResponse {
    pub run_id: String,
    pub artifacts: Vec<ArtifactSummary>,
}

// ===== DOC ARTIFACT READ =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactReadRequest {
    pub run_id: String,
    pub artifact_id: String,
    /// Line offset (0-based). Default: 0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// Max lines to return. Default: 500, Max: 2000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Include inline comment markers. Default: true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_comments: Option<bool>,
}

/// Inline comment for read response
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InlineComment {
    pub id: String,
    pub comment_type: String,
    pub status: String,
    pub line_number: i64,
    pub author: String,
    pub preview: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactReadResponse {
    pub artifact_id: String,
    pub filename: String,
    pub content: String,
    pub content_hash: String,
    pub total_lines: i64,
    pub returned_lines: i64,
    pub offset: i64,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<Vec<InlineComment>>,
}

// ===== DOC ARTIFACT CREATE =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactCreateRequest {
    pub run_id: String,
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_content: Option<String>,
    /// Document type: prd, spec, design, notes, other
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,
    /// Who created this artifact (agent role)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactCreateResponse {
    pub artifact_id: String,
    pub filename: String,
    pub content_hash: String,
}

// ===== DOC ARTIFACT EDIT =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactEditRequest {
    pub run_id: String,
    pub artifact_id: String,
    /// Edit type: replace_range, insert, append, full_replace
    pub edit_type: String,
    /// Hash from last read - required for conflict detection
    pub content_hash: String,
    /// New content to write
    pub new_content: String,
    /// For replace_range: start character offset
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "option_string_or_i64::deserialize"
    )]
    pub start_offset: Option<i64>,
    /// For replace_range: end character offset
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "option_string_or_i64::deserialize"
    )]
    pub end_offset: Option<i64>,
    /// For insert: insert position
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "option_string_or_i64::deserialize"
    )]
    pub insert_offset: Option<i64>,
    /// Why this edit is being made (shown in UI)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_reason: Option<String>,
    /// Who made this edit (agent role)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Conflict info when edit fails due to hash mismatch
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditConflict {
    pub expected_hash: String,
    pub actual_hash: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactEditResponse {
    pub success: bool,
    pub artifact_id: String,
    pub new_content_hash: String,
    pub edit_id: String,
    #[serde(default)]
    pub affected_comments: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict: Option<EditConflict>,
}

// ===== DOC ARTIFACT SEARCH =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactSearchRequest {
    pub run_id: String,
    pub artifact_id: String,
    /// Search query (regex supported)
    pub query: String,
    /// Match type: exact, regex, fuzzy. Default: exact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_type: Option<String>,
    /// Limit search to specific markdown elements: all, headings, body, code_blocks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Lines of context around match. Default: 2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_lines: Option<i64>,
    /// Maximum results. Default: 20
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i64>,
}

/// A search match result
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchMatch {
    pub line_number: i64,
    pub char_start: i64,
    pub char_end: i64,
    pub match_text: String,
    pub context: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactSearchResponse {
    pub artifact_id: String,
    pub query: String,
    pub total_matches: i64,
    pub results: Vec<SearchMatch>,
}

// ===== DOC ARTIFACT ADD COMMENT =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactAddCommentRequest {
    pub run_id: String,
    pub artifact_id: String,
    /// Comment type: comment, question, suggestion, issue
    pub comment_type: String,
    /// Character offset where selection starts
    #[serde(deserialize_with = "string_or_i64::deserialize")]
    pub selection_start: i64,
    /// Character offset where selection ends
    #[serde(deserialize_with = "string_or_i64::deserialize")]
    pub selection_end: i64,
    /// Comment text
    pub content: String,
    /// For suggestions: the proposed replacement text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_text: Option<String>,
    /// Who created this comment (agent role)
    pub author: String,
    /// Parent comment ID for replies (creates a thread)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_comment_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactAddCommentResponse {
    pub comment_id: String,
    pub artifact_id: String,
    pub selection_text: String,
}

// ===== DOC ARTIFACT RESOLVE COMMENT =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactResolveCommentRequest {
    pub run_id: String,
    pub artifact_id: String,
    pub comment_id: String,
    /// Action: accept, reject, reply, address, resolve
    pub action: String,
    /// Reply text or resolution note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    /// Who resolved this (agent role)
    pub resolved_by: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactResolveCommentResponse {
    pub success: bool,
    pub comment_id: String,
    pub new_status: String,
}

// ===== DOC ARTIFACT LIST COMMENTS =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactListCommentsRequest {
    pub run_id: String,
    pub artifact_id: String,
    /// Filter by status: open, resolved, rejected, all. Default: open
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Filter by type: comment, question, suggestion, issue, all
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_type: Option<String>,
    /// Filter to comments in line range
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "option_string_or_i64::deserialize"
    )]
    pub line_start: Option<i64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "option_string_or_i64::deserialize"
    )]
    pub line_end: Option<i64>,
}

/// Full comment details for listing
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommentDetail {
    pub id: String,
    pub comment_type: String,
    pub status: String,
    pub selection_start: i64,
    pub selection_end: i64,
    pub selection_text: Option<String>,
    pub content: String,
    pub suggested_text: Option<String>,
    pub author: String,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<String>,
    pub resolution_note: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactListCommentsResponse {
    pub artifact_id: String,
    pub comments: Vec<CommentDetail>,
}

// =============================================================================
// EDIT SUGGESTIONS (Mode 2: Suggest with diff preview)
// =============================================================================

// ===== DOC ARTIFACT SUGGEST EDIT =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactSuggestEditRequest {
    pub run_id: String,
    pub artifact_id: String,
    /// The comment ID this suggestion addresses (links suggestion to comment thread)
    pub comment_id: String,
    /// Edit type: replace_range, insert, append, full_replace
    pub edit_type: String,
    /// For replace_range: start character offset
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "option_string_or_i64::deserialize"
    )]
    pub start_offset: Option<i64>,
    /// For replace_range: end character offset
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "option_string_or_i64::deserialize"
    )]
    pub end_offset: Option<i64>,
    /// The suggested replacement/new text
    pub suggested_text: String,
    /// Rationale for the suggestion (shown to user)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// Who created this suggestion (agent role)
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactSuggestEditResponse {
    pub suggestion_id: String,
    pub comment_id: String,
    pub artifact_id: String,
}

// ===== DOC ARTIFACT ACCEPT SUGGESTION =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactAcceptSuggestionRequest {
    pub run_id: String,
    pub artifact_id: String,
    pub suggestion_id: String,
    /// Who accepted this suggestion
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactAcceptSuggestionResponse {
    pub success: bool,
    pub suggestion_id: String,
    pub new_content_hash: String,
    /// Comments that were resolved as a result of this acceptance
    #[serde(default)]
    pub resolved_comments: Vec<String>,
}

// ===== DOC ARTIFACT REJECT SUGGESTION =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactRejectSuggestionRequest {
    pub run_id: String,
    pub artifact_id: String,
    pub suggestion_id: String,
    /// Reason for rejecting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Who rejected this suggestion
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactRejectSuggestionResponse {
    pub success: bool,
    pub suggestion_id: String,
}

// ===== DOC ARTIFACT LIST SUGGESTIONS =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactListSuggestionsRequest {
    pub run_id: String,
    pub artifact_id: String,
    /// Filter by status: pending, accepted, rejected, all. Default: pending
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Full suggestion details for listing
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionDetail {
    pub id: String,
    pub comment_id: String,
    pub artifact_id: String,
    pub suggested_by: String,
    pub edit_type: String,
    pub start_offset: Option<i64>,
    pub end_offset: Option<i64>,
    pub original_text: Option<String>,
    pub suggested_text: String,
    pub rationale: Option<String>,
    pub status: String,
    pub accepted_by: Option<String>,
    pub accepted_at: Option<String>,
    pub rejection_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocArtifactListSuggestionsResponse {
    pub artifact_id: String,
    pub suggestions: Vec<SuggestionDetail>,
}

// =============================================================================
// TERMINAL WORKFLOW TOOLS (/hotwire, /pair)
// =============================================================================

// ===== HOTWIRE - Initiate run from terminal =====

/// Artifact suggestion for hotwire command
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotwireArtifact {
    pub path: String,
    /// Action: "create" or "use_existing"
    pub action: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HotwireRequest {
    /// The project directory path
    pub project_path: String,
    /// The Zellij session name
    pub zellij_session: String,
    /// User's intent/description of what they want to do
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    /// Suggested playbook ID if agent determined one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_playbook: Option<String>,
    /// Suggested artifacts to create or use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_artifacts: Option<Vec<HotwireArtifact>>,
}

/// Response when run starts immediately
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotwireStarted {
    pub run_id: String,
    pub role: String,
    pub playbook: String,
    pub protocol: String,
}

/// Response when user must confirm in app
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotwireNeedsConfirmation {
    pub pending_run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_playbook: Option<String>,
    pub message: String,
}

/// Hotwire response - tagged enum for easy parsing
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum HotwireResponse {
    Started(HotwireStarted),
    NeedsConfirmation(HotwireNeedsConfirmation),
    Error { error: String },
}

// ===== PAIR - Join run as second agent =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PairRequest {
    /// The Zellij session name
    pub zellij_session: String,
    /// The project directory path
    pub project_path: String,
}

/// Context about the primary agent's state when joining
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PairingContext {
    pub primary_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_artifact: Option<String>,
    pub conversation_summary: String,
}

/// Response when successfully joined a run
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PairJoined {
    pub run_id: String,
    pub role: String,
    pub role_name: String,
    pub playbook: String,
    pub protocol: String,
    pub context: PairingContext,
}

/// A run waiting for a second agent
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PendingPairRun {
    pub run_id: String,
    pub playbook: String,
    pub intent: String,
    pub role_needed: String,
}

/// Response when multiple runs need selection
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PairNeedsSelection {
    pub pending_runs: Vec<PendingPairRun>,
    pub message: String,
}

/// Response when project path doesn't match
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PairProjectMismatch {
    pub required_path: String,
    pub current_path: String,
    pub message: String,
}

/// Pair response - tagged enum for easy parsing
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PairResponse {
    Joined(PairJoined),
    NeedsSelection(PairNeedsSelection),
    #[serde(rename = "none")]
    NoneAvailable { message: String },
    ProjectMismatch(PairProjectMismatch),
    Error { error: String },
}

// ===== LIST ACTIVE RUNS =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListActiveRunsRequest {
    /// Filter by project path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
    /// Filter by Zellij session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zellij_session: Option<String>,
}

/// An active or resumable run
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActiveRun {
    pub run_id: String,
    pub playbook: String,
    pub intent: String,
    pub status: String,
    /// If this session was attached, the role it had
    #[serde(skip_serializing_if = "Option::is_none")]
    pub my_role: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListActiveRunsResponse {
    pub runs: Vec<ActiveRun>,
}

// ===== LIST PLAYBOOKS =====

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListPlaybooksRequest {}

/// Role information within a playbook
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaybookRoleInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    /// True if this role initiates the workflow (used by /hotwire)
    pub is_initiating: bool,
}

/// Hints for how to initialize a playbook
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaybookInitHints {
    /// Whether this playbook expects a document path
    pub expects_document: bool,
    /// Whether this playbook expects a goal/objective
    pub expects_goal: bool,
    /// Suggested directory paths for artifacts
    pub suggested_paths: Vec<String>,
}

/// Playbook metadata for intent matching
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaybookInfo {
    pub id: String,
    pub name: String,
    pub tagline: String,
    pub description: String,
    pub artifact_mode: bool,
    pub roles: Vec<PlaybookRoleInfo>,
    /// What this playbook is best for
    pub best_for: Vec<String>,
    /// Keywords for intent matching
    pub keywords: Vec<String>,
    pub initialization: PlaybookInitHints,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListPlaybooksResponse {
    pub playbooks: Vec<PlaybookInfo>,
}
