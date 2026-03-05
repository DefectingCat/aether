use anyhow::Result;

/// Matrix AI 机器人的配置结构体。
///
/// 包含连接 Matrix 服务器、调用 AI API 以及控制机器人行为所需的全部配置项。
/// 配置可通过环境变量或 `.env` 文件加载，详见 [`Config::from_env`]。
///
/// # 必需配置
///
/// - `MATRIX_HOMESERVER`: Matrix 服务器地址
/// - `MATRIX_USERNAME`: Matrix 用户名
/// - `MATRIX_PASSWORD`: Matrix 密码
/// - `OPENAI_API_KEY`: OpenAI API 密钥
///
/// # 可选配置
///
/// 所有可选配置都有合理的默认值，详见各字段文档。
///
/// # Example
///
/// ```no_run
/// use aether_matrix::config::Config;
///
/// // 从环境变量加载配置
/// let config = Config::from_env().expect("配置加载失败");
///
/// assert!(!config.matrix_homeserver.is_empty());
/// assert!(!config.openai_api_key.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    // --- Matrix 配置 ---
    /// Matrix 服务器地址（必需）。
    ///
    /// 示例: `https://matrix.org`
    pub matrix_homeserver: String,

    /// Matrix 用户名（必需）。
    ///
    /// 通常是完整的用户 ID，如 `@user:matrix.org`
    pub matrix_username: String,

    /// Matrix 密码（必需）。
    pub matrix_password: String,

    /// Matrix 设备 ID（可选）。
    ///
    /// 设置固定的设备 ID 可以避免重复登录创建新设备。
    /// 建议使用一个有意义的标识符，如 `AETHER_BOT_001`。
    pub matrix_device_id: Option<String>,

    /// 设备显示名称。
    ///
    /// 在 Matrix 客户端的设备列表中显示的名称。
    pub device_display_name: String,

    /// Matrix SDK 存储路径。
    ///
    /// 用于存储会话状态、同步令牌等持久化数据。
    pub store_path: String,

    // --- AI API 配置 ---
    /// OpenAI API 密钥（必需）。
    pub openai_api_key: String,

    /// OpenAI API 基础 URL。
    ///
    /// 可设置为兼容的 API 端点，如 Azure OpenAI 或自托管服务。
    pub openai_base_url: String,

    /// 使用的模型名称。
    ///
    /// 支持所有 OpenAI 兼容的模型，如 `gpt-4o-mini`、`gpt-4` 等。
    pub openai_model: String,

    /// 系统提示词。
    ///
    /// 用于设置 AI 的行为和角色。
    pub system_prompt: Option<String>,

    // --- 机器人配置 ---
    /// 命令前缀。
    ///
    /// 在群聊中触发 AI 响应的前缀，默认为 `!ai`。
    pub command_prefix: String,

    /// 最大历史轮数。
    ///
    /// 每个会话保留的最大对话轮数（一轮 = 一问一答）。
    /// 超出限制时会自动丢弃最早的历史。
    pub max_history: usize,

    /// 机器人拥有者列表。
    ///
    /// 拥有者可以使用管理命令（如 `!leave`）。
    /// 格式为 Matrix 用户 ID 列表，如 `["@user:matrix.org", "@admin:server.com"]`。
    pub bot_owners: Vec<String>,

    /// 数据库文件路径。
    ///
    /// 用于存储 Persona 等持久化数据。
    /// 默认为 `./data/aether.db`。
    pub db_path: String,

    // --- 流式输出配置 ---
    /// 是否启用流式输出。
    ///
    /// 启用后 AI 响应以打字机效果逐步显示。
    pub streaming_enabled: bool,

    /// 流式更新最小间隔（毫秒）。
    ///
    /// 控制消息更新的频率，避免过于频繁的 API 调用。
    pub streaming_min_interval_ms: u64,

    /// 流式更新最小字符数。
    ///
    /// 累积到此数量的字符后才更新消息，与时间间隔共同控制更新节奏。
    pub streaming_min_chars: usize,

    // --- 日志配置 ---
    /// 日志级别。
    ///
    /// 支持: `trace`, `debug`, `info`, `warn`, `error`
    pub log_level: String,

    // --- Vision 配置 ---
    /// 是否启用图片理解功能。
    ///
    /// 启用后机器人可以理解用户发送的图片内容。
    pub vision_enabled: bool,

    /// 图片理解使用的模型（需要支持 Vision API）。
    ///
    /// 如未设置，使用 `openai_model` 配置的模型。
    /// 推荐使用 `gpt-4o`、`gpt-4o-mini` 等支持 Vision 的模型。
    pub vision_model: Option<String>,

    /// 图片最大尺寸（像素）。
    ///
    /// 超过此尺寸的图片会被自动缩放，以避免 API 限制和减少处理时间。
    /// 保持宽高比，将图片缩放到最大边不超过此值。
    pub vision_max_image_size: u32,

    /// HTTP 代理 URL（可选）。
    ///
    /// 用于通过代理服务器连接 Matrix 和 OpenAI API。
    /// 格式为 `http://host:port` 或 `socks5://host:port`。
    pub proxy: Option<String>,
}

/// 为 `Config` 提供合理的默认值。
///
/// 默认值适用于大多数场景，必需字段会被设置为空字符串，
/// 调用 [`Config::from_env`] 时会验证这些字段是否已配置。
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
            command_prefix: "!".to_string(),
            max_history: 10,
            bot_owners: Vec::new(),
            db_path: "./data/aether.db".to_string(),
            streaming_enabled: true,
            streaming_min_interval_ms: 1000,
            streaming_min_chars: 50,
            // 日志配置
            log_level: "info".to_string(),
            // Vision 配置
            vision_enabled: true,
            vision_model: None,
            vision_max_image_size: 1024,
            proxy: None,
        }
    }
}

impl Config {
    /// 从环境变量加载配置。
    ///
    /// 优先尝试加载当前目录下的 `.env` 文件，然后从环境变量读取配置。
    /// 如果 `.env` 文件不存在或加载失败，会记录警告并继续从环境变量读取。
    ///
    /// # Arguments
    ///
    /// 无参数，所有配置从环境变量读取。
    ///
    /// # Returns
    ///
    /// 成功时返回填充好的 `Config` 实例。
    ///
    /// # Errors
    ///
    /// 当以下必需环境变量未设置时返回错误：
    /// - `MATRIX_HOMESERVER`
    /// - `MATRIX_USERNAME`
    /// - `MATRIX_PASSWORD`
    /// - `OPENAI_API_KEY`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aether_matrix::config::Config;
    ///
    /// // 确保已设置必需的环境变量
    /// // MATRIX_HOMESERVER, MATRIX_USERNAME, MATRIX_PASSWORD, OPENAI_API_KEY
    ///
    /// let config = Config::from_env().expect("配置加载失败");
    /// ```
    ///
    /// # Environment Variables
    ///
    /// | 变量名 | 必需 | 默认值 | 说明 |
    /// |--------|------|--------|------|
    /// | `MATRIX_HOMESERVER` | 是 | - | Matrix 服务器地址 |
    /// | `MATRIX_USERNAME` | 是 | - | Matrix 用户名 |
    /// | `MATRIX_PASSWORD` | 是 | - | Matrix 密码 |
    /// | `MATRIX_DEVICE_ID` | 否 | `None` | 设备 ID |
    /// | `DEVICE_DISPLAY_NAME` | 否 | `AI Bot` | 设备显示名称 |
    /// | `STORE_PATH` | 否 | `./store` | 存储路径 |
    /// | `OPENAI_API_KEY` | 是 | - | API 密钥 |
    /// | `OPENAI_BASE_URL` | 否 | OpenAI 默认 | API 基础 URL |
    /// | `OPENAI_MODEL` | 否 | `gpt-4o-mini` | 模型名称 |
    /// | `SYSTEM_PROMPT` | 否 | `None` | 系统提示词 |
    /// | `BOT_COMMAND_PREFIX` | 否 | `!ai` | 命令前缀 |
    /// | `MAX_HISTORY` | 否 | `10` | 最大历史轮数 |
    /// | `STREAMING_ENABLED` | 否 | `true` | 启用流式输出 |
    /// | `STREAMING_MIN_INTERVAL_MS` | 否 | `1000` | 流式更新间隔 |
    /// | `STREAMING_MIN_CHARS` | 否 | `50` | 流式更新字符数 |
    /// | `LOG_LEVEL` | 否 | `info` | 日志级别 |
    pub fn from_env() -> Result<Self> {
        // 加载 .env 文件（如果存在）
        // 在测试模式下跳过，以便测试可以完全控制环境变量
        // 避免测试环境受到开发环境 .env 文件的影响
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
            command_prefix: std::env::var("BOT_COMMAND_PREFIX").unwrap_or_else(|_| "!".to_string()),
            max_history: std::env::var("MAX_HISTORY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            bot_owners: std::env::var("BOT_OWNERS")
                .ok()
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            db_path: std::env::var("DB_PATH").unwrap_or_else(|_| "./data/aether.db".to_string()),
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
            // Vision 配置
            vision_enabled: std::env::var("VISION_ENABLED")
                .ok()
                .map(|s| s.to_lowercase() != "false")
                .unwrap_or(true),
            vision_model: std::env::var("VISION_MODEL").ok(),
            vision_max_image_size: std::env::var("VISION_MAX_IMAGE_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1024),
            // 代理配置
            proxy: std::env::var("PROXY").ok(),
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
                "VISION_ENABLED",
                "VISION_MODEL",
                "VISION_MAX_IMAGE_SIZE",
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
                "VISION_ENABLED",
                "VISION_MODEL",
                "VISION_MAX_IMAGE_SIZE",
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
        assert_eq!(config.command_prefix, "!");
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
