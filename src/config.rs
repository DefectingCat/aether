use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    // Matrix 配置
    pub matrix_homeserver: String,
    pub matrix_username: String,
    pub matrix_password: String,

    // AI API 配置
    pub openai_api_key: String,
    pub openai_base_url: String,
    pub openai_model: String,
    pub system_prompt: Option<String>,

    // 机器人配置
    pub command_prefix: String,
    pub max_history: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // 加载 .env 文件（如果存在）
        dotenvy::dotenv().ok();

        Ok(Self {
            matrix_homeserver: std::env::var("MATRIX_HOMESERVER")?,
            matrix_username: std::env::var("MATRIX_USERNAME")?,
            matrix_password: std::env::var("MATRIX_PASSWORD")?,
            openai_api_key: std::env::var("OPENAI_API_KEY")?,
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
        })
    }
}
