//! Element (Matrix) 兼容消息模板 · v3
//!
//! 核心策略：
//!   - <h3>        → 渲染为大号加粗标题（真正的视觉层级）
//!   - <blockquote>→ 渲染为带左边框 + 浅色背景的块（卡片效果）
//!   - <table>     → 对齐的数据网格
//!   - <code>      → 等宽 + 背景色（命令高亮）
//!   - <font color>→ 彩色文字
//!   - 零 style= 属性

mod color {
    pub const TITLE: &str = "#e8eeff";
    pub const ACCENT: &str = "#6ea8ff";
    pub const KEY: &str = "#6a7fa8";
    pub const VALUE: &str = "#c8d4f0";
    pub const DIM: &str = "#4a5a78";
    pub const CMD: &str = "#7ab4ff";
    pub const SUCCESS: &str = "#48bb78";
    pub const ERROR: &str = "#f06070";
    pub const WARNING: &str = "#f0c060";
    pub const GOLD: &str = "#f0c060";
}

fn fc(color: &str, s: &str) -> String {
    format!(r#"<font color="{color}">{s}</font>"#)
}
fn bold(s: &str) -> String {
    format!("<b>{s}</b>")
}
fn code(s: &str) -> String {
    format!("<code>{s}</code>")
}

pub struct GlassTemplate;

impl GlassTemplate {
    // ── 信息卡片 ──────────────────────────────────────────────────────────────
    //
    // Element 渲染：
    //   [大标题]  ◈ Bot 信息
    //   ┌──────────────────────────┐  ← blockquote 左边框
    //   │  字段        值
    //   │  用户 ID     @bot:…
    //   └──────────────────────────┘

    pub fn info_card(title: &str, items: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
        let rows: String = items
            .iter()
            .map(|(k, v)| {
                format!(
                    "<tr><td>{}</td><td>{}</td></tr>",
                    fc(color::KEY, k.as_ref()),
                    bold(&fc(color::VALUE, v.as_ref()))
                )
            })
            .collect();

        format!(
            "<h3>{} {}</h3><blockquote><table>{rows}</table></blockquote>",
            fc(color::ACCENT, "◈"),
            bold(&fc(color::TITLE, title))
        )
    }

    // ── 用户卡片 ──────────────────────────────────────────────────────────────

    pub fn user_card(title: &str, items: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
        let rows: String = items
            .iter()
            .map(|(k, v)| {
                format!(
                    "<tr><td>{}</td><td>{}</td></tr>",
                    fc(color::KEY, k.as_ref()),
                    bold(&fc(color::VALUE, v.as_ref()))
                )
            })
            .collect();

        format!(
            "<h3>{} {}</h3><blockquote><table>{rows}</table></blockquote>",
            fc(color::CMD, "◉"),
            bold(&fc(color::TITLE, title))
        )
    }

    // ── 命令帮助 ──────────────────────────────────────────────────────────────
    //
    // Element 渲染：
    //   [大标题]  ⌨ 命令帮助
    //   ┌──────────────────────────┐
    //   │  `!help`    显示帮助
    //   │  `!sign`    每日签到
    //   └──────────────────────────┘

    pub fn help_menu(commands: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
        let rows: String = commands
            .iter()
            .map(|(name, desc)| {
                format!(
                    "<tr><td>{}</td><td>{}</td></tr>",
                    code(name.as_ref()),
                    fc(color::DIM, desc.as_ref())
                )
            })
            .collect();

        format!(
            "<h3>{} {}</h3><blockquote><table>{rows}</table></blockquote>",
            fc(color::ACCENT, "⌨"),
            bold(&fc(color::TITLE, "命令帮助"))
        )
    }

    // ── 子命令列表 ────────────────────────────────────────────────────────────

    pub fn subcommand_list(title: &str, items: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
        let rows: String = items
            .iter()
            .map(|(name, desc)| {
                format!(
                    "<tr><td>{}</td><td>{}</td></tr>",
                    code(name.as_ref()),
                    fc(color::DIM, desc.as_ref())
                )
            })
            .collect();

        format!(
            "<h3>{} {}</h3><blockquote><table>{rows}</table></blockquote>",
            fc(color::ACCENT, "≡"),
            bold(&fc(color::TITLE, title))
        )
    }

    // ── 排行榜 ────────────────────────────────────────────────────────────────

    pub fn leaderboard(
        title: &str,
        headers: &[impl AsRef<str>],
        rows: &[Vec<impl AsRef<str>>],
    ) -> String {
        let medals = ["🥇", "🥈", "🥉"];

        let th: String = headers
            .iter()
            .map(|h| format!("<th>{}</th>", fc(color::KEY, h.as_ref())))
            .collect();

        let tr: String = rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                let cells: String = row
                    .iter()
                    .enumerate()
                    .map(|(ci, cell)| {
                        let content = if ci == 0 {
                            // 排名列：前三加奖牌
                            let medal = medals.get(i).copied().unwrap_or("");
                            if medal.is_empty() {
                                fc(color::KEY, cell.as_ref())
                            } else {
                                format!("{medal} {}", fc(color::VALUE, cell.as_ref()))
                            }
                        } else if i == 0 {
                            // 第一名数值加金色
                            bold(&fc(color::GOLD, cell.as_ref()))
                        } else {
                            fc(color::VALUE, cell.as_ref())
                        };
                        format!("<td>{content}</td>")
                    })
                    .collect();
                format!("<tr>{cells}</tr>")
            })
            .collect();

        format!(
            "<h3>{} {}</h3><blockquote><table><tr>{th}</tr>{tr}</table></blockquote>",
            fc(color::GOLD, "◆"),
            bold(&fc(color::TITLE, title))
        )
    }

    // ── 状态反馈 ──────────────────────────────────────────────────────────────
    //
    // 单行状态：blockquote 左边框颜色随状态变化（由 Element 主题决定，
    // 但 <font color> 的标题颜色会明确传达语义）

    pub fn status(kind: Status, message: &str) -> String {
        let (icon, color) = match kind {
            Status::Success => ("✓", color::SUCCESS),
            Status::Error => ("✕", color::ERROR),
            Status::Warning => ("⚠", color::WARNING),
            Status::Info => ("ℹ", color::ACCENT),
        };
        // 用 blockquote 包裹，左边框是 Element 默认样式提供的视觉分隔
        format!(
            "<blockquote>{}</blockquote>",
            bold(&fc(color, &format!("{icon}  {message}")))
        )
    }

    // ── 状态详情（带副标题）──────────────────────────────────────────────────

    pub fn status_detail(kind: Status, title: &str, detail: &str) -> String {
        let (icon, color) = match kind {
            Status::Success => ("✓", color::SUCCESS),
            Status::Error => ("✕", color::ERROR),
            Status::Warning => ("⚠", color::WARNING),
            Status::Info => ("ℹ", color::ACCENT),
        };
        format!(
            "<blockquote>{}<br>{}</blockquote>",
            bold(&fc(color, &format!("{icon} {title}"))),
            fc(color::KEY, detail)
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Success,
    Error,
    Warning,
    Info,
}

// ─── 快捷函数 ──────────────────────────────────────────────────────────────────

pub fn info_card(title: &str, items: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    GlassTemplate::info_card(title, items)
}
pub fn user_card(title: &str, items: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    GlassTemplate::user_card(title, items)
}
pub fn help_menu(commands: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    GlassTemplate::help_menu(commands)
}
pub fn subcommand_list(title: &str, items: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    GlassTemplate::subcommand_list(title, items)
}
pub fn leaderboard(
    title: &str,
    headers: &[impl AsRef<str>],
    rows: &[Vec<impl AsRef<str>>],
) -> String {
    GlassTemplate::leaderboard(title, headers, rows)
}
pub fn success(msg: &str) -> String {
    GlassTemplate::status(Status::Success, msg)
}
pub fn error(msg: &str) -> String {
    GlassTemplate::status(Status::Error, msg)
}
pub fn warning(msg: &str) -> String {
    GlassTemplate::status(Status::Warning, msg)
}
pub fn info(msg: &str) -> String {
    GlassTemplate::status(Status::Info, msg)
}
pub fn success_detail(title: &str, detail: &str) -> String {
    GlassTemplate::status_detail(Status::Success, title, detail)
}
pub fn error_detail(title: &str, detail: &str) -> String {
    GlassTemplate::status_detail(Status::Error, title, detail)
}

// ─── 测试 ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_style_attributes() {
        let cases = [
            info_card("T", &[("k", "v")]),
            help_menu(&[("!x", "d")]),
            leaderboard("T", &["A", "B"], &[vec!["1", "Alice"]]),
            success("ok"),
            error("fail"),
            warning("w"),
            info("i"),
        ];
        for html in &cases {
            assert!(!html.contains("style="), "style= found in:\n{html}");
        }
    }

    #[test]
    fn uses_structural_tags() {
        let card = info_card("Bot", &[("ID", "123")]);
        assert!(card.contains("<h3>"));
        assert!(card.contains("<blockquote>"));
        assert!(card.contains("<table>"));

        let menu = help_menu(&[("!ping", "测试")]);
        assert!(menu.contains("<code>!ping</code>"));
        assert!(menu.contains("<blockquote>"));
    }

    #[test]
    fn status_uses_blockquote() {
        for html in [success("ok"), error("e"), warning("w"), info("i")] {
            assert!(html.contains("<blockquote>"));
            assert!(!html.contains("style="));
        }
    }
}
