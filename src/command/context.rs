//! 命令上下文

use matrix_sdk::{Client, Room};
use matrix_sdk::ruma::{OwnedEventId, OwnedUserId};

use super::gateway::CommandGateway;

/// 命令执行上下文
pub struct CommandContext<'a> {
    /// Matrix 客户端
    pub client: &'a Client,
    /// 房间
    pub room: Room,
    /// 发送者
    pub sender: OwnedUserId,
    /// 主命令
    pub cmd: &'a str,
    /// 参数列表（第一个参数可作为子命令）
    pub args: Vec<&'a str>,
    /// 原始消息
    pub raw_msg: &'a str,
    /// 事件 ID
    pub event_id: OwnedEventId,
    /// Bot 所有者列表
    pub bot_owners: &'a [String],
    /// 命令网关（可选，用于热更新）
    pub gateway: Option<&'a CommandGateway>,
}

impl<'a> CommandContext<'a> {
    /// 创建新的命令上下文
    pub fn new(
        client: &'a Client,
        room: Room,
        sender: OwnedUserId,
        cmd: &'a str,
        args: Vec<&'a str>,
        raw_msg: &'a str,
        event_id: OwnedEventId,
        bot_owners: &'a [String],
    ) -> Self {
        Self {
            client,
            room,
            sender,
            cmd,
            args,
            raw_msg,
            event_id,
            bot_owners,
            gateway: None,
        }
    }

    /// 创建带网关的命令上下文
    pub fn with_gateway(
        client: &'a Client,
        room: Room,
        sender: OwnedUserId,
        cmd: &'a str,
        args: Vec<&'a str>,
        raw_msg: &'a str,
        event_id: OwnedEventId,
        bot_owners: &'a [String],
        gateway: &'a CommandGateway,
    ) -> Self {
        Self {
            client,
            room,
            sender,
            cmd,
            args,
            raw_msg,
            event_id,
            bot_owners,
            gateway: Some(gateway),
        }
    }

    /// 获取房间 ID
    pub fn room_id(&self) -> &matrix_sdk::ruma::RoomId {
        self.room.room_id()
    }

    /// 获取第一个参数（可作为子命令）
    pub fn first_arg(&self) -> Option<&'a str> {
        self.args.first().copied()
    }

    /// 获取子命令（第一个参数）
    pub fn sub_command(&self) -> Option<&'a str> {
        self.args.first().copied()
    }

    /// 获取子命令后的参数
    pub fn sub_args(&self) -> &[&'a str] {
        if self.args.len() > 1 {
            &self.args[1..]
        } else {
            &[]
        }
    }

    /// 获取参数作为单个字符串（用空格连接）
    pub fn args_joined(&self) -> String {
        self.args.join(" ")
    }
}