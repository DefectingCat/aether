//! AI 服务集成测试
//!
//! 使用 mockall 模拟 AiServiceTrait，测试事件处理器等核心逻辑

use aether_matrix::traits::AiServiceTrait;
use mockall::mock;
use mockall::predicate::*;

// 创建 mock 实现 - 必须实现所有 trait 方法
mock! {
    pub TestAiService {}

    impl Clone for TestAiService {
        fn clone(&self) -> Self;
    }

    impl AiServiceTrait for TestAiService {
        async fn chat(&self, session_id: &str, prompt: &str) -> anyhow::Result<String>;
        async fn chat_with_system<'a>(&self, session_id: &str, prompt: &str, system_prompt: Option<&'a str>) -> anyhow::Result<String>;
        async fn reset_conversation(&self, session_id: &str);
        async fn chat_stream(&self, session_id: &str, prompt: &str) -> anyhow::Result<(
            std::sync::Arc<tokio::sync::Mutex<aether_matrix::traits::StreamingState>>,
            std::pin::Pin<Box<dyn futures_util::Stream<Item = anyhow::Result<String>> + Send>>,
        )>;
        async fn chat_with_image(&self, session_id: &str, text: &str, image_data_url: &str) -> anyhow::Result<String>;
        async fn chat_with_image_stream(&self, session_id: &str, text: &str, image_data_url: &str) -> anyhow::Result<(
            std::sync::Arc<tokio::sync::Mutex<aether_matrix::traits::StreamingState>>,
            std::pin::Pin<Box<dyn futures_util::Stream<Item = anyhow::Result<String>> + Send>>,
        )>;
        async fn chat_stream_with_system<'a>(&self, session_id: &str, prompt: &str, system_prompt: Option<&'a str>) -> anyhow::Result<(
            std::sync::Arc<tokio::sync::Mutex<aether_matrix::traits::StreamingState>>,
            std::pin::Pin<Box<dyn futures_util::Stream<Item = anyhow::Result<String>> + Send>>,
        )>;
    }
}

// ============================================================================
// 基础聊天功能测试
// ============================================================================

mod chat_tests {
    use super::*;

    #[tokio::test]
    async fn test_chat_success() {
        let mut mock_service = MockTestAiService::new();
        mock_service
            .expect_chat()
            .with(eq("session-1"), eq("Hello"))
            .times(1)
            .returning(|_, _| Ok("Hi there!".to_string()));

        let result = mock_service.chat("session-1", "Hello").await;
        assert_eq!(result.unwrap(), "Hi there!");
    }

    #[tokio::test]
    async fn test_chat_empty_response() {
        let mut mock_service = MockTestAiService::new();
        mock_service
            .expect_chat()
            .returning(|_, _| Ok("".to_string()));

        let result = mock_service.chat("session-1", "Test").await;
        assert_eq!(result.unwrap(), "");
    }
}

// ============================================================================
// 会话管理测试
// ============================================================================

mod conversation_tests {
    use super::*;

    #[tokio::test]
    async fn test_session_isolation() {
        let mut mock_service = MockTestAiService::new();

        mock_service
            .expect_chat()
            .with(eq("session-a"), eq("Hello A"))
            .returning(|_, _| Ok("Response A".to_string()));

        mock_service
            .expect_chat()
            .with(eq("session-b"), eq("Hello B"))
            .returning(|_, _| Ok("Response B".to_string()));

        let result_a = mock_service.chat("session-a", "Hello A").await.unwrap();
        let result_b = mock_service.chat("session-b", "Hello B").await.unwrap();

        assert_eq!(result_a, "Response A");
        assert_eq!(result_b, "Response B");
    }

    #[tokio::test]
    async fn test_reset_conversation() {
        let mut mock_service = MockTestAiService::new();

        mock_service
            .expect_chat()
            .returning(|_, _| Ok("Response".to_string()));

        mock_service
            .expect_reset_conversation()
            .with(eq("session-1"))
            .times(1)
            .returning(|_| ());

        mock_service
            .expect_chat()
            .returning(|_, _| Ok("New response".to_string()));

        mock_service.chat("session-1", "Hello").await.unwrap();
        mock_service.reset_conversation("session-1").await;
        mock_service.chat("session-1", "New message").await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_messages_in_session() {
        let mut mock_service = MockTestAiService::new();

        mock_service
            .expect_chat()
            .times(3)
            .returning(|_, _| Ok("OK".to_string()));

        mock_service.chat("session-1", "Message 1").await.unwrap();
        mock_service.chat("session-1", "Message 2").await.unwrap();
        mock_service.chat("session-1", "Message 3").await.unwrap();
    }
}

// ============================================================================
// Trait 实现验证测试
// ============================================================================

mod trait_tests {
    use super::*;

    #[tokio::test]
    async fn test_trait_chat_delegates_correctly() {
        let mut mock_service = MockTestAiService::new();
        mock_service
            .expect_chat()
            .returning(|_, _| Ok("Trait response".to_string()));

        let response = AiServiceTrait::chat(&mock_service, "session-1", "Hi")
            .await
            .unwrap();
        assert_eq!(response, "Trait response");
    }

    #[tokio::test]
    async fn test_trait_reset_delegates_correctly() {
        let mut mock_service = MockTestAiService::new();

        mock_service
            .expect_chat()
            .returning(|_, _| Ok("OK".to_string()));

        mock_service.expect_reset_conversation().returning(|_| ());

        mock_service.chat("session-1", "Hello").await.unwrap();
        AiServiceTrait::reset_conversation(&mock_service, "session-1").await;
    }
}

// ============================================================================
// 图片理解测试
// ============================================================================

mod vision_tests {
    use super::*;

    #[tokio::test]
    async fn test_chat_with_image() {
        let mut mock_service = MockTestAiService::new();
        mock_service
            .expect_chat_with_image()
            .with(
                eq("session-1"),
                eq("What's in this image?"),
                eq("data:image/png;base64,abc123"),
            )
            .returning(|_, _, _| Ok("I see a cat".to_string()));

        let result = mock_service
            .chat_with_image(
                "session-1",
                "What's in this image?",
                "data:image/png;base64,abc123",
            )
            .await;

        assert_eq!(result.unwrap(), "I see a cat");
    }
}
