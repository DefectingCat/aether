use anyhow::Result;
use futures_util::Stream;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::Mutex;

use crate::ai_service::StreamingState;

/// 流式聊天的响应类型
pub type ChatStreamResponse = (
    Arc<Mutex<StreamingState>>,
    Pin<Box<dyn Stream<Item = Result<String>> + Send>>,
);

/// AI 服务的 trait 抽象，用于支持 mock 测试
pub trait AiServiceTrait: Clone + Send + Sync + 'static {
    /// 普通聊天
    fn chat(&self, session_id: &str, prompt: &str) -> impl Future<Output = Result<String>> + Send;

    /// 重置会话
    fn reset_conversation(&self, session_id: &str) -> impl Future<Output = ()> + Send;

    /// 流式聊天
    /// 返回共享状态用于追踪累积内容，以及 Stream 用于消费
    fn chat_stream(
        &self,
        session_id: &str,
        prompt: &str,
    ) -> impl Future<Output = Result<ChatStreamResponse>> + Send;
}

/// 发送消息的结果
#[derive(Debug, Clone)]
pub struct SendMessageResult {
    /// 事件 ID，用于后续编辑
    pub event_id: matrix_sdk::ruma::OwnedEventId,
}

/// Room 客户端的 trait 抽象，用于支持 mock 测试
pub trait RoomClient: Clone + Send + Sync + 'static {
    /// 获取房间 ID
    fn room_id(&self) -> matrix_sdk::ruma::OwnedRoomId;

    /// 判断是否为私聊
    fn is_direct(&self) -> impl Future<Output = bool> + Send;

    /// 发送文本消息
    fn send_text(&self, content: &str) -> impl Future<Output = Result<SendMessageResult>> + Send;

    /// 编辑消息
    fn edit_message(
        &self,
        original_event_id: matrix_sdk::ruma::OwnedEventId,
        new_content: &str,
    ) -> impl Future<Output = Result<()>> + Send;
}
