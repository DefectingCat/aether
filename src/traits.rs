//! # Trait 抽象层
//!
//! 提供 AI 服务的 trait 抽象，支持依赖注入和 mock 测试。
//!
//! ## 核心类型
//!
//! - [`AiServiceTrait`][]: AI 服务的核心 trait，定义聊天、流式输出、图片理解等接口
//! - [`StreamingState`][]: 流式响应的状态追踪器
//! - [`ChatStreamResponse`][]: 流式聊天的响应类型别名
//!
//! ## 设计目的
//!
//! 通过 trait 抽象实现：
//! - **依赖注入**: 在测试中可以使用 mock 实现
//! - **解耦**: 事件处理器不依赖具体的 AI 服务实现
//! - **可扩展**: 可以轻松切换不同的 AI 后端
//!
//! # Example
//!
//! ```no_run
//! use aether_matrix::traits::AiServiceTrait;
//! use aether_matrix::ai_service::AiService;
//!
//! // 在生产环境中使用真实实现
//! async fn create_service(config: &aether_matrix::config::Config) -> impl AiServiceTrait {
//!     AiService::new(config).await
//! }
//! ```

use anyhow::Result;
use futures_util::Stream;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::{Mutex, RwLock};

use crate::mcp::McpServerManager;

/// 流式响应的状态追踪器。
///
/// 在流式输出过程中累积 AI 返回的内容，支持多次追加和查询。
/// 使用 `Arc<Mutex<StreamingState>>` 在多个异步任务间共享。
#[derive(Default)]
pub struct StreamingState {
    pub accumulated: String,
}

impl StreamingState {
    /// 创建一个空的流式状态。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::traits::StreamingState;
    ///
    /// let state = StreamingState::new();
    /// assert!(state.content().is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加文本到累积内容。
    ///
    /// # Arguments
    ///
    /// * `delta` - 要追加的文本片段
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::traits::StreamingState;
    ///
    /// let mut state = StreamingState::new();
    /// state.append("Hello");
    /// state.append(" World");
    /// assert_eq!(state.content(), "Hello World");
    /// ```
    pub fn append(&mut self, delta: &str) {
        self.accumulated.push_str(delta);
    }

    /// 获取当前累积的完整内容。
    ///
    /// # Returns
    ///
    /// 返回已累积的所有文本内容的引用。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::traits::StreamingState;
    ///
    /// let mut state = StreamingState::new();
    /// state.append("Test");
    /// assert_eq!(state.content(), "Test");
    /// ```
    pub fn content(&self) -> &str {
        &self.accumulated
    }
}

/// 流式聊天的响应类型。
///
/// 包含两个部分：
/// - `Arc<Mutex<StreamingState>>`: 共享状态，用于追踪已累积的内容
/// - `Pin<Box<dyn Stream>>`: 异步流，每次 yield 一个文本片段
///
/// # Example
///
/// ```no_run
/// use aether_matrix::traits::{AiServiceTrait, StreamingState};
/// use futures_util::StreamExt;
///
/// async fn example<S: AiServiceTrait>(service: &S) -> anyhow::Result<()> {
///     let (state, mut stream) = service.chat_stream("user-1", "Hello").await?;
///
///     while let Some(delta) = stream.next().await {
///         // delta 是新增的文本片段
///         println!("Delta: {}", delta?);
///     }
///
///     // 获取完整内容
///     let full_content = state.lock().await.content();
///     Ok(())
/// }
/// ```
pub type ChatStreamResponse = (
    Arc<Mutex<StreamingState>>,
    Pin<Box<dyn Stream<Item = Result<String>> + Send>>,
);

/// AI 服务的 trait 抽象。
///
/// 定义了 AI 服务必须实现的接口，支持：
/// - 普通聊天（一次性返回完整回复）
/// - 流式聊天（打字机效果）
/// - 图片理解（Vision API）
/// - 多会话管理
///
/// # Trait Bounds
///
/// - `Clone`: 支持在多处共享服务实例
/// - `Send + Sync + 'static`: 支持跨线程传递和异步使用
///
/// # Implementors
///
/// 实现者需要保证：
/// - 所有方法都是线程安全的
/// - 会话 ID 用于隔离不同用户/房间的对话历史
/// - 流式方法返回的 Stream 在消费完毕后会自动保存完整回复
///
/// # Example
///
/// ```no_run
/// use aether_matrix::traits::AiServiceTrait;
///
/// async fn handle_message<S: AiServiceTrait>(service: &S, user: &str, msg: &str) {
///     let reply = service.chat(user, msg).await.unwrap();
///     println!("AI: {}", reply);
/// }
/// ```
#[allow(dead_code)]
pub trait AiServiceTrait: Clone + Send + Sync + 'static {
    /// 执行普通（非流式）聊天。
    ///
    /// 发送用户消息并返回 AI 的完整回复。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符，用于隔离不同用户/房间的对话
    /// * `prompt` - 用户输入的消息内容
    ///
    /// # Returns
    ///
    /// 成功时返回 AI 的完整回复文本。
    ///
    /// # Errors
    ///
    /// 当 API 调用失败时返回错误。
    fn chat(&self, session_id: &str, prompt: &str) -> impl Future<Output = Result<String>> + Send;

    /// 执行带自定义系统提示词的聊天。
    ///
    /// 与 [`chat`](AiServiceTrait::chat) 类似，但允许覆盖默认的系统提示词。
    /// 适用于人设系统等需要动态改变 AI 行为的场景。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `prompt` - 用户输入的消息内容
    /// * `system_prompt` - 自定义系统提示词，如果为 None 则使用默认提示词
    ///
    /// # Returns
    ///
    /// 成功时返回 AI 的完整回复文本。
    fn chat_with_system(
        &self,
        session_id: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> impl Future<Output = Result<String>> + Send;

    /// 重置指定会话的历史记录。
    ///
    /// 清除指定会话的所有对话历史，保留系统提示词。
    /// 通常用于用户请求"新对话"或"清除记忆"。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 要重置的会话标识符
    fn reset_conversation(&self, session_id: &str) -> impl Future<Output = ()> + Send;

    /// 执行流式聊天。
    ///
    /// 发送用户消息并返回一个流式响应，支持打字机效果。
    /// 流消费完毕后会自动保存完整回复到会话历史。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `prompt` - 用户输入的消息内容
    ///
    /// # Returns
    ///
    /// 返回一个元组：
    /// - `Arc<Mutex<StreamingState>>`: 共享状态，可随时查询已累积的内容
    /// - `Stream`: 异步流，每次 yield 一个文本片段
    ///
    /// # Errors
    ///
    /// 当 API 调用失败时返回错误。
    fn chat_stream(
        &self,
        session_id: &str,
        prompt: &str,
    ) -> impl Future<Output = Result<ChatStreamResponse>> + Send;

    /// 执行带自定义系统提示词的流式聊天。
    ///
    /// 结合 [`chat_with_system`](AiServiceTrait::chat_with_system) 和
    /// [`chat_stream`](AiServiceTrait::chat_stream) 的功能。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `prompt` - 用户输入的消息内容
    /// * `system_prompt` - 自定义系统提示词，如果为 None 则使用默认提示词
    ///
    /// # Returns
    ///
    /// 返回流式响应，详见 [`chat_stream`](AiServiceTrait::chat_stream)。
    fn chat_stream_with_system(
        &self,
        session_id: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> impl Future<Output = Result<ChatStreamResponse>> + Send;

    /// 执行带图片的聊天（Vision API）。
    ///
    /// 发送用户消息（包含文本和图片）并返回 AI 的完整回复。
    /// 适用于需要理解图片内容的场景。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `text` - 用户输入的文本内容
    /// * `image_data_url` - 图片的 base64 data URL，格式为 `data:{media_type};base64,{data}`
    ///
    /// # Returns
    ///
    /// 成功时返回 AI 的完整回复文本。
    ///
    /// # Errors
    ///
    /// 当 API 调用失败或模型不支持 Vision 时返回错误。
    fn chat_with_image(
        &self,
        session_id: &str,
        text: &str,
        image_data_url: &str,
    ) -> impl Future<Output = Result<String>> + Send;

    /// 执行带图片的流式聊天（Vision API）。
    ///
    /// 结合 [`chat_with_image`](AiServiceTrait::chat_with_image) 和
    /// [`chat_stream`](AiServiceTrait::chat_stream) 的功能。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `text` - 用户输入的文本内容
    /// * `image_data_url` - 图片的 base64 data URL
    ///
    /// # Returns
    ///
    /// 返回流式响应，详见 [`chat_stream`](AiServiceTrait::chat_stream)。
    fn chat_with_image_stream(
        &self,
        session_id: &str,
        text: &str,
        image_data_url: &str,
    ) -> impl Future<Output = Result<ChatStreamResponse>> + Send;

    /// 执行带工具调用的聊天。
    ///
    /// AI 可以自动判断是否需要调用工具，并执行工具后返回最终响应。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `prompt` - 用户输入的消息内容
    /// * `system_prompt` - 自定义系统提示词
    ///
    /// # Returns
    ///
    /// 成功时返回 AI 的完整回复文本（已处理工具调用）。
    fn chat_with_tools(
        &self,
        session_id: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> impl Future<Output = Result<String>> + Send;

    /// 获取 MCP 服务器管理器。
    ///
    /// # Returns
    ///
    /// 返回 MCP 服务器管理器的 Arc 引用（如果启用）。
    fn mcp_server_manager(&self) -> Option<Arc<RwLock<McpServerManager>>>;

    /// 列出所有可用的 MCP 工具。
    ///
    /// # Returns
    ///
    /// 返回工具定义列表，如果 MCP 未启用则返回空列表。
    fn list_mcp_tools(&self) -> impl Future<Output = Vec<crate::mcp::ToolDefinition>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_state_new_creates_empty() {
        let state = StreamingState::new();
        assert!(state.content().is_empty());
    }

    #[test]
    fn test_append_adds_text() {
        let mut state = StreamingState::new();
        state.append("Hello");
        assert_eq!(state.content(), "Hello");
    }

    #[test]
    fn test_content_returns_accumulated() {
        let mut state = StreamingState::new();
        state.accumulated = "Test content".to_string();
        assert_eq!(state.content(), "Test content");
    }

    #[test]
    fn test_multiple_appends() {
        let mut state = StreamingState::new();
        state.append("Hello");
        state.append(" ");
        state.append("World");
        assert_eq!(state.content(), "Hello World");
    }

    #[test]
    fn test_streaming_state_default_is_empty() {
        let state = StreamingState::default();
        assert!(state.content().is_empty());
    }

    #[tokio::test]
    async fn test_chat_stream_response_type_works() {
        let state = Arc::new(Mutex::new(StreamingState::new()));

        {
            let mut s = state.lock().await;
            s.append("Test");
        }

        assert_eq!(state.lock().await.content(), "Test");
    }
}
