//! 房间邀请处理。

use anyhow::Result;
use matrix_sdk::ruma::events::room::member::{MembershipState, StrippedRoomMemberEvent};
use matrix_sdk::{Client, Room};
use tracing::{info, warn};

use crate::traits::{ClientWrapper, MatrixClient};

pub async fn handle_invite(ev: StrippedRoomMemberEvent, client: Client, room: Room) -> Result<()> {
    handle_invite_with_client(ev, ClientWrapper(client), room.room_id()).await
}

pub async fn handle_invite_with_client<C: MatrixClient>(
    ev: StrippedRoomMemberEvent,
    client: C,
    room_id: &matrix_sdk::ruma::RoomId,
) -> Result<()> {
    if ev.content.membership != MembershipState::Invite {
        return Ok(());
    }

    let user_id = &ev.state_key;
    let my_user_id = client.user_id().expect("user_id should be available");

    if *user_id != my_user_id {
        return Ok(());
    }

    info!("收到房间邀请: {}", room_id);

    match client.join_room_by_id(room_id).await {
        Ok(_) => info!("成功加入房间: {}", room_id),
        Err(e) => warn!("加入房间失败: {}", e),
    }

    Ok(())
}
