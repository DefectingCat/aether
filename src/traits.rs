use anyhow::Result;
use futures_util::Stream;
use matrix_sdk::ruma::OwnedEventId;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::Mutex;

/// 流式响应的状态追踪。
///
/// 在流式响应过程中累积所有已接收的文本片段，
/// 允许消费者随时获取当前累积的完整内容。
///
/// # Thread Safety
///
/// 通常与 `Arc<Mutex<StreamingState>>` 配合使用，
/// 确保流生产者和消费者之间的安全共享。
///
/// # Example
///
/// ```
/// use aether_matrix::traits::StreamingState;
///
/// let mut state = StreamingState::new();
/// state.append("Hello");
/// state.append(" World");
///
/// assert_eq!(state.content(), "Hello World");
/// ```
#[derive(Default)]
pub struct StreamingState {
    /// 累积的完整响应内容
    pub accumulated: String,
}

impl StreamingState {
    /// 创建新的空状态。
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加新的文本片段。
    ///
    /// # Arguments
    ///
    /// * `delta` - 新收到的文本片段
    pub fn append(&mut self, delta: &str) {
        self.accumulated.push_str(delta);
    }

    /// 获取当前累积的完整内容。
    pub fn content(&self) -> &str {
        &self.accumulated
    }
}

/// 流式聊天的响应类型。
///
/// 返回一个元组，包含：
/// - `Arc<Mutex<StreamingState>>`: 共享状态，用于追踪累积的响应内容
/// - `Pin<Box<dyn Stream<Item = Result<String>> + Send>>`: 可消费的流，每次产生一个文本片段
///
/// 这种设计允许生产者（AI 服务）和消费者（事件处理器）并发工作：
/// - 消费者可以从 Stream 读取每个 chunk
/// - 同时可以通过 StreamingState 获取当前累积的完整内容
pub type ChatStreamResponse = (
    Arc<Mutex<StreamingState>>,
    Pin<Box<dyn Stream<Item = Result<String>> + Send>>,
);

/// AI 服务的 trait 抽象。
///
/// 定义了 AI 服务必须实现的接口，支持依赖注入和 mock 测试。
/// 所有方法都是异步的，返回 `Future` 以兼容 `async fn` trait。
///
/// # Trait Bounds
///
/// - `Clone`: 允许在多个地方共享服务实例
/// - `Send + Sync`: 确保可以跨线程安全使用
/// - `'static`: 确保可以存储在需要静态生命周期的上下文中
///
/// # Example
///
/// ```rust
/// use anyhow::Result;
/// use aether_matrix::traits::AiServiceTrait;
/// use std::future::Future;
///
/// // 实现 trait 的 mock 服务
/// #[derive(Clone)]
/// struct MockAiService;
///
/// impl AiServiceTrait for MockAiService {
///     async fn chat(&self, session_id: &str, prompt: &str) -> Result<String> {
///         Ok(format!("Echo: {}", prompt))
///     }
///
///     async fn reset_conversation(&self, session_id: &str) {}
///
///     async fn chat_stream(
///         &self,
///         session_id: &str,
///         prompt: &str,
///     ) -> Result<aether_matrix::traits::ChatStreamResponse> {
///         unimplemented!("mock implementation")
///     }
///
///     async fn chat_with_image(
///         &self,
///         session_id: &str,
///         text: &str,
///         image_data_url: &str,
///     ) -> Result<String> {
///         Ok("Mock vision response".to_string())
///     }
///
///     async fn chat_with_image_stream(
///         &self,
///         session_id: &str,
///         text: &str,
///         image_data_url: &str,
///     ) -> Result<aether_matrix::traits::ChatStreamResponse> {
///         unimplemented!("mock implementation")
///     }
/// }
/// ```
pub trait AiServiceTrait: Clone + Send + Sync + 'static {
    /// 执行普通（非流式）聊天。
    ///
    /// 发送用户消息并返回 AI 的完整回复。
    /// 会自动将消息添加到会话历史中。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符，用于隔离不同用户/房间的对话
    /// * `prompt` - 用户输入的消息内容
    ///
    /// # Returns
    ///
    /// 成功时返回 AI 的回复文本。
    ///
    /// # Errors
    ///
    /// 当 API 调用失败时返回错误，例如：
    /// - 网络连接问题
    /// - API 认证失败
    /// - 服务端错误
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
    /// 成功时返回 AI 的回复文本。
    fn chat_with_system(
        &self,
        session_id: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> impl Future<Output = Result<String>> + Send;

    /// 重置指定会话的历史记录。
    ///
    /// 清除该会话的所有历史消息，但保留系统提示词。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 要重置的会话标识符
    fn reset_conversation(&self, session_id: &str) -> impl Future<Output = ()> + Send;

    /// 执行流式聊天。
    ///
    /// 与 [`chat`](AiServiceTrait::chat) 类似，但返回流式响应，
    /// 允许实时显示 AI 的输出（打字机效果）。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符，用于隔离不同用户/房间的对话
    /// * `prompt` - 用户输入的消息内容
    ///
    /// # Returns
    ///
    /// 成功时返回 [`ChatStreamResponse`]，包含：
    /// - 共享状态，可随时获取累积的完整内容
    /// - 流，消费时产生每个文本片段
    ///
    /// # Errors
    ///
    /// 当 API 调用初始化失败时返回错误。
    fn chat_stream(
        &self,
        session_id: &str,
        prompt: &str,
    ) -> impl Future<Output = Result<ChatStreamResponse>> + Send;

    /// 执行带自定义系统提示词的流式聊天。
    ///
    /// 与 [`chat_with_system`](AiServiceTrait::chat_with_system) 类似，但返回流式响应。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `prompt` - 用户输入的消息内容
    /// * `system_prompt` - 自定义系统提示词，如果为 None 则使用默认提示词
    ///
    /// # Returns
    ///
    /// 成功时返回 [`ChatStreamResponse`]。
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
    /// * `session_id` - 会话标识符，用于隔离不同用户/房间的对话
    /// * `text` - 用户输入的文本内容（可以是关于图片的问题或描述）
    /// * `image_data_url` - 图片的 base64 data URL，格式为 `data:{media_type};base64,{data}`
    ///
    /// # Returns
    ///
    /// 成功时返回 AI 的回复文本。
    ///
    /// # Errors
    ///
    /// 当 API 调用失败时返回错误，例如：
    /// - 网络连接问题
    /// - API 认证失败
    /// - 模型不支持 Vision API
    /// - 图片格式无效
    fn chat_with_image(
        &self,
        session_id: &str,
        text: &str,
        image_data_url: &str,
    ) -> impl Future<Output = Result<String>> + Send;

    /// 执行带图片的流式聊天（Vision API）。
    ///
    /// 与 [`chat_with_image`](AiServiceTrait::chat_with_image) 类似，但返回流式响应，
    /// 允许实时显示 AI 的输出（打字机效果）。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `text` - 用户输入的文本内容
    /// * `image_data_url` - 图片的 base64 data URL
    ///
    /// # Returns
    ///
    /// 成功时返回 [`ChatStreamResponse`]，包含：
    /// - 共享状态，可随时获取累积的完整内容
    /// - 流，消费时产生每个文本片段
    ///
    /// # Errors
    ///
    /// 当 API 调用初始化失败时返回错误。
    fn chat_with_image_stream(
        &self,
        session_id: &str,
        text: &str,
        image_data_url: &str,
    ) -> impl Future<Output = Result<ChatStreamResponse>> + Send;
}

pub trait MessageSender: Clone + Send + Sync + 'static {
    fn send(&self, content: &str) -> impl Future<Output = Result<OwnedEventId>> + Send;
    fn edit(
        &self,
        event_id: OwnedEventId,
        new_content: &str,
    ) -> impl Future<Output = Result<()>> + Send;
}

pub trait MatrixClient: Clone + Send + Sync + 'static {
    fn user_id(&self) -> Option<matrix_sdk::ruma::OwnedUserId>;
    fn join_room_by_id(
        &self,
        room_id: &matrix_sdk::ruma::RoomId,
    ) -> impl Future<Output = Result<()>> + Send;
}

#[derive(Clone)]
pub struct ClientWrapper(pub matrix_sdk::Client);

impl MatrixClient for ClientWrapper {
    fn user_id(&self) -> Option<matrix_sdk::ruma::OwnedUserId> {
        self.0.user_id().map(|id| id.to_owned())
    }

    async fn join_room_by_id(&self, room_id: &matrix_sdk::ruma::RoomId) -> Result<()> {
        self.0.join_room_by_id(room_id).await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct RoomSender(pub matrix_sdk::Room);

impl MessageSender for RoomSender {
    async fn send(&self, content: &str) -> Result<OwnedEventId> {
        use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
        let response = self
            .0
            .send(RoomMessageEventContent::text_plain(content))
            .await?;
        Ok(response.event_id)
    }

    async fn edit(&self, event_id: OwnedEventId, new_content: &str) -> Result<()> {
        use matrix_sdk::ruma::events::room::message::{
            ReplacementMetadata, RoomMessageEventContent,
        };
        let metadata = ReplacementMetadata::new(event_id, None);
        let msg_content =
            RoomMessageEventContent::text_plain(new_content).make_replacement(metadata);
        self.0.send(msg_content).await?;
        Ok(())
    }
}