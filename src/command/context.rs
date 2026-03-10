//! # 命令上下文
//!
//! 提供命令执行所需的上下文信息，包括 Matrix 客户端、房间、发送者等。

use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::{Client, Room};

/// 用于构建 [`CommandContext`] 的参数结构体。
///
/// 使用 builder 模式收集参数，避免 `CommandContext::new()` 参数过多。
///
/// # Example
///
/// ```ignore
/// let args = CommandContextArgs {
///     client: &client,
///     room: room,
///     sender: event.sender,
///     args: parsed.args,
///     bot_owners: &config.bot_owners,
/// };
/// let ctx = CommandContext::new(args);
/// ```
pub struct CommandContextArgs<'a> {
    /// Matrix 客户端实例，用于发送消息等操作
    pub client: &'a Client,
    /// 消息来源房间，用于回复消息
    pub room: Room,
    /// 命令发送者的用户 ID
    pub sender: OwnedUserId,
    /// 命令参数列表（第一个参数可作为子命令）
    pub args: Vec<&'a str>,
    /// Bot 所有者列表，用于权限判断
    pub bot_owners: &'a [String],
}

/// 命令执行上下文。
///
/// 包含命令处理器执行所需的所有信息，通过 `CommandContextArgs` 构建。
///
/// # Fields
///
/// - `client`: Matrix 客户端，用于执行 API 操作
/// - `room`: 消息来源房间，用于发送回复
/// - `sender`: 命令发送者，用于权限判断和个性化响应
/// - `args`: 命令参数，第一个参数可作为子命令
/// - `bot_owners`: Bot 所有者列表，用于管理员权限判断
pub struct CommandContext<'a> {
    /// Matrix 客户端实例，用于发送消息、获取用户信息等操作。
    pub client: &'a Client,
    /// 消息来源房间，用于发送回复消息。
    pub room: Room,
    /// 命令发送者的用户 ID，格式如 `@user:matrix.org`。
    pub sender: OwnedUserId,
    /// 命令参数列表。
    ///
    /// 第一个参数（`args[0]`）可作为子命令，剩余参数为子命令参数。
    /// 例如 `!bot name 新名称` 解析后 args 为 `["name", "新名称"]`。
    pub args: Vec<&'a str>,
    /// Bot 所有者列表，用于判断用户是否拥有管理员权限。
    ///
    /// 格式为 Matrix 用户 ID 列表，如 `["@user:matrix.org"]`。
    pub bot_owners: &'a [String],
}

impl<'a> CommandContext<'a> {
    /// 从参数创建命令上下文。
    ///
    /// # Arguments
    ///
    /// * `args` - 构建参数，包含 `client`, `room`, `sender`, `args`, `bot_owners` 字段
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ctx = CommandContext::new(CommandContextArgs {
    ///     client: &client,
    ///     room,
    ///     sender,
    ///     args: vec!["help"],
    ///     bot_owners: &config.bot_owners,
    /// });
    /// ```
    pub fn new(args: CommandContextArgs<'a>) -> Self {
        Self {
            client: args.client,
            room: args.room,
            sender: args.sender,
            args: args.args,
            bot_owners: args.bot_owners,
        }
    }

    /// 获取房间 ID。
    ///
    /// # Returns
    ///
    /// 返回消息来源房间的 ID 引用。
    pub fn room_id(&self) -> &matrix_sdk::ruma::RoomId {
        self.room.room_id()
    }

    /// 获取子命令（第一个参数）。
    ///
    /// 子命令用于实现多级命令结构，如 `!bot name` 中的 `name`。
    ///
    /// # Returns
    ///
    /// 如果存在参数，返回第一个参数作为子命令；否则返回 `None`。
    ///
    /// # Example
    ///
    /// ```ignore
    /// // 命令: !bot name 新名称
    /// // args: ["name", "新名称"]
    /// assert_eq!(ctx.sub_command(), Some("name"));
    /// ```
    pub fn sub_command(&self) -> Option<&'a str> {
        self.args.first().copied()
    }

    /// 获取子命令参数（除第一个参数外的所有参数）。
    ///
    /// 用于获取子命令需要的参数，如 `!bot name 新名称` 中的 `新名称`。
    ///
    /// # Returns
    ///
    /// 返回除第一个参数外的所有参数切片；如果只有一个或没有参数，返回空切片。
    ///
    /// # Example
    ///
    /// ```ignore
    /// // 命令: !bot name 新名称
    /// // args: ["name", "新名称"]
    /// assert_eq!(ctx.sub_args(), &["新名称"]);
    /// ```
    pub fn sub_args(&self) -> &[&'a str] {
        if self.args.len() > 1 {
            &self.args[1..]
        } else {
            &[]
        }
    }
}
