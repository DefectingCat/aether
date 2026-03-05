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

/// 处理房间邀请（独立函数，不依赖 EventHandler 实例）
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
pub struct EventHandler<T: AiServiceTrait> {
    ai_service: T,
    client: Client,
    bot_user_id: OwnedUserId,
    command_prefix: String,
    command_gateway: CommandGateway,
    streaming_enabled: bool,
    streaming_min_interval: Duration,
    streaming_min_chars: usize,
    vision_enabled: bool,
    vision_max_image_size: u32,
}

impl<T: AiServiceTrait> EventHandler<T> {
    pub fn new(
        ai_service: T,
        bot_user_id: OwnedUserId,
        client: Client,
        config: &Config,
        persona_store: Option<PersonaStore>,
    ) -> Self {
        let mut command_gateway = CommandGateway::new(
            config.command_prefix.clone(),
            config.bot_owners.clone(),
        );

        command_gateway.register(Arc::new(BotInfoHandler));
        command_gateway.register(Arc::new(BotLeaveHandler));
        command_gateway.register(Arc::new(BotPingHandler));

        if let Some(store) = persona_store {
            command_gateway.register(Arc::new(PersonaHandler::new(store)));
        }

        Self {
            ai_service,
            client,
            bot_user_id,
            command_prefix: config.command_prefix.clone(),
            command_gateway,
            streaming_enabled: config.streaming_enabled,
            streaming_min_interval: Duration::from_millis(config.streaming_min_interval_ms),
            streaming_min_chars: config.streaming_min_chars,
            vision_enabled: config.vision_enabled,
            vision_max_image_size: config.vision_max_image_size,
        }
    }

    /// 处理消息
    pub async fn handle_message(
        &self,
        ev: matrix_sdk::ruma::events::room::message::SyncRoomMessageEvent,
        room: Room,
        client: Client,
    ) -> Result<()> {
        // 使用 as_original() 获取原始消息事件
        let original = match ev.as_original() {
            Some(o) => o,
            None => return Ok(()), // 忽略已删除的消息
        };

        // 跳过自己发送的消息
        if original.sender == self.bot_user_id {
            return Ok(());
        }

        // 获取消息文本
        let text = original.content.body();
        let event_id = original.event_id.clone();

        let room_id = room.room_id();

        // 判断是否应该响应
        let is_direct = room.is_direct().await.unwrap_or(false);

        // 检查是否通过 Intentional Mentions (MSC 3456) 被提及
        let mentions_bot = original
            .content
            .mentions
            .as_ref()
            .is_some_and(|m| m.user_ids.contains(&self.bot_user_id));

        // 检查是否是命令（以命令前缀开头）
        let is_command = self.command_gateway.is_command(text);

        info!("收到消息: '{}', 是否命令: {}", text, is_command);

        // 处理命令
        if is_command {
            info!("分发命令: {}", text);
            // 尝试通过命令网关分发
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

        // 非命令消息：检查是否应该响应 AI
        let should_respond = if is_direct {
            // 私聊：总是响应
            true
        } else {
            // 房间：检查文本中的 user_id（兼容旧客户端）或 mentions 字段（现代客户端）
            text.contains(&self.bot_user_id.to_string()) || mentions_bot
        };

        if !should_respond {
            return Ok(());
        }

        let session_id = if is_direct {
            original.sender.to_string()
        } else {
            room_id.to_string()
        };

        match &original.content.msgtype {
            MessageType::Text(text_msg) => {
                let clean_text = self.extract_message(&text_msg.body);
                debug!("处理消息 [{}]: {}", session_id, clean_text);

                if self.streaming_enabled {
                    self.handle_streaming_response(&room, &session_id, &clean_text)
                        .await?;
                } else {
                    self.handle_normal_response(&room, &session_id, &clean_text)
                        .await?;
                }
            }
            MessageType::Image(image_msg) if self.vision_enabled => {
                debug!("处理图片消息 [{}]", session_id);
                match self.handle_image_message(&room, &session_id, image_msg).await {
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
            _ => {}
        }

        Ok(())
    }

    async fn handle_image_message(
        &self,
        room: &Room,
        session_id: &str,
        image_msg: &matrix_sdk::ruma::events::room::message::ImageMessageEventContent,
    ) -> Result<()> {
        let mxc_uri = match &image_msg.source {
            matrix_sdk::ruma::events::room::MediaSource::Plain(uri) => uri,
            matrix_sdk::ruma::events::room::MediaSource::Encrypted(_) => {
                anyhow::bail!("不支持加密图片");
            }
        };

        let image_data_url = download_image_as_base64(
            &self.client,
            mxc_uri,
            None,
            self.vision_max_image_size,
        )
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
    ) -> Result<()> {
        match self.ai_service.chat(session_id, clean_text).await {
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

    /// 流式响应（打字机效果）
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

                    // 检查是否需要更新消息（混合策略）
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
                            // 编辑已有消息
                            let metadata =
                                ReplacementMetadata::new(original_event_id.clone(), None);
                            let msg_content = RoomMessageEventContent::text_plain(&content)
                                .make_replacement(metadata);
                            room.send(msg_content).await?;
                        } else {
                            // 发送新消息
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
                    // 如果已经发送了一些内容，显示错误
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
                        } else {
                            room.send(RoomMessageEventContent::text_plain(&error_msg))
                                .await?;
                        }
                    } else {
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

        // 流结束，发送最终消息（如果有残留内容）
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

    fn extract_message(&self, text: &str) -> String {
        let mut result = text.to_string();

        // 移除命令前缀
        if result.starts_with(&self.command_prefix) {
            result = result[self.command_prefix.len()..].to_string();
        }

        // 移除 @提及
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
            std::sync::Arc<tokio::sync::Mutex<crate::traits::StreamingState>>,
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
