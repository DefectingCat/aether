//! Vision API 集成测试

use aether_matrix::ai_service::AiService;
use aether_matrix::config::Config;
use aether_matrix::traits::AiServiceTrait;
use serde_json::json;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

fn mock_vision_response(description: &str) -> serde_json::Value {
    json!({
        "id": "chatcmpl-vision-test",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": description
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 100,
            "completion_tokens": 50,
            "total_tokens": 150
        }
    })
}

mod vision_tests {
    use super::*;

    #[tokio::test]
    async fn test_chat_with_image_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(body_string_contains("image_url"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mock_vision_response("This is a cat sitting on a chair.")),
            )
            .mount(&server)
            .await;

        let config = create_test_config(&server.uri());
        let service = AiService::new(&config);

        let image_data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

        let response = service
            .chat_with_image("session-1", "What's in this image?", image_data_url)
            .await
            .unwrap();

        assert_eq!(response, "This is a cat sitting on a chair.");
    }

    #[tokio::test]
    async fn test_chat_with_image_empty_response() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-vision-test",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "gpt-4o",
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

        let image_data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

        let response = service
            .chat_with_image("session-1", "What's in this image?", image_data_url)
            .await
            .unwrap();

        assert_eq!(response, "");
    }

    #[tokio::test]
    async fn test_chat_with_image_trait_method() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mock_vision_response("A beautiful landscape.")),
            )
            .mount(&server)
            .await;

        let config = create_test_config(&server.uri());
        let service = AiService::new(&config);

        let image_data_url = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/2wBDAQkJCQwLDBgNDRgyIRwhMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjL/wAARCAABAAEDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAn/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFQEBAQAAAAAAAAAAAAAAAAAAAAX/xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oADAMBEQACEQA/ALUABo//2Q==";

        let response = AiServiceTrait::chat_with_image(
            &service,
            "session-1",
            "Describe this image",
            image_data_url,
        )
        .await
        .unwrap();

        assert_eq!(response, "A beautiful landscape.");
    }

    #[tokio::test]
    async fn test_chat_with_image_multiple_sessions() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mock_vision_response("Response")),
            )
            .mount(&server)
            .await;

        let config = create_test_config(&server.uri());
        let service = AiService::new(&config);

        let image_data_url = "data:image/png;base64,test";

        let response_a = service
            .chat_with_image("session-a", "Question A", image_data_url)
            .await
            .unwrap();
        let response_b = service
            .chat_with_image("session-b", "Question B", image_data_url)
            .await
            .unwrap();

        assert_eq!(response_a, "Response");
        assert_eq!(response_b, "Response");
    }
}
