use anyhow::Result;
use aether_matrix::traits::StreamingState;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn create_test_stream_with_state(
    chunks: Vec<String>,
    state: Arc<Mutex<StreamingState>>,
) -> Pin<Box<dyn Stream<Item = Result<String>> + Send>> {
    use futures_util::stream;
    Box::pin(stream::iter(chunks).then(move |chunk| {
        let state = state.clone();
        async move {
            state.lock().await.append(&chunk);
            Ok(chunk)
        }
    }))
}

pub fn create_error_stream(error_msg: String) -> Pin<Box<dyn Stream<Item = Result<String>> + Send>> {
    use futures_util::stream;
    let msg = error_msg.clone();
    Box::pin(stream::iter(vec![
        Ok("Partial".to_string()),
        Err(anyhow::anyhow!(msg)),
    ]))
}