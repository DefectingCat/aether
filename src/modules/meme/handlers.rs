//! Meme 命令处理器。

use anyhow::Result;
use async_trait::async_trait;
use matrix_sdk::ruma::events::room::message::{ImageMessageEventContent, MessageType, RoomMessageEventContent};
use matrix_sdk::ruma::MxcUri;

use crate::command::{CommandContext, CommandHandler, Permission};
use crate::modules::meme::tenor::TenorClient;
use crate::ui::{error, info_card};

/// Meme 梗图命令处理器。
///
/// 使用 Tenor GIF API 搜索并发送梗图。
///
/// # 命令
///
/// `!meme <关键词>` - 搜索并发送一张匹配的 GIF
///
/// # 权限
///
/// 任何房间成员都可以执行。
pub struct MemeHandler {
    tenor: Option<TenorClient>,
}

impl MemeHandler {
    /// 创建新的 Meme 命令处理器。
    pub fn new(tenor: Option<TenorClient>) -> Self {
        Self { tenor }
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
        let query: String = ctx.sub_args().join(" ");
        if query.is_empty() {
            let html = info_card("Meme 命令", &[("!meme <关键词>", "搜索并发送梗图")]);
            return send_html(&ctx.room, &html).await;
        }

        // 检查是否配置了 Tenor API Key
        let tenor = match &self.tenor {
            Some(t) => t,
            None => {
                let html = error("梗图功能未配置。请在 .env 中设置 TENOR_API_KEY");
                return send_html(&ctx.room, &html).await;
            }
        };

        // 搜索 GIF
        let gif_result = match tenor.search(&query).await {
            Ok(Some(result)) => result,
            Ok(None) => {
                let html = error(&format!("没有找到匹配「{}」的梗图", query));
                return send_html(&ctx.room, &html).await;
            }
            Err(e) => {
                tracing::error!("Tenor API 错误: {}", e);
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

        // 上传到 Matrix media server
        let media = ctx.client.media();
        let mime_type: mime::Mime = "image/gif".parse()?;
        let upload_response = match media.upload(&mime_type, bytes.to_vec(), None).await {
            Ok(r) => r,
            Err(e) => {
                let html = error(&format!("上传梗图失败: {}", e));
                return send_html(&ctx.room, &html).await;
            }
        };

        // 发送图片消息
        let mxc_uri: &MxcUri = &upload_response.content_uri;
        let image_content = ImageMessageEventContent::plain(
            query.clone(),
            mxc_uri.to_owned().into(),
        );

        let message = RoomMessageEventContent::new(MessageType::Image(image_content));
        ctx.room.send(message).await?;

        tracing::info!("已发送梗图: {}", query);
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

    let content = RoomMessageEventContent::text_html(plain_text, html);
    room.send(content).await?;
    Ok(())
}