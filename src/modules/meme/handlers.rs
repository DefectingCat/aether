//! Meme 命令处理器。

use anyhow::Result;
use async_trait::async_trait;
use matrix_sdk::attachment::AttachmentConfig;

use crate::command::{CommandContext, CommandHandler, Permission};
use crate::modules::meme::klipy::KlipyClient;
use crate::ui::{error, info_card};

/// Meme 梗图命令处理器。
///
/// 使用 KLIPY GIF API 搜索并发送梗图。
///
/// # 命令
///
/// `!meme <关键词>` - 搜索并发送一张匹配的 GIF
///
/// # 权限
///
/// 任何房间成员都可以执行。
pub struct MemeHandler {
    klipy: Option<KlipyClient>,
}

impl MemeHandler {
    /// 创建新的 Meme 命令处理器。
    pub fn new(klipy: Option<KlipyClient>) -> Self {
        Self { klipy }
    }
}

#[async_trait]
impl CommandHandler for MemeHandler {
    fn name(&self) -> &str {
        "meme"
    }

    fn description(&self) -> &str {
        "搜索并发送梗图"
    }

    fn usage(&self) -> &str {
        "meme <关键词>"
    }

    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // !meme 命令没有子命令结构，所有参数都是搜索关键词
        let query: String = ctx.args.join(" ");
        if query.is_empty() {
            let html = info_card("Meme 命令", &[("!meme <关键词>", "搜索并发送梗图")]);
            return send_html(&ctx.room, &html).await;
        }

        // 检查是否配置了 KLIPY API Key
        let klipy = match &self.klipy {
            Some(k) => k,
            None => {
                let html = error("梗图功能未配置。请在 .env 中设置 KLIPY_API_KEY\n注册地址: https://partner.klipy.com");
                return send_html(&ctx.room, &html).await;
            }
        };

        // 搜索 GIF
        let gif_result = match klipy.search(&query).await {
            Ok(Some(result)) => result,
            Ok(None) => {
                let html = error(&format!("没有找到匹配「{}」的梗图", query));
                return send_html(&ctx.room, &html).await;
            }
            Err(e) => {
                tracing::error!("KLIPY API 错误: {}", e);
                let html = error(&format!("搜索梗图失败: {}", e));
                return send_html(&ctx.room, &html).await;
            }
        };

        // 下载 GIF
        let http_client = reqwest::Client::new();
        let response = match http_client.get(&gif_result.url).send().await {
            Ok(r) => r,
            Err(e) => {
                let html = error(&format!("下载梗图失败: {}", e));
                return send_html(&ctx.room, &html).await;
            }
        };

        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                let html = error(&format!("读取梗图数据失败: {}", e));
                return send_html(&ctx.room, &html).await;
            }
        };

        // 使用 send_attachment 发送图片（自动处理图片信息和缩略图）
        let mime_type: mime::Mime = "image/gif".parse()?;
        let filename = format!("{}.gif", query);
        let config = AttachmentConfig::new();

        match ctx.room.send_attachment(&filename, &mime_type, bytes.to_vec(), config).await {
            Ok(_) => {
                tracing::info!("已发送梗图: {}", query);
            }
            Err(e) => {
                let html = error(&format!("发送梗图失败: {}", e));
                return send_html(&ctx.room, &html).await;
            }
        }

        Ok(())
    }
}

/// 发送 HTML 消息
async fn send_html(room: &matrix_sdk::Room, html: &str) -> Result<()> {
    let plain_text = html
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != ' ', "")
        .chars()
        .take(100)
        .collect::<String>();

    let content = matrix_sdk::ruma::events::room::message::RoomMessageEventContent::text_html(plain_text, html);
    room.send(content).await?;
    Ok(())
}