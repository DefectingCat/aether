//! 流式响应处理。

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use futures_util::{Stream, StreamExt};
use matrix_sdk::Room;
use matrix_sdk::ruma::OwnedEventId;
use tokio::sync::Mutex;
use tracing::warn;

use crate::traits::{MessageSender, RoomSender, StreamingState};

#[derive(Clone)]
pub struct StreamingHandler {
    pub min_interval: Duration,
    pub min_chars: usize,
}

impl StreamingHandler {
    pub fn new(min_interval: Duration, min_chars: usize) -> Self {
        Self {
            min_interval,
            min_chars,
        }
    }

    pub async fn handle(
        &self,
        room: &Room,
        state: Arc<Mutex<StreamingState>>,
        stream: impl Stream<Item = Result<String>> + Send + Unpin,
    ) -> Result<()> {
        self.handle_with_sender(RoomSender(room.clone()), state, stream, None).await
    }

    pub async fn handle_with_initial_event(
        &self,
        room: &Room,
        state: Arc<Mutex<StreamingState>>,
        stream: impl Stream<Item = Result<String>> + Send + Unpin,
        initial_event_id: OwnedEventId,
    ) -> Result<()> {
        self.handle_with_sender(RoomSender(room.clone()), state, stream, Some(initial_event_id)).await
    }

    pub async fn handle_with_sender<S: MessageSender>(
        &self,
        sender: S,
        state: Arc<Mutex<StreamingState>>,
        mut stream: impl Stream<Item = Result<String>> + Send + Unpin,
        initial_event_id: Option<OwnedEventId>,
    ) -> Result<()> {
        let mut event_id = initial_event_id;
        let mut chars_since_update: usize = 0;
        let mut last_update = Instant::now();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(delta) => {
                    chars_since_update += delta.chars().count();

                    let time_elapsed = last_update.elapsed() >= self.min_interval;
                    let chars_accumulated = chars_since_update >= self.min_chars;

                    if time_elapsed || chars_accumulated {
                        let content = {
                            let s = state.lock().await;
                            s.content().to_string()
                        };

                        event_id = self.send_or_edit(&sender, &content, event_id).await?;

                        chars_since_update = 0;
                        last_update = Instant::now();
                    }
                }
                Err(e) => {
                    warn!("流式响应错误: {}", e);
                    self.handle_error(&sender, &state, event_id, &e.to_string()).await?;
                    return Ok(());
                }
            }
        }

        let final_content = {
            let s = state.lock().await;
            s.content().to_string()
        };

        if !final_content.is_empty() {
            self.send_final(&sender, &final_content, event_id).await?;
        }

        Ok(())
    }

    async fn send_or_edit<S: MessageSender>(
        &self,
        sender: &S,
        content: &str,
        event_id: Option<OwnedEventId>,
    ) -> Result<Option<OwnedEventId>> {
        if let Some(original_event_id) = event_id {
            sender.edit(original_event_id.clone(), content).await?;
            Ok(Some(original_event_id))
        } else {
            let new_event_id = sender.send(content).await?;
            Ok(Some(new_event_id))
        }
    }

    async fn handle_error<S: MessageSender>(
        &self,
        sender: &S,
        state: &Arc<Mutex<StreamingState>>,
        event_id: Option<OwnedEventId>,
        error_msg: &str,
    ) -> Result<()> {
        let content = {
            let s = state.lock().await;
            s.content().to_string()
        };

        if !content.is_empty() {
            let error_content = format!("{}\n\n[错误: {}]", content, error_msg);
            if let Some(original_event_id) = event_id {
                sender.edit(original_event_id, &error_content).await?;
            } else {
                sender.send(&error_content).await?;
            }
        } else {
            let error_text = format!("AI 服务暂时不可用: {}", error_msg);
            if let Some(original_event_id) = event_id {
                sender.edit(original_event_id, &error_text).await?;
            } else {
                sender.send(&error_text).await?;
            }
        }

        Ok(())
    }

    async fn send_final<S: MessageSender>(
        &self,
        sender: &S,
        content: &str,
        event_id: Option<OwnedEventId>,
    ) -> Result<()> {
        if let Some(original_event_id) = event_id {
            sender.edit(original_event_id, content).await?;
        } else if !content.is_empty() {
            sender.send(content).await?;
        }
        Ok(())
    }
}
