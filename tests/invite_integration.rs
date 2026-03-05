//! 房间邀请处理集成测试

mod common;

use aether_matrix::event_handler::handle_invite_with_client;
use common::mock_client::MockClient;
use matrix_sdk::ruma::{
    events::room::member::StrippedRoomMemberEvent,
    owned_room_id, owned_user_id,
    serde::Raw,
};

fn create_member_event_json(user_id: &str, membership: &str) -> StrippedRoomMemberEvent {
    let json = format!(
        r#"{{
            "type": "m.room.member",
            "content": {{
                "membership": "{}"
            }},
            "state_key": "{}",
            "sender": "@sender:example.com"
        }}"#,
        membership, user_id
    );
    let raw: Raw<StrippedRoomMemberEvent> = Raw::from_json(
        serde_json::from_str(&json).unwrap()
    );
    raw.deserialize().unwrap()
}

#[tokio::test]
async fn test_handle_invite_joins_room() {
    let bot_user = owned_user_id!("@bot:example.com");
    let client = MockClient::new(Some(bot_user));
    let room_id = owned_room_id!("!test:example.com");

    let ev = create_member_event_json("@bot:example.com", "invite");

    let _: anyhow::Result<()> = handle_invite_with_client(ev, client.clone(), &room_id)
        .await;

    let joined = client.get_joined_rooms().await;
    assert_eq!(joined.len(), 1);
    assert_eq!(joined[0], room_id);
}

#[tokio::test]
async fn test_handle_invite_ignores_other_users() {
    let bot_user = owned_user_id!("@bot:example.com");
    let client = MockClient::new(Some(bot_user));
    let room_id = owned_room_id!("!test:example.com");

    let ev = create_member_event_json("@other:example.com", "invite");

    let _: anyhow::Result<()> = handle_invite_with_client(ev, client.clone(), &room_id)
        .await;

    let joined = client.get_joined_rooms().await;
    assert!(joined.is_empty());
}

#[tokio::test]
async fn test_handle_invite_ignores_non_invite_events() {
    let bot_user = owned_user_id!("@bot:example.com");
    let client = MockClient::new(Some(bot_user));
    let room_id = owned_room_id!("!test:example.com");

    let ev = create_member_event_json("@bot:example.com", "join");

    let _: anyhow::Result<()> = handle_invite_with_client(ev, client.clone(), &room_id)
        .await;

    let joined = client.get_joined_rooms().await;
    assert!(joined.is_empty());
}

#[tokio::test]
async fn test_handle_invite_join_failure_is_handled() {
    let bot_user = owned_user_id!("@bot:example.com");
    let client = MockClient::new(Some(bot_user)).with_join_failure();
    let room_id = owned_room_id!("!test:example.com");

    let ev = create_member_event_json("@bot:example.com", "invite");

    let result: anyhow::Result<()> = handle_invite_with_client(ev, client.clone(), &room_id).await;
    assert!(result.is_ok());
}