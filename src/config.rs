//! # 配置管理模块
//!
//! 提供机器人的配置加载和管理功能。
//!
//! ## 核心类型
//!
//! - [`Config`] - 配置结构体，包含所有配置项
//!
//! ## 配置来源
//!
//! 配置可通过以下方式加载（优先级从高到低）：
//! 1. 环境变量
//! 2. `config.toml` 配置文件
//! 3. `.env` 文件（使用 dotenvy 库）
//! 4. 代码默认值
//!
//! ## 配置分组
//!
//! 配置项分为以下几组：
//! - **Matrix 配置**: 服务器地址、用户凭据、设备信息
//! - **AI API 配置**: API 密钥、模型、系统提示词
//! - **机器人配置**: 命令前缀、历史长度、拥有者列表
//! - **流式输出配置**: 启用状态、更新间隔、字符阈值
//! - **Vision 配置**: 图片理解、模型、最大尺寸
//! - **日志配置**: 日志级别
//!
//! # Example
//!
//! ```no_run
//! use aether_matrix::config::Config;
//!
//! // 从配置文件和环境变量加载配置
//! let config = Config::load("config.toml").expect("配置加载失败");
//!
//! // 使用默认值创建
//! let default_config = Config::default();
//! ```

use anyhow::{Context, Result};
use serde::Deserialize;

/// Matrix AI 机器人的配置结构体。
///
/// 包含连接 Matrix 服务器、调用 AI API 以及控制机器人行为所需的全部配置项。
/// 配置可通过 TOML 文件或环境变量加载，详见 [`Config::load`]。
///
/// # 必需配置
///
/// - `matrix.homeserver`: Matrix 服务器地址
/// - `matrix.username`: Matrix 用户名
/// - `matrix.password`: Matrix 密码
/// - `openai.api_key`: OpenAI API 密钥
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
/// // 从配置文件和环境变量加载配置
/// let config = Config::load("config.toml").expect("配置加载失败");
///
/// assert!(!config.matrix.homeserver.is_empty());
/// assert!(!config.openai.api_key.is_empty());
/// ```
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    /// Matrix 连接配置
    #[serde(default)]
    pub matrix: MatrixConfig,

    /// OpenAI API 配置
    #[serde(default)]
    pub openai: OpenAiConfig,

    /// 机器人行为配置
    #[serde(default)]
    pub bot: BotConfig,

    /// 流式输出配置
    #[serde(default)]
    pub streaming: StreamingConfig,

    /// Vision 配置
    #[serde(default)]
    pub vision: VisionConfig,

    /// 日志配置
    #[serde(default)]
    pub log: LogConfig,

    /// HTTP 代理 URL（可选）
    pub proxy: Option<String>,

    /// MCP 配置
    #[serde(default)]
    pub mcp: crate::mcp::McpConfig,

    /// Meme 梗图配置
    #[serde(default)]
    pub meme: MemeConfig,
}

/// Matrix 连接配置
#[derive(Debug, Clone, Deserialize)]
pub struct MatrixConfig {
    /// Matrix 服务器地址（必需）。
    ///
    /// 示例: `https://matrix.org`
    #[serde(default)]
    pub homeserver: String,

    /// Matrix 用户名（必需）。
    ///
    /// 通常是完整的用户 ID，如 `@user:matrix.org`
    #[serde(default)]
    pub username: String,

    /// Matrix 密码（必需）。
    #[serde(default)]
    pub password: String,

    /// Matrix 设备 ID（可选）。
    ///
    /// 设置固定的设备 ID 可以避免重复登录创建新设备。
    /// 建议使用一个有意义的标识符，如 `AETHER_BOT_001`。
    pub device_id: Option<String>,

    /// 设备显示名称。
    ///
    /// 在 Matrix 客户端的设备列表中显示的名称。
    #[serde(default = "default_device_display_name")]
    pub device_display_name: String,

    /// Matrix SDK 存储路径。
    ///
    /// 用于存储会话状态、同步令牌等持久化数据。
    #[serde(default = "default_store_path")]
    pub store_path: String,
}

/// OpenAI API 配置
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiConfig {
    /// OpenAI API 密钥（必需）。
    #[serde(default)]
    pub api_key: String,

    /// OpenAI API 基础 URL。
    ///
    /// 可设置为兼容的 API 端点，如 Azure OpenAI 或自托管服务。
    #[serde(default = "default_openai_base_url")]
    pub base_url: String,

    /// 使用的模型名称。
    ///
    /// 支持所有 OpenAI 兼容的模型，如 `gpt-4o-mini`、`gpt-4` 等。
    #[serde(default = "default_openai_model")]
    pub model: String,

    /// 系统提示词。
    ///
    /// 用于设置 AI 的行为和角色。
    pub system_prompt: Option<String>,
}

/// 机器人行为配置
#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    /// 命令前缀。
    ///
    /// 在群聊中触发 AI 响应的前缀，默认为 `!`。
    #[serde(default = "default_command_prefix")]
    pub command_prefix: String,

    /// 最大历史轮数。
    ///
    /// 每个会话保留的最大对话轮数（一轮 = 一问一答）。
    /// 超出限制时会自动丢弃最早的历史。
    #[serde(default = "default_max_history")]
    pub max_history: usize,

    /// 机器人拥有者列表。
    ///
    /// 拥有者可以使用管理命令（如 `!leave`）。
    /// 格式为 Matrix 用户 ID 列表，如 `["@user:matrix.org", "@admin:server.com"]`。
    #[serde(default)]
    pub owners: Vec<String>,

    /// 数据库文件路径。
    ///
    /// 用于存储 Persona、Muyu 等持久化数据。
    /// 默认为 `./data/aether.db`。
    #[serde(default = "default_db_path")]
    pub db_path: String,
}

/// 流式输出配置
#[derive(Debug, Clone, Deserialize)]
pub struct StreamingConfig {
    /// 是否启用流式输出。
    ///
    /// 启用后 AI 响应以打字机效果逐步显示。
    #[serde(default = "default_streaming_enabled")]
    pub enabled: bool,

    /// 流式更新最小间隔（毫秒）。
    ///
    /// 控制消息更新的频率，避免过于频繁的 API 调用。
    #[serde(default = "default_streaming_min_interval")]
    pub min_interval_ms: u64,

    /// 流式更新最小字符数。
    ///
    /// 累积到此数量的字符后才更新消息，与时间间隔共同控制更新节奏。
    #[serde(default = "default_streaming_min_chars")]
    pub min_chars: usize,
}

/// Vision 配置
#[derive(Debug, Clone, Deserialize)]
pub struct VisionConfig {
    /// 是否启用图片理解功能。
    ///
    /// 启用后机器人可以理解用户发送的图片内容。
    #[serde(default = "default_vision_enabled")]
    pub enabled: bool,

    /// 图片理解使用的模型（需要支持 Vision API）。
    ///
    /// 如未设置，使用 `openai.model` 配置的模型。
    /// 推荐使用 `gpt-4o`、`gpt-4o-mini` 等支持 Vision 的模型。
    pub model: Option<String>,

    /// 图片最大尺寸（像素）。
    ///
    /// 超过此尺寸的图片会被自动缩放，以避免 API 限制和减少处理时间。
    /// 保持宽高比，将图片缩放到最大边不超过此值。
    #[serde(default = "default_vision_max_image_size")]
    pub max_image_size: u32,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    /// 日志级别。
    ///
    /// 支持: `trace`, `debug`, `info`, `warn`, `error`
    #[serde(default = "default_log_level")]
    pub level: String,
}

/// Meme 梗图配置
#[derive(Debug, Clone, Deserialize)]
pub struct MemeConfig {
    /// 是否启用梗图功能。
    #[serde(default = "default_meme_enabled")]
    pub enabled: bool,

    /// KLIPY API Key。
    ///
    /// 从 https://partner.klipy.com 获取。
    /// 也支持旧的 TENOR_API_KEY 环境变量（向后兼容）。
    pub api_key: Option<String>,

    /// 每次搜索返回的结果数量。
    #[serde(default = "default_meme_limit")]
    pub limit: u32,
}

// 默认值函数
fn default_device_display_name() -> String {
    "AI Bot".to_string()
}
fn default_store_path() -> String {
    "./store".to_string()
}
fn default_openai_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}
fn default_openai_model() -> String {
    "gpt-4o-mini".to_string()
}
fn default_command_prefix() -> String {
    "!".to_string()
}
fn default_max_history() -> usize {
    10
}
fn default_db_path() -> String {
    "./data/aether.db".to_string()
}
fn default_streaming_enabled() -> bool {
    true
}
fn default_streaming_min_interval() -> u64 {
    1000
}
fn default_streaming_min_chars() -> usize {
    50
}
fn default_vision_enabled() -> bool {
    true
}
fn default_vision_max_image_size() -> u32 {
    1024
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_meme_enabled() -> bool {
    true
}
fn default_meme_limit() -> u32 {
    8
}

impl Default for MatrixConfig {
    fn default() -> Self {
        Self {
            homeserver: String::new(),
            username: String::new(),
            password: String::new(),
            device_id: None,
            device_display_name: default_device_display_name(),
            store_path: default_store_path(),
        }
    }
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: default_openai_base_url(),
            model: default_openai_model(),
            system_prompt: None,
        }
    }
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            command_prefix: default_command_prefix(),
            max_history: default_max_history(),
            owners: Vec::new(),
            db_path: default_db_path(),
        }
    }
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enabled: default_streaming_enabled(),
            min_interval_ms: default_streaming_min_interval(),
            min_chars: default_streaming_min_chars(),
        }
    }
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            enabled: default_vision_enabled(),
            model: None,
            max_image_size: default_vision_max_image_size(),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

impl Default for MemeConfig {
    fn default() -> Self {
        Self {
            enabled: default_meme_enabled(),
            api_key: None,
            limit: default_meme_limit(),
        }
    }
}

impl Config {
    /// 从配置文件和环境变量加载配置。
    ///
    /// 加载顺序（优先级从高到低）：
    /// 1. 环境变量
    /// 2. TOML 配置文件
    /// 3. `.env` 文件
    /// 4. 代码默认值
    ///
    /// # Arguments
    ///
    /// * `path` - 配置文件路径
    ///
    /// # Returns
    ///
    /// 成功时返回填充好的 `Config` 实例。
    ///
    /// # Errors
    ///
    /// 当以下必需字段未设置时返回错误：
    /// - `matrix.homeserver` / `MATRIX_HOMESERVER`
    /// - `matrix.username` / `MATRIX_USERNAME`
    /// - `matrix.password` / `MATRIX_PASSWORD`
    /// - `openai.api_key` / `OPENAI_API_KEY`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aether_matrix::config::Config;
    ///
    /// let config = Config::load("config.toml").expect("配置加载失败");
    /// ```
    pub fn load(path: &str) -> Result<Self> {
        // 1. 加载 .env 文件（兼容）
        #[cfg(not(test))]
        match dotenvy::dotenv() {
            Ok(p) => tracing::debug!(".env 文件已加载: {:?}", p),
            Err(e) => tracing::debug!(".env 文件加载失败: {}", e),
        }

        // 2. 加载 TOML 或警告 + 默认值
        let mut config = if std::path::Path::new(path).exists() {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("无法读取配置文件: {}", path))?;
            toml::from_str(&content).with_context(|| format!("配置文件格式错误: {}", path))?
        } else {
            tracing::warn!("配置文件 {} 不存在，将使用默认值和环境变量", path);
            Self::default()
        };

        // 3. 环境变量覆盖
        config.apply_env_overrides();

        // 4. 验证必需字段
        config.validate()?;

        Ok(config)
    }

    /// 环境变量覆盖（保持原有变量名）
    fn apply_env_overrides(&mut self) {
        // Matrix 配置
        if let Ok(v) = std::env::var("MATRIX_HOMESERVER") {
            self.matrix.homeserver = v;
        }
        if let Ok(v) = std::env::var("MATRIX_USERNAME") {
            self.matrix.username = v;
        }
        if let Ok(v) = std::env::var("MATRIX_PASSWORD") {
            self.matrix.password = v;
        }
        if let Ok(v) = std::env::var("MATRIX_DEVICE_ID") {
            self.matrix.device_id = Some(v);
        }
        if let Ok(v) = std::env::var("DEVICE_DISPLAY_NAME") {
            self.matrix.device_display_name = v;
        }
        if let Ok(v) = std::env::var("STORE_PATH") {
            self.matrix.store_path = v;
        }

        // OpenAI 配置
        if let Ok(v) = std::env::var("OPENAI_API_KEY") {
            self.openai.api_key = v;
        }
        if let Ok(v) = std::env::var("OPENAI_BASE_URL") {
            self.openai.base_url = v;
        }
        if let Ok(v) = std::env::var("OPENAI_MODEL") {
            self.openai.model = v;
        }
        if let Ok(v) = std::env::var("SYSTEM_PROMPT") {
            self.openai.system_prompt = Some(v);
        }

        // Bot 配置
        if let Ok(v) = std::env::var("BOT_COMMAND_PREFIX") {
            self.bot.command_prefix = v;
        }
        if let Ok(v) = std::env::var("MAX_HISTORY")
            && let Ok(n) = v.parse()
        {
            self.bot.max_history = n;
        }
        if let Ok(v) = std::env::var("BOT_OWNERS") {
            self.bot.owners = v.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(v) = std::env::var("DB_PATH") {
            self.bot.db_path = v;
        }

        // 流式输出配置
        if let Ok(v) = std::env::var("STREAMING_ENABLED") {
            self.streaming.enabled = v.to_lowercase() != "false";
        }
        if let Ok(v) = std::env::var("STREAMING_MIN_INTERVAL_MS")
            && let Ok(n) = v.parse()
        {
            self.streaming.min_interval_ms = n;
        }
        if let Ok(v) = std::env::var("STREAMING_MIN_CHARS")
            && let Ok(n) = v.parse()
        {
            self.streaming.min_chars = n;
        }

        // Vision 配置
        if let Ok(v) = std::env::var("VISION_ENABLED") {
            self.vision.enabled = v.to_lowercase() != "false";
        }
        if let Ok(v) = std::env::var("VISION_MODEL") {
            self.vision.model = Some(v);
        }
        if let Ok(v) = std::env::var("VISION_MAX_IMAGE_SIZE")
            && let Ok(n) = v.parse()
        {
            self.vision.max_image_size = n;
        }

        // 日志配置
        if let Ok(v) = std::env::var("LOG_LEVEL") {
            self.log.level = v;
        }

        // 代理配置
        if let Ok(v) = std::env::var("PROXY") {
            self.proxy = Some(v);
        }

        // MCP 配置
        self.mcp.apply_env_overrides();

        // Meme 配置
        if let Ok(v) = std::env::var("MEME_ENABLED") {
            self.meme.enabled = v.to_lowercase() != "false";
        }
        if let Ok(v) = std::env::var("KLIPY_API_KEY") {
            self.meme.api_key = Some(v);
        }
        // 同时支持旧的 TENOR_API_KEY 环境变量（向后兼容）
        if self.meme.api_key.is_none() {
            if let Ok(v) = std::env::var("TENOR_API_KEY") {
                self.meme.api_key = Some(v);
            }
        }
        if let Ok(v) = std::env::var("MEME_LIMIT")
            && let Ok(n) = v.parse()
        {
            self.meme.limit = n;
        }
        // 如果设置了 KLIPY_API_KEY，自动启用 meme 功能
        if self.meme.api_key.is_some() {
            self.meme.enabled = true;
        }
    }

    /// 验证必需字段
    fn validate(&self) -> Result<()> {
        if self.matrix.homeserver.is_empty() {
            anyhow::bail!(
                "MATRIX_HOMESERVER 未设置。\n\
                 请在 config.toml 的 [matrix] 部分或环境变量中配置 Matrix 服务器地址。\n\
                 示例: homeserver = \"https://matrix.org\""
            );
        }
        if self.matrix.username.is_empty() {
            anyhow::bail!(
                "MATRIX_USERNAME 未设置。\n\
                 请在 config.toml 的 [matrix] 部分或环境变量中配置 Matrix 用户名。\n\
                 示例: username = \"@bot:matrix.org\""
            );
        }
        if self.matrix.password.is_empty() {
            anyhow::bail!(
                "MATRIX_PASSWORD 未设置。\n\
                 请在 config.toml 的 [matrix] 部分或环境变量中配置 Matrix 密码。"
            );
        }
        if self.openai.api_key.is_empty() {
            anyhow::bail!(
                "OPENAI_API_KEY 未设置。\n\
                 请在 config.toml 的 [openai] 部分或环境变量中配置 API 密钥。\n\
                 示例: api_key = \"sk-...\""
            );
        }
        Ok(())
    }

    /// 向后兼容方法：从环境变量加载配置。
    #[allow(dead_code)]
    pub fn from_env() -> Result<Self> {
        Self::load("config.toml")
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
        assert_eq!(config.matrix.homeserver, "https://custom.server");
        assert_eq!(config.matrix.username, "custom_user");
        assert_eq!(config.matrix.password, "custom_pass");
        assert_eq!(config.openai.api_key, "sk-custom");
        // 可选字段
        assert_eq!(config.matrix.device_id, Some("DEVICE123".to_string()));
        assert_eq!(config.matrix.device_display_name, "Custom Bot");
        assert_eq!(config.matrix.store_path, "/custom/store");
        assert_eq!(config.openai.base_url, "https://api.custom.com/v1");
        assert_eq!(config.openai.model, "gpt-4");
        assert_eq!(
            config.openai.system_prompt,
            Some("You are a helpful assistant.".to_string())
        );
        assert_eq!(config.bot.command_prefix, "!custom");
        assert_eq!(config.bot.max_history, 20);
        assert!(!config.streaming.enabled);
        assert_eq!(config.streaming.min_interval_ms, 500);
        assert_eq!(config.streaming.min_chars, 25);
        assert_eq!(config.log.level, "debug");

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
        assert_eq!(config.matrix.device_id, None);
        assert_eq!(config.matrix.device_display_name, "AI Bot");
        assert_eq!(config.matrix.store_path, "./store");
        assert_eq!(config.openai.base_url, "https://api.openai.com/v1");
        assert_eq!(config.openai.model, "gpt-4o-mini");
        assert_eq!(config.openai.system_prompt, None);
        assert_eq!(config.bot.command_prefix, "!");
        assert_eq!(config.bot.max_history, 10);
        assert!(config.streaming.enabled);
        assert_eq!(config.streaming.min_interval_ms, 1000);
        assert_eq!(config.streaming.min_chars, 50);
        assert_eq!(config.log.level, "info");

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
        assert!(!config.streaming.enabled);
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
        assert!(config.streaming.enabled);
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
        assert!(!config.streaming.enabled);
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
        assert!(config.streaming.enabled);

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
        assert_eq!(config.bot.max_history, 100);
        assert_eq!(config.streaming.min_interval_ms, 2000);
        assert_eq!(config.streaming.min_chars, 200);

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
        assert_eq!(config.bot.max_history, 10);
        assert_eq!(config.streaming.min_interval_ms, 1000);
        assert_eq!(config.streaming.min_chars, 50);

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
        assert_eq!(config.matrix.device_id, None);
        teardown_env();

        // device_id 设置为空字符串
        setup_env(HashMap::from([
            ("MATRIX_HOMESERVER", "https://matrix.org"),
            ("MATRIX_USERNAME", "test_user"),
            ("MATRIX_PASSWORD", "test_pass"),
            ("OPENAI_API_KEY", "test_key"),
            ("MATRIX_DEVICE_ID", ""),
        ]));
        let config = Config::from_env().expect("配置应成功加载");
        assert_eq!(config.matrix.device_id, Some("".to_string()));
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
        assert_eq!(config.matrix.device_id, Some("MYDEVICE".to_string()));

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
        assert_eq!(config.openai.system_prompt, None);
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
            config.openai.system_prompt,
            Some("Be concise and helpful.".to_string())
        );

        teardown_env();
    }

    // ========== TOML 解析测试 ==========

    #[test]
    fn test_toml_parsing() {
        let toml_str = r#"
            [matrix]
            homeserver = "https://matrix.org"
            username = "@bot:matrix.org"
            password = "secret"
            device_display_name = "Test Bot"

            [openai]
            api_key = "sk-test"
            model = "gpt-4"

            [bot]
            command_prefix = "?"
            max_history = 20

            [streaming]
            enabled = false

            [vision]
            enabled = false

            [log]
            level = "debug"
        "#;

        let config: Config = toml::from_str(toml_str).expect("TOML 解析应成功");
        assert_eq!(config.matrix.homeserver, "https://matrix.org");
        assert_eq!(config.matrix.username, "@bot:matrix.org");
        assert_eq!(config.matrix.password, "secret");
        assert_eq!(config.matrix.device_display_name, "Test Bot");
        assert_eq!(config.openai.api_key, "sk-test");
        assert_eq!(config.openai.model, "gpt-4");
        assert_eq!(config.bot.command_prefix, "?");
        assert_eq!(config.bot.max_history, 20);
        assert!(!config.streaming.enabled);
        assert!(!config.vision.enabled);
        assert_eq!(config.log.level, "debug");
    }

    #[test]
    fn test_toml_minimal() {
        let toml_str = r#"
            [matrix]
            homeserver = "https://matrix.org"
            username = "@bot:matrix.org"
            password = "secret"

            [openai]
            api_key = "sk-test"
        "#;

        let config: Config = toml::from_str(toml_str).expect("TOML 解析应成功");
        assert_eq!(config.matrix.homeserver, "https://matrix.org");
        assert_eq!(config.openai.api_key, "sk-test");
        // 默认值
        assert_eq!(config.matrix.device_display_name, "AI Bot");
        assert_eq!(config.openai.model, "gpt-4o-mini");
        assert_eq!(config.bot.command_prefix, "!");
    }
}
