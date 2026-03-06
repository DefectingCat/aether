use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use anyhow::Result;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::CreateChatCompletionRequest;
use futures_util::{Stream, StreamExt};

use crate::config::Config;
use crate::conversation::ConversationManager;
use crate::traits::{AiServiceTrait, ChatStreamResponse, StreamingState};

/// OpenAI API 封装服务。
///
/// 提供与 OpenAI 兼容 API 的交互功能，支持：
/// - 普通聊天（一次性返回完整回复）
/// - 流式聊天（打字机效果）
/// - 多会话管理
///
/// # Cloning
///
/// `AiService` 实现了 `Clone`，内部使用 `Arc` 共享状态，
/// 克隆开销很小，可以在多处共享同一实例。
///
/// # Example
///
/// ```no_run
/// use aether_matrix::ai_service::AiService;
/// use aether_matrix::config::Config;
///
/// async fn example() {
///     let config = Config::default();
///     let service = AiService::new(&config);
///
///     // 克隆服务（共享内部状态）
///     let service_clone = service.clone();
///
///     // 发送消息
///     let reply = service.chat("user-1", "Hello!").await.unwrap();
/// }
/// ```
#[derive(Clone)]
pub struct AiService {
    inner: Arc<AiServiceInner>,
}

/// `AiService` 的内部实现。
///
/// 使用 `Arc` 包装，支持高效的克隆和共享。
struct AiServiceInner {
    /// OpenAI 客户端
    client: Client<OpenAIConfig>,
    /// 使用的模型名称
    model: String,
    /// 图片理解使用的模型名称
    vision_model: String,
    /// 会话管理器（使用 RwLock 支持并发读写）
    conversation: Arc<RwLock<ConversationManager>>,
}

impl AiService {
    /// 从配置创建新的 AI 服务实例。
    ///
    /// # Arguments
    ///
    /// * `config` - 机器人配置，包含 API 密钥、模型等设置
    pub fn new(config: &Config) -> Self {
        let openai_config = OpenAIConfig::new()
            .with_api_key(&config.openai_api_key)
            .with_api_base(&config.openai_base_url);

        Self {
            inner: Arc::new(AiServiceInner {
                client: Client::with_config(openai_config),
                model: config.openai_model.clone(),
                vision_model: config
                    .vision_model
                    .clone()
                    .unwrap_or_else(|| config.openai_model.clone()),
                conversation: Arc::new(RwLock::new(ConversationManager::new(
                    config.system_prompt.clone(),
                    config.max_history,
                ))),
            }),
        }
    }

    /// 执行普通（非流式）聊天。
    #[allow(dead_code)]
    pub async fn chat(&self, session_id: &str, prompt: &str) -> Result<String> {
        // 添加用户消息到历史（使用独立作用域限制锁的生命周期）
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_user_message(session_id, prompt);
        }

        // 获取完整消息历史（包含系统提示词）
        let messages = {
            let conv = self.inner.conversation.read().await;
            conv.get_messages(session_id)
        };

        // 构建并发送 API 请求
        let request = CreateChatCompletionRequest {
            model: self.inner.model.clone(),
            messages,
            ..Default::default()
        };

        let response = self.inner.client.chat().create(request).await?;

        // 提取回复内容
        let content = response.choices[0]
            .message
            .content
            .clone()
            .unwrap_or_default();

        // 保存 AI 回复到历史
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_assistant_message(session_id, &content);
        }

        Ok(content)
    }

    /// 执行带自定义系统提示词的聊天。
    ///
    /// 与 [`chat`](AiService::chat) 类似，但允许覆盖默认的系统提示词。
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
    pub async fn chat_with_system(
        &self,
        session_id: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<String> {
        // 添加用户消息到历史
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_user_message(session_id, prompt);
        }

        // 获取消息历史，如果有自定义系统提示词则使用
        let messages = {
            let conv = self.inner.conversation.read().await;
            if let Some(sp) = system_prompt {
                conv.get_messages_with_system(session_id, sp)
            } else {
                conv.get_messages(session_id)
            }
        };

        // 调用 API
        let request = CreateChatCompletionRequest {
            model: self.inner.model.clone(),
            messages,
            ..Default::default()
        };

        let response = self.inner.client.chat().create(request).await?;

        // 提取回复内容
        let content = response.choices[0]
            .message
            .content
            .clone()
            .unwrap_or_default();

        // 添加助手回复到历史
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_assistant_message(session_id, &content);
        }

        Ok(content)
    }

    /// 重置指定会话的历史记录。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 要重置的会话标识符
    #[allow(dead_code)]
    pub async fn reset_conversation(&self, session_id: &str) {
        let mut conv = self.inner.conversation.write().await;
        conv.reset(session_id);
    }

    /// 执行流式聊天。
    #[allow(dead_code)]
    pub async fn chat_stream(&self, session_id: &str, prompt: &str) -> Result<ChatStreamResponse> {
        // 添加用户消息到历史
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_user_message(session_id, prompt);
        }

        // 获取完整消息历史
        let messages = {
            let conv = self.inner.conversation.read().await;
            conv.get_messages(session_id)
        };

        // 创建流式请求
        let request = CreateChatCompletionRequest {
            model: self.inner.model.clone(),
            messages,
            stream: Some(true),
            ..Default::default()
        };

        let stream = self.inner.client.chat().create_stream(request).await?;

        // 创建共享状态，用于追踪累积内容
        let state = Arc::new(Mutex::new(StreamingState::new()));
        let state_clone = state.clone();
        let conversation = self.inner.conversation.clone();
        let session_id_owned = session_id.to_string();

        // 包装 stream：在消费时更新共享状态
        // 使用 filter_map 而非 map，以过滤空 delta 和结束标记
        // 流结束时自动保存完整回复到会话历史，避免消费者手动处理
        let wrapped_stream = stream.filter_map(move |chunk_result| {
            let state = state_clone.clone();
            let conversation = conversation.clone();
            let session_id_owned = session_id_owned.clone();
            async move {
                match chunk_result {
                    Ok(chunk) => {
                        // 提取 delta 内容
                        if let Some(delta) =
                            chunk.choices.first().and_then(|c| c.delta.content.clone())
                        {
                            // 更新共享状态
                            {
                                let mut s = state.lock().await;
                                s.append(&delta);
                            }
                            Some(Ok(delta))
                        } else {
                            // 检查是否是结束标记
                            if chunk
                                .choices
                                .first()
                                .is_some_and(|c| c.finish_reason.is_some())
                            {
                                // 流结束，保存完整回复到会话历史
                                let s = state.lock().await;
                                let content = s.content().to_string();
                                drop(s); // 显式释放锁，避免在持有锁时再次获取写锁
                                let mut conv = conversation.write().await;
                                conv.add_assistant_message(&session_id_owned, &content);
                            }
                            None
                        }
                    }
                    Err(e) => Some(Err(anyhow::anyhow!("流式响应错误: {}", e))),
                }
            }
        });

        Ok((state, Box::pin(wrapped_stream)))
    }

    /// 执行带图片的聊天（Vision API）。
    ///
    /// 发送用户消息（包含文本和图片）并返回 AI 的完整回复。
    /// 适用于需要理解图片内容的场景。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `text` - 用户输入的文本内容
    /// * `image_data_url` - 图片的 base64 data URL
    ///
    /// # Returns
    ///
    /// 成功时返回 AI 的完整回复文本。
    ///
    /// # Errors
    ///
    /// 当 API 调用失败时返回错误。
    pub async fn chat_with_image(
        &self,
        session_id: &str,
        text: &str,
        image_data_url: &str,
    ) -> Result<String> {
        // 添加带图片的用户消息到历史
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_user_message_with_image(session_id, text, image_data_url);
        }

        // 获取完整消息历史（包含系统提示词）
        let messages = {
            let conv = self.inner.conversation.read().await;
            conv.get_messages(session_id)
        };

        // 构建并发送 API 请求
        let request = CreateChatCompletionRequest {
            model: self.inner.vision_model.clone(),
            messages,
            ..Default::default()
        };

        let response = self.inner.client.chat().create(request).await?;

        // 提取回复内容
        let content = response.choices[0]
            .message
            .content
            .clone()
            .unwrap_or_default();

        // 保存 AI 回复到历史
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_assistant_message(session_id, &content);
        }

        Ok(content)
    }

    /// 执行带图片的流式聊天（Vision API）。
    #[allow(dead_code)]
    pub async fn chat_with_image_stream(
        &self,
        session_id: &str,
        text: &str,
        image_data_url: &str,
    ) -> Result<ChatStreamResponse> {
        // 添加带图片的用户消息到历史
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_user_message_with_image(session_id, text, image_data_url);
        }

        // 获取完整消息历史
        let messages = {
            let conv = self.inner.conversation.read().await;
            conv.get_messages(session_id)
        };

        // 创建流式请求
        let request = CreateChatCompletionRequest {
            model: self.inner.vision_model.clone(),
            messages,
            stream: Some(true),
            ..Default::default()
        };

        let stream = self.inner.client.chat().create_stream(request).await?;

        // 创建共享状态，用于追踪累积内容
        let state = Arc::new(Mutex::new(StreamingState::new()));
        let state_clone = state.clone();
        let conversation = self.inner.conversation.clone();
        let session_id_owned = session_id.to_string();

        // 包装 stream：在消费时更新共享状态
        let wrapped_stream = stream.filter_map(move |chunk_result| {
            let state = state_clone.clone();
            let conversation = conversation.clone();
            let session_id_owned = session_id_owned.clone();
            async move {
                match chunk_result {
                    Ok(chunk) => {
                        if let Some(delta) =
                            chunk.choices.first().and_then(|c| c.delta.content.clone())
                        {
                            {
                                let mut s = state.lock().await;
                                s.append(&delta);
                            }
                            Some(Ok(delta))
                        } else {
                            if chunk
                                .choices
                                .first()
                                .is_some_and(|c| c.finish_reason.is_some())
                            {
                                let s = state.lock().await;
                                let content = s.content().to_string();
                                drop(s);
                                let mut conv = conversation.write().await;
                                conv.add_assistant_message(&session_id_owned, &content);
                            }
                            None
                        }
                    }
                    Err(e) => Some(Err(anyhow::anyhow!("流式响应错误: {}", e))),
                }
            }
        });

        Ok((state, Box::pin(wrapped_stream)))
    }

    /// 带自定义系统提示词的流式聊天
    pub async fn chat_stream_with_system(
        &self,
        session_id: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<(
        Arc<Mutex<StreamingState>>,
        Pin<Box<dyn Stream<Item = Result<String>> + Send>>,
    )> {
        // 添加用户消息到历史
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_user_message(session_id, prompt);
        }

        // 获取消息历史，如果有自定义系统提示词则使用
        let messages = {
            let conv = self.inner.conversation.read().await;
            if let Some(sp) = system_prompt {
                conv.get_messages_with_system(session_id, sp)
            } else {
                conv.get_messages(session_id)
            }
        };

        // 创建流式请求
        let request = CreateChatCompletionRequest {
            model: self.inner.model.clone(),
            messages,
            stream: Some(true),
            ..Default::default()
        };

        let stream = self.inner.client.chat().create_stream(request).await?;

        // 创建共享状态
        let state = Arc::new(Mutex::new(StreamingState::new()));
        let state_clone = state.clone();
        let conversation = self.inner.conversation.clone();
        let session_id_owned = session_id.to_string();

        // 包装 stream，在消费时更新状态
        let wrapped_stream = stream.filter_map(move |chunk_result| {
            let state = state_clone.clone();
            let conversation = conversation.clone();
            let session_id_owned = session_id_owned.clone();
            async move {
                match chunk_result {
                    Ok(chunk) => {
                        // 提取 delta 内容
                        if let Some(delta) =
                            chunk.choices.first().and_then(|c| c.delta.content.clone())
                        {
                            // 更新共享状态
                            {
                                let mut s = state.lock().await;
                                s.append(&delta);
                            }
                            Some(Ok(delta))
                        } else {
                            // 检查是否是结束标记
                            if chunk
                                .choices
                                .first()
                                .is_some_and(|c| c.finish_reason.is_some())
                            {
                                // 保存完整回复到会话历史
                                let s = state.lock().await;
                                let content = s.content().to_string();
                                drop(s); // 释放锁
                                let mut conv = conversation.write().await;
                                conv.add_assistant_message(&session_id_owned, &content);
                            }
                            None
                        }
                    }
                    Err(e) => Some(Err(anyhow::anyhow!("流式响应错误: {}", e))),
                }
            }
        });

        Ok((state, Box::pin(wrapped_stream)))
    }
}

impl AiServiceTrait for AiService {
    async fn chat(&self, session_id: &str, prompt: &str) -> Result<String> {
        self.chat(session_id, prompt).await
    }

    async fn chat_with_system(
        &self,
        session_id: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<String> {
        self.chat_with_system(session_id, prompt, system_prompt)
            .await
    }

    async fn reset_conversation(&self, session_id: &str) {
        self.reset_conversation(session_id).await
    }

    async fn chat_stream(&self, session_id: &str, prompt: &str) -> Result<ChatStreamResponse> {
        self.chat_stream(session_id, prompt).await
    }

    async fn chat_with_image(
        &self,
        session_id: &str,
        text: &str,
        image_data_url: &str,
    ) -> Result<String> {
        self.chat_with_image(session_id, text, image_data_url).await
    }

    async fn chat_with_image_stream(
        &self,
        session_id: &str,
        text: &str,
        image_data_url: &str,
    ) -> Result<ChatStreamResponse> {
        self.chat_with_image_stream(session_id, text, image_data_url)
            .await
    }

    async fn chat_stream_with_system(
        &self,
        session_id: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<(
        Arc<Mutex<StreamingState>>,
        Pin<Box<dyn Stream<Item = Result<String>> + Send>>,
    )> {
        self.chat_stream_with_system(session_id, prompt, system_prompt)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_state_new() {
        let state = StreamingState::new();
        assert!(state.content().is_empty());
    }

    #[test]
    fn test_streaming_state_append() {
        let mut state = StreamingState::new();
        state.append("Hello");
        assert_eq!(state.content(), "Hello");

        state.append(" World");
        assert_eq!(state.content(), "Hello World");
    }

    #[test]
    fn test_streaming_state_multiple_appends() {
        let mut state = StreamingState::new();
        state.append("First");
        state.append(" ");
        state.append("Second");
        state.append(" ");
        state.append("Third");

        assert_eq!(state.content(), "First Second Third");
    }
}
