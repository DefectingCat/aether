use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    // Matrix 配置
    pub matrix_homeserver: String,
    pub matrix_username: String,
    pub matrix_password: String,
    pub matrix_device_id: Option<String>,
    pub device_display_name: String,
    pub store_path: String,

    // AI API 配置
    pub openai_api_key: String,
    pub openai_base_url: String,
    pub openai_model: String,
    pub system_prompt: Option<String>,

    // 机器人配置
    pub command_prefix: String,
    pub max_history: usize,

    // 流式输出配置
    pub streaming_enabled: bool,
    pub streaming_min_interval_ms: u64,
    pub streaming_min_chars: usize,

    // 日志配置
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            matrix_homeserver: "https://matrix.org".to_string(),
            matrix_username: String::new(),
            matrix_password: String::new(),
            matrix_device_id: None,
            device_display_name: "AI Bot".to_string(),
            store_path: "./store".to_string(),
            openai_api_key: String::new(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
            openai_model: "gpt-4o-mini".to_string(),
            system_prompt: None,
            command_prefix: "!ai".to_string(),
            max_history: 10,
            streaming_enabled: true,
            streaming_min_interval_ms: 1000,
            streaming_min_chars: 50,
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // 加载 .env 文件（如果存在）
        // 在测试模式下跳过，以便测试可以完全控制环境变量
        #[cfg(not(test))]
        match dotenvy::dotenv() {
            Ok(path) => {
                tracing::debug!(".env 文件已加载: {:?}", path);
            }
            Err(e) => {
                tracing::warn!(
                    ".env 文件加载失败: {}。\n\
                     请检查：\n\
                     1. 文件是否存在于当前目录\n\
                     2. 文件格式是否正确（包含空格的值需要用引号包裹，如: NAME=\"value with spaces\"）\n\
                     将从环境变量读取配置。",
                    e
                );
            }
        }

        Ok(Self {
            matrix_homeserver: std::env::var("MATRIX_HOMESERVER").map_err(|_| {
                anyhow::anyhow!(
                    "MATRIX_HOMESERVER 未设置。\n\
                         请在 .env 文件或环境变量中配置 Matrix 服务器地址。\n\
                         示例: MATRIX_HOMESERVER=https://matrix.org"
                )
            })?,
            matrix_username: std::env::var("MATRIX_USERNAME").map_err(|_| {
                anyhow::anyhow!(
                    "MATRIX_USERNAME 未设置。\n\
                         请在 .env 文件或环境变量中配置 Matrix 用户名。\n\
                         示例: MATRIX_USERNAME=your_username"
                )
            })?,
            matrix_password: std::env::var("MATRIX_PASSWORD").map_err(|_| {
                anyhow::anyhow!(
                    "MATRIX_PASSWORD 未设置。\n\
                         请在 .env 文件或环境变量中配置 Matrix 密码。"
                )
            })?,
            matrix_device_id: std::env::var("MATRIX_DEVICE_ID").ok(),
            device_display_name: std::env::var("DEVICE_DISPLAY_NAME")
                .unwrap_or_else(|_| "AI Bot".to_string()),
            store_path: std::env::var("STORE_PATH").unwrap_or_else(|_| "./store".to_string()),
            openai_api_key: std::env::var("OPENAI_API_KEY").map_err(|_| {
                anyhow::anyhow!(
                    "OPENAI_API_KEY 未设置。\n\
                         请在 .env 文件或环境变量中配置 API 密钥。\n\
                         示例: OPENAI_API_KEY=sk-..."
                )
            })?,
            openai_base_url: std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            openai_model: std::env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            system_prompt: std::env::var("SYSTEM_PROMPT").ok(),
            command_prefix: std::env::var("BOT_COMMAND_PREFIX")
                .unwrap_or_else(|_| "!ai".to_string()),
            max_history: std::env::var("MAX_HISTORY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            // 流式输出配置
            streaming_enabled: std::env::var("STREAMING_ENABLED")
                .ok()
                .map(|s| s.to_lowercase() != "false")
                .unwrap_or(true),
            streaming_min_interval_ms: std::env::var("STREAMING_MIN_INTERVAL_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            streaming_min_chars: std::env::var("STREAMING_MIN_CHARS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
            // 日志配置
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    lazy_static::lazy_static! {
        // 防止环境变量测试并行执行
        static ref ENV_MUTEX: Mutex<()> = Mutex::new(());
    }

    fn setup_env(vars: HashMap<&str, &str>) {
        // 清除所有可能影响测试的环境变量
        // SAFETY: 仅在测试中使用，且通过 ENV_MUTEX 保证串行执行
        unsafe {
            let keys_to_remove = [
                "MATRIX_HOMESERVER",
                "MATRIX_USERNAME",
                "MATRIX_PASSWORD",
                "MATRIX_DEVICE_ID",
                "DEVICE_DISPLAY_NAME",
                "STORE_PATH",
                "OPENAI_API_KEY",
                "OPENAI_BASE_URL",
                "OPENAI_MODEL",
                "SYSTEM_PROMPT",
                "BOT_COMMAND_PREFIX",
                "MAX_HISTORY",
                "STREAMING_ENABLED",
                "STREAMING_MIN_INTERVAL_MS",
                "STREAMING_MIN_CHARS",
                "LOG_LEVEL",
            ];
            for key in &keys_to_remove {
                std::env::remove_var(key);
            }
            for (key, value) in vars {
                std::env::set_var(key, value);
            }
        }
    }

    fn teardown_env() {
        // SAFETY: 仅在测试中使用，且通过 ENV_MUTEX 保证串行执行
        unsafe {
            let keys_to_remove = [
                "MATRIX_HOMESERVER",
                "MATRIX_USERNAME",
                "MATRIX_PASSWORD",
                "MATRIX_DEVICE_ID",
                "DEVICE_DISPLAY_NAME",
                "STORE_PATH",
                "OPENAI_API_KEY",
                "OPENAI_BASE_URL",
                "OPENAI_MODEL",
                "SYSTEM_PROMPT",
                "BOT_COMMAND_PREFIX",
                "MAX_HISTORY",
                "STREAMING_ENABLED",
                "STREAMING_MIN_INTERVAL_MS",
                "STREAMING_MIN_CHARS",
                "LOG_LEVEL",
            ];
            for key in &keys_to_remove {
                std::env::remove_var(key);
            }
        }
    }

    // ========== 必需字段缺失测试 ==========

    #[test]
    fn test_from_env_missing_homeserver() {
        // 忽略之前测试可能导致的 mutex poisoning
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        setup_env(HashMap::from([
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
        ]));

        let result = Config::from_env();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("MATRIX_HOMESERVER"),
            "错误消息应包含 MATRIX_HOMESERVER: {err}"
        );

        teardown_env();
    }

    #[test]
    fn test_from_env_missing_username() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
        ]));

        let result = Config::from_env();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("MATRIX_USERNAME"),
            "错误消息应包含 MATRIX_USERNAME: {err}"
        );

        teardown_env();
    }

    #[test]
    fn test_from_env_missing_password() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("OPENAI_API_KEY", "test_key"),
        ]));

        let result = Config::from_env();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("MATRIX_PASSWORD"),
            "错误消息应包含 MATRIX_PASSWORD: {err}"
        );

        teardown_env();
    }

    #[test]
    fn test_from_env_missing_api_key() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
        ]));

        let result = Config::from_env();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("OPENAI_API_KEY"),
            "错误消息应包含 OPENAI_API_KEY: {err}"
        );

        teardown_env();
    }

    // ========== 可选字段解析测试 ==========

    #[test]
    fn test_from_env_all_optional_fields() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        setup_env(HashMap::from([
            // 必需字段
            ("MATRIX_HOMESERVER", "https://custom.server"),
            ("MATRIX_USERNAME", "custom_user"),
            ("MATRIX_PASSWORD", "custom_pass"),
            ("OPENAI_API_KEY", "sk-custom"),
            // 可选字段 - 自定义值
            ("MATRIX_DEVICE_ID", "DEVICE123"),
            ("DEVICE_DISPLAY_NAME", "Custom Bot"),
            ("STORE_PATH", "/custom/store"),
            ("OPENAI_BASE_URL", "https://api.custom.com/v1"),
            ("OPENAI_MODEL", "gpt-4"),
            ("SYSTEM_PROMPT", "You are a helpful assistant."),
            ("BOT_COMMAND_PREFIX", "!custom"),
            ("MAX_HISTORY", "20"),
            ("STREAMING_ENABLED", "false"),
            ("STREAMING_MIN_INTERVAL_MS", "500"),
            ("STREAMING_MIN_CHARS", "25"),
            ("LOG_LEVEL", "debug"),
        ]));

        let config = Config::from_env().expect("配置应成功加载");
        assert_eq!(config.matrix_homeserver, "https://custom.server");
        assert_eq!(config.matrix_username, "custom_user");
        assert_eq!(config.matrix_password, "custom_pass");
        assert_eq!(config.openai_api_key, "sk-custom");
        // 可选字段
        assert_eq!(config.matrix_device_id, Some("DEVICE123".to_string()));
        assert_eq!(config.device_display_name, "Custom Bot");
        assert_eq!(config.store_path, "/custom/store");
        assert_eq!(config.openai_base_url, "https://api.custom.com/v1");
        assert_eq!(config.openai_model, "gpt-4");
        assert_eq!(
            config.system_prompt,
            Some("You are a helpful assistant.".to_string())
        );
        assert_eq!(config.command_prefix, "!custom");
        assert_eq!(config.max_history, 20);
        assert!(!config.streaming_enabled);
        assert_eq!(config.streaming_min_interval_ms, 500);
        assert_eq!(config.streaming_min_chars, 25);
        assert_eq!(config.log_level, "debug");

        teardown_env();
    }

    // ========== 默认值测试 ==========

    #[test]
    fn test_from_env_uses_defaults_for_optional() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
        ]));

        let config = Config::from_env().expect("配置应成功加载");
        // 可选字段应使用默认值
        assert_eq!(config.matrix_device_id, None);
        assert_eq!(config.device_display_name, "AI Bot");
        assert_eq!(config.store_path, "./store");
        assert_eq!(config.openai_base_url, "https://api.openai.com/v1");
        assert_eq!(config.openai_model, "gpt-4o-mini");
        assert_eq!(config.system_prompt, None);
        assert_eq!(config.command_prefix, "!ai");
        assert_eq!(config.max_history, 10);
        assert!(config.streaming_enabled);
        assert_eq!(config.streaming_min_interval_ms, 1000);
        assert_eq!(config.streaming_min_chars, 50);
        assert_eq!(config.log_level, "info");

        teardown_env();
    }

    // ========== 类型转换测试 ==========

    #[test]
    fn test_from_env_boolean_parsing() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        // 测试 "false" 值
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
            ("STREAMING_ENABLED", "false"),
        ]));
        let config = Config::from_env().expect("配置应成功加载");
        assert!(!config.streaming_enabled);
        teardown_env();

        // 测试 "true" 值
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
            ("STREAMING_ENABLED", "true"),
        ]));
        let config = Config::from_env().expect("配置应成功加载");
        assert!(config.streaming_enabled);
        teardown_env();

        // 测试 "FALSE" 值（大写）
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
            ("STREAMING_ENABLED", "FALSE"),
        ]));
        let config = Config::from_env().expect("配置应成功加载");
        assert!(!config.streaming_enabled);
        teardown_env();

        // 测试其他值（非 "false"，应视为 true）
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
            ("STREAMING_ENABLED", "anything"),
        ]));
        let config = Config::from_env().expect("配置应成功加载");
        assert!(config.streaming_enabled);

        teardown_env();
    }

    #[test]
    fn test_from_env_number_parsing_valid() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
            ("MAX_HISTORY", "100"),
            ("STREAMING_MIN_INTERVAL_MS", "2000"),
            ("STREAMING_MIN_CHARS", "200"),
        ]));

        let config = Config::from_env().expect("配置应成功加载");
        assert_eq!(config.max_history, 100);
        assert_eq!(config.streaming_min_interval_ms, 2000);
        assert_eq!(config.streaming_min_chars, 200);

        teardown_env();
    }

    #[test]
    fn test_from_env_number_parsing_invalid_uses_defaults() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
            ("MAX_HISTORY", "not_a_number"),
            ("STREAMING_MIN_INTERVAL_MS", "invalid"),
            ("STREAMING_MIN_CHARS", "abc"),
        ]));

        let config = Config::from_env().expect("配置应成功加载");
        // 无效数字应使用默认值
        assert_eq!(config.max_history, 10);
        assert_eq!(config.streaming_min_interval_ms, 1000);
        assert_eq!(config.streaming_min_chars, 50);

        teardown_env();
    }

    // ========== 边界情况测试 ==========

    #[test]
    fn test_from_env_device_id_optional() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        // device_id 未设置
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
        ]));
        let config = Config::from_env().expect("配置应成功加载");
        assert_eq!(config.matrix_device_id, None);
        teardown_env();

        // device_id 设置为空字符串（env::var 会将空字符串视为有效值）
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
            ("MATRIX_DEVICE_ID", ""),
        ]));
        let config = Config::from_env().expect("配置应成功加载");
        assert_eq!(config.matrix_device_id, Some("".to_string()));
        teardown_env();

        // device_id 设置为有效值
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
            ("MATRIX_DEVICE_ID", "MYDEVICE"),
        ]));
        let config = Config::from_env().expect("配置应成功加载");
        assert_eq!(config.matrix_device_id, Some("MYDEVICE".to_string()));

        teardown_env();
    }

    #[test]
    fn test_from_env_system_prompt_optional() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        // system_prompt 未设置
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
        ]));
        let config = Config::from_env().expect("配置应成功加载");
        assert_eq!(config.system_prompt, None);
        teardown_env();

        // system_prompt 设置为有效值
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
            ("SYSTEM_PROMPT", "Be concise and helpful."),
        ]));
        let config = Config::from_env().expect("配置应成功加载");
        assert_eq!(
            config.system_prompt,
            Some("Be concise and helpful.".to_string())
        );

        teardown_env();
    }
}
