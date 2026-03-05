//! 流式响应处理集成测试

mod common;

use aether_matrix::event_handler::StreamingHandler;
use aether_matrix::traits::{MessageSender, StreamingState};
use common::mock_room::MockRoom;
use common::test_helpers::{create_error_stream, create_test_stream_with_state};
use matrix_sdk::ruma::OwnedEventId;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

fn make_event_id(id: &str) -> OwnedEventId {
    OwnedEventId::try_from(id).unwrap()
}

#[tokio::test]
async fn test_streaming_empty_stream() {
    let room = MockRoom::new();
    let handler = StreamingHandler::new(Duration::from_millis(100), 5);
    let state = Arc::new(Mutex::new(StreamingState::new()));
    let stream = create_test_stream_with_state(vec![], state.clone());

    let _: anyhow::Result<()> = handler
        .handle_with_sender(room.clone(), state, stream, None)
        .await;

    let messages = room.get_messages().await;
    assert!(messages.is_empty());
}

#[tokio::test]
async fn test_streaming_single_chunk() {
    let room = MockRoom::new();
    let handler = StreamingHandler::new(Duration::from_millis(100), 100);
    let state = Arc::new(Mutex::new(StreamingState::new()));
    let stream = create_test_stream_with_state(vec!["Hello".to_string()], state.clone());

    let _: anyhow::Result<()> = handler
        .handle_with_sender(room.clone(), state, stream, None)
        .await;

    let messages = room.get_messages().await;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].0, "Hello");
}

#[tokio::test]
async fn test_streaming_time_throttle_trigger() {
    let room = MockRoom::new();
    let handler = StreamingHandler::new(Duration::from_millis(10), 1000);
    let state = Arc::new(Mutex::new(StreamingState::new()));

    let chunks: Vec<String> = (0..5).map(|i| format!("Chunk{}", i)).collect();
    let stream = create_test_stream_with_state(chunks.clone(), state.clone());

    let _: anyhow::Result<()> = handler
        .handle_with_sender(room.clone(), state, stream, None)
        .await;

    let messages = room.get_messages().await;
    assert!(messages.len() >= 1, "Expected at least 1 message, got {}", messages.len());
    let final_content = messages.last().unwrap().0.clone();
    assert!(final_content.contains("Chunk"));
}

#[tokio::test]
async fn test_streaming_chars_throttle_trigger() {
    let room = MockRoom::new();
    let handler = StreamingHandler::new(Duration::from_secs(10), 3);
    let state = Arc::new(Mutex::new(StreamingState::new()));
    let stream = create_test_stream_with_state(
        vec!["abc".to_string(), "def".to_string()],
        state.clone(),
    );

    let _: anyhow::Result<()> = handler
        .handle_with_sender(room.clone(), state, stream, None)
        .await;

    let messages = room.get_messages().await;
    assert!(messages.len() >= 1);
    assert!(messages.last().unwrap().0.contains("abcdef"));
}

#[tokio::test]
async fn test_streaming_error_mid_stream() {
    let room = MockRoom::new();
    let handler = StreamingHandler::new(Duration::from_millis(100), 5);
    let state = Arc::new(Mutex::new(StreamingState::new()));
    
    state.lock().await.append("Partial");
    
    let stream = create_error_stream("API error".to_string());

    let _: anyhow::Result<()> = handler
        .handle_with_sender(room.clone(), state, stream, None)
        .await;

    let messages = room.get_messages().await;
    assert!(!messages.is_empty());
    let last_msg = messages.last().unwrap();
    assert!(last_msg.0.contains("错误") || last_msg.0.contains("不可用"));
}

#[tokio::test]
async fn test_streaming_with_initial_event_id() {
    let room = MockRoom::new();
    let handler = StreamingHandler::new(Duration::from_millis(100), 5);
    let state = Arc::new(Mutex::new(StreamingState::new()));
    let stream = create_test_stream_with_state(
        vec!["Updated content".to_string()],
        state.clone(),
    );
    let initial_event_id = make_event_id("$processing_msg");

    let _: anyhow::Result<()> = handler
        .handle_with_sender(room.clone(), state, stream, Some(initial_event_id.clone()))
        .await;

    let messages = room.get_messages().await;
    assert!(!messages.is_empty());
    assert_eq!(messages[0].1, Some(initial_event_id));
}

#[tokio::test]
async fn test_streaming_edit_after_send() {
    let room = MockRoom::new();
    let handler = StreamingHandler::new(Duration::from_millis(10), 3);
    let state = Arc::new(Mutex::new(StreamingState::new()));

    let stream = create_test_stream_with_state(
        vec!["Hel".to_string(), "lo ".to_string(), "Wor".to_string(), "ld".to_string()],
        state.clone(),
    );

    let _: anyhow::Result<()> = handler
        .handle_with_sender(room.clone(), state, stream, None)
        .await;

    let messages = room.get_messages().await;
    assert!(messages.len() >= 1);

    let final_content = messages.last().unwrap().0.clone();
    assert!(final_content.contains("Hello World") || final_content.contains("World"));
}

#[tokio::test]
async fn test_streaming_multiple_updates() {
    let room = MockRoom::new();
    let handler = StreamingHandler::new(Duration::from_millis(10), 2);
    let state = Arc::new(Mutex::new(StreamingState::new()));

    let stream = create_test_stream_with_state(
        vec!["AB".to_string(), "CD".to_string(), "EF".to_string(), "GH".to_string()],
        state.clone(),
    );

    let _: anyhow::Result<()> = handler
        .handle_with_sender(room.clone(), state, stream, None)
        .await;

    let messages = room.get_messages().await;
    assert!(messages.len() >= 2);
}