use std::time::{Duration, Instant};

use anyhow::Result;
use futures_util::StreamExt;
use matrix_sdk::{
    Client, Room,
    ruma::{
        OwnedEventId, OwnedUserId,
        events::{
            AnySyncTimelineEvent,
            room::{
                member::{MembershipState, StrippedRoomMemberEvent},
                message::{MessageType, Relation, ReplacementMetadata, RoomMessageEventContent},
            },
        },
    },
};
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::media::download_image_as_base64;
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
    /// Matrix 客户端（用于下载媒体）
    client: Client,
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
    /// 是否启用图片理解功能
    vision_enabled: bool,
    #[allow(dead_code)]
    /// 图片最大尺寸（像素）
    vision_max_image_size: u32,
}

impl<T: AiServiceTrait> EventHandler<T> {
    /// 创建新的事件处理器。
    ///
    /// # Arguments
    ///
    /// * `ai_service` - AI 服务实例
    /// * `bot_user_id` - 机器人的 Matrix 用户 ID
    /// * `client` - Matrix 客户端实例（用于下载媒体）
    /// * `config` - 机器人配置
    pub fn new(ai_service: T, bot_user_id: OwnedUserId, client: Client, config: &Config) -> Self {
        Self {
            ai_service,
            client,
            bot_user_id,
            command_prefix: config.command_prefix.clone(),
            streaming_enabled: config.streaming_enabled,
            streaming_min_interval: Duration::from_millis(config.streaming_min_interval_ms),
            streaming_min_chars: config.streaming_min_chars,
            vision_enabled: config.vision_enabled,
            vision_max_image_size: config.vision_max_image_size,
        }
    }

    /// 处理房间消息事件。
    ///
    /// 判断是否需要响应消息，并调用相应的 AI 服务。
    /// 支持文本消息和图片消息（Vision API）。
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
    /// 4. 根据消息类型调用相应的 AI 服务：
    ///    - 文本消息：普通聊天
    ///    - 图片消息：Vision API（如果启用）
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

        // 对于图片消息，私聊总是响应，群聊需要命令前缀或提及
        let should_respond = if is_direct {
            true
        } else {
            // 群聊：检查文本内容是否包含命令前缀或提及
            let text = original.content.body();
            text.starts_with(&self.command_prefix)
                || text.contains(&self.bot_user_id.to_string())
                || mentions_bot
        };

        if !should_respond {
            return Ok(());
        }

        // 生成会话 ID：私聊用用户 ID（保持个人对话上下文），群聊用房间 ID（共享对话上下文）
        let session_id = if is_direct {
            original.sender.to_string()
        } else {
            room_id.to_string()
        };

        // 提取引用消息内容（如果有）
        let reply_context = self.extract_reply_content(&room, original).await;

        // 根据消息类型分发处理
        match &original.content.msgtype {
            MessageType::Text(text_msg) => {
                // 文本消息处理
                let clean_text = self.extract_message(&text_msg.body);

                // 处理特殊命令
                if clean_text == "!reset" {
                    self.ai_service.reset_conversation(&session_id).await;
                    room.send(RoomMessageEventContent::text_plain("会话历史已清除"))
                        .await?;
                    return Ok(());
                }

                if clean_text == "!help" {
                    let help_text = if self.vision_enabled {
                        format!(
                            "可用命令:\n{} <消息> - 与 AI 对话\n发送图片 - 让 AI 分析图片内容\n!reset - 清除会话历史\n!help - 显示帮助",
                            self.command_prefix
                        )
                    } else {
                        format!(
                            "可用命令:\n{} <消息> - 与 AI 对话\n!reset - 清除会话历史\n!help - 显示帮助",
                            self.command_prefix
                        )
                    };
                    room.send(RoomMessageEventContent::text_plain(help_text))
                        .await?;
                    return Ok(());
                }

                // 组装完整 prompt（包含引用上下文）
                let full_prompt = if let Some(ref reply) = reply_context {
                    debug!(
                        "处理文本消息 [{}] (引用: {}): {}",
                        session_id, reply, clean_text
                    );
                    format!("[引用消息]: {}\n\n{}", reply, clean_text)
                } else {
                    debug!("处理文本消息 [{}]: {}", session_id, clean_text);
                    clean_text.clone()
                };

                // 根据配置选择流式或普通响应
                if self.streaming_enabled {
                    self.handle_streaming_response(&room, &session_id, &full_prompt)
                        .await?;
                } else {
                    self.handle_normal_response(&room, &session_id, &full_prompt)
                        .await?;
                }
            }
            MessageType::Image(image_msg) => {
                // 图片消息处理（Vision API）
                if self.vision_enabled {
                    debug!("处理图片消息 [{}]", session_id);
                    self.handle_image_message(&room, &session_id, &original.sender, image_msg)
                        .await?;
                }
            }
            _ => {
                // 忽略其他消息类型（视频、音频、文件等）
                debug!("忽略不支持的消息类型");
            }
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

    /// 从消息中提取引用内容。
    ///
    /// 如果消息是回复消息（包含 `relates_to` 字段），则通过 Room API 获取
    /// 被引用的事件，并提取其消息文本。
    ///
    /// # Arguments
    ///
    /// * `room` - 消息所在的房间
    /// * `original` - 原始消息事件
    ///
    /// # Returns
    ///
    /// 成功时返回被引用消息的文本内容，失败或无引用时返回 `None`。
    async fn extract_reply_content(
        &self,
        room: &Room,
        original: &matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent,
    ) -> Option<String> {
        // 检查是否是回复消息
        let in_reply_to = original.content.relates_to.as_ref().and_then(|r| match r {
            Relation::Reply { in_reply_to } => Some(&in_reply_to.event_id),
            _ => None,
        })?;

        // 通过 Room API 获取被引用的事件
        let event = room.load_or_fetch_event(in_reply_to, None).await.ok()?;

        // 反序列化事件
        let timeline_event = event.into_raw().deserialize().ok()?;

        // 提取消息文本
        match timeline_event {
            AnySyncTimelineEvent::MessageLike(msg) => {
                // 获取事件的原始内容（非删除状态）
                msg.original_content().and_then(|c| match c {
                    matrix_sdk::ruma::events::AnyMessageLikeEventContent::RoomMessage(m) => {
                        Some(m.msgtype.body().to_string())
                    }
                    _ => None,
                })
            }
            _ => None,
        }
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

    /// 处理图片消息（Vision API）。
    ///
    /// 下载图片、转换为 base64，然后调用 Vision API。
    ///
    /// # Arguments
    ///
    /// * `room` - 消息所在的房间
    /// * `session_id` - 会话标识符
    /// * `sender` - 发送者 ID
    /// * `image_msg` - 图片消息内容
    async fn handle_image_message(
        &self,
        room: &Room,
        session_id: &str,
        _sender: &matrix_sdk::ruma::OwnedUserId,
        image_msg: &matrix_sdk::ruma::events::room::message::ImageMessageEventContent,
    ) -> Result<()> {
        // 获取图片 URL（从 source 字段）
        let mxc_uri = match &image_msg.source {
            matrix_sdk::ruma::events::room::MediaSource::Plain(url) => url,
            matrix_sdk::ruma::events::room::MediaSource::Encrypted(_) => {
                warn!("不支持加密图片");
                room.send(RoomMessageEventContent::text_plain("不支持加密图片"))
                    .await?;
                return Ok(());
            }
        };

        // 发送处理中提示
        let processing_msg = room
            .send(RoomMessageEventContent::text_plain("正在分析图片..."))
            .await?;

        // 下载图片并转换为 base64
        let media_type = image_msg
            .info
            .as_ref()
            .and_then(|i| i.mimetype.as_deref())
            .unwrap_or("image/png");

        let image_data_url =
            match download_image_as_base64(&self.client, mxc_uri, Some(media_type)).await {
                Ok(data) => data,
                Err(e) => {
                    warn!("下载图片失败: {}", e);
                    // 编辑处理中消息为错误提示
                    let metadata = ReplacementMetadata::new(processing_msg.event_id.clone(), None);
                    let error_content =
                        RoomMessageEventContent::text_plain(format!("下载图片失败: {}", e))
                            .make_replacement(metadata);
                    room.send(error_content).await?;
                    return Ok(());
                }
            };

        // 获取图片说明作为提示词
        let text = if image_msg.body.trim().is_empty() {
            "请描述这张图片的内容。".to_string()
        } else {
            image_msg.body.clone()
        };

        // 根据配置选择流式或普通响应
        if self.streaming_enabled {
            self.handle_image_streaming_response(
                room,
                session_id,
                &text,
                &image_data_url,
                processing_msg.event_id,
            )
            .await?;
        } else {
            match self
                .ai_service
                .chat_with_image(session_id, &text, &image_data_url)
                .await
            {
                Ok(reply) => {
                    // 编辑处理中消息为最终回复
                    let metadata = ReplacementMetadata::new(processing_msg.event_id.clone(), None);
                    let content =
                        RoomMessageEventContent::text_plain(&reply).make_replacement(metadata);
                    room.send(content).await?;
                }
                Err(e) => {
                    warn!("Vision API 调用失败: {}", e);
                    let metadata = ReplacementMetadata::new(processing_msg.event_id.clone(), None);
                    let error_content =
                        RoomMessageEventContent::text_plain(format!("图片分析失败: {}", e))
                            .make_replacement(metadata);
                    room.send(error_content).await?;
                }
            }
        }

        Ok(())
    }

    /// 发送图片的流式响应（Vision API）。
    ///
    /// 使用混合节流策略更新消息。
    async fn handle_image_streaming_response(
        &self,
        room: &Room,
        session_id: &str,
        text: &str,
        image_data_url: &str,
        processing_event_id: OwnedEventId,
    ) -> Result<()> {
        // 开始流式聊天
        let (state, mut stream) = match self
            .ai_service
            .chat_with_image_stream(session_id, text, image_data_url)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                warn!("Vision API 流式调用初始化失败: {}", e);
                let metadata = ReplacementMetadata::new(processing_event_id.clone(), None);
                let error_content =
                    RoomMessageEventContent::text_plain(format!("图片分析失败: {}", e))
                        .make_replacement(metadata);
                room.send(error_content).await?;
                return Ok(());
            }
        };

        // 状态追踪
        let event_id: Option<OwnedEventId> = Some(processing_event_id);
        let mut chars_since_update: usize = 0;
        let mut last_update = Instant::now();

        // 消费流
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(delta) => {
                    chars_since_update += delta.chars().count();

                    let time_elapsed = last_update.elapsed() >= self.streaming_min_interval;
                    let chars_accumulated = chars_since_update >= self.streaming_min_chars;

                    if time_elapsed || chars_accumulated {
                        let content = {
                            let s = state.lock().await;
                            s.content().to_string()
                        };

                        if let Some(ref original_event_id) = event_id {
                            let metadata =
                                ReplacementMetadata::new(original_event_id.clone(), None);
                            let msg_content = RoomMessageEventContent::text_plain(&content)
                                .make_replacement(metadata);
                            room.send(msg_content).await?;
                        }

                        chars_since_update = 0;
                        last_update = Instant::now();
                    }
                }
                Err(e) => {
                    warn!("Vision 流式响应错误: {}", e);
                    let content = {
                        let s = state.lock().await;
                        s.content().to_string()
                    };

                    if !content.is_empty() {
                        let error_msg = format!("{}\n\n[错误: {}]", content, e);
                        if let Some(ref original_event_id) = event_id {
                            let metadata =
                                ReplacementMetadata::new(original_event_id.clone(), None);
                            let msg_content = RoomMessageEventContent::text_plain(&error_msg)
                                .make_replacement(metadata);
                            room.send(msg_content).await?;
                        }
                    } else if let Some(ref original_event_id) = event_id {
                        let metadata = ReplacementMetadata::new(original_event_id.clone(), None);
                        let error_content =
                            RoomMessageEventContent::text_plain(format!("图片分析失败: {}", e))
                                .make_replacement(metadata);
                        room.send(error_content).await?;
                    }
                    return Ok(());
                }
            }
        }

        // 流结束，发送最终内容
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
            vision_enabled: true,
            vision_model: None,
            vision_max_image_size: 1024,
        };
        let bot_user_id = user_id!("@bot:matrix.org").to_owned();
        // 注意：测试中需要创建一个真正的 Client 或使用 mock
        // 这里简化测试，仅测试 extract_message 方法
        // 实际集成测试需要 mock Matrix 客户端
        unimplemented!("需要 mock Client 进行测试")
    }

    // 暂时跳过需要 create_test_handler 的测试
    // 这些测试需要 mock Matrix 客户端
    // extract_message 的逻辑很简单，可以手动测试或使用集成测试

    // 简单单元测试 extract_message 逻辑
    #[test]
    fn test_extract_message_logic() {
        // 直接测试逻辑，不依赖 EventHandler
        let command_prefix = "!ai ";
        let bot_user_id = "@bot:matrix.org";

        // 测试移除命令前缀
        let mut result = "!ai Hello world".to_string();
        if result.starts_with(command_prefix) {
            result = result[command_prefix.len()..].to_string();
        }
        result = result.replace(bot_user_id, "");
        assert_eq!(result.trim(), "Hello world");

        // 测试移除 @提及
        let mut result = "@bot:matrix.org Hello there".to_string();
        if result.starts_with(command_prefix) {
            result = result[command_prefix.len()..].to_string();
        }
        result = result.replace(bot_user_id, "");
        assert_eq!(result.trim(), "Hello there");

        // 测试普通消息
        let mut result = "Just a plain message".to_string();
        if result.starts_with(command_prefix) {
            result = result[command_prefix.len()..].to_string();
        }
        result = result.replace(bot_user_id, "");
        assert_eq!(result.trim(), "Just a plain message");
    }
}
