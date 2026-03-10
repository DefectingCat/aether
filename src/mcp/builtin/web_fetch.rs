//! # Web Fetch 工具
//!
//! 从 URL 获取网页内容并提取文本。

use anyhow::{Context, Result};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::mcp::{Tool, ToolDefinition, ToolResult, ToolSource};

/// Web Fetch 工具参数
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WebFetchParams {
    /// 要获取的 URL
    pub url: String,

    /// 可选：CSS 选择器，提取特定内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,

    /// 可选：最大内容长度（字符）
    #[serde(default = "default_max_length")]
    pub max_length: usize,
}

fn default_max_length() -> usize {
    10000
}

/// Web Fetch 工具实现
pub struct WebFetchTool {
    /// HTTP 客户端
    client: reqwest::Client,
    /// 最大内容长度
    max_length: usize,
}

impl WebFetchTool {
    /// 创建新的 Web Fetch 工具
    pub fn new(config: super::super::WebFetchConfig) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(config.timeout))
                .user_agent("Aether-Matrix-Bot/1.0")
                .build()
                .expect("Failed to create HTTP client"),
            max_length: config.max_length,
        }
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "web_fetch".to_string(),
            description: "Fetch content from a web URL and extract text. Useful for getting information from websites.".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(WebFetchParams))
                .expect("Failed to generate schema"),
        }
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<ToolResult> {
        let params: WebFetchParams =
            serde_json::from_value(arguments).context("Invalid arguments for web_fetch")?;

        // 验证 URL
        let url = url::Url::parse(&params.url).context("Invalid URL")?;

        tracing::info!("Fetching URL: {}", url);

        // 获取网页内容
        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .context("Failed to fetch URL")?;

        if !response.status().is_success() {
            tracing::warn!("HTTP error {}: {}", response.status(), params.url);
            return Ok(ToolResult {
                success: false,
                content: String::new(),
                error: Some(format!("HTTP {}: {}", response.status(), params.url)),
            });
        }

        let html = response
            .text()
            .await
            .context("Failed to read response body")?;

        tracing::debug!("Fetched {} bytes from {}", html.len(), url);

        // 提取文本内容
        let content = if let Some(selector) = &params.selector {
            self.extract_with_selector(&html, selector)?
        } else {
            self.extract_text(&html)
        };

        // 限制长度（使用参数或默认值）
        let max_len = if params.max_length > 0 {
            params.max_length
        } else {
            self.max_length
        };

        let truncated = if content.len() > max_len {
            tracing::debug!(
                "Truncating content from {} to {} characters",
                content.len(),
                max_len
            );
            format!(
                "{}...\n\n[Truncated at {} characters]",
                &content[..max_len],
                max_len
            )
        } else {
            content
        };

        Ok(ToolResult {
            success: true,
            content: truncated,
            error: None,
        })
    }

    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }
}

impl WebFetchTool {
    /// 从 HTML 中提取纯文本
    fn extract_text(&self, html: &str) -> String {
        // 移除 script 和 style 标签
        let re_script = regex::Regex::new(r"<script[^>]*>.*?</script>").unwrap();
        let re_style = regex::Regex::new(r"<style[^>]*>.*?</style>").unwrap();

        let html = re_script.replace_all(html, " ");
        let html = re_style.replace_all(&html, " ");

        // 移除所有 HTML 标签
        let re_tags = regex::Regex::new(r"<[^>]+>").unwrap();
        let text = re_tags.replace_all(&html, " ");

        // 清理空白
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// 使用 CSS 选择器提取内容
    fn extract_with_selector(&self, html: &str, selector: &str) -> Result<String> {
        let document = scraper::Html::parse_document(html);
        let selector = scraper::Selector::parse(selector)
            .map_err(|e| anyhow::anyhow!("Invalid CSS selector: {:?}", e))?;

        let text = document
            .select(&selector)
            .map(|el| el.text().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::WebFetchConfig;

    #[test]
    fn test_extract_text() {
        let tool = WebFetchTool::new(WebFetchConfig::default());

        let html = r#"<html><body><p>Hello World</p></body></html>"#;
        let text = tool.extract_text(html);

        assert!(text.contains("Hello World"));
    }

    #[test]
    fn test_extract_text_removes_script() {
        let tool = WebFetchTool::new(WebFetchConfig::default());

        let html = r#"<html><body><script>alert('test');</script><p>Content</p></body></html>"#;
        let text = tool.extract_text(html);

        assert!(text.contains("Content"));
        assert!(!text.contains("alert"));
    }

    #[test]
    fn test_tool_definition() {
        let tool = WebFetchTool::new(WebFetchConfig::default());
        let def = tool.definition();

        assert_eq!(def.name, "web_fetch");
        assert!(!def.description.is_empty());
    }

    #[test]
    fn test_params_deserialization() {
        let json = r#"{"url": "https://example.com", "max_length": 5000}"#;
        let params: WebFetchParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.url, "https://example.com");
        assert_eq!(params.max_length, 5000);
        assert!(params.selector.is_none());
    }
}
