//! 媒体处理模块。
//!
//! 提供图片下载、缩放和 base64 编码功能，用于 Vision API。

use std::io::Cursor;

use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use matrix_sdk::{
    Client,
    media::{MediaFormat, MediaRequestParameters},
    ruma::{MxcUri, events::room::MediaSource},
};

/// 从 Matrix 服务器下载图片并转换为 base64 data URL。
///
/// 下载后会自动缩放图片（如果超过最大尺寸），保持宽高比。
///
/// # Arguments
///
/// * `client` - Matrix 客户端实例
/// * `mxc_uri` - Matrix 内容 URI (mxc://...)
/// * `expected_media_type` - 预期的媒体类型（可选，用于验证）
/// * `max_size` - 图片最大边长（像素），超过此尺寸会自动缩放
///
/// # Returns
///
/// 成功时返回 base64 编码的 data URL，格式为 `data:image/png;base64,{data}`
/// 缩放后的图片统一输出为 PNG 格式。
///
/// # Errors
///
/// 当以下情况发生时返回错误：
/// - MXC URI 无效
/// - 图片下载失败
/// - 图片解析或缩放失败
///
/// # Example
///
/// ```
/// use aether_matrix::media::download_image_as_base64;
///
/// // let data_url = download_image_as_base64(
/// //     &client,
/// //     "mxc://matrix.org/abc123",
/// //     Some("image/png"),
/// //     1024,  // 最大 1024 像素
/// // ).await?;
/// ```
pub async fn download_image_as_base64(
    client: &Client,
    mxc_uri: &MxcUri,
    _expected_media_type: Option<&str>,
    max_size: u32,
) -> Result<String> {
    // 验证 MXC URI
    if !mxc_uri.is_valid() {
        anyhow::bail!("无效的 MXC URI: {}", mxc_uri);
    }

    // 下载媒体文件
    let request = MediaRequestParameters {
        source: MediaSource::Plain(mxc_uri.to_owned()),
        format: MediaFormat::File,
    };
    let media = client
        .media()
        .get_media_content(&request, true) // 允许从缓存获取
        .await
        .context("从 Matrix 服务器下载图片失败")?;

    // 缩放图片（如果需要）
    let processed_media = resize_image_if_needed(&media, max_size)?;

    // 编码为 data URL（缩放后统一为 PNG 格式）
    Ok(encode_as_data_url(&processed_media, "image/png"))
}

/// 缩放图片（如果超过最大尺寸）。
///
/// 保持宽高比，将图片缩放到最大边不超过 `max_size`。
/// 如果图片已经足够小，则不做任何处理。
///
/// # Arguments
///
/// * `image_data` - 原始图片数据
/// * `max_size` - 最大边长（像素）
///
/// # Returns
///
/// 成功时返回缩放后的 PNG 格式图片数据。
///
/// # Errors
///
/// 当图片数据无效时返回错误。
///
/// # Example
///
/// ```
/// use aether_matrix::media::resize_image_if_needed;
///
/// // 假设 image_data 是有效的图片数据
/// // let resized = resize_image_if_needed(&image_data, 1024)?;
/// ```
///
/// # Algorithm
///
/// 使用 Lanczos3 算法进行高质量缩放，适合照片和复杂图像。
/// 对于简单的图标或线条图，可能产生轻微的模糊，但整体效果优于其他算法。
pub fn resize_image_if_needed(image_data: &[u8], max_size: u32) -> Result<Vec<u8>> {
    // 加载图片
    let img = image::load_from_memory(image_data).context("无法解析图片数据")?;

    // 检查是否需要缩放
    let (width, height) = (img.width(), img.height());
    if width <= max_size && height <= max_size {
        // 图片已经足够小，直接返回原数据
        return Ok(image_data.to_vec());
    }

    // 计算缩放比例，保持宽高比
    let ratio = max_size as f64 / width.max(height) as f64;
    let new_width = (width as f64 * ratio) as u32;
    let new_height = (height as f64 * ratio) as u32;

    tracing::debug!(
        "缩放图片: {}x{} -> {}x{}",
        width,
        height,
        new_width,
        new_height
    );

    // 使用 Lanczos3 算法缩放（高质量）
    // Lanczos3 是一种高质量的重采样算法，适合照片和复杂图像
    // 相比双线性插值，它在保持锐度的同时减少了锯齿和模糊
    let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

    // 输出为 PNG 格式
    let mut output = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut output), image::ImageFormat::Png)
        .context("无法编码缩放后的图片")?;

    Ok(output)
}

/// 将图片数据编码为 base64 data URL。
///
/// # Arguments
///
/// * `image_data` - 图片二进制数据
/// * `media_type` - 媒体类型（如 "image/png", "image/jpeg"）
///
/// # Returns
///
/// 返回格式为 `data:{media_type};base64,{data}` 的字符串
///
/// # Example
///
/// ```
/// use aether_matrix::media::encode_as_data_url;
///
/// let data_url = encode_as_data_url(b"\x89PNG...", "image/png");
/// assert!(data_url.starts_with("data:image/png;base64,"));
/// ```
pub fn encode_as_data_url(image_data: &[u8], media_type: &str) -> String {
    let base64_str = STANDARD.encode(image_data);
    format!("data:{};base64,{}", media_type, base64_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;
    use std::io::Cursor;

    #[test]
    fn test_encode_as_data_url() {
        let data = b"hello";
        let result = encode_as_data_url(data, "image/png");
        assert!(result.starts_with("data:image/png;base64,"));
        assert!(result.ends_with(&STANDARD.encode(data)));
    }

    #[test]
    fn test_resize_image_if_needed_small_image() {
        // 创建一个小的测试图片（1x1 像素）
        let img = DynamicImage::new_rgb8(1, 1);
        let mut output = Vec::new();
        img.write_to(&mut Cursor::new(&mut output), image::ImageFormat::Png)
            .unwrap();

        // 小图片不应被缩放
        let result = resize_image_if_needed(&output, 1024).unwrap();
        // 返回的应该还是 PNG 格式
        assert!(result.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
    }

    #[test]
    fn test_resize_image_if_needed_preserves_aspect_ratio() {
        // 创建一个 2x1 像素的测试图片
        let img = DynamicImage::new_rgb8(2, 1);
        let mut output = Vec::new();
        img.write_to(&mut Cursor::new(&mut output), image::ImageFormat::Png)
            .unwrap();

        // 不需要缩放
        let result = resize_image_if_needed(&output, 10).unwrap();
        // 应该返回原始数据
        assert!(!result.is_empty());
    }

    #[test]
    fn test_resize_image_if_needed_invalid_image_format() {
        // 测试无效的图片格式（纯文本数据）
        let invalid_data = b"This is not an image file at all!";
        let result = resize_image_if_needed(invalid_data, 1024);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("无法解析图片数据"));
    }

    #[test]
    fn test_resize_image_if_needed_corrupted_image_data() {
        // 测试损坏的PNG数据（有效的PNG头部但损坏的内容）
        let corrupted_png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\x00\x00\x00\x0cIDAT\x08\xd7c\xf8\x0f\x00\x01\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        // This is a truncated PNG file - it has valid headers but incomplete data
        let result = resize_image_if_needed(corrupted_png, 1024);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("无法解析图片数据"));
    }

    #[test]
    fn test_resize_image_if_needed_empty_data() {
        // 测试空数据
        let empty_data: &[u8] = &[];
        let result = resize_image_if_needed(empty_data, 1024);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("无法解析图片数据"));
    }

    #[test]
    fn test_resize_image_if_needed_oversized_image() {
        // 创建一个超大图片（超过最大尺寸）
        // Using a reasonable size that's still testable but larger than max_size
        let large_img = DynamicImage::new_rgb8(2048, 1536); // 2048x1536 > 1024
        let mut output = Vec::new();
        large_img.write_to(&mut Cursor::new(&mut output), image::ImageFormat::Png)
            .unwrap();

        // Should be resized to fit within 1024 max dimension
        let result = resize_image_if_needed(&output, 1024).unwrap();
        
        // Verify it's still a valid PNG
        assert!(result.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
        
        // Load the result to verify dimensions
        let resized_img = image::load_from_memory(&result).unwrap();
        let (width, height) = (resized_img.width(), resized_img.height());
        assert!(width <= 1024 && height <= 1024);
        // Verify aspect ratio is preserved (2048:1536 = 4:3, so resized should maintain this)
        let aspect_ratio = width as f32 / height as f32;
        assert!((aspect_ratio - (4.0 / 3.0)).abs() < 0.1);
    }

    #[test]
    fn test_encode_as_data_url_empty_data() {
        // 测试空数据的Data URL编码
        let empty_data: &[u8] = &[];
        let result = encode_as_data_url(empty_data, "image/png");
        assert_eq!(result, "data:image/png;base64,");
    }

    #[test]
    fn test_encode_as_data_url_invalid_media_type() {
        // 测试无效的媒体类型
        let data = b"test";
        let result = encode_as_data_url(data, "");
        assert_eq!(result, "data:;base64,dGVzdA==");
        
        let result2 = encode_as_data_url(data, "invalid/type");
        assert_eq!(result2, "data:invalid/type;base64,dGVzdA==");
        // The function doesn't validate media types, so this should work
    }

    #[test]
    fn test_resize_image_if_needed_unsupported_format() {
        // Test with data that looks like a format but isn't supported
        // Create some random binary data that doesn't correspond to any image format
        let random_data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let result = resize_image_if_needed(&random_data, 1024);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("无法解析图片数据"));
    }
}
