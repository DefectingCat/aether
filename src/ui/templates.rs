//! Element (Matrix) 兼容消息模板 · v3
//!
//! 本模块提供 Matrix 消息的 HTML 模板生成，专为 Element 客户端优化。
//! 使用语义化标签而非内联样式，确保跨客户端兼容性。
//!
//! ## 功能特性
//!
//! - **卡片式布局**: 使用 `<blockquote>` 实现左边框 + 浅色背景效果
//! - **层级标题**: `<h3>` 渲染为大号加粗标题
//! - **数据表格**: `<table>` 展示对齐的键值对
//! - **代码高亮**: `<code>` 等宽字体 + 背景色
//! - **彩色文本**: `<font color>` 实现状态色（成功/错误/警告）
//! - **零 style 属性**: 避免客户端过滤，确保最大兼容性
//!
//! ## 核心设计原则
//!
//! 1. **语义优先**: 使用结构化标签（`<blockquote>`, `<table>`）而非 `style=`
//! 2. **渐进增强**: 基础文本在所有客户端可读，高级客户端渲染增强效果
//! 3. **颜色系统**: 预定义颜色常量，保持视觉一致性
//!
//! ## 使用示例
//!
//! ```
//! use aether_matrix::ui::templates::{info_card, help_menu, success, error};
//!
//! // 信息卡片
//! let card = info_card("Bot 信息", &[
//!     ("名称", "AI Assistant"),
//!     ("版本", "1.0.0"),
//! ]);
//!
//! // 帮助菜单
//! let menu = help_menu(&[
//!     ("!ping", "测试响应"),
//!     ("!help", "显示帮助"),
//! ]);
//!
//! // 状态消息
//! let ok = success("操作成功！");
//! let err = error("发生错误");
//! ```
//!
//! ## 渲染效果
//!
//! | 模板函数 | 用途 | 视觉特征 |
//! |----------|------|----------|
//! | [`info_card`] | 信息展示 | 标题 + 表格卡片 |
//! | [`help_menu`] | 命令列表 | 代码高亮 + 描述 |
//! | [`success`] | 成功状态 | 绿色 ✓ 图标 |
//! | [`error`] | 错误状态 | 红色 ✕ 图标 |
//! | [`warning`] | 警告状态 | 黄色 ⚠ 图标 |
//! | [`leaderboard`] | 排行榜 | 表头 + 数据行 |

/// 预定义颜色系统。
///
/// 所有颜色均使用十六进制格式，专为 Element 客户端优化。
/// 避免使用 `style=` 属性，改用 `<font color>` 标签。
///
/// ## 颜色语义
///
/// | 常量 | 用途 | 色值 |
/// |------|------|------|
/// | [`TITLE`] | 标题文字 | `#3164f0ff` (蓝色) |
/// | [`ACCENT`] | 强调图标 | `#6ea8ff` (亮蓝) |
/// | [`KEY`] | 键名/表头 | `#6a7fa8` (灰蓝) |
/// | [`VALUE`] | 数值/内容 | `#c8d4f0` (浅蓝) |
/// | [`DIM`] | 次要描述 | `#4a5a78` (暗蓝) |
/// | [`SUCCESS`] | 成功状态 | `#48bb78` (绿色) |
/// | [`ERROR`] | 错误状态 | `#f06070` (红色) |
/// | [`WARNING`] | 警告状态 | `#f0c060` (黄色) |
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

/// 生成带颜色的 `<font>` 标签。
///
/// # Arguments
///
/// * `color` - 颜色值（十六进制或命名颜色）
/// * `s` - 要包裹的文本内容
///
/// # Returns
///
/// 返回格式化的 HTML 字符串：`<font color="{color}">{s}</font>`
///
/// # Example
///
/// ```
/// # fn fc(color: &str, s: &str) -> String {
/// #     format!("<font color=\"{}\">{}</font>", color, s)
/// # }
/// let red_text = fc("#f00", "error");
/// assert_eq!(red_text, "<font color=\"#f00\">error</font>");
/// ```
fn fc(color: &str, s: &str) -> String {
    format!(r#"<font color="{color}">{s}</font>"#)
}

/// 生成加粗 `<b>` 标签。
///
/// # Arguments
///
/// * `s` - 要加粗的文本内容
///
/// # Returns
///
/// 返回格式化的 HTML 字符串：`<b>{s}</b>`
fn bold(s: &str) -> String {
    format!("<b>{s}</b>")
}

/// 生成代码 `<code>` 标签。
///
/// 用于命令、路径等技术内容的等宽字体展示。
///
/// # Arguments
///
/// * `s` - 代码文本内容
///
/// # Returns
///
/// 返回格式化的 HTML 字符串：`<code>{s}</code>`
fn code(s: &str) -> String {
    format!("<code>{s}</code>")
}

/// 玻璃态模板生成器。
///
/// 使用结构化 HTML 标签实现卡片式布局，专为 Element 客户端优化。
///
/// ## 设计特点
///
/// - **语义化标签**: 使用 `<h3>`, `<blockquote>`, `<table>` 构建层级
/// - **视觉层次**: 标题 → 卡片容器 → 数据表格
/// - **颜色系统**: 统一使用 `color` 模块预定义颜色
///
/// ## 使用示例
///
/// ```
/// # use aether_matrix::ui::templates::GlassTemplate;
/// let card = GlassTemplate::info_card("Bot 信息", &[
///     ("名称", "AI Assistant"),
///     ("版本", "1.0.0"),
/// ]);
/// // 渲染为：标题 + 表格卡片
/// ```
pub struct GlassTemplate;

impl GlassTemplate {
    /// 生成信息卡片模板。
    ///
    /// 适用于展示键值对信息（如 Bot 信息、配置状态等）。
    ///
    /// ## 渲染结构
    ///
    /// ```html
    /// <h3>[图标] [标题]</h3>
    /// <blockquote>
    ///   <table>
    ///     <tr><td>键 1</td><td>值 1</td></tr>
    ///     <tr><td>键 2</td><td>值 2</td></tr>
    ///   </table>
    /// </blockquote>
    /// ```
    ///
    /// # Arguments
    ///
    /// * `title` - 卡片标题
    /// * `items` - 键值对列表，支持任意实现 `AsRef<str>` 的类型
    ///
    /// # Returns
    ///
    /// 返回格式化的 HTML 字符串。
    ///
    /// # Example
    ///
    /// ```
    /// # use aether_matrix::ui::templates::GlassTemplate;
    /// let card = GlassTemplate::info_card("Bot 信息", &[
    ///     ("名称", "AI Assistant"),
    ///     ("状态", "在线"),
    /// ]);
    /// assert!(card.contains("<h3>"));
    /// assert!(card.contains("<blockquote>"));
    /// assert!(card.contains("<table>"));
    /// ```
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

    /// 生成帮助菜单模板。
    ///
    /// 适用于展示命令列表，命令名使用代码高亮样式。
    ///
    /// ## 渲染结构
    ///
    /// ```html
    /// <h3>[图标] 命令帮助</h3>
    /// <blockquote>
    ///   <table>
    ///     <tr><td><code>命令 1</code></td><td>描述 1</td></tr>
    ///     <tr><td><code>命令 2</code></td><td>描述 2</td></tr>
    ///   </table>
    /// </blockquote>
    /// ```
    ///
    /// # Arguments
    ///
    /// * `commands` - 命令列表，每个元素为 `(命令名，描述)` 元组
    ///
    /// # Returns
    ///
    /// 返回格式化的 HTML 字符串。
    ///
    /// # Example
    ///
    /// ```
    /// # use aether_matrix::ui::templates::GlassTemplate;
    /// let menu = GlassTemplate::help_menu(&[
    ///     ("!ping", "测试响应"),
    ///     ("!help", "显示帮助"),
    /// ]);
    /// assert!(menu.contains("<code>!ping</code>"));
    /// ```
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

    /// 生成状态消息模板。
    ///
    /// 根据状态类型显示不同的图标和颜色：
    ///
    /// ## 状态类型
    ///
    /// | 状态 | 图标 | 颜色 |
    /// |------|------|------|
    /// | [`Success`](Status::Success) | ✓ | 绿色 |
    /// | [`Error`](Status::Error) | ✕ | 红色 |
    /// | [`Warning`](Status::Warning) | ⚠ | 黄色 |
    ///
    /// ## 渲染结构
    ///
    /// ```html
    /// <blockquote>[图标] [消息内容]</blockquote>
    /// ```
    ///
    /// # Arguments
    ///
    /// * `kind` - 状态类型（成功/错误/警告）
    /// * `message` - 状态消息内容
    ///
    /// # Returns
    ///
    /// 返回格式化的 HTML 字符串。
    ///
    /// # Example
    ///
    /// ```
    /// # use aether_matrix::ui::templates::{GlassTemplate, Status};
    /// let success = GlassTemplate::status(Status::Success, "操作成功");
    /// assert!(success.contains("✓"));
    /// assert!(success.contains("<blockquote>"));
    /// ```
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

/// 消息状态类型。
///
/// 用于 [`GlassTemplate::status`] 方法，决定显示的图标和颜色。
///
/// ## 变体说明
///
/// | 变体 | 图标 | 颜色 | 使用场景 |
/// |------|------|------|----------|
/// | [`Self::Success`] | ✓ | 绿色 | 操作成功、任务完成 |
/// | [`Self::Error`] | ✕ | 红色 | 操作失败、发生错误 |
/// | [`Self::Warning`] | ⚠ | 黄色 | 注意事项、潜在问题 |
///
/// # Example
///
/// ```
/// use aether_matrix::ui::templates::Status;
///
/// let status = Status::Success;
/// match status {
///     Status::Success => println!("成功"),
///     Status::Error => println!("错误"),
///     Status::Warning => println!("警告"),
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub enum Status {
    /// 操作成功，显示绿色 ✓ 图标。
    Success,
    /// 操作失败，显示红色 ✕ 图标。
    Error,
    /// 注意事项，显示黄色 ⚠ 图标。
    Warning,
}

/// 生成信息卡片（[`GlassTemplate::info_card`] 的便捷封装）。
///
/// 适用于展示键值对信息。
///
/// # Arguments
///
/// * `title` - 卡片标题
/// * `items` - 键值对列表
///
/// # Returns
///
/// 返回格式化的 HTML 字符串。
///
/// # Example
///
/// ```
/// use aether_matrix::ui::templates::info_card;
///
/// let card = info_card("Bot 信息", &[
///     ("名称", "AI Assistant"),
///     ("版本", "1.0.0"),
/// ]);
/// assert!(card.contains("<h3>"));
/// ```
pub fn info_card(title: &str, items: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    GlassTemplate::info_card(title, items)
}

/// 生成帮助菜单（[`GlassTemplate::help_menu`] 的便捷封装）。
///
/// 适用于展示命令列表。
///
/// # Arguments
///
/// * `commands` - 命令列表，每个元素为 `(命令名，描述)` 元组
///
/// # Returns
///
/// 返回格式化的 HTML 字符串。
pub fn help_menu(commands: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    GlassTemplate::help_menu(commands)
}

/// 生成成功状态消息（[`GlassTemplate::status`] 的便捷封装）。
///
/// 显示绿色 ✓ 图标，用于操作成功的提示。
///
/// # Arguments
///
/// * `msg` - 成功消息内容
///
/// # Returns
///
/// 返回格式化的 HTML 字符串。
///
/// # Example
///
/// ```
/// use aether_matrix::ui::templates::success;
///
/// let msg = success("操作已完成");
/// assert!(msg.contains("✓"));
/// assert!(msg.contains("<blockquote>"));
/// ```
pub fn success(msg: &str) -> String {
    GlassTemplate::status(Status::Success, msg)
}

/// 生成错误状态消息（[`GlassTemplate::status`] 的便捷封装）。
///
/// 显示红色 ✕ 图标，用于操作失败的提示。
///
/// # Arguments
///
/// * `msg` - 错误消息内容
///
/// # Returns
///
/// 返回格式化的 HTML 字符串。
pub fn error(msg: &str) -> String {
    GlassTemplate::status(Status::Error, msg)
}

/// 生成警告状态消息（[`GlassTemplate::status`] 的便捷封装）。
///
/// 显示黄色 ⚠ 图标，用于注意事项的提示。
///
/// # Arguments
///
/// * `msg` - 警告消息内容
///
/// # Returns
///
/// 返回格式化的 HTML 字符串。
pub fn warning(msg: &str) -> String {
    GlassTemplate::status(Status::Warning, msg)
}

/// 生成信息状态消息。
///
/// 显示蓝色强调色，用于一般信息提示（非状态性消息）。
///
/// # Arguments
///
/// * `msg` - 信息消息内容
///
/// # Returns
///
/// 返回格式化的 HTML 字符串。
///
/// # Note
///
/// 此函数标记为 `#[allow(dead_code)]`，目前未使用，但保留供未来扩展。
#[allow(dead_code)]
pub fn info(msg: &str) -> String {
    format!("<blockquote>{}</blockquote>", bold(&fc(color::ACCENT, msg)))
}

/// 生成排行榜模板。
///
/// 适用于展示排名、积分榜等表格数据。
///
/// ## 渲染结构
///
/// ```html
/// <h3>[图标] [标题]</h3>
/// <blockquote>
///   <table>
///     <thead>
///       <tr><th>表头 1</th><th>表头 2</th></tr>
///     </thead>
///     <tbody>
///       <tr><td>数据 1</td><td>数据 2</td></tr>
///     </tbody>
///   </table>
/// </blockquote>
/// ```
///
/// # Arguments
///
/// * `title` - 排行榜标题
/// * `headers` - 表头列名列表
/// * `rows` - 数据行列表，每行为字符串向量
///
/// # Returns
///
/// 返回格式化的 HTML 字符串。
///
/// # Example
///
/// ```
/// use aether_matrix::ui::templates::leaderboard;
///
/// let board = leaderboard("功德榜", &["排名", "用户", "功德"], &[
///     vec!["1", "Alice", "1000"],
///     vec!["2", "Bob", "800"],
/// ]);
/// assert!(board.contains("<thead>"));
/// assert!(board.contains("<tbody>"));
/// ```
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
