//! AI 服务集成测试
//!
//! 使用 wiremock 模拟 OpenAI API，测试 AiService 的核心功能

use aether_matrix::ai_service::AiService;
use aether_matrix::config::Config;
use aether_matrix::traits::AiServiceTrait;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// 创建测试用的 Config，指向 mock 服务器
fn create_test_config(api_url: &str) -> Config {
    Config {
        matrix_homeserver: "https://matrix.org".to_string(),
        matrix_username: "test".to_string(),
        matrix_password: "test".to_string(),
        matrix_device_id: None,
        device_display_name: "Test Bot".to_string(),
        store_path: "./store".to_string(),
        openai_api_key: "sk-test".to_string(),
        openai_base_url: api_url.to_string(),
        openai_model: "gpt-4o-mini".to_string(),
        system_prompt: None,
        command_prefix: "!ai".to_string(),
        max_history: 10,
        bot_owners: vec![],
        db_path: "./data/aether.db".to_string(),
        streaming_enabled: true,
        streaming_min_interval_ms: 500,
        streaming_min_chars: 10,
        log_level: "info".to_string(),
        vision_enabled: true,
        vision_model: None,
        vision_max_image_size: 1024,
        proxy: None,
    }
}

/// 创建 OpenAI API 成功响应的 mock
fn mock_chat_success_response(content: &str) -> serde_json::Value {
    json!({
        "id": "chatcmpl-test",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gpt-4o-mini",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 20,
            "total_tokens": 30
        }
    })
}

// ============================================================================
// 普通聊天测试
// ============================================================================

mod chat_tests {
    use super::*;

    #[tokio::test]
    async fn test_chat_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mock_chat_success_response("Hello! How can I help?")),
            )
            .mount(&server)
            .await;

        let config = create_test_config(&server.uri());
        let service = AiService::new(&config);

        let response = service.chat("session-1", "Hi").await.unwrap();
        assert_eq!(response, "Hello! How can I help?");
    }

    #[tokio::test]
    async fn test_chat_empty_response() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-test",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "gpt-4o-mini",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": null
                    },
                    "finish_reason": "stop"
                }]
            })))
            .mount(&server)
            .await;

        let config = create_test_config(&server.uri());
        let service = AiService::new(&config);

        let response = service.chat("session-1", "Hi").await.unwrap();
        assert_eq!(response, "");
    }

    #[tokio::test]
    async fn test_chat_with_system_prompt() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mock_chat_success_response("I am helpful!")),
            )
            .mount(&server)
            .await;

        let mut config = create_test_config(&server.uri());
        config.system_prompt = Some("You are a helpful assistant.".to_string());

        let service = AiService::new(&config);

        let response = service.chat("session-1", "What are you?").await.unwrap();
        assert_eq!(response, "I am helpful!");
    }
}

// ============================================================================
// 会话管理测试
// ============================================================================

mod conversation_tests {
    use super::*;

    #[tokio::test]
    async fn test_session_isolation() {
        let server = MockServer::start().await;

        // 通用响应
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mock_chat_success_response("Response")),
            )
            .mount(&server)
            .await;

        let config = create_test_config(&server.uri());
        let service = AiService::new(&config);

        // 发送消息到不同会话
        let response_a = service.chat("session-a", "Hello A").await.unwrap();
        let response_b = service.chat("session-b", "Hello B").await.unwrap();

        // 两个会话都能正常响应
        assert_eq!(response_a, "Response");
        assert_eq!(response_b, "Response");

        // 重置 session-a 后，session-b 历史应保留
        service.reset_conversation("session-a").await;

        // session-b 仍可正常使用
        let response_b2 = service.chat("session-b", "Hello B2").await.unwrap();
        assert_eq!(response_b2, "Response");
    }

    #[tokio::test]
    async fn test_reset_conversation() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mock_chat_success_response("Response")),
            )
            .mount(&server)
            .await;

        let config = create_test_config(&server.uri());
        let service = AiService::new(&config);

        // 发送消息并获取响应
        service.chat("session-1", "Hello").await.unwrap();

        // 重置会话
        service.reset_conversation("session-1").await;

        // 再次发送消息，历史应该已被清除
        service.chat("session-1", "New message").await.unwrap();

        // 如果重置成功，请求体中不应包含之前的消息历史
        // 这里主要通过无错误来验证
    }

    #[tokio::test]
    async fn test_multiple_messages_in_session() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mock_chat_success_response("OK")),
            )
            .mount(&server)
            .await;

        let config = create_test_config(&server.uri());
        let service = AiService::new(&config);

        // 同一会话发送多条消息
        service.chat("session-1", "Message 1").await.unwrap();
        service.chat("session-1", "Message 2").await.unwrap();
        service.chat("session-1", "Message 3").await.unwrap();

        // 所有消息应该成功处理
    }
}

// ============================================================================
// AiServiceTrait 实现测试
// ============================================================================

mod trait_tests {
    use super::*;

    #[tokio::test]
    async fn test_trait_chat_delegates_correctly() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mock_chat_success_response("Trait response")),
            )
            .mount(&server)
            .await;

        let config = create_test_config(&server.uri());
        let service = AiService::new(&config);

        // 通过 trait 调用
        let response = AiServiceTrait::chat(&service, "session-1", "Hi")
            .await
            .unwrap();
        assert_eq!(response, "Trait response");
    }

    #[tokio::test]
    async fn test_trait_reset_delegates_correctly() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mock_chat_success_response("OK")),
            )
            .mount(&server)
            .await;

        let config = create_test_config(&server.uri());
        let service = AiService::new(&config);

        // 发送消息
        service.chat("session-1", "Hello").await.unwrap();

        // 通过 trait 重置
        AiServiceTrait::reset_conversation(&service, "session-1").await;

        // 验证可以继续使用
        service.chat("session-1", "New message").await.unwrap();
    }
}
