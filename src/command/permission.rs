//! 权限模型

use matrix_sdk::Room;
use matrix_sdk::ruma::OwnedUserId;

/// 权限级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    /// 任何房间成员
    Anyone,
    /// 房间管理员 (power_level >= 50)
    RoomMod,
    /// Bot 所有者
    BotOwner,
}

impl Permission {
    /// 检查用户是否具有此权限
    pub async fn check(
        &self,
        room: &Room,
        user_id: &OwnedUserId,
        bot_owners: &[String],
    ) -> bool {
        match self {
            Permission::Anyone => true,
            Permission::RoomMod => {
                // 私聊房间允许所有操作
                if room.is_direct().await.unwrap_or(false) {
                    return true;
                }
                // 检查用户是否是房间成员
                // get_member 返回 Option<RoomMember>，如果用户是成员则返回 Some
                room.get_member(user_id.as_ref()).await.ok().flatten().is_some()
            }
            Permission::BotOwner => {
                // 检查用户是否是 Bot 所有者
                bot_owners.iter().any(|owner| owner == user_id.as_str())
            }
        }
    }

    /// 获取权限级别的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Permission::Anyone => "任何人",
            Permission::RoomMod => "房间管理员",
            Permission::BotOwner => "Bot 所有者",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_ordering() {
        assert!(Permission::BotOwner > Permission::RoomMod);
        assert!(Permission::RoomMod > Permission::Anyone);
    }

    #[test]
    fn test_permission_display_name() {
        assert_eq!(Permission::Anyone.display_name(), "任何人");
        assert_eq!(Permission::RoomMod.display_name(), "房间管理员");
        assert_eq!(Permission::BotOwner.display_name(), "Bot 所有者");
    }
}