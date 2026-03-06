use anyhow::Result;
use futures_util::Stream;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::Mutex;

/// 流式响应的状态追踪。
#[derive(Default)]
pub struct StreamingState {
    pub accumulated: String,
}

impl StreamingState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, delta: &str) {
        self.accumulated.push_str(delta);
    }

    pub fn content(&self) -> &str {
        &self.accumulated
    }
}

/// 流式聊天的响应类型。
pub type ChatStreamResponse = (
    Arc<Mutex<StreamingState>>,
    Pin<Box<dyn Stream<Item = Result<String>> + Send>>,
);

/// AI 服务的 trait 抽象。
#[allow(dead_code)]
pub trait AiServiceTrait: Clone + Send + Sync + 'static {
    fn chat(&self, session_id: &str, prompt: &str) -> impl Future<Output = Result<String>> + Send;

    fn chat_with_system(
        &self,
        session_id: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> impl Future<Output = Result<String>> + Send;

    fn reset_conversation(&self, session_id: &str) -> impl Future<Output = ()> + Send;

    fn chat_stream(
        &self,
        session_id: &str,
        prompt: &str,
    ) -> impl Future<Output = Result<ChatStreamResponse>> + Send;

    fn chat_stream_with_system(
        &self,
        session_id: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> impl Future<Output = Result<ChatStreamResponse>> + Send;

    fn chat_with_image(
        &self,
        session_id: &str,
        text: &str,
        image_data_url: &str,
    ) -> impl Future<Output = Result<String>> + Send;

    fn chat_with_image_stream(
        &self,
        session_id: &str,
        text: &str,
        image_data_url: &str,
    ) -> impl Future<Output = Result<ChatStreamResponse>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_state_new_creates_empty() {
        let state = StreamingState::new();
        assert!(state.content().is_empty());
    }

    #[test]
    fn test_append_adds_text() {
        let mut state = StreamingState::new();
        state.append("Hello");
        assert_eq!(state.content(), "Hello");
    }

    #[test]
    fn test_content_returns_accumulated() {
        let mut state = StreamingState::new();
        state.accumulated = "Test content".to_string();
        assert_eq!(state.content(), "Test content");
    }

    #[test]
    fn test_multiple_appends() {
        let mut state = StreamingState::new();
        state.append("Hello");
        state.append(" ");
        state.append("World");
        assert_eq!(state.content(), "Hello World");
    }

    #[test]
    fn test_streaming_state_default_is_empty() {
        let state = StreamingState::default();
        assert!(state.content().is_empty());
    }

    #[tokio::test]
    async fn test_chat_stream_response_type_works() {
        let state = Arc::new(Mutex::new(StreamingState::new()));

        {
            let mut s = state.lock().await;
            s.append("Test");
        }

        assert_eq!(state.lock().await.content(), "Test");
    }
}
