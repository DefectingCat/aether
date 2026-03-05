use std::collections::HashMap;

use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
    ChatCompletionRequestMessageContentPartText, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent, ImageDetail,
    ImageUrl,
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

    /// 重置（删除）指定会话的历史记录。
    ///
    /// # Arguments
    ///
    /// * `session_id` - 要重置的会话标识符
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::conversation::ConversationManager;
    ///
    /// let mut manager = ConversationManager::new(None, 10);
    /// manager.add_user_message("user-1", "Hello!");
    /// manager.reset("user-1");
    ///
    /// assert!(manager.get_messages("user-1").is_empty());
    /// ```
    pub fn reset(&mut self, session_id: &str) {
        self.conversations.remove(session_id);
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
}
