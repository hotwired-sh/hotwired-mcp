//! Doc-artifact tools for document editing operations.
//!
//! These tools allow agents to read, edit, search, and comment on
//! tracked document artifacts in the Hotwired doc-editor.

use crate::ipc::messages::{
    DocArtifactAcceptSuggestionRequest,
    DocArtifactAcceptSuggestionResponse,
    DocArtifactAddCommentRequest,
    DocArtifactAddCommentResponse,
    DocArtifactCreateRequest,
    DocArtifactCreateResponse,
    DocArtifactEditRequest,
    DocArtifactEditResponse,
    DocArtifactListCommentsRequest,
    DocArtifactListCommentsResponse,
    DocArtifactListRequest,
    DocArtifactListResponse,
    DocArtifactListSuggestionsRequest,
    DocArtifactListSuggestionsResponse,
    DocArtifactReadRequest,
    DocArtifactReadResponse,
    DocArtifactRejectSuggestionRequest,
    DocArtifactRejectSuggestionResponse,
    DocArtifactResolveCommentRequest,
    DocArtifactResolveCommentResponse,
    DocArtifactSearchRequest,
    DocArtifactSearchResponse,
    // Edit suggestions (Mode 2)
    DocArtifactSuggestEditRequest,
    DocArtifactSuggestEditResponse,
};
use crate::ipc::traits::IpcClient;
use crate::types::errors::IpcError;

// =============================================================================
// ARTIFACT LISTING AND READING
// =============================================================================

/// List all artifacts in a run.
pub async fn list_artifacts<C: IpcClient>(
    client: &C,
    run_id: &str,
) -> Result<DocArtifactListResponse, IpcError> {
    let request = DocArtifactListRequest {
        run_id: run_id.to_string(),
    };

    let endpoint = format!("/api/runs/{}/artifacts", run_id);
    client.request(&endpoint, &request).await
}

/// Read artifact content with pagination.
pub async fn read_artifact<C: IpcClient>(
    client: &C,
    run_id: &str,
    artifact_id: &str,
    offset: Option<i64>,
    limit: Option<i64>,
    include_comments: Option<bool>,
) -> Result<DocArtifactReadResponse, IpcError> {
    let request = DocArtifactReadRequest {
        run_id: run_id.to_string(),
        artifact_id: artifact_id.to_string(),
        offset,
        limit,
        include_comments,
    };

    let endpoint = format!("/api/runs/{}/artifacts/{}", run_id, artifact_id);
    client.request(&endpoint, &request).await
}

/// Format the read response for agent consumption.
pub fn format_read_response(response: &DocArtifactReadResponse) -> String {
    let mut output = String::new();

    output.push_str(&format!("## Document: {}\n\n", response.filename));
    output.push_str(&format!("**Content Hash:** `{}`\n", response.content_hash));
    output.push_str(&format!(
        "**Lines:** {}/{} (offset: {}, has_more: {})\n\n",
        response.returned_lines, response.total_lines, response.offset, response.has_more
    ));

    if let Some(comments) = &response.comments {
        if !comments.is_empty() {
            output.push_str("### Inline Comments\n");
            for c in comments {
                output.push_str(&format!(
                    "- [{}] Line {}: {} ({}) - \"{}\"\n",
                    c.id, c.line_number, c.comment_type, c.author, c.preview
                ));
            }
            output.push('\n');
        }
    }

    output.push_str("### Content\n\n```markdown\n");
    output.push_str(&response.content);
    output.push_str("\n```\n");

    output
}

// =============================================================================
// ARTIFACT CREATION
// =============================================================================

/// Create a new artifact.
pub async fn create_artifact<C: IpcClient>(
    client: &C,
    run_id: &str,
    filename: &str,
    initial_content: Option<&str>,
    document_type: Option<&str>,
    created_by: Option<&str>,
) -> Result<DocArtifactCreateResponse, IpcError> {
    let request = DocArtifactCreateRequest {
        run_id: run_id.to_string(),
        filename: filename.to_string(),
        initial_content: initial_content.map(String::from),
        document_type: document_type.map(String::from),
        created_by: created_by.map(String::from),
    };

    let endpoint = "/api/artifacts".to_string();
    client.request(&endpoint, &request).await
}

// =============================================================================
// ARTIFACT EDITING
// =============================================================================

/// Edit artifact content with conflict detection.
pub async fn edit_artifact<C: IpcClient>(
    client: &C,
    run_id: &str,
    artifact_id: &str,
    edit_type: &str,
    content_hash: &str,
    new_content: &str,
    start_offset: Option<i64>,
    end_offset: Option<i64>,
    insert_offset: Option<i64>,
    edit_reason: Option<&str>,
    source: Option<&str>,
) -> Result<DocArtifactEditResponse, IpcError> {
    let request = DocArtifactEditRequest {
        run_id: run_id.to_string(),
        artifact_id: artifact_id.to_string(),
        edit_type: edit_type.to_string(),
        content_hash: content_hash.to_string(),
        new_content: new_content.to_string(),
        start_offset,
        end_offset,
        insert_offset,
        edit_reason: edit_reason.map(String::from),
        source: source.map(String::from),
    };

    let endpoint = format!("/api/runs/{}/artifacts/{}/edit", run_id, artifact_id);
    client.request(&endpoint, &request).await
}

/// Validate edit type value.
pub fn validate_edit_type(edit_type: &str) -> Result<(), String> {
    const VALID: &[&str] = &["replace_range", "insert", "append", "full_replace"];
    if VALID.contains(&edit_type) {
        Ok(())
    } else {
        Err(format!(
            "Invalid edit type '{}'. Must be one of: {}",
            edit_type,
            VALID.join(", ")
        ))
    }
}

/// Format edit response for agent consumption.
pub fn format_edit_response(response: &DocArtifactEditResponse) -> String {
    if !response.success {
        if let Some(conflict) = &response.conflict {
            return format!(
                "⚠️ **CONFLICT DETECTED**\n\n\
                The document was modified since you last read it.\n\
                - Expected hash: `{}`\n\
                - Actual hash: `{}`\n\n\
                **Resolution:** Call `doc_artifact_read` to get the latest content and hash, \
                then retry your edit.",
                conflict.expected_hash, conflict.actual_hash
            );
        }
        return "Edit failed for unknown reason.".to_string();
    }

    let mut output = format!(
        "✓ Edit successful (ID: {})\n\n\
        **New content hash:** `{}`\n",
        response.edit_id, response.new_content_hash
    );

    if !response.affected_comments.is_empty() {
        output.push_str(&format!(
            "\n**Affected comments:** {}\n\
            These comments may need to be reviewed as their positions may have shifted.",
            response.affected_comments.join(", ")
        ));
    }

    output
}

// =============================================================================
// ARTIFACT SEARCH
// =============================================================================

/// Search within an artifact.
pub async fn search_artifact<C: IpcClient>(
    client: &C,
    run_id: &str,
    artifact_id: &str,
    query: &str,
    match_type: Option<&str>,
    scope: Option<&str>,
    context_lines: Option<i64>,
    max_results: Option<i64>,
) -> Result<DocArtifactSearchResponse, IpcError> {
    let request = DocArtifactSearchRequest {
        run_id: run_id.to_string(),
        artifact_id: artifact_id.to_string(),
        query: query.to_string(),
        match_type: match_type.map(String::from),
        scope: scope.map(String::from),
        context_lines,
        max_results,
    };

    let endpoint = format!("/api/runs/{}/artifacts/{}/search", run_id, artifact_id);
    client.request(&endpoint, &request).await
}

/// Format search response for agent consumption.
pub fn format_search_response(response: &DocArtifactSearchResponse) -> String {
    let mut output = format!(
        "## Search Results for: \"{}\"\n\n\
        **Total matches:** {}\n\n",
        response.query, response.total_matches
    );

    if response.results.is_empty() {
        output.push_str("No matches found.\n");
        return output;
    }

    for (i, result) in response.results.iter().enumerate() {
        output.push_str(&format!(
            "### Match {} (Line {}, chars {}-{})\n",
            i + 1,
            result.line_number,
            result.char_start,
            result.char_end
        ));

        if let Some(section) = &result.section {
            output.push_str(&format!("**Section:** {}\n", section));
        }

        output.push_str(&format!(
            "**Matched text:** `{}`\n\n\
            ```\n{}\n```\n\n",
            result.match_text, result.context
        ));
    }

    output
}

// =============================================================================
// COMMENTS
// =============================================================================

/// Add a comment to an artifact.
/// If `parent_comment_id` is provided, this creates a reply in an existing thread.
pub async fn add_comment<C: IpcClient>(
    client: &C,
    run_id: &str,
    artifact_id: &str,
    comment_type: &str,
    selection_start: i64,
    selection_end: i64,
    content: &str,
    suggested_text: Option<&str>,
    author: &str,
    parent_comment_id: Option<&str>,
) -> Result<DocArtifactAddCommentResponse, IpcError> {
    let request = DocArtifactAddCommentRequest {
        run_id: run_id.to_string(),
        artifact_id: artifact_id.to_string(),
        comment_type: comment_type.to_string(),
        selection_start,
        selection_end,
        content: content.to_string(),
        suggested_text: suggested_text.map(String::from),
        author: author.to_string(),
        parent_comment_id: parent_comment_id.map(String::from),
    };

    let endpoint = format!(
        "/api/runs/{}/artifacts/{}/comments/add",
        run_id, artifact_id
    );
    client.request(&endpoint, &request).await
}

/// Validate comment type value.
pub fn validate_comment_type(comment_type: &str) -> Result<(), String> {
    const VALID: &[&str] = &["comment", "question", "suggestion", "issue"];
    if VALID.contains(&comment_type) {
        Ok(())
    } else {
        Err(format!(
            "Invalid comment type '{}'. Must be one of: {}",
            comment_type,
            VALID.join(", ")
        ))
    }
}

/// Resolve or respond to a comment.
pub async fn resolve_comment<C: IpcClient>(
    client: &C,
    run_id: &str,
    artifact_id: &str,
    comment_id: &str,
    action: &str,
    response_text: Option<&str>,
    resolved_by: &str,
) -> Result<DocArtifactResolveCommentResponse, IpcError> {
    let request = DocArtifactResolveCommentRequest {
        run_id: run_id.to_string(),
        artifact_id: artifact_id.to_string(),
        comment_id: comment_id.to_string(),
        action: action.to_string(),
        response: response_text.map(String::from),
        resolved_by: resolved_by.to_string(),
    };

    let endpoint = format!(
        "/api/runs/{}/artifacts/{}/comments/{}/resolve",
        run_id, artifact_id, comment_id
    );
    client.request(&endpoint, &request).await
}

/// Validate resolve action value.
/// Actions:
/// - "accept": User approves the feedback, signals agent to address it (non-terminal)
/// - "reject": User disagrees with the feedback, closes the thread (terminal)
/// - "reply": Continue discussion, thread stays open (non-terminal)
/// - "address": Agent has addressed the feedback, closes the thread (terminal)
/// - "resolve": Legacy action, same as address (terminal)
pub fn validate_resolve_action(action: &str) -> Result<(), String> {
    const VALID: &[&str] = &["accept", "reject", "reply", "address", "resolve"];
    if VALID.contains(&action) {
        Ok(())
    } else {
        Err(format!(
            "Invalid action '{}'. Must be one of: {}",
            action,
            VALID.join(", ")
        ))
    }
}

/// List comments on an artifact.
pub async fn list_comments<C: IpcClient>(
    client: &C,
    run_id: &str,
    artifact_id: &str,
    status: Option<&str>,
    comment_type: Option<&str>,
    line_start: Option<i64>,
    line_end: Option<i64>,
) -> Result<DocArtifactListCommentsResponse, IpcError> {
    let request = DocArtifactListCommentsRequest {
        run_id: run_id.to_string(),
        artifact_id: artifact_id.to_string(),
        status: status.map(String::from),
        comment_type: comment_type.map(String::from),
        line_start,
        line_end,
    };

    let endpoint = format!(
        "/api/runs/{}/artifacts/{}/comments/list",
        run_id, artifact_id
    );
    client.request(&endpoint, &request).await
}

/// Format comments list response for agent consumption.
pub fn format_comments_response(response: &DocArtifactListCommentsResponse) -> String {
    let mut output = format!(
        "## Comments on Artifact: {}\n\n\
        **Total:** {} comments\n\n",
        response.artifact_id,
        response.comments.len()
    );

    if response.comments.is_empty() {
        output.push_str("No comments found matching the filter.\n");
        return output;
    }

    for comment in &response.comments {
        let status_icon = match comment.status.as_str() {
            "open" => "🔵",
            "resolved" => "✅",
            "rejected" => "❌",
            _ => "⚪",
        };

        let type_icon = match comment.comment_type.as_str() {
            "comment" => "💬",
            "question" => "❓",
            "suggestion" => "💡",
            "issue" => "⚠️",
            _ => "📝",
        };

        output.push_str(&format!(
            "### {} {} [{}] - {} (by {})\n",
            status_icon, type_icon, comment.id, comment.comment_type, comment.author
        ));

        output.push_str(&format!(
            "**Selection:** chars {}-{}\n",
            comment.selection_start, comment.selection_end
        ));

        if let Some(text) = &comment.selection_text {
            let preview = if text.len() > 50 {
                format!("{}...", &text[..50])
            } else {
                text.clone()
            };
            output.push_str(&format!("**Selected text:** \"{}\"\n", preview));
        }

        output.push_str(&format!("**Content:** {}\n", comment.content));

        if let Some(suggested) = &comment.suggested_text {
            output.push_str(&format!("**Suggested replacement:** \"{}\"\n", suggested));
        }

        if comment.status != "open" {
            if let Some(by) = &comment.resolved_by {
                output.push_str(&format!("**Resolved by:** {}\n", by));
            }
            if let Some(note) = &comment.resolution_note {
                output.push_str(&format!("**Resolution note:** {}\n", note));
            }
        }

        output.push_str(&format!("**Created:** {}\n\n", comment.created_at));
    }

    output
}

/// Format artifact list response for agent consumption.
pub fn format_list_response(response: &DocArtifactListResponse) -> String {
    let mut output = format!(
        "## Artifacts in Run: {}\n\n\
        **Total:** {} artifacts\n\n",
        response.run_id,
        response.artifacts.len()
    );

    if response.artifacts.is_empty() {
        output.push_str("No artifacts found in this run.\n");
        return output;
    }

    for artifact in &response.artifacts {
        output.push_str(&format!(
            "- **{}** ({})\n  \
            ID: `{}`\n  \
            Lines: {} | Hash: `{}`\n  \
            Updated: {}\n\n",
            artifact.filename,
            artifact.document_type,
            artifact.id,
            artifact.total_lines,
            &artifact.content_hash[..8], // Show just first 8 chars of hash
            artifact.updated_at
        ));
    }

    output
}

// =============================================================================
// EDIT SUGGESTIONS (Mode 2: Suggest with diff preview)
// =============================================================================

/// Suggest an edit for an artifact (linked to a comment).
/// This creates a suggestion that the human can Accept or Reject.
/// When accepted, the edit is applied and the linked comment is resolved.
pub async fn suggest_edit<C: IpcClient>(
    client: &C,
    run_id: &str,
    artifact_id: &str,
    comment_id: &str,
    edit_type: &str,
    start_offset: Option<i64>,
    end_offset: Option<i64>,
    suggested_text: &str,
    rationale: Option<&str>,
    source: &str,
) -> Result<DocArtifactSuggestEditResponse, IpcError> {
    let request = DocArtifactSuggestEditRequest {
        run_id: run_id.to_string(),
        artifact_id: artifact_id.to_string(),
        comment_id: comment_id.to_string(),
        edit_type: edit_type.to_string(),
        start_offset,
        end_offset,
        suggested_text: suggested_text.to_string(),
        rationale: rationale.map(String::from),
        source: source.to_string(),
    };

    let endpoint = format!("/api/runs/{}/artifacts/{}/suggestions", run_id, artifact_id);
    client.request(&endpoint, &request).await
}

/// Accept an edit suggestion and apply it to the document.
pub async fn accept_suggestion<C: IpcClient>(
    client: &C,
    run_id: &str,
    artifact_id: &str,
    suggestion_id: &str,
    source: &str,
) -> Result<DocArtifactAcceptSuggestionResponse, IpcError> {
    let request = DocArtifactAcceptSuggestionRequest {
        run_id: run_id.to_string(),
        artifact_id: artifact_id.to_string(),
        suggestion_id: suggestion_id.to_string(),
        source: source.to_string(),
    };

    let endpoint = format!(
        "/api/runs/{}/artifacts/{}/suggestions/{}/accept",
        run_id, artifact_id, suggestion_id
    );
    client.request(&endpoint, &request).await
}

/// Reject an edit suggestion.
pub async fn reject_suggestion<C: IpcClient>(
    client: &C,
    run_id: &str,
    artifact_id: &str,
    suggestion_id: &str,
    reason: Option<&str>,
    source: &str,
) -> Result<DocArtifactRejectSuggestionResponse, IpcError> {
    let request = DocArtifactRejectSuggestionRequest {
        run_id: run_id.to_string(),
        artifact_id: artifact_id.to_string(),
        suggestion_id: suggestion_id.to_string(),
        reason: reason.map(String::from),
        source: source.to_string(),
    };

    let endpoint = format!(
        "/api/runs/{}/artifacts/{}/suggestions/{}/reject",
        run_id, artifact_id, suggestion_id
    );
    client.request(&endpoint, &request).await
}

/// List edit suggestions for an artifact.
pub async fn list_suggestions<C: IpcClient>(
    client: &C,
    run_id: &str,
    artifact_id: &str,
    status: Option<&str>,
) -> Result<DocArtifactListSuggestionsResponse, IpcError> {
    let request = DocArtifactListSuggestionsRequest {
        run_id: run_id.to_string(),
        artifact_id: artifact_id.to_string(),
        status: status.map(String::from),
    };

    // Use /list suffix to distinguish from create endpoint
    let endpoint = format!(
        "/api/runs/{}/artifacts/{}/suggestions/list",
        run_id, artifact_id
    );
    client.request(&endpoint, &request).await
}

/// Validate suggestion status filter value.
pub fn validate_suggestion_status(status: &str) -> Result<(), String> {
    const VALID: &[&str] = &["pending", "accepted", "rejected", "all"];
    if VALID.contains(&status) {
        Ok(())
    } else {
        Err(format!(
            "Invalid status '{}'. Must be one of: {}",
            status,
            VALID.join(", ")
        ))
    }
}

/// Format suggestions list response for agent consumption.
pub fn format_suggestions_response(response: &DocArtifactListSuggestionsResponse) -> String {
    let mut output = format!(
        "## Edit Suggestions for Artifact: {}\n\n\
        **Total:** {} suggestions\n\n",
        response.artifact_id,
        response.suggestions.len()
    );

    if response.suggestions.is_empty() {
        output.push_str("No suggestions found matching the filter.\n");
        return output;
    }

    for suggestion in &response.suggestions {
        let status_icon = match suggestion.status.as_str() {
            "pending" => "🟡",
            "accepted" => "✅",
            "rejected" => "❌",
            _ => "⚪",
        };

        output.push_str(&format!(
            "### {} [{}] - {} (by {})\n",
            status_icon, suggestion.id, suggestion.edit_type, suggestion.suggested_by
        ));

        output.push_str(&format!(
            "**Linked to comment:** `{}`\n",
            suggestion.comment_id
        ));

        if let (Some(start), Some(end)) = (suggestion.start_offset, suggestion.end_offset) {
            output.push_str(&format!("**Selection:** chars {}-{}\n", start, end));
        }

        if let Some(original) = &suggestion.original_text {
            let preview = if original.len() > 100 {
                format!("{}...", &original[..100])
            } else {
                original.clone()
            };
            output.push_str(&format!("**Original text:** \"{}\"\n", preview));
        }

        let suggested_preview = if suggestion.suggested_text.len() > 100 {
            format!("{}...", &suggestion.suggested_text[..100])
        } else {
            suggestion.suggested_text.clone()
        };
        output.push_str(&format!("**Suggested text:** \"{}\"\n", suggested_preview));

        if let Some(rationale) = &suggestion.rationale {
            output.push_str(&format!("**Rationale:** {}\n", rationale));
        }

        if suggestion.status == "accepted" {
            if let Some(by) = &suggestion.accepted_by {
                output.push_str(&format!("**Accepted by:** {}\n", by));
            }
            if let Some(at) = &suggestion.accepted_at {
                output.push_str(&format!("**Accepted at:** {}\n", at));
            }
        } else if suggestion.status == "rejected" {
            if let Some(reason) = &suggestion.rejection_reason {
                output.push_str(&format!("**Rejection reason:** {}\n", reason));
            }
        }

        output.push_str(&format!("**Created:** {}\n\n", suggestion.created_at));
    }

    output
}

/// Format suggest_edit response for agent consumption.
pub fn format_suggest_edit_response(response: &DocArtifactSuggestEditResponse) -> String {
    format!(
        "✓ Suggestion created successfully\n\n\
        **Suggestion ID:** `{}`\n\
        **Linked to comment:** `{}`\n\
        **Artifact:** `{}`\n\n\
        The human will see a diff preview and can Accept or Reject this suggestion.\n\
        - If accepted, the edit is applied and the linked comment is marked as addressed.\n\
        - If rejected, no changes are made.",
        response.suggestion_id, response.comment_id, response.artifact_id
    )
}

/// Format accept_suggestion response for agent consumption.
pub fn format_accept_suggestion_response(response: &DocArtifactAcceptSuggestionResponse) -> String {
    let mut output = format!(
        "✓ Suggestion accepted and applied\n\n\
        **Suggestion ID:** `{}`\n\
        **New content hash:** `{}`\n",
        response.suggestion_id, response.new_content_hash
    );

    if !response.resolved_comments.is_empty() {
        output.push_str(&format!(
            "\n**Resolved comments:** {}\n",
            response.resolved_comments.join(", ")
        ));
    }

    output
}

/// Format reject_suggestion response for agent consumption.
pub fn format_reject_suggestion_response(response: &DocArtifactRejectSuggestionResponse) -> String {
    format!(
        "✓ Suggestion rejected\n\n\
        **Suggestion ID:** `{}`\n\n\
        The suggested edit was NOT applied. Consider a different approach or ask for clarification.",
        response.suggestion_id
    )
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::messages::{
        ArtifactSummary, CommentDetail, EditConflict, SearchMatch, SuggestionDetail,
    };

    // -------------------------------------------------------------------------
    // Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_edit_type_valid() {
        assert!(validate_edit_type("replace_range").is_ok());
        assert!(validate_edit_type("insert").is_ok());
        assert!(validate_edit_type("append").is_ok());
        assert!(validate_edit_type("full_replace").is_ok());
    }

    #[test]
    fn test_validate_edit_type_invalid() {
        let result = validate_edit_type("invalid_type");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid edit type"));
        assert!(err.contains("invalid_type"));
        assert!(err.contains("replace_range"));
    }

    #[test]
    fn test_validate_edit_type_empty() {
        let result = validate_edit_type("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_edit_type_case_sensitive() {
        // Edit types are case-sensitive
        assert!(validate_edit_type("REPLACE_RANGE").is_err());
        assert!(validate_edit_type("Insert").is_err());
    }

    #[test]
    fn test_validate_comment_type_valid() {
        assert!(validate_comment_type("comment").is_ok());
        assert!(validate_comment_type("question").is_ok());
        assert!(validate_comment_type("suggestion").is_ok());
        assert!(validate_comment_type("issue").is_ok());
    }

    #[test]
    fn test_validate_comment_type_invalid() {
        let result = validate_comment_type("feedback");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid comment type"));
        assert!(err.contains("feedback"));
    }

    #[test]
    fn test_validate_resolve_action_valid() {
        assert!(validate_resolve_action("accept").is_ok());
        assert!(validate_resolve_action("reject").is_ok());
        assert!(validate_resolve_action("reply").is_ok());
        assert!(validate_resolve_action("address").is_ok());
        assert!(validate_resolve_action("resolve").is_ok());
    }

    #[test]
    fn test_validate_resolve_action_invalid() {
        let result = validate_resolve_action("dismiss");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid action"));
        assert!(err.contains("dismiss"));
    }

    #[test]
    fn test_validate_suggestion_status_valid() {
        assert!(validate_suggestion_status("pending").is_ok());
        assert!(validate_suggestion_status("accepted").is_ok());
        assert!(validate_suggestion_status("rejected").is_ok());
        assert!(validate_suggestion_status("all").is_ok());
    }

    #[test]
    fn test_validate_suggestion_status_invalid() {
        let result = validate_suggestion_status("waiting");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid status"));
    }

    // -------------------------------------------------------------------------
    // Format Edit Response Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_edit_response_success() {
        let response = DocArtifactEditResponse {
            success: true,
            artifact_id: "artifact-1".to_string(),
            edit_id: "edit-123".to_string(),
            new_content_hash: "abc123def456".to_string(),
            affected_comments: vec![],
            conflict: None,
        };

        let output = format_edit_response(&response);
        assert!(output.contains("Edit successful"));
        assert!(output.contains("edit-123"));
        assert!(output.contains("abc123def456"));
        assert!(!output.contains("Affected comments"));
    }

    #[test]
    fn test_format_edit_response_success_with_affected_comments() {
        let response = DocArtifactEditResponse {
            success: true,
            artifact_id: "artifact-1".to_string(),
            edit_id: "edit-456".to_string(),
            new_content_hash: "xyz789".to_string(),
            affected_comments: vec!["comment-1".to_string(), "comment-2".to_string()],
            conflict: None,
        };

        let output = format_edit_response(&response);
        assert!(output.contains("Edit successful"));
        assert!(output.contains("Affected comments"));
        assert!(output.contains("comment-1, comment-2"));
    }

    #[test]
    fn test_format_edit_response_conflict() {
        let response = DocArtifactEditResponse {
            success: false,
            artifact_id: "artifact-1".to_string(),
            edit_id: String::new(),
            new_content_hash: String::new(),
            affected_comments: vec![],
            conflict: Some(EditConflict {
                expected_hash: "expected123".to_string(),
                actual_hash: "actual456".to_string(),
                message: "Hash mismatch".to_string(),
            }),
        };

        let output = format_edit_response(&response);
        assert!(output.contains("CONFLICT DETECTED"));
        assert!(output.contains("expected123"));
        assert!(output.contains("actual456"));
        assert!(output.contains("doc_artifact_read"));
    }

    #[test]
    fn test_format_edit_response_unknown_failure() {
        let response = DocArtifactEditResponse {
            success: false,
            artifact_id: "artifact-1".to_string(),
            edit_id: String::new(),
            new_content_hash: String::new(),
            affected_comments: vec![],
            conflict: None,
        };

        let output = format_edit_response(&response);
        assert!(output.contains("Edit failed for unknown reason"));
    }

    // -------------------------------------------------------------------------
    // Format Read Response Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_read_response_basic() {
        let response = DocArtifactReadResponse {
            artifact_id: "artifact-1".to_string(),
            filename: "document.md".to_string(),
            content: "# Hello World\n\nThis is content.".to_string(),
            content_hash: "hash123".to_string(),
            offset: 0,
            returned_lines: 3,
            total_lines: 3,
            has_more: false,
            comments: None,
        };

        let output = format_read_response(&response);
        assert!(output.contains("Document: document.md"));
        assert!(output.contains("hash123"));
        assert!(output.contains("3/3"));
        assert!(output.contains("# Hello World"));
    }

    #[test]
    fn test_format_read_response_with_comments() {
        let response = DocArtifactReadResponse {
            artifact_id: "artifact-1".to_string(),
            filename: "document.md".to_string(),
            content: "# Hello World".to_string(),
            content_hash: "hash123".to_string(),
            offset: 0,
            returned_lines: 1,
            total_lines: 1,
            has_more: false,
            comments: Some(vec![crate::ipc::messages::InlineComment {
                id: "comment-1".to_string(),
                comment_type: "suggestion".to_string(),
                status: "open".to_string(),
                line_number: 1,
                author: "critiquer".to_string(),
                preview: "Add more detail...".to_string(),
            }]),
        };

        let output = format_read_response(&response);
        assert!(output.contains("Inline Comments"));
        assert!(output.contains("comment-1"));
        assert!(output.contains("Line 1"));
        assert!(output.contains("suggestion"));
        assert!(output.contains("critiquer"));
    }

    #[test]
    fn test_format_read_response_with_pagination() {
        let response = DocArtifactReadResponse {
            artifact_id: "artifact-1".to_string(),
            filename: "long-document.md".to_string(),
            content: "First 100 lines...".to_string(),
            content_hash: "hash123".to_string(),
            offset: 100,
            returned_lines: 100,
            total_lines: 500,
            has_more: true,
            comments: None,
        };

        let output = format_read_response(&response);
        assert!(output.contains("100/500"));
        assert!(output.contains("offset: 100"));
        assert!(output.contains("has_more: true"));
    }

    // -------------------------------------------------------------------------
    // Format Search Response Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_search_response_no_matches() {
        let response = DocArtifactSearchResponse {
            artifact_id: "artifact-1".to_string(),
            query: "nonexistent".to_string(),
            total_matches: 0,
            results: vec![],
        };

        let output = format_search_response(&response);
        assert!(output.contains("Search Results for: \"nonexistent\""));
        assert!(output.contains("Total matches:** 0"));
        assert!(output.contains("No matches found"));
    }

    #[test]
    fn test_format_search_response_with_matches() {
        let response = DocArtifactSearchResponse {
            artifact_id: "artifact-1".to_string(),
            query: "hello".to_string(),
            total_matches: 2,
            results: vec![
                SearchMatch {
                    line_number: 5,
                    char_start: 0,
                    char_end: 5,
                    match_text: "hello".to_string(),
                    context: "hello world".to_string(),
                    section: Some("Introduction".to_string()),
                },
                SearchMatch {
                    line_number: 10,
                    char_start: 3,
                    char_end: 8,
                    match_text: "hello".to_string(),
                    context: "   hello again".to_string(),
                    section: None,
                },
            ],
        };

        let output = format_search_response(&response);
        assert!(output.contains("Total matches:** 2"));
        assert!(output.contains("Match 1 (Line 5"));
        assert!(output.contains("Match 2 (Line 10"));
        assert!(output.contains("Section:** Introduction"));
        assert!(output.contains("Matched text:** `hello`"));
    }

    // -------------------------------------------------------------------------
    // Format Comments Response Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_comments_response_empty() {
        let response = DocArtifactListCommentsResponse {
            artifact_id: "artifact-1".to_string(),
            comments: vec![],
        };

        let output = format_comments_response(&response);
        assert!(output.contains("Total:** 0 comments"));
        assert!(output.contains("No comments found"));
    }

    #[test]
    fn test_format_comments_response_with_comments() {
        let response = DocArtifactListCommentsResponse {
            artifact_id: "artifact-1".to_string(),
            comments: vec![CommentDetail {
                id: "comment-123".to_string(),
                comment_type: "suggestion".to_string(),
                status: "open".to_string(),
                author: "critiquer".to_string(),
                selection_start: 0,
                selection_end: 50,
                selection_text: Some("Some selected text".to_string()),
                content: "This needs improvement".to_string(),
                suggested_text: Some("Better text".to_string()),
                created_at: "2024-01-15T10:30:00Z".to_string(),
                resolved_by: None,
                resolved_at: None,
                resolution_note: None,
            }],
        };

        let output = format_comments_response(&response);
        assert!(output.contains("comment-123"));
        assert!(output.contains("🔵")); // open status icon
        assert!(output.contains("💡")); // suggestion type icon
        assert!(output.contains("critiquer"));
        assert!(output.contains("chars 0-50"));
        assert!(output.contains("This needs improvement"));
        assert!(output.contains("Suggested replacement"));
    }

    #[test]
    fn test_format_comments_response_resolved() {
        let response = DocArtifactListCommentsResponse {
            artifact_id: "artifact-1".to_string(),
            comments: vec![CommentDetail {
                id: "comment-456".to_string(),
                comment_type: "question".to_string(),
                status: "resolved".to_string(),
                author: "writer".to_string(),
                selection_start: 10,
                selection_end: 20,
                selection_text: None,
                content: "What does this mean?".to_string(),
                suggested_text: None,
                created_at: "2024-01-15T10:30:00Z".to_string(),
                resolved_by: Some("human".to_string()),
                resolved_at: Some("2024-01-15T11:00:00Z".to_string()),
                resolution_note: Some("Clarified the meaning".to_string()),
            }],
        };

        let output = format_comments_response(&response);
        assert!(output.contains("✅")); // resolved status
        assert!(output.contains("❓")); // question type
        assert!(output.contains("Resolved by:** human"));
        assert!(output.contains("Resolution note:** Clarified"));
    }

    // -------------------------------------------------------------------------
    // Format List Response Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_list_response_empty() {
        let response = DocArtifactListResponse {
            run_id: "run-123".to_string(),
            artifacts: vec![],
        };

        let output = format_list_response(&response);
        assert!(output.contains("Artifacts in Run: run-123"));
        assert!(output.contains("Total:** 0 artifacts"));
        assert!(output.contains("No artifacts found"));
    }

    #[test]
    fn test_format_list_response_with_artifacts() {
        let response = DocArtifactListResponse {
            run_id: "run-123".to_string(),
            artifacts: vec![
                ArtifactSummary {
                    id: "artifact-1".to_string(),
                    filename: "prd.md".to_string(),
                    document_type: "prd".to_string(),
                    total_lines: 150,
                    content_hash: "abcdef123456789012345678".to_string(),
                    created_at: "2024-01-15T10:00:00Z".to_string(),
                    updated_at: "2024-01-15T10:30:00Z".to_string(),
                },
                ArtifactSummary {
                    id: "artifact-2".to_string(),
                    filename: "design.md".to_string(),
                    document_type: "design".to_string(),
                    total_lines: 75,
                    content_hash: "xyz789abcdef0123456789ab".to_string(),
                    created_at: "2024-01-15T10:30:00Z".to_string(),
                    updated_at: "2024-01-15T11:00:00Z".to_string(),
                },
            ],
        };

        let output = format_list_response(&response);
        assert!(output.contains("Total:** 2 artifacts"));
        assert!(output.contains("prd.md"));
        assert!(output.contains("design.md"));
        assert!(output.contains("Lines: 150"));
        assert!(output.contains("Lines: 75"));
        // Hash should be truncated to 8 chars
        assert!(output.contains("abcdef12"));
    }

    // -------------------------------------------------------------------------
    // Format Suggestions Response Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_suggestions_response_empty() {
        let response = DocArtifactListSuggestionsResponse {
            artifact_id: "artifact-1".to_string(),
            suggestions: vec![],
        };

        let output = format_suggestions_response(&response);
        assert!(output.contains("Total:** 0 suggestions"));
        assert!(output.contains("No suggestions found"));
    }

    #[test]
    fn test_format_suggestions_response_pending() {
        let response = DocArtifactListSuggestionsResponse {
            artifact_id: "artifact-1".to_string(),
            suggestions: vec![SuggestionDetail {
                id: "suggestion-1".to_string(),
                comment_id: "comment-1".to_string(),
                artifact_id: "artifact-1".to_string(),
                edit_type: "replace_range".to_string(),
                status: "pending".to_string(),
                suggested_by: "writer".to_string(),
                start_offset: Some(0),
                end_offset: Some(50),
                original_text: Some("Original content here".to_string()),
                suggested_text: "Improved content here".to_string(),
                rationale: Some("Better clarity".to_string()),
                created_at: "2024-01-15T10:30:00Z".to_string(),
                updated_at: "2024-01-15T10:30:00Z".to_string(),
                accepted_by: None,
                accepted_at: None,
                rejection_reason: None,
            }],
        };

        let output = format_suggestions_response(&response);
        assert!(output.contains("🟡")); // pending icon
        assert!(output.contains("suggestion-1"));
        assert!(output.contains("comment-1"));
        assert!(output.contains("replace_range"));
        assert!(output.contains("writer"));
        assert!(output.contains("chars 0-50"));
        assert!(output.contains("Better clarity"));
    }

    #[test]
    fn test_format_suggestions_response_accepted() {
        let response = DocArtifactListSuggestionsResponse {
            artifact_id: "artifact-1".to_string(),
            suggestions: vec![SuggestionDetail {
                id: "suggestion-2".to_string(),
                comment_id: "comment-2".to_string(),
                artifact_id: "artifact-1".to_string(),
                edit_type: "insert".to_string(),
                status: "accepted".to_string(),
                suggested_by: "writer".to_string(),
                start_offset: None,
                end_offset: None,
                original_text: None,
                suggested_text: "New content".to_string(),
                rationale: None,
                created_at: "2024-01-15T10:30:00Z".to_string(),
                updated_at: "2024-01-15T11:00:00Z".to_string(),
                accepted_by: Some("human".to_string()),
                accepted_at: Some("2024-01-15T11:00:00Z".to_string()),
                rejection_reason: None,
            }],
        };

        let output = format_suggestions_response(&response);
        assert!(output.contains("✅")); // accepted icon
        assert!(output.contains("Accepted by:** human"));
        assert!(output.contains("Accepted at:** 2024-01-15T11:00:00Z"));
    }

    #[test]
    fn test_format_suggestions_response_rejected() {
        let response = DocArtifactListSuggestionsResponse {
            artifact_id: "artifact-1".to_string(),
            suggestions: vec![SuggestionDetail {
                id: "suggestion-3".to_string(),
                comment_id: "comment-3".to_string(),
                artifact_id: "artifact-1".to_string(),
                edit_type: "append".to_string(),
                status: "rejected".to_string(),
                suggested_by: "writer".to_string(),
                start_offset: None,
                end_offset: None,
                original_text: None,
                suggested_text: "Unwanted content".to_string(),
                rationale: None,
                created_at: "2024-01-15T10:30:00Z".to_string(),
                updated_at: "2024-01-15T10:35:00Z".to_string(),
                accepted_by: None,
                accepted_at: None,
                rejection_reason: Some("Not relevant to scope".to_string()),
            }],
        };

        let output = format_suggestions_response(&response);
        assert!(output.contains("❌")); // rejected icon
        assert!(output.contains("Rejection reason:** Not relevant to scope"));
    }

    // -------------------------------------------------------------------------
    // Format Other Response Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_suggest_edit_response() {
        let response = DocArtifactSuggestEditResponse {
            suggestion_id: "suggestion-new".to_string(),
            comment_id: "comment-linked".to_string(),
            artifact_id: "artifact-1".to_string(),
        };

        let output = format_suggest_edit_response(&response);
        assert!(output.contains("Suggestion created successfully"));
        assert!(output.contains("suggestion-new"));
        assert!(output.contains("comment-linked"));
        assert!(output.contains("artifact-1"));
        assert!(output.contains("Accept or Reject"));
    }

    #[test]
    fn test_format_accept_suggestion_response() {
        let response = DocArtifactAcceptSuggestionResponse {
            success: true,
            suggestion_id: "suggestion-accepted".to_string(),
            new_content_hash: "newhash123".to_string(),
            resolved_comments: vec!["comment-1".to_string(), "comment-2".to_string()],
        };

        let output = format_accept_suggestion_response(&response);
        assert!(output.contains("Suggestion accepted and applied"));
        assert!(output.contains("suggestion-accepted"));
        assert!(output.contains("newhash123"));
        assert!(output.contains("Resolved comments"));
        assert!(output.contains("comment-1, comment-2"));
    }

    #[test]
    fn test_format_accept_suggestion_response_no_resolved() {
        let response = DocArtifactAcceptSuggestionResponse {
            success: true,
            suggestion_id: "suggestion-123".to_string(),
            new_content_hash: "hash456".to_string(),
            resolved_comments: vec![],
        };

        let output = format_accept_suggestion_response(&response);
        assert!(output.contains("Suggestion accepted"));
        assert!(!output.contains("Resolved comments"));
    }

    #[test]
    fn test_format_reject_suggestion_response() {
        let response = DocArtifactRejectSuggestionResponse {
            success: true,
            suggestion_id: "suggestion-rejected".to_string(),
        };

        let output = format_reject_suggestion_response(&response);
        assert!(output.contains("Suggestion rejected"));
        assert!(output.contains("suggestion-rejected"));
        assert!(output.contains("NOT applied"));
    }
}
