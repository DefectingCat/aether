use std::time::{Duration, Instant};

use anyhow::Result;
use futures_util::StreamExt;
use matrix_sdk::{
    Client, Room,
    ruma::{
        OwnedEventId, OwnedUserId,
        events::room::{
            member::{MembershipState, StrippedRoomMemberEvent},
            message::{ReplacementMetadata, RoomMessageEventContent},
        },
    },
};
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::traits::AiServiceTrait;

/// 处理房间邀请事件。
///
/// 当机器人收到加入房间的邀请时自动加入。这是独立函数而非方法，
/// 以便在事件处理器注册时直接使用。
///
/// # Arguments
///
/// * `ev` - 房间成员事件（邀请）
/// * `client` - Matrix 客户端实例
/// * `room` - 发送邀请的房间
///
/// # Returns
///
/// 成功时返回 `Ok(())`，失败时返回错误。
///
/// # Example
///
/// ```ignore
/// client.add_event_handler(
///     |ev: StrippedRoomMemberEvent, client: Client, room: Room| async move {
///         if let Err(e) = handle_invite(ev, client, room).await {
///             tracing::error!("处理邀请失败: {}", e);
///         }
///     }
/// );
/// ```
pub async fn handle_invite(ev: StrippedRoomMemberEvent, client: Client, room: Room) -> Result<()> {
    // 只处理邀请事件，忽略其他成员状态变更
    if ev.content.membership != MembershipState::Invite {
        return Ok(());
    }

    let user_id = &ev.state_key;
    let my_user_id = client.user_id().expect("user_id should be available");

    // 只处理邀请自己的事件
    if user_id != my_user_id {
        return Ok(()); // 不是邀请机器人
    }

    let room_id = room.room_id();
    info!("收到房间邀请: {}", room_id);

    match client.join_room_by_id(room_id).await {
        Ok(_) => info!("成功加入房间: {}", room_id),
        Err(e) => warn!("加入房间失败: {}", e),
    }

    Ok(())
}

/// Matrix 消息事件处理器。
///
/// 处理房间消息事件，判断是否需要响应并调用 AI 服务。
/// 支持普通响应和流式响应两种模式。
///
/// # 响应条件
///
/// - **私聊**: 总是响应所有消息
/// - **群聊**: 只响应以下情况
///   - 消息以命令前缀开头（如 `!ai`）
///   - 消息包含机器人的 user ID（@提及，兼容旧客户端）
///   - 消息的 `mentions` 字段包含机器人（MSC 3456）
///
/// # 流式响应策略
///
/// 当启用流式输出时，使用混合节流策略更新消息：
/// - 时间触发：超过配置的最小间隔
/// - 字符触发：累积超过配置的最小字符数
///
/// # Example
///
/// ```ignore
/// use aether_matrix::event_handler::EventHandler;
/// use aether_matrix::ai_service::AiService;
///
/// let handler = EventHandler::new(ai_service, bot_user_id, &config);
///
/// client.add_event_handler(move |ev, room| {
///     let handler = handler.clone();
///     async move {
///         handler.handle_message(ev, room).await
///     }
/// });
/// ```
#[derive(Clone)]
pub struct EventHandler<T: AiServiceTrait> {
    /// AI 服务实例
    ai_service: T,
    /// 机器人的 Matrix 用户 ID
    bot_user_id: OwnedUserId,
    /// 命令前缀（如 `!ai`）
    command_prefix: String,
    /// 是否启用流式输出
    streaming_enabled: bool,
    /// 流式更新的最小时间间隔
    streaming_min_interval: Duration,
    /// 流式更新的最小字符数阈值
    streaming_min_chars: usize,
}

impl<T: AiServiceTrait> EventHandler<T> {
    /// 创建新的事件处理器。
    ///
    /// # Arguments
    ///
    /// * `ai_service` - AI 服务实例
    /// * `bot_user_id` - 机器人的 Matrix 用户 ID
    /// * `config` - 机器人配置
    pub fn new(ai_service: T, bot_user_id: OwnedUserId, config: &Config) -> Self {
        Self {
            ai_service,
            bot_user_id,
            command_prefix: config.command_prefix.clone(),
            streaming_enabled: config.streaming_enabled,
            streaming_min_interval: Duration::from_millis(config.streaming_min_interval_ms),
            streaming_min_chars: config.streaming_min_chars,
        }
    }

    /// 处理房间消息事件。
    ///
    /// 判断是否需要响应消息，并调用相应的 AI 服务。
    ///
    /// # Arguments
    ///
    /// * `ev` - 消息事件
    /// * `room` - 消息所在的房间
    ///
    /// # Returns
    ///
    /// 成功时返回 `Ok(())`，失败时返回错误。
    ///
    /// # 响应逻辑
    ///
    /// 1. 忽略已删除的消息和自己发送的消息
    /// 2. 判断是否应该响应（私聊/群聊规则不同）
    /// 3. 处理特殊命令（`!reset`, `!help`）
    /// 4. 调用 AI 服务生成响应
    pub async fn handle_message(
        &self,
        ev: matrix_sdk::ruma::events::room::message::SyncRoomMessageEvent,
        room: Room,
    ) -> Result<()> {
        // 使用 as_original() 获取原始消息事件
        let original = match ev.as_original() {
            Some(o) => o,
            None => return Ok(()), // 忽略已删除的消息
        };

        // 跳过自己发送的消息，避免无限循环
        if original.sender == self.bot_user_id {
            return Ok(());
        }

        let text = original.content.body();
        let room_id = room.room_id();

        // 判断是否应该响应
        let is_direct = room.is_direct().await.unwrap_or(false);

        // 检查是否通过 Intentional Mentions (MSC 3456) 被提及
        // 这是现代客户端的标准提及方式
        let mentions_bot = original
            .content
            .mentions
            .as_ref()
            .is_some_and(|m| m.user_ids.contains(&self.bot_user_id));

        let should_respond = if is_direct {
            // 私聊：总是响应
            true
        } else {
            // 群聊：检查命令前缀、文本中的 user_id（兼容旧客户端）或 mentions 字段（现代客户端）
            text.starts_with(&self.command_prefix)
                || text.contains(&self.bot_user_id.to_string())
                || mentions_bot
        };

        if !should_respond {
            return Ok(());
        }

        // 提取纯净的消息文本（移除命令前缀和 @提及）
        let clean_text = self.extract_message(text);

        // 处理特殊命令
        if clean_text == "!reset" {
            let session_id = room_id.to_string();
            self.ai_service.reset_conversation(&session_id).await;
            room.send(RoomMessageEventContent::text_plain("会话历史已清除"))
                .await?;
            return Ok(());
        }

        if clean_text == "!help" {
            let help_text = format!(
                "可用命令:\n{} <消息> - 与 AI 对话\n!reset - 清除会话历史\n!help - 显示帮助",
                self.command_prefix
            );
            room.send(RoomMessageEventContent::text_plain(help_text))
                .await?;
            return Ok(());
        }

        // 生成会话 ID：私聊用用户 ID（保持个人对话上下文），群聊用房间 ID（共享对话上下文）
        let session_id = if is_direct {
            original.sender.to_string()
        } else {
            room_id.to_string()
        };

        debug!("处理消息 [{}]: {}", session_id, clean_text);

        // 根据配置选择流式或普通响应
        if self.streaming_enabled {
            self.handle_streaming_response(&room, &session_id, &clean_text)
                .await?;
        } else {
            self.handle_normal_response(&room, &session_id, &clean_text)
                .await?;
        }

        Ok(())
    }

    /// 发送普通（非流式）响应。
    ///
    /// 调用 AI 服务获取完整回复，然后一次性发送。
    async fn handle_normal_response(
        &self,
        room: &Room,
        session_id: &str,
        clean_text: &str,
    ) -> Result<()> {
        match self.ai_service.chat(session_id, clean_text).await {
            Ok(reply) => {
                room.send(RoomMessageEventContent::text_plain(reply))
                    .await?;
            }
            Err(e) => {
                warn!("AI 调用失败: {}", e);
                // 向用户显示友好的错误消息
                room.send(RoomMessageEventContent::text_plain(format!(
                    "AI 服务暂时不可用: {}",
                    e
                )))
                .await?;
            }
        }
        Ok(())
    }

    /// 发送流式响应（打字机效果）。
    ///
    /// 使用混合节流策略更新消息：
    /// - 时间触发：超过最小间隔时更新
    /// - 字符触发：累积超过最小字符数时更新
    ///
    /// 首次发送新消息，后续使用 Matrix 消息编辑 API 更新内容。
    async fn handle_streaming_response(
        &self,
        room: &Room,
        session_id: &str,
        clean_text: &str,
    ) -> Result<()> {
        // 开始流式聊天
        let (state, mut stream) = match self.ai_service.chat_stream(session_id, clean_text).await {
            Ok(result) => result,
            Err(e) => {
                warn!("流式 AI 调用初始化失败: {}", e);
                room.send(RoomMessageEventContent::text_plain(format!(
                    "AI 服务暂时不可用: {}",
                    e
                )))
                .await?;
                return Ok(());
            }
        };

        // 状态追踪
        let mut event_id: Option<OwnedEventId> = None;
        let mut chars_since_update: usize = 0;
        let mut last_update = Instant::now();

        // 消费流
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(delta) => {
                    chars_since_update += delta.chars().count();

                    // 混合节流策略：时间或字符数任一满足即更新
                    let time_elapsed = last_update.elapsed() >= self.streaming_min_interval;
                    let chars_accumulated = chars_since_update >= self.streaming_min_chars;

                    if time_elapsed || chars_accumulated {
                        // 获取当前累积的完整内容
                        let content = {
                            let s = state.lock().await;
                            s.content().to_string()
                        };

                        // 发送或编辑消息
                        if let Some(ref original_event_id) = event_id {
                            // 使用 Matrix 消息编辑 API 更新已有消息
                            let metadata =
                                ReplacementMetadata::new(original_event_id.clone(), None);
                            let msg_content = RoomMessageEventContent::text_plain(&content)
                                .make_replacement(metadata);
                            room.send(msg_content).await?;
                        } else {
                            // 首次发送新消息
                            let response = room
                                .send(RoomMessageEventContent::text_plain(&content))
                                .await?;
                            event_id = Some(response.event_id);
                        }

                        // 重置节流计数器
                        chars_since_update = 0;
                        last_update = Instant::now();
                    }
                }
                Err(e) => {
                    warn!("流式响应错误: {}", e);
                    // 优雅处理错误：显示已接收内容并追加错误信息
                    let content = {
                        let s = state.lock().await;
                        s.content().to_string()
                    };

                    if !content.is_empty() {
                        // 已有内容，追加错误信息
                        let error_msg = format!("{}\n\n[错误: {}]", content, e);
                        if let Some(ref original_event_id) = event_id {
                            let metadata =
                                ReplacementMetadata::new(original_event_id.clone(), None);
                            let msg_content = RoomMessageEventContent::text_plain(&error_msg)
                                .make_replacement(metadata);
                            room.send(msg_content).await?;
                        } else {
                            room.send(RoomMessageEventContent::text_plain(&error_msg))
                                .await?;
                        }
                    } else {
                        // 无内容，仅显示错误
                        room.send(RoomMessageEventContent::text_plain(format!(
                            "AI 服务暂时不可用: {}",
                            e
                        )))
                        .await?;
                    }
                    return Ok(());
                }
            }
        }

        // 流结束，确保发送最终内容（处理最后可能残留的更新）
        let final_content = {
            let s = state.lock().await;
            s.content().to_string()
        };

        if !final_content.is_empty()
            && let Some(ref original_event_id) = event_id
        {
            let metadata = ReplacementMetadata::new(original_event_id.clone(), None);
            let msg_content =
                RoomMessageEventContent::text_plain(&final_content).make_replacement(metadata);
            room.send(msg_content).await?;
        }

        Ok(())
    }

    /// 从原始消息文本中提取纯净的消息内容。
    ///
    /// 移除命令前缀和 @提及，返回修剪后的文本。
    ///
    /// # Arguments
    ///
    /// * `text` - 原始消息文本
    ///
    /// # Returns
    ///
    /// 移除前缀和提及后的纯净文本
    fn extract_message(&self, text: &str) -> String {
        let mut result = text.to_string();

        // 移除命令前缀（如 `!ai`）
        if result.starts_with(&self.command_prefix) {
            result = result[self.command_prefix.len()..].to_string();
        }

        // 移除 @提及（兼容旧客户端）
        result = result.replace(&self.bot_user_id.to_string(), "");

        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use matrix_sdk::ruma::user_id;

    // 用于测试的 mock AiService
    #[derive(Clone)]
    struct MockAiService;

    impl AiServiceTrait for MockAiService {
        async fn chat(&self, _session_id: &str, _prompt: &str) -> anyhow::Result<String> {
            Ok("mock response".to_string())
        }

        async fn reset_conversation(&self, _session_id: &str) {}

        async fn chat_stream(
            &self,
            _session_id: &str,
            _prompt: &str,
        ) -> anyhow::Result<(
            std::sync::Arc<tokio::sync::Mutex<crate::ai_service::StreamingState>>,
            std::pin::Pin<Box<dyn futures_util::Stream<Item = anyhow::Result<String>> + Send>>,
        )> {
            unimplemented!()
        }
    }

    fn create_test_handler() -> EventHandler<MockAiService> {
        let config = Config {
            matrix_homeserver: "https://matrix.org".to_string(),
            matrix_username: "test".to_string(),
            matrix_password: "test".to_string(),
            matrix_device_id: None,
            device_display_name: "Test Bot".to_string(),
            store_path: "./store".to_string(),
            openai_api_key: "test".to_string(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
            openai_model: "gpt-4o-mini".to_string(),
            system_prompt: None,
            command_prefix: "!ai ".to_string(),
            max_history: 10,
            streaming_enabled: false,
            streaming_min_interval_ms: 500,
            streaming_min_chars: 10,
            log_level: "info".to_string(),
        };
        let bot_user_id = user_id!("@bot:matrix.org").to_owned();
        EventHandler::new(MockAiService, bot_user_id, &config)
    }

    #[test]
    fn test_extract_message_with_prefix() {
        let handler = create_test_handler();
        let result = handler.extract_message("!ai Hello world");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_extract_message_with_prefix_and_spaces() {
        let handler = create_test_handler();
        let result = handler.extract_message("!ai   Multiple spaces   ");
        assert_eq!(result, "Multiple spaces");
    }

    #[test]
    fn test_extract_message_with_mention() {
        let handler = create_test_handler();
        let result = handler.extract_message("@bot:matrix.org Hello there");
        assert_eq!(result, "Hello there");
    }

    #[test]
    fn test_extract_message_with_prefix_and_mention() {
        let handler = create_test_handler();
        let result = handler.extract_message("!ai @bot:matrix.org Combined message");
        assert_eq!(result, "Combined message");
    }

    #[test]
    fn test_extract_message_plain_text() {
        let handler = create_test_handler();
        let result = handler.extract_message("Just a plain message");
        assert_eq!(result, "Just a plain message");
    }

    #[test]
    fn test_extract_message_empty_after_trim() {
        let handler = create_test_handler();
        let result = handler.extract_message("!ai    ");
        assert_eq!(result, "");
    }
}
