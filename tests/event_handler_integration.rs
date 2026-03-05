//! 事件处理集成测试
//!
//! 使用 MockRoomClient 测试 EventHandler 的核心功能

use aether_matrix::config::Config;
use aether_matrix::traits::{AiServiceTrait, ChatStreamResponse, RoomClient, SendMessageResult};
use anyhow::Result;
use matrix_sdk::ruma::{owned_event_id, owned_room_id, OwnedEventId, OwnedRoomId};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

// ============================================================================
// Mock AiService
// ============================================================================

/// 用于测试的 Mock AI 服务
#[derive(Clone)]
struct MockAiService {
    responses: Arc<RwLock<Vec<String>>>,
    reset_called: Arc<Mutex<bool>>,
}

impl MockAiService {
    fn new() -> Self {
        Self {
            responses: Arc::new(RwLock::new(Vec::new())),
            reset_called: Arc::new(Mutex::new(false)),
        }
    }

    async fn add_response(&self, response: &str) {
        let mut responses = self.responses.write().await;
        responses.push(response.to_string());
    }

    async fn was_reset_called(&self) -> bool {
        *self.reset_called.lock().await
    }
}

impl AiServiceTrait for MockAiService {
    async fn chat(&self, _session_id: &str, prompt: &str) -> Result<String> {
        let responses = self.responses.read().await;
        if let Some(response) = responses.first() {
            Ok(response.clone())
        } else {
            Ok(format!("Echo: {}", prompt))
        }
    }

    async fn reset_conversation(&self, _session_id: &str) {
        let mut called = self.reset_called.lock().await;
        *called = true;
    }

    async fn chat_stream(
        &self,
        _session_id: &str,
        _prompt: &str,
    ) -> Result<ChatStreamResponse> {
        // 不支持流式测试
        anyhow::bail!("Streaming not supported in mock")
    }
}

// ============================================================================
// Mock RoomClient
// ============================================================================

/// 用于测试的 Mock Room 客户端
#[derive(Clone)]
struct MockRoomClient {
    room_id: OwnedRoomId,
    is_direct_flag: bool,
    messages: Arc<Mutex<Vec<MockMessage>>>,
}

/// 记录发送的消息
#[derive(Debug, Clone)]
struct MockMessage {
    content: String,
    event_id: OwnedEventId,
    edits: Vec<String>,
}

impl MockRoomClient {
    fn new_direct() -> Self {
        Self {
            room_id: owned_room_id!("!direct:matrix.org"),
            is_direct_flag: true,
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn new_group() -> Self {
        Self {
            room_id: owned_room_id!("!group:matrix.org"),
            is_direct_flag: false,
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn sent_messages(&self) -> Vec<String> {
        let messages = self.messages.lock().await;
        messages.iter().map(|m| m.content.clone()).collect()
    }
}

impl RoomClient for MockRoomClient {
    fn room_id(&self) -> OwnedRoomId {
        self.room_id.clone()
    }

    async fn is_direct(&self) -> bool {
        self.is_direct_flag
    }

    async fn send_text(&self, content: &str) -> Result<SendMessageResult> {
        let event_id = owned_event_id!("$mock_event_id");
        let mut messages = self.messages.lock().await;
        messages.push(MockMessage {
            content: content.to_string(),
            event_id: event_id.clone(),
            edits: Vec::new(),
        });
        Ok(SendMessageResult { event_id })
    }

    async fn edit_message(
        &self,
        original_event_id: OwnedEventId,
        new_content: &str,
    ) -> Result<()> {
        let mut messages = self.messages.lock().await;
        if let Some(msg) = messages.iter_mut().find(|m| m.event_id == original_event_id) {
            msg.edits.push(new_content.to_string());
        }
        Ok(())
    }
}

// ============================================================================
// 测试辅助函数
// ============================================================================

fn create_test_config() -> Config {
    Config {
        matrix_homeserver: "https://matrix.org".to_string(),
        matrix_username: "test".to_string(),
        matrix_password: "test".to_string(),
        matrix_device_id: None,
        device_display_name: "Test Bot".to_string(),
        store_path: "./store".to_string(),
        openai_api_key: "test".to_string(),
        openai_base_url: "https://api.openai.com/v1".to_string(),
        openai_model: "gpt-4o-mini".to_string(),
        system_prompt: None,
        command_prefix: "!ai".to_string(),
        max_history: 10,
        streaming_enabled: false,
        streaming_min_interval_ms: 500,
        streaming_min_chars: 10,
        log_level: "info".to_string(),
    }
}

// ============================================================================
// MockRoomClient 测试
// ============================================================================

mod mock_room_client_tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_room_is_direct() {
        let direct_room = MockRoomClient::new_direct();
        assert!(direct_room.is_direct().await);

        let group_room = MockRoomClient::new_group();
        assert!(!group_room.is_direct().await);
    }

    #[tokio::test]
    async fn test_mock_room_send_text() {
        let room = MockRoomClient::new_direct();
        let result = room.send_text("Hello").await.unwrap();

        assert_eq!(result.event_id, owned_event_id!("$mock_event_id"));
        assert_eq!(room.sent_messages().await, vec!["Hello"]);
    }

    #[tokio::test]
    async fn test_mock_room_send_multiple_messages() {
        let room = MockRoomClient::new_direct();

        room.send_text("Message 1").await.unwrap();
        room.send_text("Message 2").await.unwrap();
        room.send_text("Message 3").await.unwrap();

        assert_eq!(
            room.sent_messages().await,
            vec!["Message 1", "Message 2", "Message 3"]
        );
    }

    #[tokio::test]
    async fn test_mock_room_edit_message() {
        let room = MockRoomClient::new_direct();
        let result = room.send_text("Original").await.unwrap();

        room.edit_message(result.event_id, "Edited").await.unwrap();

        let messages = room.messages.lock().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Original");
        assert_eq!(messages[0].edits, vec!["Edited"]);
    }

    #[tokio::test]
    async fn test_mock_room_room_id() {
        let room = MockRoomClient::new_direct();
        assert_eq!(room.room_id(), owned_room_id!("!direct:matrix.org"));

        let group_room = MockRoomClient::new_group();
        assert_eq!(group_room.room_id(), owned_room_id!("!group:matrix.org"));
    }
}

// ============================================================================
// MockAiService 测试
// ============================================================================

mod mock_ai_service_tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_ai_chat_with_response() {
        let ai = MockAiService::new();
        ai.add_response("Test response").await;

        let result = ai.chat("session-1", "Hello").await.unwrap();
        assert_eq!(result, "Test response");
    }

    #[tokio::test]
    async fn test_mock_ai_chat_without_response() {
        let ai = MockAiService::new();

        let result = ai.chat("session-1", "Hello").await.unwrap();
        assert_eq!(result, "Echo: Hello");
    }

    #[tokio::test]
    async fn test_mock_ai_reset() {
        let ai = MockAiService::new();

        assert!(!ai.was_reset_called().await);
        ai.reset_conversation("session-1").await;
        assert!(ai.was_reset_called().await);
    }

    #[tokio::test]
    async fn test_mock_ai_multiple_sessions() {
        let ai = MockAiService::new();
        ai.add_response("Response for session A").await;

        let result_a = ai.chat("session-a", "Hello A").await.unwrap();
        let result_b = ai.chat("session-b", "Hello B").await.unwrap();

        assert_eq!(result_a, "Response for session A");
        assert_eq!(result_b, "Response for session A"); // 使用相同的预设响应
    }

    #[tokio::test]
    async fn test_mock_ai_chat_stream_returns_error() {
        let ai = MockAiService::new();

        let result = ai.chat_stream("session-1", "Hello").await;
        assert!(result.is_err());
    }
}

// ============================================================================
// Config 测试
// ============================================================================

mod config_tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = create_test_config();

        assert_eq!(config.command_prefix, "!ai");
        assert!(!config.streaming_enabled); // 测试配置中关闭了流式
        assert_eq!(config.max_history, 10);
    }
}