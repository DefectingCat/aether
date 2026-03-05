use aether_matrix::config::Config;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod config_tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let config = Config::default();

        assert_eq!(config.matrix_homeserver, "https://matrix.org");
        assert_eq!(config.matrix_username, "");
        assert_eq!(config.matrix_password, "");
        assert_eq!(config.matrix_device_id, None);
        assert_eq!(config.device_display_name, "AI Bot");
        assert_eq!(config.store_path, "./store");
        assert_eq!(config.openai_api_key, "");
        assert_eq!(config.openai_base_url, "https://api.openai.com/v1");
        assert_eq!(config.openai_model, "gpt-4o-mini");
        assert_eq!(config.system_prompt, None);
        assert_eq!(config.command_prefix, "!");
        assert_eq!(config.max_history, 10);
        assert!(config.streaming_enabled);
        assert_eq!(config.streaming_min_interval_ms, 1000);
        assert_eq!(config.streaming_min_chars, 50);
        assert_eq!(config.log_level, "info");
        assert!(config.vision_enabled);
        assert_eq!(config.vision_model, None);
        assert_eq!(config.vision_max_image_size, 1024);
    }

    #[test]
    fn test_config_can_be_cloned() {
        let config = Config::default();
        let cloned = config.clone();
        assert_eq!(config.matrix_homeserver, cloned.matrix_homeserver);
    }

    #[test]
    fn test_config_debug_impl() {
        let config = Config::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("matrix_homeserver"));
    }

    #[test]
    fn test_config_custom_values() {
        let config = Config {
            matrix_homeserver: "https://custom.server".to_string(),
            matrix_username: "custom_user".to_string(),
            matrix_password: "custom_pass".to_string(),
            matrix_device_id: Some("DEVICE123".to_string()),
            device_display_name: "Custom Bot".to_string(),
            store_path: "/tmp/custom_store".to_string(),
            openai_api_key: "sk-custom".to_string(),
            openai_base_url: "https://custom.api/v1".to_string(),
            openai_model: "custom-model".to_string(),
            system_prompt: Some("You are helpful".to_string()),
            command_prefix: "!custom".to_string(),
            max_history: 20,
            bot_owners: vec!["@admin:matrix.org".to_string()],
            db_path: "./data/aether.db".to_string(),
            streaming_enabled: false,
            streaming_min_interval_ms: 2000,
            streaming_min_chars: 100,
            log_level: "debug".to_string(),
            vision_enabled: false,
            vision_model: Some("gpt-4o".to_string()),
            vision_max_image_size: 2048,
            proxy: None,
        };

        assert_eq!(config.matrix_homeserver, "https://custom.server");
        assert_eq!(config.matrix_username, "custom_user");
        assert_eq!(config.matrix_device_id, Some("DEVICE123".to_string()));
        assert_eq!(config.store_path, "/tmp/custom_store");
        assert_eq!(config.openai_model, "custom-model");
        assert_eq!(config.max_history, 20);
        assert!(!config.streaming_enabled);
        assert!(!config.vision_enabled);
        assert_eq!(config.vision_model, Some("gpt-4o".to_string()));
    }
}

mod bot_tests {
    use super::*;
    use aether_matrix::bot::Bot;
    use tempfile::TempDir;

    /// 创建测试用的 Config，使用独立的 store 路径
    fn create_test_config(homeserver: &str, store_path: &str) -> Config {
        Config {
            matrix_homeserver: homeserver.to_string(),
            matrix_username: "test_user".to_string(),
            matrix_password: "test_password".to_string(),
            matrix_device_id: None,
            device_display_name: "Test Bot".to_string(),
            store_path: store_path.to_string(),
            openai_api_key: "sk-test-key".to_string(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
            openai_model: "gpt-4o-mini".to_string(),
            system_prompt: None,
            command_prefix: "!ai".to_string(),
            max_history: 10,
            bot_owners: vec![],
            db_path: "./data/aether.db".to_string(),
            streaming_enabled: false,
            streaming_min_interval_ms: 500,
            streaming_min_chars: 10,
            log_level: "info".to_string(),
            vision_enabled: true,
            vision_model: None,
            vision_max_image_size: 1024,
            proxy: None,
        }
    }

    /// 设置完整的 Matrix homeserver mock
    async fn setup_mock_matrix_server() -> MockServer {
        let server = MockServer::start().await;

        // Mock .well-known endpoint（返回 404，表示没有 delegated server）
        Mock::given(method("GET"))
            .and(path("/.well-known/matrix/client"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        // Mock versions endpoint
        Mock::given(method("GET"))
            .and(path("/_matrix/client/versions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "versions": ["v1.0", "v1.1", "v1.2"]
            })))
            .mount(&server)
            .await;

        // Mock server .well-known
        Mock::given(method("GET"))
            .and(path("/.well-known/matrix/server"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        // Mock login endpoint
        Mock::given(method("POST"))
            .and(path("/_matrix/client/v3/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "test_access_token",
                "user_id": "@test_user:matrix.org",
                "device_id": "TESTDEVICE"
            })))
            .mount(&server)
            .await;

        server
    }

    #[tokio::test]
    async fn test_bot_new_with_valid_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store_path = temp_dir.path().join("store").to_string_lossy().to_string();

        let server = setup_mock_matrix_server().await;
        let config = create_test_config(&server.uri(), &store_path);

        let result = Bot::new(config).await;

        assert!(
            result.is_ok(),
            "Bot::new should succeed with valid config: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_bot_new_with_device_id() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store_path = temp_dir.path().join("store").to_string_lossy().to_string();

        let server = MockServer::start().await;

        // Mock endpoints
        Mock::given(method("GET"))
            .and(path("/.well-known/matrix/client"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/versions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "versions": ["v1.0", "v1.1", "v1.2", "v3"]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/.well-known/matrix/server"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/_matrix/client/v3/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "test_access_token",
                "user_id": "@test_user:matrix.org",
                "device_id": "MY_DEVICE_ID"
            })))
            .mount(&server)
            .await;

        let mut config = create_test_config(&server.uri(), &store_path);
        config.matrix_device_id = Some("MY_DEVICE_ID".to_string());

        let result = Bot::new(config).await;

        assert!(
            result.is_ok(),
            "Bot::new should succeed with device_id: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_bot_new_with_invalid_homeserver() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store_path = temp_dir.path().join("store").to_string_lossy().to_string();

        // 使用无效的 URL
        let config = create_test_config("not-a-valid-url", &store_path);

        let result = Bot::new(config).await;

        assert!(result.is_err(), "Bot::new should fail with invalid URL");
    }
}
