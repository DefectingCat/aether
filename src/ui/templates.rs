//! Element (Matrix) 兼容消息模板 · v3
//!
//! 核心策略：
//!   - `<h3>`        → 渲染为大号加粗标题（真正的视觉层级）
//!   - `<blockquote>`→ 渲染为带左边框 + 浅色背景的块（卡片效果）
//!   - `<table>`     → 对齐的数据网格
//!   - `<code>`      → 等宽 + 背景色（命令高亮）
//!   - `<font color>`→ 彩色文字
//!   - 零 style= 属性

mod color {
    pub const TITLE: &str = "#3164f0ff";
    pub const ACCENT: &str = "#6ea8ff";
    pub const KEY: &str = "#6a7fa8";
    pub const VALUE: &str = "#c8d4f0";
    pub const DIM: &str = "#4a5a78";
    pub const SUCCESS: &str = "#48bb78";
    pub const ERROR: &str = "#f06070";
    pub const WARNING: &str = "#f0c060";
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

    pub fn status(kind: Status, message: &str) -> String {
        let (icon, color) = match kind {
            Status::Success => ("✓", color::SUCCESS),
            Status::Error => ("✕", color::ERROR),
            Status::Warning => ("⚠", color::WARNING),
        };
        format!(
            "<blockquote>{}</blockquote>",
            bold(&fc(color, &format!("{icon}  {message}")))
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Success,
    Error,
    Warning,
}

pub fn info_card(title: &str, items: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    GlassTemplate::info_card(title, items)
}

pub fn help_menu(commands: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    GlassTemplate::help_menu(commands)
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

pub fn leaderboard(title: &str, headers: &[&str], rows: &[Vec<&str>]) -> String {
    let header_row: String = headers
        .iter()
        .map(|h| format!("<th>{}</th>", fc(color::KEY, h)))
        .collect();

    let data_rows: String = rows
        .iter()
        .map(|row| {
            let cells: String = row
                .iter()
                .map(|cell| format!("<td>{}</td>", bold(&fc(color::VALUE, cell))))
                .collect();
            format!("<tr>{cells}</tr>")
        })
        .collect();

    format!(
        "<h3>{} {}</h3><blockquote><table><thead><tr>{header_row}</tr></thead><tbody>{data_rows}</tbody></table></blockquote>",
        fc(color::ACCENT, "🏆"),
        bold(&fc(color::TITLE, title))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_style_attributes() {
        let cases = [
            info_card("T", &[("k", "v")]),
            help_menu(&[("!x", "d")]),
            success("ok"),
            error("fail"),
            warning("w"),
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
        for html in [success("ok"), error("e"), warning("w")] {
            assert!(html.contains("<blockquote>"));
            assert!(!html.contains("style="));
        }
    }
}
