//! # 会话管理模块
//!
//! 管理多个独立会话的消息历史，支持按 session_id 隔离对话。
//!
//! ## 核心类型
//!
//! - [`ConversationManager`][]: 会话历史管理器
//!
//! ## 功能特性
//!
//! - **多会话隔离**: 按 session_id 隔离不同用户/房间的对话
//! - **系统提示词**: 可配置的系统提示词，支持动态覆盖
//! - **历史长度限制**: 自动删除最早的消息，防止上下文过长
//! - **图片消息支持**: 支持带图片的用户消息（Vision API）
//!
//! ## 消息历史结构
//!
//! 每个会话的消息历史结构：
//! ```text
//! [System Message (可选)]
//! [User Message 1]
//! [Assistant Message 1]
//! [User Message 2]
//! [Assistant Message 2]
//! ...
//! ```
//!
//! # Example
//!
//! ```
//! use aether_matrix::conversation::ConversationManager;
//!
//! let mut manager = ConversationManager::new(
//!     Some("You are helpful.".to_string()),
//!     10
//! );
//!
//! // 添加用户消息和 AI 回复
//! manager.add_user_message("user-1", "Hello!");
//! manager.add_assistant_message("user-1", "Hi there!");
//!
//! // 获取完整消息历史（包含系统提示词）
//! let messages = manager.get_messages("user-1");
//! assert_eq!(messages.len(), 3); // system + user + assistant
//!
//! // 重置特定会话
//! manager.reset("user-1");
//! ```

use std::collections::HashMap;

use async_openai::types::chat::{
    ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessage,
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestMessage,
    ChatCompletionRequestMessageContentPartImage, ChatCompletionRequestMessageContentPartText,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestToolMessage,
    ChatCompletionRequestToolMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, FunctionCall, ImageDetail, ImageUrl,
};

/// 会话历史管理器。
///
/// 管理多个独立会话的消息历史，支持：
/// - 按 `session_id` 隔离不同用户/房间的对话
/// - 可配置的系统提示词
/// - 自动限制历史长度，防止上下文过长
///
/// # Example
///
/// ```
/// use aether_matrix::conversation::ConversationManager;
///
/// let mut manager = ConversationManager::new(Some("You are helpful.".to_string()), 10);
///
/// // 添加用户消息和 AI 回复
/// manager.add_user_message("user-1", "Hello!");
/// manager.add_assistant_message("user-1", "Hi there!");
///
/// // 获取包含系统提示词的完整消息历史
/// let messages = manager.get_messages("user-1");
/// assert_eq!(messages.len(), 3); // system + user + assistant
///
/// // 重置特定会话
/// manager.reset("user-1");
/// assert_eq!(manager.get_messages("user-1").len(), 1); // 仅剩 system
/// ```
pub struct ConversationManager {
    /// 各会话的消息历史，key 为 session_id
    conversations: HashMap<String, Vec<ChatCompletionRequestMessage>>,
    /// 系统提示词，会在每个请求的开头添加
    system_prompt: Option<String>,
    /// 每个会话保留的最大历史轮数（一轮 = 一问一答）
    max_history: usize,
}

impl ConversationManager {
    /// 创建新的会话管理器。
    ///
    /// # Arguments
    ///
    /// * `system_prompt` - 可选的系统提示词，会在每个请求开头添加
    /// * `max_history` - 最大历史轮数（一轮 = 一问一答），超过限制会自动删除最早的消息
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::conversation::ConversationManager;
    ///
    /// // 无系统提示词，保留最近 20 轮对话
    /// let manager = ConversationManager::new(None, 20);
    ///
    /// // 有系统提示词
    /// let manager = ConversationManager::new(
    ///     Some("You are a helpful assistant.".to_string()),
    ///     10
    /// );
    /// ```
    pub fn new(system_prompt: Option<String>, max_history: usize) -> Self {
        Self {
            conversations: HashMap::new(),
            system_prompt,
            max_history,
        }
    }

    /// 添加用户消息到指定会话。
    ///
    /// 如果会话不存在，会自动创建。添加后会检查历史长度，
    /// 超过限制时删除最早的消息。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `content` - 用户消息内容
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::conversation::ConversationManager;
    ///
    /// let mut manager = ConversationManager::new(None, 10);
    /// manager.add_user_message("user-1", "Hello!");
    /// ```
    pub fn add_user_message(&mut self, session_id: &str, content: &str) {
        let history = self
            .conversations
            .entry(session_id.to_string())
            .or_default();

        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: content.to_string().into(),
                name: None,
            },
        ));

        // 历史长度限制：保留最近 N 轮对话（2N 条消息）
        // 使用 split_off 高效截断，避免迭代器开销
        if history.len() > self.max_history * 2 {
            *history = history.split_off(history.len() - self.max_history * 2);
        }
    }

    /// 添加带图片的用户消息到指定会话。
    ///
    /// 用于 Vision API，支持发送文本和图片的组合消息。
    /// 如果会话不存在，会自动创建。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `text` - 用户消息文本内容
    /// * `image_data_url` - 图片的 base64 data URL，格式为 `data:{media_type};base64,{data}`
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::conversation::ConversationManager;
    ///
    /// let mut manager = ConversationManager::new(None, 10);
    /// manager.add_user_message_with_image(
    ///     "user-1",
    ///     "What's in this image?",
    ///     "data:image/png;base64,abc123",
    /// );
    /// ```
    pub fn add_user_message_with_image(
        &mut self,
        session_id: &str,
        text: &str,
        image_data_url: &str,
    ) {
        let history = self
            .conversations
            .entry(session_id.to_string())
            .or_default();

        // 构造多部分消息内容（文本 + 图片）
        let content = ChatCompletionRequestUserMessageContent::Array(vec![
            ChatCompletionRequestMessageContentPartText {
                text: text.to_string(),
            }
            .into(),
            ChatCompletionRequestMessageContentPartImage {
                image_url: ImageUrl {
                    url: image_data_url.to_string(),
                    detail: Some(ImageDetail::Auto),
                },
            }
            .into(),
        ]);

        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content,
                name: None,
            },
        ));

        // 历史长度限制
        if history.len() > self.max_history * 2 {
            *history = history.split_off(history.len() - self.max_history * 2);
        }
    }

    /// 添加 AI 助手回复到指定会话。
    ///
    /// 只在会话已存在时添加消息。通常在 `add_user_message` 之后调用。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `content` - AI 助手的回复内容
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::conversation::ConversationManager;
    ///
    /// let mut manager = ConversationManager::new(None, 10);
    /// manager.add_user_message("user-1", "Hello!");
    /// manager.add_assistant_message("user-1", "Hi there!");
    /// ```
    pub fn add_assistant_message(&mut self, session_id: &str, content: &str) {
        if let Some(history) = self.conversations.get_mut(session_id) {
            let msg = ChatCompletionRequestAssistantMessage {
                content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                    content.to_string(),
                )),
                ..Default::default()
            };
            history.push(ChatCompletionRequestMessage::Assistant(msg));
        }
    }

    /// 获取指定会话的完整消息列表。
    ///
    /// 返回包含系统提示词（如有）和历史消息的完整列表，
    /// 可直接用于 API 请求。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    ///
    /// # Returns
    ///
    /// 消息列表，顺序为：系统提示词（如有）+ 历史消息（按时间顺序）
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::conversation::ConversationManager;
    ///
    /// let mut manager = ConversationManager::new(Some("Be helpful.".to_string()), 10);
    /// manager.add_user_message("user-1", "Hello!");
    ///
    /// let messages = manager.get_messages("user-1");
    /// assert_eq!(messages.len(), 2); // system + user
    /// ```
    pub fn get_messages(&self, session_id: &str) -> Vec<ChatCompletionRequestMessage> {
        let mut messages = Vec::new();

        // 添加系统提示词（始终在开头）
        if let Some(ref prompt) = self.system_prompt {
            messages.push(ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: prompt.clone().into(),
                    name: None,
                },
            ));
        }

        // 添加历史消息
        if let Some(history) = self.conversations.get(session_id) {
            messages.extend(history.clone());
        }

        messages
    }

    /// 获取消息历史，使用自定义系统提示词覆盖默认值
    ///
    /// 与 [`get_messages`](ConversationManager::get_messages) 类似，
    /// 但使用提供的系统提示词替代默认的系统提示词。
    /// 适用于人设系统等需要动态改变 AI 行为的场景。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 会话标识符
    /// * `system_prompt` - 自定义系统提示词
    ///
    /// # Returns
    ///
    /// 消息列表，顺序为：自定义系统提示词 + 历史消息
    pub fn get_messages_with_system(
        &self,
        session_id: &str,
        system_prompt: &str,
    ) -> Vec<ChatCompletionRequestMessage> {
        let mut messages = Vec::new();

        // 使用自定义系统提示词
        messages.push(ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage {
                content: system_prompt.to_string().into(),
                name: None,
            },
        ));

        // 添加历史消息
        if let Some(history) = self.conversations.get(session_id) {
            messages.extend(history.clone());
        }

        messages
    }

    /// 重置（删除）指定会话的历史记录。
    #[allow(dead_code)]
    pub fn reset(&mut self, session_id: &str) {
        self.conversations.remove(session_id);
    }

    /// 添加工具调用消息（assistant 调用工具）
    #[allow(dead_code)]
    pub fn add_tool_call_message(
        &mut self,
        session_id: &str,
        tool_call_id: String,
        tool_name: String,
        arguments: serde_json::Value,
    ) {
        let history = self
            .conversations
            .entry(session_id.to_string())
            .or_default();

        let tool_call = ChatCompletionMessageToolCalls::Function(
            async_openai::types::chat::ChatCompletionMessageToolCall {
                id: tool_call_id,
                function: FunctionCall {
                    name: tool_name,
                    arguments: serde_json::to_string(&arguments).unwrap_or_default(),
                },
            },
        );

        // 检查最后一条消息是否是 assistant 消息
        // 如果是，追加 tool_call；否则创建新的 assistant 消息
        if let Some(last_msg) = history.last_mut()
            && let ChatCompletionRequestMessage::Assistant(msg) = last_msg
        {
            if let Some(tool_calls) = &mut msg.tool_calls {
                tool_calls.push(tool_call);
            } else {
                msg.tool_calls = Some(vec![tool_call]);
            }
            return;
        }

        // 创建新的 assistant 消息
        let msg = ChatCompletionRequestAssistantMessage {
            content: None,
            tool_calls: Some(vec![tool_call]),
            ..Default::default()
        };
        history.push(ChatCompletionRequestMessage::Assistant(msg));

        // 历史长度限制
        if history.len() > self.max_history * 2 {
            *history = history.split_off(history.len() - self.max_history * 2);
        }
    }

    /// 添加工具结果消息（tool 返回结果）
    #[allow(dead_code)]
    pub fn add_tool_result_message(
        &mut self,
        session_id: &str,
        tool_call_id: String,
        result: serde_json::Value,
    ) {
        let history = self
            .conversations
            .entry(session_id.to_string())
            .or_default();

        let content = serde_json::to_string(&result).unwrap_or_default();

        let msg = ChatCompletionRequestToolMessage {
            content: ChatCompletionRequestToolMessageContent::Text(content),
            tool_call_id,
        };

        history.push(ChatCompletionRequestMessage::Tool(msg));

        // 历史长度限制
        if history.len() > self.max_history * 2 {
            *history = history.split_off(history.len() - self.max_history * 2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager_is_empty() {
        let manager = ConversationManager::new(None, 10);
        let messages = manager.get_messages("test-session");
        assert!(messages.is_empty());
    }

    #[test]
    fn test_add_user_message() {
        let mut manager = ConversationManager::new(None, 10);
        manager.add_user_message("session-1", "Hello");

        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 1);

        match &messages[0] {
            ChatCompletionRequestMessage::User(msg) => match &msg.content {
                async_openai::types::chat::ChatCompletionRequestUserMessageContent::Text(text) => {
                    assert_eq!(text, "Hello");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected user message"),
        }
    }

    #[test]
    fn test_add_assistant_message() {
        let mut manager = ConversationManager::new(None, 10);
        manager.add_user_message("session-1", "Hello");
        manager.add_assistant_message("session-1", "Hi there!");

        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 2);

        match &messages[1] {
            ChatCompletionRequestMessage::Assistant(msg) => match &msg.content {
                Some(ChatCompletionRequestAssistantMessageContent::Text(text)) => {
                    assert_eq!(text, "Hi there!");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected assistant message"),
        }
    }

    #[test]
    fn test_history_limit() {
        let mut manager = ConversationManager::new(None, 2); // max_history = 2

        // 添加 5 条消息，第 5 条会触发截断（add_user_message 时检查）
        // u1, a1, u2, a2, u3
        manager.add_user_message("s1", "u1");
        manager.add_assistant_message("s1", "a1");
        manager.add_user_message("s1", "u2");
        manager.add_assistant_message("s1", "a2");
        manager.add_user_message("s1", "u3"); // 触发截断到 4 条：a1, u2, a2, u3

        let messages = manager.get_messages("s1");
        // max_history * 2 = 4 条消息被保留
        assert_eq!(messages.len(), 4);

        // 最新的消息应该被保留（第一条是 a1）
        match &messages[0] {
            ChatCompletionRequestMessage::Assistant(msg) => match &msg.content {
                Some(ChatCompletionRequestAssistantMessageContent::Text(text)) => {
                    assert_eq!(text, "a1");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected assistant message a1"),
        }
    }

    #[test]
    fn test_reset_conversation() {
        let mut manager = ConversationManager::new(None, 10);
        manager.add_user_message("session-1", "Hello");
        manager.add_user_message("session-2", "World");

        manager.reset("session-1");

        assert_eq!(manager.get_messages("session-1").len(), 0);
        assert_eq!(manager.get_messages("session-2").len(), 1);
    }

    #[test]
    fn test_system_prompt() {
        let mut manager = ConversationManager::new(Some("You are helpful.".to_string()), 10);
        manager.add_user_message("s1", "Hello");

        let messages = manager.get_messages("s1");
        assert_eq!(messages.len(), 2);

        match &messages[0] {
            ChatCompletionRequestMessage::System(msg) => match &msg.content {
                async_openai::types::chat::ChatCompletionRequestSystemMessageContent::Text(
                    text,
                ) => {
                    assert_eq!(text, "You are helpful.");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected system message"),
        }
    }

    #[test]
    fn test_multiple_sessions() {
        let mut manager = ConversationManager::new(None, 10);
        manager.add_user_message("session-1", "Hello from s1");
        manager.add_user_message("session-2", "Hello from s2");
        manager.add_assistant_message("session-1", "Response to s1");

        let s1_messages = manager.get_messages("session-1");
        let s2_messages = manager.get_messages("session-2");

        assert_eq!(s1_messages.len(), 2);
        assert_eq!(s2_messages.len(), 1);
    }

    #[test]
    fn test_add_tool_call_message() {
        let mut manager = ConversationManager::new(None, 10);
        manager.add_user_message("session-1", "Get weather for Beijing");

        // 添加工具调用消息
        let args = serde_json::json!({"city": "Beijing"});
        manager.add_tool_call_message(
            "session-1",
            "call_123".to_string(),
            "weather_fetch".to_string(),
            args,
        );

        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 2); // user + assistant (with tool_call)

        // 检查最后一条消息是 assistant 消息且包含 tool_calls
        match &messages[1] {
            ChatCompletionRequestMessage::Assistant(msg) => {
                assert!(msg.tool_calls.is_some());
                let tool_calls = msg.tool_calls.as_ref().unwrap();
                assert_eq!(tool_calls.len(), 1);
            }
            _ => panic!("Expected assistant message with tool_calls"),
        }
    }

    #[test]
    fn test_add_tool_result_message() {
        let mut manager = ConversationManager::new(None, 10);
        manager.add_user_message("session-1", "Get weather");

        // 添加工具调用
        let args = serde_json::json!({"city": "Beijing"});
        manager.add_tool_call_message(
            "session-1",
            "call_123".to_string(),
            "weather_fetch".to_string(),
            args,
        );

        // 添加工具结果
        let result = serde_json::json!({"temperature": 20, "condition": "sunny"});
        manager.add_tool_result_message("session-1", "call_123".to_string(), result);

        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 3); // user + assistant (tool_call) + tool (result)

        // 检查最后一条消息是 tool 消息
        match &messages[2] {
            ChatCompletionRequestMessage::Tool(msg) => {
                assert_eq!(msg.tool_call_id, "call_123");
            }
            _ => panic!("Expected tool message"),
        }
    }

    #[test]
    fn test_multiple_tool_calls_in_one_message() {
        let mut manager = ConversationManager::new(None, 10);
        manager.add_user_message("session-1", "Get weather for multiple cities");

        // 添加第一个工具调用
        let args1 = serde_json::json!({"city": "Beijing"});
        manager.add_tool_call_message(
            "session-1",
            "call_1".to_string(),
            "weather_fetch".to_string(),
            args1,
        );

        // 添加第二个工具调用（应该追加到同一条 assistant 消息）
        let args2 = serde_json::json!({"city": "Shanghai"});
        manager.add_tool_call_message(
            "session-1",
            "call_2".to_string(),
            "weather_fetch".to_string(),
            args2,
        );

        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 2); // user + assistant (with 2 tool_calls)

        // 检查 assistant 消息包含两个 tool_calls
        match &messages[1] {
            ChatCompletionRequestMessage::Assistant(msg) => {
                let tool_calls = msg.tool_calls.as_ref().unwrap();
                assert_eq!(tool_calls.len(), 2);
            }
            _ => panic!("Expected assistant message with 2 tool_calls"),
        }
    }

    #[test]
    fn test_empty_message_handling() {
        let mut manager = ConversationManager::new(Some("System prompt".to_string()), 10);
        
        manager.add_user_message("session-1", "");
        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 2);
        
        match &messages[1] {
            ChatCompletionRequestMessage::User(msg) => match &msg.content {
                async_openai::types::chat::ChatCompletionRequestUserMessageContent::Text(text) => {
                    assert_eq!(text, "");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected user message"),
        }

        manager.add_assistant_message("session-1", "");
        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 3);
        
        match &messages[2] {
            ChatCompletionRequestMessage::Assistant(msg) => match &msg.content {
                Some(ChatCompletionRequestAssistantMessageContent::Text(text)) => {
                    assert_eq!(text, "");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected assistant message"),
        }
    }

    #[test]
    fn test_ultra_long_message_handling() {
        let mut manager = ConversationManager::new(None, 5);
        let long_message = "x".repeat(100_000);
        
        manager.add_user_message("session-1", &long_message);
        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 1);
        
        match &messages[0] {
            ChatCompletionRequestMessage::User(msg) => match &msg.content {
                async_openai::types::chat::ChatCompletionRequestUserMessageContent::Text(text) => {
                    assert_eq!(text.len(), 100_000);
                    assert_eq!(text.as_str(), long_message.as_str());
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected user message"),
        }
        
        let mut manager_with_system = ConversationManager::new(Some("System".to_string()), 5);
        manager_with_system.add_user_message("session-2", &long_message);
        let messages = manager_with_system.get_messages("session-2");
        assert_eq!(messages.len(), 2);
        
        match &messages[1] {
            ChatCompletionRequestMessage::User(msg) => match &msg.content {
                async_openai::types::chat::ChatCompletionRequestUserMessageContent::Text(text) => {
                    assert_eq!(text.len(), 100_000);
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected user message"),
        }
    }

    #[test]
    fn test_extreme_history_length_zero() {
        let mut manager = ConversationManager::new(None, 0);
        
        manager.add_user_message("session-1", "message1");
        let messages = manager.get_messages("session-1");
        // With max_history=0, behavior depends on implementation
        // The key is that it doesn't panic
        assert!(messages.len() <= 2);
        
        manager.add_assistant_message("session-1", "response1");
        let messages = manager.get_messages("session-1");
        assert!(messages.len() <= 2);
        
        manager.add_user_message("session-1", "message2");
        let messages = manager.get_messages("session-1");
        assert!(messages.len() <= 2);
    }

    #[test]
    fn test_extreme_history_length_one() {
        let mut manager = ConversationManager::new(None, 1);
        
        manager.add_user_message("session-1", "u1");
        manager.add_assistant_message("session-1", "a1");
        manager.add_user_message("session-1", "u2");
        manager.add_assistant_message("session-1", "a2");
        manager.add_user_message("session-1", "u3");
        
        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 2);
        
        match &messages[0] {
            ChatCompletionRequestMessage::Assistant(msg) => match &msg.content {
                Some(ChatCompletionRequestAssistantMessageContent::Text(text)) => {
                    assert_eq!(text, "a2");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected assistant message a2"),
        }
        match &messages[1] {
            ChatCompletionRequestMessage::User(msg) => match &msg.content {
                async_openai::types::chat::ChatCompletionRequestUserMessageContent::Text(text) => {
                    assert_eq!(text, "u3");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected user message u3"),
        }
    }

    #[test]
    fn test_extreme_history_length_large() {
        let mut manager = ConversationManager::new(None, 100_000);
        
        for i in 0..10 {
            manager.add_user_message("session-1", &format!("user_msg_{}", i));
            manager.add_assistant_message("session-1", &format!("assistant_msg_{}", i));
        }
        
        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 20);
        
        for i in 0..10 {
            let user_idx = i * 2;
            let assistant_idx = i * 2 + 1;
            
            match &messages[user_idx] {
                ChatCompletionRequestMessage::User(msg) => match &msg.content {
                    async_openai::types::chat::ChatCompletionRequestUserMessageContent::Text(text) => {
                        assert_eq!(text, &format!("user_msg_{}", i));
                    }
                    _ => panic!("Expected text content"),
                },
                _ => panic!("Expected user message"),
            }
            
            match &messages[assistant_idx] {
                ChatCompletionRequestMessage::Assistant(msg) => match &msg.content {
                    Some(ChatCompletionRequestAssistantMessageContent::Text(text)) => {
                        assert_eq!(text, &format!("assistant_msg_{}", i));
                    }
                    _ => panic!("Expected text content"),
                },
                _ => panic!("Expected assistant message"),
            }
        }
    }

    #[test]
    fn test_concurrent_session_operations() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let manager = Arc::new(Mutex::new(ConversationManager::new(Some("System".to_string()), 10)));
        
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let manager_clone = Arc::clone(&manager);
                thread::spawn(move || {
                    let session_id = format!("session-{}", i);
                    for j in 0..100 {
                        let mut mgr = manager_clone.lock().unwrap();
                        mgr.add_user_message(&session_id, &format!("msg-{}-{}", i, j));
                        mgr.add_assistant_message(&session_id, &format!("resp-{}-{}", i, j));
                    }
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_manager = manager.lock().unwrap();
        for i in 0..10 {
            let session_id = format!("session-{}", i);
            let messages = final_manager.get_messages(&session_id);
            // With max_history=10, we expect at most 10 pairs (20 messages)
            // but the exact count depends on implementation details
            assert!(messages.len() <= 22, "Session {} has {} messages", session_id, messages.len());
            
            match &messages[messages.len() - 2] {
                ChatCompletionRequestMessage::User(msg) => match &msg.content {
                    async_openai::types::chat::ChatCompletionRequestUserMessageContent::Text(text) => {
                        assert_eq!(text, &format!("msg-{}-99", i));
                    }
                    _ => panic!("Expected text content"),
                },
                _ => panic!("Expected user message"),
            }
            match &messages[messages.len() - 1] {
                ChatCompletionRequestMessage::Assistant(msg) => match &msg.content {
                    Some(ChatCompletionRequestAssistantMessageContent::Text(text)) => {
                        assert_eq!(text, &format!("resp-{}-99", i));
                    }
                    _ => panic!("Expected text content"),
                },
                _ => panic!("Expected assistant message"),
            }
        }
    }

    #[test]
    fn test_image_message_with_empty_content() {
        let mut manager = ConversationManager::new(None, 10);
        
        manager.add_user_message_with_image("session-1", "", "data:image/png;base64,abc123");
        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 1);
        
        match &messages[0] {
            ChatCompletionRequestMessage::User(msg) => {
                match &msg.content {
                    async_openai::types::chat::ChatCompletionRequestUserMessageContent::Array(parts) => {
                        assert_eq!(parts.len(), 2);
                    }
                    _ => panic!("Expected array content"),
                }
            }
            _ => panic!("Expected user message"),
        }
    }

    #[test]
    fn test_nonexistent_session_behavior() {
        let mut manager = ConversationManager::new(Some("System prompt".to_string()), 10);
        
        manager.add_assistant_message("nonexistent-session", "This should not be added");
        let messages = manager.get_messages("nonexistent-session");
        assert_eq!(messages.len(), 1);
        
        match &messages[0] {
            ChatCompletionRequestMessage::System(_) => {
            }
            _ => panic!("Expected only system message"),
        }
        
        manager.add_user_message("nonexistent-session", "Now it exists");
        let messages = manager.get_messages("nonexistent-session");
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_get_messages_with_system_override() {
        let mut manager = ConversationManager::new(Some("Default system".to_string()), 10);
        manager.add_user_message("session-1", "Hello");
        
        let messages = manager.get_messages_with_system("session-1", "Custom system");
        assert_eq!(messages.len(), 2);
        
        match &messages[0] {
            ChatCompletionRequestMessage::System(msg) => match &msg.content {
                async_openai::types::chat::ChatCompletionRequestSystemMessageContent::Text(text) => {
                    assert_eq!(text, "Custom system");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected system message"),
        }
        
        let original_messages = manager.get_messages("session-1");
        match &original_messages[0] {
            ChatCompletionRequestMessage::System(msg) => match &msg.content {
                async_openai::types::chat::ChatCompletionRequestSystemMessageContent::Text(text) => {
                    assert_eq!(text, "Default system");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected system message"),
        }
    }
}
