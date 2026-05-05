use crate::ipc::messages::ChannelReplyRequest;
use crate::ipc::traits::IpcClient;
use crate::types::errors::IpcError;

/// Submit an acknowledgement for a channel message previously delivered to this
/// agent as a `<channel source="hotwired-mcp" ...>` tag.
///
/// This is the agent half of the Claude Code Channels two-way protocol: outbound
/// channel notifications carry orchestrator/peer messages into the agent's input,
/// and inbound replies travel back via this tool.
///
/// The transport to `hotwired-core` is intentionally not yet wired: a follow-up
/// release will add a `submit_channel_reply` socket endpoint and a duplex
/// subscription path on the existing Unix socket. For the current release, the
/// reply is structurally validated and logged so that the MCP-level shape is
/// stable for plugin/CLI consumers.
pub async fn submit_reply<C: IpcClient>(
    _client: &C,
    request: &ChannelReplyRequest,
) -> Result<(), IpcError> {
    tracing::info!(
        status = %request.status,
        message_id = request.message_id.as_deref().unwrap_or("(unset)"),
        detail = request.detail.as_deref().unwrap_or(""),
        "channel reply received"
    );
    Ok(())
}

/// Render a reply request for human-readable confirmation back to the agent.
pub fn format_reply_ack(request: &ChannelReplyRequest) -> String {
    let mut s = format!("Acknowledged channel reply: status={}", request.status);
    if let Some(id) = &request.message_id {
        s.push_str(&format!(", message_id={}", id));
    }
    if let Some(detail) = &request.detail {
        if !detail.is_empty() {
            s.push_str(&format!(", detail={:?}", detail));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::messages::ChannelReplyStatus;
    use crate::ipc::mock::MockIpcClient;

    #[tokio::test]
    async fn test_submit_reply_succeeds_with_minimal_request() {
        let mock = MockIpcClient::new();
        let req = ChannelReplyRequest {
            status: ChannelReplyStatus::Received,
            detail: None,
            message_id: None,
        };
        submit_reply(&mock, &req)
            .await
            .expect("submit_reply must succeed");
    }

    #[tokio::test]
    async fn test_submit_reply_succeeds_with_full_request() {
        let mock = MockIpcClient::new();
        let req = ChannelReplyRequest {
            status: ChannelReplyStatus::Completed,
            detail: Some("artifact updated".into()),
            message_id: Some("msg-42".into()),
        };
        submit_reply(&mock, &req)
            .await
            .expect("submit_reply must succeed");
    }

    #[test]
    fn test_format_reply_ack_minimal() {
        let req = ChannelReplyRequest {
            status: ChannelReplyStatus::Working,
            detail: None,
            message_id: None,
        };
        assert_eq!(
            format_reply_ack(&req),
            "Acknowledged channel reply: status=working"
        );
    }

    #[test]
    fn test_format_reply_ack_full() {
        let req = ChannelReplyRequest {
            status: ChannelReplyStatus::Error,
            detail: Some("blocked".into()),
            message_id: Some("abc".into()),
        };
        let formatted = format_reply_ack(&req);
        assert!(formatted.contains("status=error"));
        assert!(formatted.contains("message_id=abc"));
        assert!(formatted.contains(r#"detail="blocked""#));
    }

    #[test]
    fn test_status_serializes_lowercase() {
        let json = serde_json::to_string(&ChannelReplyStatus::Completed).unwrap();
        assert_eq!(json, "\"completed\"");
        let parsed: ChannelReplyStatus = serde_json::from_str("\"received\"").unwrap();
        assert_eq!(parsed, ChannelReplyStatus::Received);
    }

    #[test]
    fn test_status_display_matches_serialization() {
        for s in [
            ChannelReplyStatus::Received,
            ChannelReplyStatus::Working,
            ChannelReplyStatus::Completed,
            ChannelReplyStatus::Error,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let unquoted = json.trim_matches('"');
            assert_eq!(unquoted, s.to_string());
        }
    }
}
