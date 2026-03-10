use aether_matrix::mcp::builtin::web_fetch::{WebFetchParams, WebFetchTool};
use aether_matrix::mcp::{Tool, WebFetchConfig};
use anyhow::Result;
use serde_json::json;

fn create_test_web_fetch_tool(max_length: usize, timeout: u64) -> WebFetchTool {
    WebFetchTool::new(WebFetchConfig {
        enabled: true,
        max_length,
        timeout,
    })
}

#[tokio::test]
async fn test_web_fetch_tool_definition() {
    let tool = create_test_web_fetch_tool(10000, 10);
    let def = tool.definition();
    assert_eq!(def.name, "web_fetch");
    assert!(!def.description.is_empty());
}

#[tokio::test]
async fn test_web_fetch_invalid_url_missing_scheme() {
    let tool = create_test_web_fetch_tool(10000, 10);
    
    let params = WebFetchParams {
        url: "invalid-url".to_string(),
        selector: None,
        max_length: 1000,
    };
    
    let json_params = serde_json::to_value(params).unwrap();
    let result = tool.execute(json_params).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid URL"));
}

#[tokio::test]
async fn test_web_fetch_malformed_url() {
    let tool = create_test_web_fetch_tool(10000, 10);
    
    let params = WebFetchParams {
        url: "http://".to_string(),
        selector: None,
        max_length: 1000,
    };
    
    let json_params = serde_json::to_value(params).unwrap();
    let result = tool.execute(json_params).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid URL"));
}

#[test]
fn test_web_fetch_text_extraction_removes_html_tags() {
    let tool = create_test_web_fetch_tool(10000, 10);
    
    let html = r#"<html><body><h1>Title</h1><p>Paragraph with <em>emphasis</em> and <strong>bold</strong>.</p></body></html>"#;
    let text = tool.extract_text(html);
    
    assert!(text.contains("Title"));
    assert!(text.contains("Paragraph with emphasis and bold"));
    assert!(!text.contains("<h1>"));
    assert!(!text.contains("<em>"));
}

#[test]
fn test_web_fetch_text_extraction_removes_script_and_style() {
    let tool = create_test_web_fetch_tool(10000, 10);
    
    let html = r#"<html><head><style>.hidden { display: none; }</style></head><body><script>console.log('test');</script><div>Visible content</div></body></html>"#;
    let text = tool.extract_text(html);
    
    assert!(text.contains("Visible content"));
    assert!(!text.contains("console.log"));
    assert!(!text.contains(".hidden"));
}

#[test]
fn test_web_fetch_css_selector_valid_extraction() {
    let tool = create_test_web_fetch_tool(10000, 10);
    
    let html = r#"<html><body><div class="main"><h1>Main Title</h1><p>Main content</p></div><div class="sidebar"><p>Sidebar content</p></div></body></html>"#;
    
    let result = tool.extract_with_selector(html, ".main h1");
    assert!(result.is_ok());
    let extracted = result.unwrap();
    assert!(extracted.contains("Main Title"));
    assert!(!extracted.contains("Sidebar content"));
}

#[test]
fn test_web_fetch_css_selector_multiple_elements() {
    let tool = create_test_web_fetch_tool(10000, 10);
    
    let html = r#"<html><body><div class="main">Main</div><div class="sidebar">Sidebar</div></body></html>"#;
    
    let result = tool.extract_with_selector(html, "div");
    assert!(result.is_ok());
    let extracted = result.unwrap();
    assert!(extracted.contains("Main"));
    assert!(extracted.contains("Sidebar"));
}

#[test]
fn test_web_fetch_css_selector_invalid_syntax() {
    let tool = create_test_web_fetch_tool(10000, 10);
    
    let html = r#"<html><body><p>Content</p></body></html>"#;
    
    let result = tool.extract_with_selector(html, "invalid[selector");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid CSS selector"));
}

#[test]
fn test_web_fetch_params_with_selector_deserialization() {
    let json = r#"{"url": "https://example.com", "selector": "div.content", "max_length": 5000}"#;
    let params: WebFetchParams = serde_json::from_str(json).unwrap();
    
    assert_eq!(params.url, "https://example.com");
    assert_eq!(params.selector, Some("div.content".to_string()));
    assert_eq!(params.max_length, 5000);
}

#[test]
fn test_web_fetch_params_default_max_length() {
    let json = r#"{"url": "https://example.com"}"#;
    let params: WebFetchParams = serde_json::from_str(json).unwrap();
    
    assert_eq!(params.url, "https://example.com");
    assert!(params.selector.is_none());
    assert_eq!(params.max_length, 10000);
}

#[test]
fn test_web_fetch_empty_html_returns_empty_string() {
    let tool = create_test_web_fetch_tool(10000, 10);
    
    let html = "";
    let text = tool.extract_text(html);
    assert_eq!(text, "");
    
    let html = "<html><body></body></html>";
    let text = tool.extract_text(html);
    assert_eq!(text.trim(), "");
}

#[test]
fn test_web_fetch_whitespace_normalized_to_single_spaces() {
    let tool = create_test_web_fetch_tool(10000, 10);
    
    let html = r#"<html><body>
        <p>   Multiple   spaces   </p>
        <div>
            Newlines
            and tabs		
        </div>
    </body></html>"#;
    
    let text = tool.extract_text(html);
    assert!(text.contains("Multiple spaces"));
    assert!(text.contains("Newlines and tabs"));
    assert!(!text.contains("   "));
}