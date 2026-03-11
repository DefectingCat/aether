//! 事件处理集成测试

use aether_matrix::config::Config;
use aether_matrix::traits::{AiServiceTrait, ChatStreamResponse};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

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

    async fn chat_with_system(
        &self,
        _session_id: &str,
        prompt: &str,
        _system_prompt: Option<&str>,
    ) -> Result<String> {
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

    async fn chat_stream(&self, _session_id: &str, _prompt: &str) -> Result<ChatStreamResponse> {
        anyhow::bail!("Streaming not supported in mock")
    }

    async fn chat_stream_with_system(
        &self,
        _session_id: &str,
        _prompt: &str,
        _system_prompt: Option<&str>,
    ) -> Result<ChatStreamResponse> {
        anyhow::bail!("Streaming not supported in mock")
    }

    async fn chat_with_image(
        &self,
        _session_id: &str,
        _text: &str,
        _image_data_url: &str,
    ) -> Result<String> {
        let responses = self.responses.read().await;
        if let Some(response) = responses.first() {
            Ok(response.clone())
        } else {
            Ok("Mock vision response".to_string())
        }
    }

    async fn chat_with_image_stream(
        &self,
        _session_id: &str,
        _text: &str,
        _image_data_url: &str,
    ) -> Result<ChatStreamResponse> {
        anyhow::bail!("Streaming not supported in mock")
    }

    async fn chat_with_tools(
        &self,
        _session_id: &str,
        prompt: &str,
        _system_prompt: Option<&str>,
    ) -> Result<String> {
        self.chat_with_system(_session_id, prompt, _system_prompt)
            .await
    }

    fn mcp_server_manager(&self) -> Option<Arc<RwLock<aether_matrix::mcp::McpServerManager>>> {
        None
    }

    fn inner_mcp_registry(&self) -> Option<Arc<RwLock<aether_matrix::mcp::ToolRegistry>>> {
        None
    }

    async fn list_mcp_tools(&self) -> Vec<aether_matrix::mcp::ToolDefinition> {
        vec![]
    }

    async fn has_tools(&self) -> bool {
        false
    }
}

fn create_test_config() -> Config {
    Config {
        matrix: aether_matrix::config::MatrixConfig {
            homeserver: "https://matrix.org".to_string(),
            username: "test".to_string(),
            password: "test".to_string(),
            device_id: None,
            device_display_name: "Test Bot".to_string(),
            store_path: "./store".to_string(),
        },
        openai: aether_matrix::config::OpenAiConfig {
            api_key: "test".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            system_prompt: None,
        },
        bot: aether_matrix::config::BotConfig {
            command_prefix: "!ai".to_string(),
            max_history: 10,
            owners: vec![],
            db_path: "./data/aether.db".to_string(),
        },
        streaming: aether_matrix::config::StreamingConfig {
            enabled: false,
            min_interval_ms: 500,
            min_chars: 10,
        },
        vision: aether_matrix::config::VisionConfig {
            enabled: true,
            model: None,
            max_image_size: 1024,
        },
        log: aether_matrix::config::LogConfig {
            level: "info".to_string(),
        },
        proxy: None,
        mcp: aether_matrix::mcp::McpConfig::default(),
        meme: aether_matrix::config::MemeConfig::default(),
    }
}

mod mock_ai_service_tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mock_ai_chat_with_response() {
        let ai = MockAiService::new();
        ai.add_response("Test response").await;

        let result = ai.chat("session-1", "Hello").await.unwrap();
        assert_eq!(result, "Test response");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mock_ai_chat_without_response() {
        let ai = MockAiService::new();

        let result = ai.chat("session-1", "Hello").await.unwrap();
        assert_eq!(result, "Echo: Hello");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mock_ai_reset() {
        let ai = MockAiService::new();

        assert!(!ai.was_reset_called().await);
        ai.reset_conversation("session-1").await;
        assert!(ai.was_reset_called().await);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mock_ai_multiple_sessions() {
        let ai = MockAiService::new();
        ai.add_response("Response for session A").await;

        let result_a = ai.chat("session-a", "Hello A").await.unwrap();
        let result_b = ai.chat("session-b", "Hello B").await.unwrap();

        assert_eq!(result_a, "Response for session A");
        assert_eq!(result_b, "Response for session A");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mock_ai_chat_stream_returns_error() {
        let ai = MockAiService::new();

        let result = ai.chat_stream("session-1", "Hello").await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mock_ai_chat_with_image() {
        let ai = MockAiService::new();
        ai.add_response("This is an image description").await;

        let result = ai
            .chat_with_image(
                "session-1",
                "What's in this image?",
                "data:image/png;base64,abc",
            )
            .await
            .unwrap();
        assert_eq!(result, "This is an image description");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mock_ai_chat_with_image_stream_returns_error() {
        let ai = MockAiService::new();

        let result = ai
            .chat_with_image_stream(
                "session-1",
                "What's in this image?",
                "data:image/png;base64,abc",
            )
            .await;
        assert!(result.is_err());
    }
}

mod config_tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = create_test_config();

        assert_eq!(config.bot.command_prefix, "!ai");
        assert!(!config.streaming.enabled);
        assert_eq!(config.bot.max_history, 10);
        assert!(config.vision.enabled);
        assert_eq!(config.vision.max_image_size, 1024);
    }
}
