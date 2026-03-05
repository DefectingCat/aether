use aether_matrix::traits::MatrixClient;
use anyhow::Result;
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct MockClient {
    user_id: Option<OwnedUserId>,
    joined_rooms: Arc<Mutex<Vec<OwnedRoomId>>>,
    join_should_fail: bool,
}

impl MockClient {
    pub fn new(user_id: Option<OwnedUserId>) -> Self {
        Self {
            user_id,
            joined_rooms: Arc::new(Mutex::new(Vec::new())),
            join_should_fail: false,
        }
    }

    pub fn with_join_failure(mut self) -> Self {
        self.join_should_fail = true;
        self
    }

    pub async fn get_joined_rooms(&self) -> Vec<OwnedRoomId> {
        self.joined_rooms.lock().await.clone()
    }
}

impl MatrixClient for MockClient {
    fn user_id(&self) -> Option<OwnedUserId> {
        self.user_id.clone()
    }

    async fn join_room_by_id(&self, room_id: &RoomId) -> Result<()> {
        if self.join_should_fail {
            anyhow::bail!("Failed to join room");
        }
        self.joined_rooms.lock().await.push(room_id.to_owned());
        Ok(())
    }
}