//! Matrix 事件处理模块。
//!
//! 处理两类 Matrix 事件：
//! - **邀请事件**: 自动接受房间邀请（[`handle_invite`]）
//! - **消息事件**: 处理用户消息并调用 AI 服务（[`EventHandler`]）
//!
//! # 消息处理流程
//!
//! 1. 过滤自己发送的消息
//! 2. 识别消息类型（命令/提及/普通消息）
//! 3. 根据房间类型决定响应策略：
//!    - 私聊：总是响应
//!    - 群聊：仅在提及或命令时响应
//! 4. 调用 AI 服务（普通/流式/Vision）

use std::sync::Arc;
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

use crate::command::CommandGateway;
use crate::config::Config;
use crate::media::download_image_as_base64;
use crate::modules::admin::{BotInfoHandler, BotLeaveHandler, BotPingHandler};
use crate::modules::persona::PersonaHandler;
use crate::store::PersonaStore;
use crate::traits::AiServiceTrait;
use matrix_sdk::ruma::events::room::message::MessageType;

/// 处理房间邀请（独立函数，不依赖 EventHandler 实例）。
///
/// 当机器人收到房间邀请时自动加入，支持私聊和群聊场景。
///
/// # Arguments
///
/// * `ev` - Matrix 邀请事件
/// * `client` - Matrix 客户端实例
/// * `room` - 邀请的房间实例
///
/// # Returns
///
/// 成功时返回 `Ok(())`，失败时返回错误。
///
/// # Example
///
/// ```ignore
/// use aether_matrix::event_handler::handle_invite;
///
/// // 注册为事件处理器
/// client.add_event_handler(
///     |ev: StrippedRoomMemberEvent, client: Client, room: Room| async move {
///         if let Err(e) = handle_invite(ev, client, room).await {
///             tracing::error!("处理邀请失败: {}", e);
///         }
///     }
/// );
/// ```
pub async fn handle_invite(ev: StrippedRoomMemberEvent, client: Client, room: Room) -> Result<()> {
    if ev.content.membership != MembershipState::Invite {
        return Ok(());
    }

    let user_id = &ev.state_key;
    let my_user_id = client.user_id().expect("user_id should be available");
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

#[derive(Clone)]
/// Matrix 消息事件处理器。
///
/// 泛型参数 `T` 支持任意实现 [`AiServiceTrait`] 的服务，
/// 便于测试和替换 AI 实现。
///
/// # 职责
///
/// - 消息路由：识别命令、提及、普通消息
/// - AI 响应：调用 AI 服务并处理响应（普通/流式/Vision）
/// - 错误处理：优雅处理 AI 服务错误
///
/// # Example
///
/// ```ignore
/// use aether_matrix::event_handler::EventHandler;
/// use aether_matrix::ai_service::AiService;
///
/// let handler = EventHandler::new(
///     ai_service,
///     bot_user_id,
///     client,
///     &config,
///     persona_store,
/// );
///
/// // 注册为事件处理器
/// client.add_event_handler(handler);
/// ```
pub struct EventHandler<T: AiServiceTrait> {
    ai_service: T,
    client: Client,
    bot_user_id: OwnedUserId,
    command_prefix: String,
    command_gateway: CommandGateway,
    persona_store: Option<PersonaStore>,
    streaming_enabled: bool,
    streaming_min_interval: Duration,
    streaming_min_chars: usize,
    vision_enabled: bool,
    vision_max_image_size: u32,
}

impl<T: AiServiceTrait> EventHandler<T> {
    /// 创建新的事件处理器实例。
    ///
    /// 初始化时会注册以下命令处理器：
    /// - [`BotInfoHandler`]\: 显示机器人信息
    /// - [`BotLeaveHandler`]\: 机器人离开房间
    /// - [`BotPingHandler`]\: Ping 测试
    /// - [`PersonaHandler`]\: 人设管理（如果数据库可用）
    ///
    /// # Arguments
    ///
    /// * `ai_service` - AI 服务实例
    /// * `bot_user_id` - 机器人的 Matrix 用户 ID
    /// * `client` - Matrix 客户端实例
    /// * `config` - 机器人配置
    /// * `persona_store` - 人设存储（可选）
    ///
    /// # Returns
    ///
    /// 返回初始化完成的事件处理器实例。
    pub fn new(
        ai_service: T,
        bot_user_id: OwnedUserId,
        client: Client,
        config: &Config,
        persona_store: Option<PersonaStore>,
    ) -> Self {
        let mut command_gateway =
            CommandGateway::new(config.command_prefix.clone(), config.bot_owners.clone());

        command_gateway.register(Arc::new(BotInfoHandler));
        command_gateway.register(Arc::new(BotLeaveHandler));
        command_gateway.register(Arc::new(BotPingHandler));

        if let Some(ref store) = persona_store {
            command_gateway.register(Arc::new(PersonaHandler::new(store.clone())));
        }

        Self {
            ai_service,
            client,
            bot_user_id,
            command_prefix: config.command_prefix.clone(),
            command_gateway,
            persona_store,
            streaming_enabled: config.streaming_enabled,
            streaming_min_interval: Duration::from_millis(config.streaming_min_interval_ms),
            streaming_min_chars: config.streaming_min_chars,
            vision_enabled: config.vision_enabled,
            vision_max_image_size: config.vision_max_image_size,
        }
    }

    /// 处理 Matrix 房间消息事件。
    ///
    /// 根据消息类型和房间类型决定响应策略：
    /// - **命令消息**: 分发给命令处理器
    /// - **提及消息**: 调用 AI 服务响应
    /// - **图片消息**: 调用 Vision API 分析
    ///
    /// # 响应策略
    ///
    /// | 房间类型 | 响应条件 |
    /// |---------|---------|
    /// | 私聊 | 总是响应 |
    /// | 群聊 | 命令前缀 / @提及 / Intentional Mentions |
    ///
    /// # Arguments
    ///
    /// * `ev` - Matrix 消息事件
    /// * `room` - 房间实例
    /// * `client` - Matrix 客户端
    ///
    /// # Returns
    ///
    /// 成功时返回 `Ok(())`，失败时返回错误。
    ///
    /// # Errors
    ///
    /// 当以下情况发生时返回错误：
    /// - 命令分发失败
    /// - AI 服务调用失败
    /// - Matrix API 调用失败
    pub async fn handle_message(
        &self,
        ev: matrix_sdk::ruma::events::room::message::SyncRoomMessageEvent,
        room: Room,
        client: Client,
    ) -> Result<()> {
        // Matrix SDK 的 SyncRoomMessageEvent 可能包含编辑/删除等衍生事件
        // 使用 as_original() 获取原始消息，过滤掉衍生事件
        let original = match ev.as_original() {
            Some(o) => o,
            None => return Ok(()), // 忽略已删除或编辑后的消息
        };

        // 跳过自己发送的消息，避免消息循环
        if original.sender == self.bot_user_id {
            return Ok(());
        }

        // 获取消息文本
        let text = original.content.body();
        let event_id = original.event_id.clone();

        let room_id = room.room_id();

        // 判断是否应该响应
        let is_direct = room.is_direct().await.unwrap_or(false);

        // MSC 3456 (Intentional Mentions) 是现代 Matrix 客户端推荐的提及方式
        // 相比文本匹配 @user_id，mentions 字段更精确且支持富文本
        let mentions_bot = original
            .content
            .mentions
            .as_ref()
            .is_some_and(|m| m.user_ids.contains(&self.bot_user_id));

        // 检查是否是命令（以命令前缀开头）
        let is_command = self.command_gateway.is_command(text);

        info!("收到消息: '{}', 是否命令: {}", text, is_command);

        // 处理命令：优先分发命令，避免命令被当作普通消息处理
        if is_command {
            info!("分发命令: {}", text);
            self.command_gateway
                .dispatch(
                    &client,
                    room.clone(),
                    original.sender.clone(),
                    text,
                    event_id.clone(),
                )
                .await?;
            return Ok(());
        }

        // 响应策略：私聊总是响应，群聊需要明确的触发条件
        // 私聊场景：用户体验优先，避免用户需要手动提及
        // 群聊场景：避免机器人刷屏，仅在明确触发时响应
        let should_respond = if is_direct {
            true
        } else {
            // 兼容两种提及方式：
            // 1. 文本包含 @user_id（旧客户端，部分 Matrix 客户端）
            // 2. mentions 字段（现代客户端，推荐方式）
            text.contains(&self.bot_user_id.to_string()) || mentions_bot
        };

        if !should_respond {
            return Ok(());
        }

        // 会话隔离策略：
        // - 私聊：按用户 ID 隔离，每个用户有独立的对话历史
        // - 群聊：按房间 ID 隔离，房间内所有用户共享历史
        let session_id = if is_direct {
            original.sender.to_string()
        } else {
            room_id.to_string()
        };

        // 获取房间的人设系统提示词
        let system_prompt = if let Some(ref store) = self.persona_store {
            match store.get_room_persona(room_id.as_str()) {
                Ok(Some(persona)) => {
                    debug!("使用人设 '{}' 的系统提示词", persona.name);
                    Some(persona.system_prompt)
                }
                Ok(None) => None,
                Err(e) => {
                    warn!("获取房间人设失败: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // 根据消息类型处理
        match &original.content.msgtype {
            MessageType::Text(text_msg) => {
                let clean_text = self.extract_message(&text_msg.body);
                debug!("处理消息 [{}]: {}", session_id, clean_text);

                if self.streaming_enabled {
                    self.handle_streaming_response(&room, &session_id, &clean_text, system_prompt.as_deref())
                        .await?;
                } else {
                    self.handle_normal_response(&room, &session_id, &clean_text, system_prompt.as_deref())
                        .await?;
                }
            }
MessageType::Image(image_msg) if self.vision_enabled => {
                debug!("处理图片消息 [{}]", session_id);
                match self.handle_image_message(&room, &session_id, image_msg, system_prompt.as_deref()).await {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("图片处理失败: {}", e);
                        room.send(RoomMessageEventContent::text_plain(format!(
                            "图片处理失败: {}",
                            e
                        )))
                        .await?;
                    }
                }
            }
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_image_message(
        &self,
        room: &Room,
        session_id: &str,
        image_msg: &matrix_sdk::ruma::events::room::message::ImageMessageEventContent,
        system_prompt: Option<&str>,
    ) -> Result<()> {
        let mxc_uri = match &image_msg.source {
            matrix_sdk::ruma::events::room::MediaSource::Plain(uri) => uri,
            matrix_sdk::ruma::events::room::MediaSource::Encrypted(_) => {
                anyhow::bail!("不支持加密图片");
            }
        };

        let image_data_url =
            download_image_as_base64(&self.client, mxc_uri, None, self.vision_max_image_size)
                .await?;

        let prompt = image_msg.body.clone();
        let reply = self
            .ai_service
            .chat_with_image(session_id, &prompt, &image_data_url)
            .await?;

        room.send(RoomMessageEventContent::text_plain(reply))
            .await?;
        Ok(())
    }

    /// 普通响应（非流式）
    async fn handle_normal_response(
        &self,
        room: &Room,
        session_id: &str,
        clean_text: &str,
        system_prompt: Option<&str>,
    ) -> Result<()> {
        match self.ai_service.chat_with_system(session_id, clean_text, system_prompt).await {
            Ok(reply) => {
                room.send(RoomMessageEventContent::text_plain(reply))
                    .await?;
            }
            Err(e) => {
                warn!("AI 调用失败: {}", e);
                room.send(RoomMessageEventContent::text_plain(format!(
                    "AI 服务暂时不可用: {}",
                    e
                )))
                .await?;
            }
        }
        Ok(())
    }

    /// 流式响应处理（打字机效果）。
    ///
    /// 使用混合节流策略控制消息更新频率，平衡用户体验和 API 调用开销：
    /// - **时间触发**: 超过 `streaming_min_interval` 时更新
    /// - **字符触发**: 累积超过 `streaming_min_chars` 时更新
    ///
    /// # 消息更新机制
    ///
    /// 1. 首次发送新消息
    /// 2. 后续使用 Matrix 消息编辑 API 更新内容
    /// 3. 流结束时发送最终版本
    ///
    /// # Error Handling
    ///
    /// 流中途出错时，会在已发送内容后追加错误信息，
    /// 避免用户看到突然中断的消息。
    ///
    /// # Arguments
    ///
    /// * `room` - Matrix 房间实例
    /// * `session_id` - 会话标识符
    /// * `clean_text` - 清理后的用户消息（已移除命令前缀和提及）
    ///
    /// # Returns
    ///
    /// 成功时返回 `Ok(())`，失败时返回错误。
    async fn handle_streaming_response(
        &self,
        room: &Room,
        session_id: &str,
        clean_text: &str,
        system_prompt: Option<&str>,
    ) -> Result<()> {
        // 开始流式聊天
        let (state, mut stream) = match self.ai_service.chat_stream_with_system(session_id, clean_text, system_prompt).await {
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

                    // 混合节流策略：避免过于频繁的消息编辑 API 调用
                    // 时间触发：保证最小更新间隔，避免刷屏
                    // 字符触发：在快速输出时提前更新，提升用户体验
                    let time_elapsed = last_update.elapsed() >= self.streaming_min_interval;
                    let chars_accumulated = chars_since_update >= self.streaming_min_chars;

                    if time_elapsed || chars_accumulated {
                        // 获取当前累积的内容
                        let content = {
                            let s = state.lock().await;
                            s.content().to_string()
                        };

                        // 发送或编辑消息
                        if let Some(ref original_event_id) = event_id {
                            // 编辑已有消息：使用 Matrix 消息编辑功能
                            let metadata =
                                ReplacementMetadata::new(original_event_id.clone(), None);
                            let msg_content = RoomMessageEventContent::text_plain(&content)
                                .make_replacement(metadata);
                            room.send(msg_content).await?;
                        } else {
                            // 发送新消息：首次响应
                            let response = room
                                .send(RoomMessageEventContent::text_plain(&content))
                                .await?;
                            event_id = Some(response.event_id);
                        }

                        // 重置计数器
                        chars_since_update = 0;
                        last_update = Instant::now();
                    }
                }
                Err(e) => {
                    warn!("流式响应错误: {}", e);
                    // 错误处理：在已发送内容后追加错误信息，保持上下文连续性
                    let content = {
                        let s = state.lock().await;
                        s.content().to_string()
                    };

                    if !content.is_empty() {
                        // 已有部分内容，追加错误信息
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

        // 流结束，发送最终消息
        // 注意：仅在事件 ID 存在时编辑，避免重复发送
        let final_content = {
            let s = state.lock().await;
            s.content().to_string()
        };

        if !final_content.is_empty()
            && let Some(ref original_event_id) = event_id
        {
            // 编辑为最终内容
            let metadata = ReplacementMetadata::new(original_event_id.clone(), None);
            let msg_content =
                RoomMessageEventContent::text_plain(&final_content).make_replacement(metadata);
            room.send(msg_content).await?;
        }

        Ok(())
    }

    /// 清理用户消息，移除命令前缀和提及。
    ///
    /// 处理以下情况：
    /// - 移除命令前缀（如 `!ai`）
    /// - 移除文本形式的 @user_id 提及
    /// - 去除首尾空白
    ///
    /// # Arguments
    ///
    /// * `text` - 原始消息文本
    ///
    /// # Returns
    ///
    /// 返回清理后的消息文本。
    fn extract_message(&self, text: &str) -> String {
        let mut result = text.to_string();

        // 移除命令前缀：允许用户使用简短的命令语法
        // 例如：!help 而不是 !ai help
        if result.starts_with(&self.command_prefix) {
            result = result[self.command_prefix.len()..].to_string();
        }

        // 移除 @提及：支持文本形式的提及（旧客户端兼容）
        // 例如：@bot:matrix.org 你好 -> 你好
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

        async fn chat_with_system(
            &self,
            _session_id: &str,
            _prompt: &str,
            _system_prompt: Option<&str>,
        ) -> anyhow::Result<String> {
            Ok("mock response".to_string())
        }

        async fn reset_conversation(&self, _session_id: &str) {}

        async fn chat_stream(
            &self,
            _session_id: &str,
            _prompt: &str,
        ) -> anyhow::Result<(
            std::sync::Arc<tokio::sync::Mutex<crate::traits::StreamingState>>,
            std::pin::Pin<Box<dyn futures_util::Stream<Item = anyhow::Result<String>> + Send>>,
        )> {
            unimplemented!()
        }

        async fn chat_stream_with_system(
            &self,
            _session_id: &str,
            _prompt: &str,
            _system_prompt: Option<&str>,
        ) -> anyhow::Result<crate::traits::ChatStreamResponse> {
            unimplemented!()
        }

        async fn chat_with_image(
            &self,
            _session_id: &str,
            _text: &str,
            _image_data_url: &str,
        ) -> anyhow::Result<String> {
            Ok("mock vision response".to_string())
        }

        async fn chat_with_image_stream(
            &self,
            _session_id: &str,
            _text: &str,
            _image_data_url: &str,
        ) -> anyhow::Result<crate::traits::ChatStreamResponse> {
            unimplemented!()
        }
    }

    fn create_test_handler() -> EventHandler<MockAiService> {
        use matrix_sdk::Client;
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
            command_prefix: "!".to_string(),
            max_history: 10,
            bot_owners: Vec::new(),
            db_path: "./data/aether.db".to_string(),
            streaming_enabled: false,
            streaming_min_interval_ms: 500,
            streaming_min_chars: 10,
            log_level: "info".to_string(),
            vision_enabled: true,
            vision_model: None,
            vision_max_image_size: 1024,
            proxy: None,
        };
        let bot_user_id = user_id!("@bot:matrix.org").to_owned();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = rt.block_on(async {
            Client::builder()
                .homeserver_url("https://matrix.org")
                .build()
                .await
                .unwrap()
        });
        EventHandler::new(MockAiService, bot_user_id, client, &config, None)
    }

    #[test]
    fn test_extract_message_with_prefix() {
        let handler = create_test_handler();
        let result = handler.extract_message("! Hello world");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_extract_message_with_prefix_and_spaces() {
        let handler = create_test_handler();
        let result = handler.extract_message("!   Multiple spaces   ");
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
        let result = handler.extract_message("! @bot:matrix.org Combined message");
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
        let result = handler.extract_message("!    ");
        assert_eq!(result, "");
    }
}
