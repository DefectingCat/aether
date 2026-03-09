//! 赛博木鱼命令处理器

use anyhow::Result;
use async_trait::async_trait;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use std::sync::Arc;

use crate::command::{CommandContext, CommandHandler, Permission};
use crate::ui::{error, info_card, leaderboard, success};

use super::logic::MuyuLogic;
use super::models::{DropItem, HitResult, Rarity, Title};
use super::store::MuyuStore;

/// 敲木鱼命令处理器
pub struct MuyuHandler {
    logic: Arc<MuyuLogic>,
}

impl MuyuHandler {
    /// 创建新的处理器
    pub fn new(store: MuyuStore) -> Self {
        Self {
            logic: Arc::new(MuyuLogic::new(store)),
        }
    }
}

#[async_trait]
impl CommandHandler for MuyuHandler {
    fn name(&self) -> &str {
        "木鱼"
    }

    fn description(&self) -> &str {
        "敲一次木鱼，积累功德"
    }

    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let user_id = ctx.sender.to_string();
        let room_id = ctx.room_id().to_string();

        let result = self.logic.hit(&user_id, &room_id)?;

        if result.merit_gained == 0 {
            let html = warning("敲得太快了，请稍后再试~");
            return send_html(&ctx.room, &html).await;
        }

        let html = render_hit_result(&result);
        send_html(&ctx.room, &html).await
    }
}

/// 查看功德命令处理器
pub struct MeritHandler {
    store: MuyuStore,
}

impl MeritHandler {
    /// 创建新的处理器
    pub fn new(store: MuyuStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl CommandHandler for MeritHandler {
    fn name(&self) -> &str {
        "功德"
    }

    fn description(&self) -> &str {
        "查看当前功德值"
    }

    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let user_id = ctx.sender.to_string();
        let room_id = ctx.room_id().to_string();

        match self.store.get_merit(&user_id, &room_id)? {
            Some(record) => {
                // 获取已装备的称号
                let titles = self.store.get_user_titles(&user_id, &room_id)?;
                let equipped = titles.iter().find(|t| t.equipped);

                let mut items: Vec<(&str, &str)> = vec![
                    ("累计功德", Box::leak(record.merit_total.to_string().into_boxed_str())),
                    ("今日功德", Box::leak(record.merit_today.to_string().into_boxed_str())),
                    ("今日敲击", Box::leak(record.hits_today.to_string().into_boxed_str())),
                    ("最大连击", Box::leak(record.max_combo.to_string().into_boxed_str())),
                    ("会心一击", Box::leak(record.critical_count.to_string().into_boxed_str())),
                ];

                if let Some(t) = equipped {
                    let icon = t.title.icon.as_deref().unwrap_or("");
                    items.push(("当前称号", Box::leak(format!("{} {}", icon, t.title.name).into_boxed_str())));
                }

                let html = info_card("功德信息", &items);
                send_html(&ctx.room, &html).await
            }
            None => {
                let html = info("还没有敲过木鱼，使用 !木鱼 开始积累功德吧~");
                send_html(&ctx.room, &html).await
            }
        }
    }
}

/// 功德榜命令处理器
pub struct RankHandler {
    store: MuyuStore,
}

impl RankHandler {
    /// 创建新的处理器
    pub fn new(store: MuyuStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl CommandHandler for RankHandler {
    fn name(&self) -> &str {
        "功德榜"
    }

    fn description(&self) -> &str {
        "查看房间功德排行榜"
    }

    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let room_id = ctx.room_id().to_string();

        let rankings = self.store.get_leaderboard(&room_id, 10)?;

        if rankings.is_empty() {
            let html = info("暂无功德记录");
            return send_html(&ctx.room, &html).await;
        }

        let headers = ["排名", "用户", "功德"];
        let rows: Vec<Vec<String>> = rankings
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let title_str = if let Some(ref t) = entry.equipped_title {
                    let icon = t.icon.as_deref().unwrap_or("");
                    format!("{} {} ", icon, t.name)
                } else {
                    String::new()
                };

                // 简化用户 ID 显示
                let user_display = entry.user_id
                    .split(':')
                    .next()
                    .unwrap_or(&entry.user_id)
                    .trim_start_matches('@');

                vec![
                    (i + 1).to_string(),
                    format!("{}{}", title_str, user_display),
                    entry.merit_total.to_string(),
                ]
            })
            .collect();

        let html = leaderboard(
            "功德排行榜",
            &headers,
            &rows.iter().map(|r| r.iter().map(|s| s.as_str()).collect()).collect::<Vec<_>>(),
        );
        send_html(&ctx.room, &html).await
    }
}

/// 称号命令处理器
pub struct TitleHandler {
    store: MuyuStore,
}

impl TitleHandler {
    /// 创建新的处理器
    pub fn new(store: MuyuStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl CommandHandler for TitleHandler {
    fn name(&self) -> &str {
        "称号"
    }

    fn description(&self) -> &str {
        "查看或装备称号"
    }

    fn usage(&self) -> &str {
        "称号 [称号名称]"
    }

    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let user_id = ctx.sender.to_string();
        let room_id = ctx.room_id().to_string();
        let args = ctx.sub_args();

        if args.is_empty() {
            // 显示已解锁的称号
            self.handle_list(ctx, &user_id, &room_id).await
        } else {
            // 装备称号
            self.handle_equip(ctx, &user_id, &room_id, &args.join(" ")).await
        }
    }
}

impl TitleHandler {
    async fn handle_list(&self, ctx: &CommandContext<'_>, user_id: &str, room_id: &str) -> Result<()> {
        let titles = self.store.get_user_titles(user_id, room_id)?;

        if titles.is_empty() {
            let html = info("还没有解锁任何称号，继续敲木鱼吧~");
            return send_html(&ctx.room, &html).await;
        }

        let items: Vec<(&str, &str)> = titles
            .iter()
            .map(|t| {
                let icon = t.title.icon.as_deref().unwrap_or("");
                let equipped = if t.equipped { " ✓" } else { "" };
                let rarity = t.title.rarity.display_name();
                (
                    Box::leak(format!("{} {} [{}]", icon, t.title.name, rarity).into_boxed_str()) as &str,
                    if t.equipped { "已装备" } else { "" },
                )
            })
            .collect();

        let html = info_card("已解锁称号", &items);
        send_html(&ctx.room, &html).await
    }

    async fn handle_equip(&self, ctx: &CommandContext<'_>, user_id: &str, room_id: &str, name: &str) -> Result<()> {
        // 查找称号
        let titles = self.store.get_user_titles(user_id, room_id)?;
        let title = titles.iter().find(|t| t.title.name == name);

        match title {
            Some(t) => {
                if self.store.equip_title(user_id, room_id, t.title.id)? {
                    let icon = t.title.icon.as_deref().unwrap_or("");
                    let html = success(&format!("已装备称号: {} {}", icon, t.title.name));
                    send_html(&ctx.room, &html).await
                } else {
                    let html = error("装备失败");
                    send_html(&ctx.room, &html).await
                }
            }
            None => {
                let html = error(&format!("未找到称号: {}", name));
                send_html(&ctx.room, &html).await
            }
        }
    }
}

/// 背包命令处理器
pub struct BagHandler {
    store: MuyuStore,
}

impl BagHandler {
    /// 创建新的处理器
    pub fn new(store: MuyuStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl CommandHandler for BagHandler {
    fn name(&self) -> &str {
        "背包"
    }

    fn description(&self) -> &str {
        "查看掉落物品背包"
    }

    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let user_id = ctx.sender.to_string();
        let room_id = ctx.room_id().to_string();

        let drops = self.store.get_drops(&user_id, &room_id)?;

        if drops.is_empty() {
            let html = info("背包空空如也，继续敲木鱼收集物品吧~");
            return send_html(&ctx.room, &html).await;
        }

        // 统计物品数量
        let mut item_counts: std::collections::HashMap<String, (i32, String, Rarity)> = std::collections::HashMap::new();
        for drop in &drops {
            let entry = item_counts.entry(drop.item_name.clone()).or_insert((
                0,
                drop.item_icon.clone().unwrap_or_default(),
                drop.rarity,
            ));
            entry.0 += 1;
        }

        let mut items: Vec<(&str, &str)> = item_counts
            .iter()
            .map(|(name, (count, icon, rarity))| {
                (
                    Box::leak(format!("{} {} x{}", icon, name, count).into_boxed_str()) as &str,
                    rarity.display_name(),
                )
            })
            .collect();

        // 按稀有度排序
        items.sort_by(|a, b| {
            let rarity_order = |s: &str| match s {
                "传说" => 0,
                "史诗" => 1,
                "稀有" => 2,
                _ => 3,
            };
            rarity_order(a.1).cmp(&rarity_order(b.1))
        });

        let total = drops.len();
        let html = info_card(&format!("背包 (共 {} 件)", total), &items);
        send_html(&ctx.room, &html).await
    }
}

// ==================== 辅助函数 ====================

/// 发送 HTML 消息
async fn send_html(room: &matrix_sdk::Room, html: &str) -> Result<()> {
    let plain_text = html
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != ' ', "")
        .chars()
        .take(100)
        .collect::<String>();

    let content = RoomMessageEventContent::text_html(plain_text, html);
    room.send(content).await?;
    Ok(())
}

/// 警告消息
fn warning(msg: &str) -> String {
    format!("<blockquote><b>⚠ {}</b></blockquote>", msg)
}

/// 信息消息
fn info(msg: &str) -> String {
    format!("<blockquote><b>ℹ {}</b></blockquote>", msg)
}

/// 渲染敲木鱼结果
fn render_hit_result(result: &HitResult) -> String {
    let mut parts = Vec::new();

    // 敲击动画
    if result.is_critical {
        parts.push(r#"<b>🪘✨🌟✨ 会心一击！</b>"#.to_string());
    } else if result.new_combo >= 20 {
        parts.push(format!(r#"<b>🪘💥 COMBO x{}！</b>"#, result.new_combo));
    } else if result.new_combo >= 5 {
        parts.push(format!(r#"<b>🪘💥 连击 x{}</b>"#, result.new_combo));
    } else {
        parts.push(r#"<b>🪘 *knock*</b>"#.to_string());
    }

    // 功德获得
    let merit_text = if result.combo_multiplier > 1.0 {
        format!(
            r#"+{} 功德（连击加成 ×{:.1}）"#,
            result.merit_gained, result.combo_multiplier
        )
    } else {
        format!("+{} 功德", result.merit_gained)
    };

    parts.push(format!(
        r#"{}  |  累计：{} 功德"#,
        merit_text, result.merit_total
    ));

    // 掉落物品
    if let Some(ref item) = result.dropped_item {
        let rarity_color = item.rarity.color();
        let rarity_name = item.rarity.display_name();
        parts.push(format!(
            r#"<br><b>🎁 <font color="{}">[{}]</font> {} {} 收入背包！</b>"#,
            rarity_color, rarity_name, item.item_icon.as_deref().unwrap_or(""), item.item_name
        ));
    }

    // 新解锁称号
    for title in &result.unlocked_titles {
        let rarity_color = title.rarity.color();
        parts.push(format!(
            r#"<br><b>🏆 解锁称号：<font color="{}">{} {}</font></b>"#,
            rarity_color,
            title.icon.as_deref().unwrap_or(""),
            title.name
        ));
    }

    format!("<blockquote>{}</blockquote>", parts.join("<br>"))
}