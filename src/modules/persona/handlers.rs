//! Persona 命令处理器

use anyhow::Result;
use async_trait::async_trait;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

use crate::command::{CommandContext, CommandHandler, Permission};
use crate::store::PersonaStore;
use crate::ui::{info_card, success, error, warning};

/// Persona 命令处理器
pub struct PersonaHandler {
    store: PersonaStore,
}

impl PersonaHandler {
    /// 创建新的 Persona 命令处理器
    pub fn new(store: PersonaStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl CommandHandler for PersonaHandler {
    fn name(&self) -> &str {
        "persona"
    }

    fn description(&self) -> &str {
        "人设管理命令"
    }

    fn usage(&self) -> &str {
        "persona <set|list|off|info>"
    }

    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let sub = ctx.sub_command();

        match sub {
            Some("set") => self.handle_set(ctx).await,
            Some("list") => self.handle_list(ctx).await,
            Some("off") => self.handle_off(ctx).await,
            Some("info") => self.handle_info(ctx).await,
            _ => self.handle_help(ctx).await,
        }
    }
}

impl PersonaHandler {
    async fn handle_help(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let items = vec![
            ("!persona list", "列出所有人设"),
            ("!persona set <id>", "设置当前房间人设（需要管理员权限）"),
            ("!persona off", "关闭当前房间人设（需要管理员权限）"),
            ("!persona info <id>", "查看人设详情"),
        ];
        let html = info_card("Persona 命令", &items);
        send_html(&ctx.room, &html).await
    }

    async fn handle_list(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let personas = self.store.get_all()?;

        if personas.is_empty() {
            let html = warning("暂无人设可用");
            return send_html(&ctx.room, &html).await;
        }

        let items: Vec<(&str, &str)> = personas
            .iter()
            .map(|p| {
                let emoji = p.avatar_emoji.as_deref().unwrap_or("");
                let builtin = if p.is_builtin { " [内置]" } else { "" };
                (p.id.as_str(), Box::leak(format!("{} {}{}", emoji, p.name, builtin).into_boxed_str()) as &str)
            })
            .collect();

        let html = info_card("可用人设", &items);
        send_html(&ctx.room, &html).await
    }

    async fn handle_set(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // 检查权限 - 需要 RoomMod
        if !Permission::RoomMod.check(&ctx.room, &ctx.sender, ctx.bot_owners).await {
            let html = error("权限不足: 需要房间管理员权限");
            return send_html(&ctx.room, &html).await;
        }

        let persona_id: String = ctx.sub_args().join(" ");
        if persona_id.is_empty() {
            let html = error("请提供人设 ID: !persona set <id>");
            return send_html(&ctx.room, &html).await;
        }

        // 检查人设是否存在
        match self.store.get_by_id(&persona_id)? {
            Some(persona) => {
                let room_id = ctx.room_id().to_string();
                let set_by = ctx.sender.to_string();

                self.store.set_room_persona(&room_id, &persona_id, &set_by)?;

                let emoji = persona.avatar_emoji.as_deref().unwrap_or("");
                let html = success(&format!("已设置人设: {} {}", emoji, persona.name));
                send_html(&ctx.room, &html).await
            }
            None => {
                let html = error(&format!("人设不存在: {}", persona_id));
                send_html(&ctx.room, &html).await
            }
        }
    }

    async fn handle_off(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // 检查权限 - 需要 RoomMod
        if !Permission::RoomMod.check(&ctx.room, &ctx.sender, ctx.bot_owners).await {
            let html = error("权限不足: 需要房间管理员权限");
            return send_html(&ctx.room, &html).await;
        }

        let room_id = ctx.room_id().to_string();
        self.store.disable_room_persona(&room_id)?;

        let html = success("已关闭当前房间的人设");
        send_html(&ctx.room, &html).await
    }

    async fn handle_info(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let persona_id: String = ctx.sub_args().join(" ");
        if persona_id.is_empty() {
            let html = error("请提供人设 ID: !persona info <id>");
            return send_html(&ctx.room, &html).await;
        }

        match self.store.get_by_id(&persona_id)? {
            Some(persona) => {
                let emoji = persona.avatar_emoji.as_deref().unwrap_or("");
                let builtin = if persona.is_builtin { " [内置]" } else { "" };
                let prompt_preview = if persona.system_prompt.len() > 200 {
                    format!("{}...", &persona.system_prompt[..200])
                } else {
                    persona.system_prompt.clone()
                };

                let items = vec![
                    ("ID", persona.id.as_str()),
                    ("名称", Box::leak(format!("{} {}{}", emoji, persona.name, builtin).into_boxed_str()) as &str),
                    ("提示词预览", Box::leak(prompt_preview.into_boxed_str()) as &str),
                ];

                let html = info_card("人设详情", &items);
                send_html(&ctx.room, &html).await
            }
            None => {
                let html = error(&format!("人设不存在: {}", persona_id));
                send_html(&ctx.room, &html).await
            }
        }
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