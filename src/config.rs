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
    pub bot_owners: Vec<String>,
    pub db_path: String,

    // 流式输出配置
    pub streaming_enabled: bool,
    pub streaming_min_interval_ms: u64,
    pub streaming_min_chars: usize,

    // 日志配置
    pub log_level: String,

    // 代理配置
    pub proxy: Option<String>,
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
            command_prefix: "!".to_string(),
            max_history: 10,
            bot_owners: Vec::new(),
            db_path: "./data/aether.db".to_string(),
            streaming_enabled: true,
            streaming_min_interval_ms: 1000,
            streaming_min_chars: 50,
            log_level: "info".to_string(),
            proxy: None,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // 加载 .env 文件（如果存在）
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
                .unwrap_or_else(|_| "!".to_string()),
            max_history: std::env::var("MAX_HISTORY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            bot_owners: std::env::var("BOT_OWNERS")
                .ok()
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            db_path: std::env::var("DB_PATH")
                .unwrap_or_else(|_| "./data/aether.db".to_string()),
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
            // 代理配置
            proxy: std::env::var("PROXY").ok(),
        })
    }
}
