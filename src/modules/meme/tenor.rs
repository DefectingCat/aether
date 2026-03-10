//! Tenor GIF API 客户端。

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::{debug, warn};

/// Tenor API 客户端。
///
/// 用于搜索 GIF 图片。
pub struct TenorClient {
    api_key: String,
    limit: u32,
    http_client: reqwest::Client,
}

/// Tenor 搜索响应。
#[derive(Debug, Deserialize)]
struct TenorSearchResponse {
    results: Vec<TenorResult>,
    #[serde(default)]
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TenorResult {
    id: String,
    media_formats: MediaFormats,
}

#[derive(Debug, Deserialize)]
struct MediaFormats {
    #[serde(default)]
    gif: Option<MediaFormat>,
    #[serde(default)]
    tinygif: Option<MediaFormat>,
    #[serde(default)]
    mediumgif: Option<MediaFormat>,
}

#[derive(Debug, Deserialize)]
struct MediaFormat {
    url: String,
}

/// GIF 搜索结果。
#[derive(Debug, Clone)]
pub struct GifResult {
    pub url: String,
}

impl TenorClient {
    /// 创建新的 Tenor 客户端。
    pub fn new(api_key: String, limit: u32) -> Self {
        Self {
            api_key,
            limit,
            http_client: reqwest::Client::new(),
        }
    }

    /// 搜索 GIF。
    ///
    /// # Arguments
    ///
    /// * `query` - 搜索关键词
    ///
    /// # Returns
    ///
    /// 返回随机一个 GIF 结果，如果没有结果则返回 None。
    pub async fn search(&self, query: &str) -> Result<Option<GifResult>> {
        let encoded_query = urlencoding::encode(query);
        let url = format!(
            "https://tenor.googleapis.com/v2/search?q={}&key={}&limit={}&media_filter=gif,tinygif,mediumgif",
            encoded_query, self.api_key, self.limit
        );

        debug!("Tenor API 请求: {}", url.replace(&self.api_key, "API_KEY"));

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .context("Tenor API 请求失败")?;

        let status = response.status();
        let body = response.text().await.context("读取响应失败")?;

        debug!("Tenor API 响应状态: {}", status);
        debug!("Tenor API 响应内容: {}", body.chars().take(500).collect::<String>());

        if !status.is_success() {
            anyhow::bail!("Tenor API 返回错误: {} - {}", status, body);
        }

        // 尝试解析响应
        let search_response: TenorSearchResponse = serde_json::from_str(&body)
            .with_context(|| format!("解析 Tenor API 响应失败，响应内容: {}", body.chars().take(200).collect::<String>()))?;

        if search_response.results.is_empty() {
            warn!("Tenor API 返回空结果，查询: {}", query);
            return Ok(None);
        }

        debug!("Tenor API 返回 {} 个结果", search_response.results.len());

        // 随机选择一个结果
        use rand::prelude::IndexedRandom;
        let result = search_response
            .results
            .choose(&mut rand::rng())
            .context("没有可用的 GIF 结果")?;

        debug!("选中的 GIF ID: {}", result.id);

        // 优先使用 tinygif（更小的文件），然后是 mediumgif，最后是 gif
        let url = result
            .media_formats
            .tinygif
            .as_ref()
            .or(result.media_formats.mediumgif.as_ref())
            .or(result.media_formats.gif.as_ref())
            .map(|m| m.url.as_str())
            .context("GIF 没有可用的媒体格式")?
            .to_string();

        debug!("GIF URL: {}", url);

        Ok(Some(GifResult { url }))
    }
}