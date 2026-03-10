use aether_matrix::config::Config;

mod config_tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let config = Config::default();

        assert_eq!(config.matrix.homeserver, "");
        assert_eq!(config.matrix.username, "");
        assert_eq!(config.matrix.password, "");
        assert_eq!(config.matrix.device_id, None);
        assert_eq!(config.matrix.device_display_name, "AI Bot");
        assert_eq!(config.matrix.store_path, "./store");
        assert_eq!(config.openai.api_key, "");
        assert_eq!(config.openai.base_url, "https://api.openai.com/v1");
        assert_eq!(config.openai.model, "gpt-4o-mini");
        assert_eq!(config.openai.system_prompt, None);
        assert_eq!(config.bot.command_prefix, "!");
        assert_eq!(config.bot.max_history, 10);
        assert!(config.streaming.enabled);
        assert_eq!(config.streaming.min_interval_ms, 1000);
        assert_eq!(config.streaming.min_chars, 50);
        assert_eq!(config.log.level, "info");
        assert!(config.vision.enabled);
        assert_eq!(config.vision.model, None);
        assert_eq!(config.vision.max_image_size, 1024);
    }

    #[test]
    fn test_config_can_be_cloned() {
        let config = Config::default();
        let cloned = config.clone();
        assert_eq!(config.matrix.homeserver, cloned.matrix.homeserver);
    }

    #[test]
    fn test_config_debug_impl() {
        let config = Config::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("homeserver"));
    }

    #[test]
    fn test_config_custom_values() {
        let config = Config {
            matrix: aether_matrix::config::MatrixConfig {
                homeserver: "https://custom.server".to_string(),
                username: "custom_user".to_string(),
                password: "custom_pass".to_string(),
                device_id: Some("DEVICE123".to_string()),
                device_display_name: "Custom Bot".to_string(),
                store_path: "/tmp/custom_store".to_string(),
            },
            openai: aether_matrix::config::OpenAiConfig {
                api_key: "sk-custom".to_string(),
                base_url: "https://custom.api/v1".to_string(),
                model: "custom-model".to_string(),
                system_prompt: Some("You are helpful".to_string()),
            },
            bot: aether_matrix::config::BotConfig {
                command_prefix: "!custom".to_string(),
                max_history: 20,
                owners: vec!["@admin:matrix.org".to_string()],
                db_path: "./data/aether.db".to_string(),
            },
            streaming: aether_matrix::config::StreamingConfig {
                enabled: false,
                min_interval_ms: 2000,
                min_chars: 100,
            },
            vision: aether_matrix::config::VisionConfig {
                enabled: false,
                model: Some("gpt-4o".to_string()),
                max_image_size: 2048,
            },
            log: aether_matrix::config::LogConfig {
                level: "debug".to_string(),
            },
            proxy: None,
            mcp: aether_matrix::mcp::McpConfig::default(),
        };

        assert_eq!(config.matrix.homeserver, "https://custom.server");
        assert_eq!(config.matrix.username, "custom_user");
        assert_eq!(config.matrix.device_id, Some("DEVICE123".to_string()));
        assert_eq!(config.matrix.store_path, "/tmp/custom_store");
        assert_eq!(config.openai.model, "custom-model");
        assert_eq!(config.bot.max_history, 20);
        assert!(!config.streaming.enabled);
        assert!(!config.vision.enabled);
        assert_eq!(config.vision.model, Some("gpt-4o".to_string()));
    }
}

mod bot_tests {
    use super::*;
    use aether_matrix::bot::Bot;
    use tempfile::TempDir;

    fn create_test_config(homeserver: &str, store_path: &str) -> Config {
        Config {
            matrix: aether_matrix::config::MatrixConfig {
                homeserver: homeserver.to_string(),
                username: "test_user".to_string(),
                password: "test_password".to_string(),
                device_id: None,
                device_display_name: "Test Bot".to_string(),
                store_path: store_path.to_string(),
            },
            openai: aether_matrix::config::OpenAiConfig {
                api_key: "sk-test-key".to_string(),
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
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_bot_new_with_invalid_homeserver() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store_path = temp_dir.path().join("store").to_string_lossy().to_string();

        let config = create_test_config("not-a-valid-url", &store_path);

        let result = Bot::new(config).await;

        assert!(result.is_err(), "Bot::new should fail with invalid URL");
    }
}
