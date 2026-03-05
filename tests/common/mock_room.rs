use aether_matrix::traits::MessageSender;
use anyhow::Result;
use matrix_sdk::ruma::OwnedEventId;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct MockRoom {
    pub sent_messages: Arc<Mutex<Vec<(String, Option<OwnedEventId>)>>>,
    next_event_id: Arc<AtomicU64>,
}

impl MockRoom {
    pub fn new() -> Self {
        Self {
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            next_event_id: Arc::new(AtomicU64::new(1)),
        }
    }

    fn next_event_id(&self) -> OwnedEventId {
        let id = self.next_event_id.fetch_add(1, Ordering::SeqCst);
        let event_id = format!("$event_{}", id);
        OwnedEventId::try_from(event_id.as_str()).unwrap()
    }

    pub async fn get_messages(&self) -> Vec<(String, Option<OwnedEventId>)> {
        self.sent_messages.lock().await.clone()
    }
}

impl Default for MockRoom {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageSender for MockRoom {
    async fn send(&self, content: &str) -> Result<OwnedEventId> {
        let event_id = self.next_event_id();
        let mut messages = self.sent_messages.lock().await;
        messages.push((content.to_string(), Some(event_id.clone())));
        Ok(event_id)
    }

    async fn edit(&self, event_id: OwnedEventId, new_content: &str) -> Result<()> {
        let mut messages = self.sent_messages.lock().await;
        messages.push((new_content.to_string(), Some(event_id)));
        Ok(())
    }
}
