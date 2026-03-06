//! 命令上下文

use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::{Client, Room};

/// 用于构建 CommandContext 的参数
pub struct CommandContextArgs<'a> {
    pub client: &'a Client,
    pub room: Room,
    pub sender: OwnedUserId,
    pub args: Vec<&'a str>,
    pub bot_owners: &'a [String],
}

/// 命令执行上下文
pub struct CommandContext<'a> {
    /// Matrix 客户端
    pub client: &'a Client,
    /// 房间
    pub room: Room,
    /// 发送者
    pub sender: OwnedUserId,
    /// 参数列表（第一个参数可作为子命令）
    pub args: Vec<&'a str>,
    /// Bot 所有者列表
    pub bot_owners: &'a [String],
}

impl<'a> CommandContext<'a> {
    pub fn new(args: CommandContextArgs<'a>) -> Self {
        Self {
            client: args.client,
            room: args.room,
            sender: args.sender,
            args: args.args,
            bot_owners: args.bot_owners,
        }
    }

    pub fn room_id(&self) -> &matrix_sdk::ruma::RoomId {
        self.room.room_id()
    }

    pub fn sub_command(&self) -> Option<&'a str> {
        self.args.first().copied()
    }

    pub fn sub_args(&self) -> &[&'a str] {
        if self.args.len() > 1 {
            &self.args[1..]
        } else {
            &[]
        }
    }
}
