//! Admin 命令处理器

use anyhow::Result;
use async_trait::async_trait;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

use crate::command::{CommandContext, CommandHandler, Permission};
use crate::ui::{error, info_card, success, warning};

/// Bot 管理命令处理器。
///
/// 提供多个子命令用于 Bot 管理和查询：
/// - `info` - 查看 Bot 基本信息（任何人）
/// - `ping` - 测试响应延迟（任何人）
/// - `name` - 修改显示名称（Bot 所有者）
/// - `avatar` - 修改头像（Bot 所有者）
/// - `join` - 加入指定房间（Bot 所有者）
/// - `rooms` - 列出已加入房间（Bot 所有者）
/// - `prefix` - 修改命令前缀（已移除）
///
/// # 权限
///
/// 基础权限为 `Anyone`，但部分子命令有独立的权限检查：
/// - `info`, `ping` - 任何房间成员
/// - `name`, `avatar`, `join`, `rooms` - 仅 Bot 所有者
///
/// # Example
///
/// ```ignore
/// use aether_matrix::command::{CommandHandler, CommandContext, Permission};
/// use async_trait::async_trait;
///
/// /// Bot 管理命令处理器。
/// pub struct BotInfoHandler;
///
/// #[async_trait]
/// impl CommandHandler for BotInfoHandler {
///     fn name(&self) -> &str {
///         "bot"
///     }
///
///     fn description(&self) -> &str {
///         "Bot 管理命令"
///     }
///
///     fn usage(&self) -> &str {
///         "bot <info|name|ping>"
///     }
///
///     fn permission(&self) -> Permission {
///         Permission::Anyone
///     }
///
///     async fn execute(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
///         // 根据子命令分发到对应处理方法
///         Ok(())
///     }
/// }
/// ```
pub struct BotInfoHandler;

#[async_trait]
impl CommandHandler for BotInfoHandler {
    /// 命令名称（不含前缀）。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotInfoHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// let handler = BotInfoHandler;
    /// assert_eq!(handler.name(), "bot");
    /// ```
    fn name(&self) -> &str {
        "bot"
    }

    /// 命令描述，用于帮助信息。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotInfoHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// let handler = BotInfoHandler;
    /// assert!(!handler.description().is_empty());
    /// ```
    fn description(&self) -> &str {
        "Bot 管理命令"
    }

    /// 使用说明，说明命令的参数和用法。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotInfoHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// let handler = BotInfoHandler;
    /// assert!(handler.usage().contains("info"));
    /// ```
    fn usage(&self) -> &str {
        "bot <info|name|ping>"
    }

    /// 所需权限级别。
    ///
    /// 基础权限为 `Anyone`，部分子命令有独立的权限检查。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotInfoHandler;
    /// use aether_matrix::command::{CommandHandler, Permission};
    ///
    /// let handler = BotInfoHandler;
    /// assert_eq!(handler.permission(), Permission::Anyone);
    /// ```
    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    /// 执行命令。
    ///
    /// 根据子命令分发到对应的处理方法。
    ///
    /// # Arguments
    ///
    /// * `ctx` - 命令执行上下文，包含客户端、房间、发送者等信息
    ///
    /// # Returns
    ///
    /// 成功时返回 `Ok(())`，失败时返回错误。
    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let sub = ctx.sub_command();

        match sub {
            Some("info") => self.handle_info(ctx).await,
            Some("name") => self.handle_name(ctx).await,
            Some("ping") => self.handle_ping(ctx).await,
            Some("rooms") => self.handle_rooms(ctx).await,
            Some("join") => self.handle_join(ctx).await,
            Some("prefix") => self.handle_prefix(ctx).await,
            Some("avatar") => self.handle_avatar(ctx).await,
            _ => self.handle_help(ctx).await,
        }
    }
}

impl BotInfoHandler {
    async fn handle_help(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let items = vec![
            ("!bot info", "查看 Bot 基本信息"),
            (
                "!bot name <名称>",
                "修改 Bot 显示名称（需要 Bot 所有者权限）",
            ),
            ("!bot avatar <url>", "修改 Bot 头像（需要 Bot 所有者权限）"),
            ("!bot join <房间ID>", "加入指定房间（需要 Bot 所有者权限）"),
            ("!bot rooms", "列出已加入的所有房间（需要 Bot 所有者权限）"),
            ("!bot prefix <前缀>", "修改命令前缀（需要 Bot 所有者权限）"),
            ("!bot ping", "测试响应延迟"),
            ("!bot leave", "离开当前房间（需要管理员权限）"),
        ];
        let html = info_card("Bot 命令", &items);
        send_html(&ctx.room, &html).await
    }

    async fn handle_info(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let user_id = ctx
            .client
            .user_id()
            .map(|u| u.to_string())
            .unwrap_or_else(|| "未知".to_string());

        let device_id = ctx
            .client
            .device_id()
            .map(|d| d.to_string())
            .unwrap_or_else(|| "未知".to_string());

        let rooms_count = ctx.client.joined_rooms().len();
        let rooms_str = format!("{} 个", rooms_count);

        let items = vec![
            ("用户 ID", user_id.as_str()),
            ("设备 ID", device_id.as_str()),
            ("已加入房间", rooms_str.as_str()),
            ("运行状态", "正常运行中"),
        ];

        let html = info_card("Bot 信息", &items);
        send_html(&ctx.room, &html).await
    }

    async fn handle_name(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // 检查权限 - 需要 BotOwner
        if !Permission::BotOwner
            .check(&ctx.room, &ctx.sender, ctx.bot_owners)
            .await
        {
            let html = error("权限不足: 需要 Bot 所有者权限");
            return send_html(&ctx.room, &html).await;
        }

        // 获取新名称参数（子命令后的参数）
        let new_name: String = ctx.sub_args().join(" ");
        if new_name.is_empty() {
            let html = error("请提供新名称: !bot name <名称>");
            return send_html(&ctx.room, &html).await;
        }

        // 调用 Matrix API 设置显示名称
        let account = ctx.client.account();
        match account.set_display_name(Some(&new_name)).await {
            Ok(()) => {
                let html = success(&format!("显示名称已修改为: {}", new_name));
                send_html(&ctx.room, &html).await
            }
            Err(e) => {
                let html = error(&format!("修改显示名称失败: {}", e));
                send_html(&ctx.room, &html).await
            }
        }
    }

    async fn handle_ping(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let html = success("Pong! 机器人响应正常");
        send_html(&ctx.room, &html).await
    }

    async fn handle_rooms(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // 检查权限 - 需要 BotOwner
        if !Permission::BotOwner
            .check(&ctx.room, &ctx.sender, ctx.bot_owners)
            .await
        {
            let html = error("权限不足: 需要 Bot 所有者权限");
            return send_html(&ctx.room, &html).await;
        }

        let joined_rooms = ctx.client.joined_rooms();
        let rooms: Vec<_> = joined_rooms.iter().collect();
        let mut rooms_info: Vec<(String, String)> = Vec::new();

        for room in rooms.iter() {
            let room_id = room.room_id().to_string();
            let name = room
                .display_name()
                .await
                .map(|n| n.to_string())
                .unwrap_or_else(|_| room_id.clone());
            rooms_info.push((room_id, name));
        }

        // 转换为 (&str, &str) 格式
        let rooms_info: Vec<(&str, &str)> = rooms_info
            .iter()
            .map(|(id, name)| (id.as_str(), name.as_str()))
            .collect();

        let html = info_card("已加入房间", &rooms_info);
        send_html(&ctx.room, &html).await
    }

    async fn handle_join(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // 检查权限 - 需要 BotOwner
        if !Permission::BotOwner
            .check(&ctx.room, &ctx.sender, ctx.bot_owners)
            .await
        {
            let html = error("权限不足: 需要 Bot 所有者权限");
            return send_html(&ctx.room, &html).await;
        }

        let room_id: String = ctx.sub_args().join(" ");
        if room_id.is_empty() {
            let html = error("请提供房间 ID: !bot join <房间ID>");
            return send_html(&ctx.room, &html).await;
        }

        // 解析房间 ID
        use matrix_sdk::ruma::OwnedRoomOrAliasId;
        match room_id.parse::<OwnedRoomOrAliasId>() {
            Ok(room_id) => {
                // 加入房间
                match ctx.client.join_room_by_id_or_alias(&room_id, &[]).await {
                    Ok(_) => {
                        let html = success(&format!("已成功加入房间: {}", room_id));
                        send_html(&ctx.room, &html).await
                    }
                    Err(e) => {
                        let html = error(&format!("加入房间失败: {}", e));
                        send_html(&ctx.room, &html).await
                    }
                }
            }
            Err(e) => {
                let html = error(&format!("无效的房间 ID: {}", e));
                send_html(&ctx.room, &html).await
            }
        }
    }

    async fn handle_prefix(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // 检查权限 - 需要 BotOwner
        if !Permission::BotOwner
            .check(&ctx.room, &ctx.sender, ctx.bot_owners)
            .await
        {
            let html = error("权限不足: 需要 Bot 所有者权限");
            return send_html(&ctx.room, &html).await;
        }

        let html = error("命令前缀热更新功能已移除");
        send_html(&ctx.room, &html).await
    }

    async fn handle_avatar(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // 检查权限 - 需要 BotOwner
        if !Permission::BotOwner
            .check(&ctx.room, &ctx.sender, ctx.bot_owners)
            .await
        {
            let html = error("权限不足: 需要 Bot 所有者权限");
            return send_html(&ctx.room, &html).await;
        }

        let avatar_url: String = ctx.sub_args().join(" ");
        if avatar_url.is_empty() {
            let html = error("请提供头像 URL: !bot avatar <url>");
            return send_html(&ctx.room, &html).await;
        }

        // 下载图片
        let http_client = reqwest::Client::new();
        let response = match http_client.get(&avatar_url).send().await {
            Ok(r) => r,
            Err(e) => {
                let html = error(&format!("下载头像失败: {}", e));
                return send_html(&ctx.room, &html).await;
            }
        };

        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                let html = error(&format!("读取头像数据失败: {}", e));
                return send_html(&ctx.room, &html).await;
            }
        };

        // 检测 MIME 类型
        let content_type = match bytes.get(0..4) {
            Some([0x89, 0x50, 0x4E, 0x47]) => "image/png",
            Some([0xFF, 0xD8, 0xFF, ..]) => "image/jpeg",
            Some([0x47, 0x49, 0x46, ..]) => "image/gif",
            Some([0x52, 0x49, 0x46, 0x46]) => "image/webp",
            _ => {
                let html = error("不支持的图片格式 (支持: PNG, JPEG, GIF, WebP)");
                return send_html(&ctx.room, &html).await;
            }
        };

        // 上传到 Matrix media server
        let media = ctx.client.media();
        let mime_type: mime::Mime = content_type.parse()?;
        match media.upload(&mime_type, bytes.to_vec(), None).await {
            Ok(response) => {
                // 设置头像 URL
                let account = ctx.client.account();
                // 从响应中获取 MXC URI
                let mxc_uri = response.content_uri;
                match account.set_avatar_url(Some(&mxc_uri)).await {
                    Ok(()) => {
                        let html = success(&format!("头像已更新: {}", avatar_url));
                        send_html(&ctx.room, &html).await
                    }
                    Err(e) => {
                        let html = error(&format!("设置头像失败: {}", e));
                        send_html(&ctx.room, &html).await
                    }
                }
            }
            Err(e) => {
                let html = error(&format!("上传头像失败: {}", e));
                send_html(&ctx.room, &html).await
            }
        }
    }
}

/// Bot 离开房间命令处理器。
///
/// 让 Bot 离开当前房间。执行时会发送告别消息，然后离开房间。
///
/// # 权限
///
/// 需要房间管理员权限（`RoomMod`）。私聊房间默认拥有此权限。
///
/// # Example
///
/// ```ignore
/// use aether_matrix::command::{CommandHandler, CommandContext, Permission};
/// use async_trait::async_trait;
///
/// /// Bot 离开房间命令处理器。
/// pub struct BotLeaveHandler;
///
/// #[async_trait]
/// impl CommandHandler for BotLeaveHandler {
///     fn name(&self) -> &str {
///         "leave"
///     }
///
///     fn description(&self) -> &str {
///         "让 Bot 离开当前房间"
///     }
///
///     fn usage(&self) -> &str {
///         "leave"
///     }
///
///     fn permission(&self) -> Permission {
///         Permission::RoomMod
///     }
///
///     async fn execute(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
///         // 发送告别消息并离开房间
///         Ok(())
///     }
/// }
/// ```
pub struct BotLeaveHandler;

#[async_trait]
impl CommandHandler for BotLeaveHandler {
    /// 命令名称（不含前缀）。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotLeaveHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// let handler = BotLeaveHandler;
    /// assert_eq!(handler.name(), "leave");
    /// ```
    fn name(&self) -> &str {
        "leave"
    }

    /// 命令描述，用于帮助信息。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotLeaveHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// let handler = BotLeaveHandler;
    /// assert!(!handler.description().is_empty());
    /// ```
    fn description(&self) -> &str {
        "让 Bot 离开当前房间"
    }

    /// 使用说明，说明命令的参数和用法。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotLeaveHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// let handler = BotLeaveHandler;
    /// assert_eq!(handler.usage(), "leave");
    /// ```
    fn usage(&self) -> &str {
        "leave"
    }

    /// 所需权限级别。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotLeaveHandler;
    /// use aether_matrix::command::{CommandHandler, Permission};
    ///
    /// let handler = BotLeaveHandler;
    /// assert_eq!(handler.permission(), Permission::RoomMod);
    /// ```
    fn permission(&self) -> Permission {
        Permission::RoomMod
    }

    /// 执行命令。
    ///
    /// 发送告别消息后离开当前房间。
    ///
    /// # Arguments
    ///
    /// * `ctx` - 命令执行上下文，包含客户端、房间、发送者等信息
    ///
    /// # Returns
    ///
    /// 成功时返回 `Ok(())`，失败时返回错误。
    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let room_id = ctx.room_id();

        // 发送告别消息
        let html = warning(&format!("再见！正在离开房间 {} ...", room_id));
        send_html(&ctx.room, &html).await?;

        // 离开房间
        ctx.room.leave().await?;

        Ok(())
    }
}

/// Bot Ping 命令处理器。
///
/// 测试 Bot 响应延迟，返回 pong 消息。用于验证 Bot 是否正常工作。
///
/// # 权限
///
/// 任何房间成员都可以执行此命令。
///
/// # Example
///
/// ```ignore
/// use aether_matrix::command::{CommandHandler, CommandContext, Permission};
/// use async_trait::async_trait;
///
/// /// Bot Ping 命令处理器。
/// pub struct BotPingHandler;
///
/// #[async_trait]
/// impl CommandHandler for BotPingHandler {
///     fn name(&self) -> &str {
///         "ping"
///     }
///
///     fn description(&self) -> &str {
///         "测试 Bot 响应"
///     }
///
///     fn usage(&self) -> &str {
///         "ping"
///     }
///
///     fn permission(&self) -> Permission {
///         Permission::Anyone
///     }
///
///     async fn execute(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
///         // 返回 pong 消息
///         Ok(())
///     }
/// }
/// ```
pub struct BotPingHandler;

#[async_trait]
impl CommandHandler for BotPingHandler {
    /// 命令名称（不含前缀）。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotPingHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// let handler = BotPingHandler;
    /// assert_eq!(handler.name(), "ping");
    /// ```
    fn name(&self) -> &str {
        "ping"
    }

    /// 命令描述，用于帮助信息。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotPingHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// let handler = BotPingHandler;
    /// assert!(!handler.description().is_empty());
    /// ```
    fn description(&self) -> &str {
        "测试 Bot 响应"
    }

    /// 使用说明，说明命令的参数和用法。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotPingHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// let handler = BotPingHandler;
    /// assert_eq!(handler.usage(), "ping");
    /// ```
    fn usage(&self) -> &str {
        "ping"
    }

    /// 所需权限级别。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::modules::admin::BotPingHandler;
    /// use aether_matrix::command::{CommandHandler, Permission};
    ///
    /// let handler = BotPingHandler;
    /// assert_eq!(handler.permission(), Permission::Anyone);
    /// ```
    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    /// 执行命令。
    ///
    /// 返回 pong 消息确认 Bot 正常响应。
    ///
    /// # Arguments
    ///
    /// * `ctx` - 命令执行上下文，包含客户端、房间、发送者等信息
    ///
    /// # Returns
    ///
    /// 成功时返回 `Ok(())`，失败时返回错误。
    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let html = success("Pong! 机器人响应正常");
        send_html(&ctx.room, &html).await
    }
}

/// 发送 HTML 消息
async fn send_html(room: &matrix_sdk::Room, html: &str) -> Result<()> {
    // 提取纯文本作为 fallback
    let plain_text = html
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != ' ', "")
        .chars()
        .take(100)
        .collect::<String>();

    let content = RoomMessageEventContent::text_html(plain_text, html);
    room.send(content).await?;
    Ok(())
}
