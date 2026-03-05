//! Matrix 消息事件处理。
//!
//! 提供房间邀请处理和消息响应功能。

mod extract;
mod invite;
mod streaming;

pub use invite::handle_invite;

use std::time::Duration;

use anyhow::Result;
use matrix_sdk::{
    Client, Room,
    ruma::{
        OwnedUserId,
        events::room::message::{MessageType, ReplacementMetadata, RoomMessageEventContent},
    },
};
use tracing::{debug, warn};

use crate::config::Config;
use crate::media::download_image_as_base64;
use crate::traits::AiServiceTrait;
use streaming::StreamingHandler;

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
    pub(super) ai_service: T,
    /// Matrix 客户端（用于下载媒体）
    pub(super) client: Client,
    /// 机器人的 Matrix 用户 ID
    pub(super) bot_user_id: OwnedUserId,
    /// 命令前缀（如 `!ai`）
    pub(super) command_prefix: String,
    /// 是否启用流式输出
    streaming_enabled: bool,
    /// 流式处理器
    streaming_handler: StreamingHandler,
    /// 是否启用图片理解功能
    vision_enabled: bool,
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
            streaming_handler: StreamingHandler::new(
                Duration::from_millis(config.streaming_min_interval_ms),
                config.streaming_min_chars,
            ),
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
                    self.handle_image_message(&room, &session_id, image_msg)
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
    /// 使用 StreamingHandler 处理流式输出。
    async fn handle_streaming_response(
        &self,
        room: &Room,
        session_id: &str,
        clean_text: &str,
    ) -> Result<()> {
        // 开始流式聊天
        let (state, stream) = match self.ai_service.chat_stream(session_id, clean_text).await {
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

        self.streaming_handler.handle(room, state, stream).await
    }

    /// 处理图片消息（Vision API）。
    ///
    /// 下载图片、转换为 base64，然后调用 Vision API。
    ///
    /// # Arguments
    ///
    /// * `room` - 消息所在的房间
    /// * `session_id` - 会话标识符
    /// * `image_msg` - 图片消息内容
    async fn handle_image_message(
        &self,
        room: &Room,
        session_id: &str,
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

        let image_data_url = match download_image_as_base64(
            &self.client,
            mxc_uri,
            Some(media_type),
            self.vision_max_image_size,
        )
        .await
        {
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
    /// 使用 StreamingHandler 处理流式输出。
    async fn handle_image_streaming_response(
        &self,
        room: &Room,
        session_id: &str,
        text: &str,
        image_data_url: &str,
        processing_event_id: matrix_sdk::ruma::OwnedEventId,
    ) -> Result<()> {
        // 开始流式聊天
        let (state, stream) = match self
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

        self.streaming_handler
            .handle_with_initial_event(room, state, stream, processing_event_id)
            .await
    }
}

#[cfg(test)]
mod tests {
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
