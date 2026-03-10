//! # Tool Registry Tests
//!
//! Comprehensive tests for MCP tool registry functionality including:
//! - Tool registration and discovery
//! - OpenAI tool format conversion
//! - Tool execution and parameter validation
//! - Edge cases and error handling

use aether_matrix::mcp::{
    BuiltinToolsConfig, Tool, ToolDefinition, ToolRegistry, ToolResult, WebFetchConfig,
};
use aether_matrix::mcp::builtin::BuiltInTools;
use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObject};
use std::sync::Arc;

/// Test creating an empty tool registry (no builtin tools enabled)
#[test]
fn test_empty_registry_creation() {
    let config = BuiltinToolsConfig {
        enabled: false,
        web_fetch: WebFetchConfig::default(),
    };
    
    let registry = ToolRegistry::new(&config);
    assert!(registry.is_empty());
}

/// Test that is_empty() returns false when tools are registered
#[test]
fn test_registry_not_empty_when_tools_present() {
    let config = BuiltinToolsConfig {
        enabled: true,
        web_fetch: WebFetchConfig {
            enabled: true,
            max_length: 10000,
            timeout: 10,
        },
    };
    
    let registry = ToolRegistry::new(&config);
    assert!(!registry.is_empty());
}

/// Test manual tool registration
#[test]
fn test_manual_tool_registration() {
    let mut registry = ToolRegistry::new(&BuiltinToolsConfig {
        enabled: false,
        web_fetch: WebFetchConfig::default(),
    });
    
    // Create a mock tool for testing
    struct MockTool;
    
    impl Tool for MockTool {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "mock_tool".to_string(),
                description: "A mock tool for testing".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "test_param": {"type": "string"}
                    },
                    "required": ["test_param"]
                }),
            }
        }
        
        async fn execute(&self, _arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
            Ok(ToolResult {
                success: true,
                content: "mock result".to_string(),
                error: None,
            })
        }
    }
    
    let tool = Arc::new(MockTool);
    registry.register(tool);
    
    assert!(!registry.is_empty());
    let tools = registry.to_openai_tools();
    assert_eq!(tools.len(), 1);
    
    if let ChatCompletionTools::Function(ChatCompletionTool { function }) = &tools[0] {
        assert_eq!(function.name, "mock_tool");
        assert_eq!(function.description, Some("A mock tool for testing".to_string()));
        assert!(function.parameters.is_some());
    } else {
        panic!("Expected Function tool type");
    }
}

/// Test OpenAI tool format conversion with web_fetch tool
#[test]
fn test_openai_tool_conversion() {
    let config = BuiltinToolsConfig {
        enabled: true,
        web_fetch: WebFetchConfig {
            enabled: true,
            max_length: 5000,
            timeout: 5,
        },
    };
    
    let registry = ToolRegistry::new(&config);
    let openai_tools = registry.to_openai_tools();
    
    assert!(!openai_tools.is_empty());
    
    // Should have web_fetch tool
    let web_fetch_tool = openai_tools
        .iter()
        .find(|tool| match tool {
            ChatCompletionTools::Function(f) => f.function.name == "web_fetch",
            _ => false,
        })
        .expect("web_fetch tool should be present");
    
    if let ChatCompletionTools::Function(ChatCompletionTool { function }) = web_fetch_tool {
        assert_eq!(function.name, "web_fetch");
        assert!(function.description.is_some_and(|desc| !desc.is_empty()));
        assert!(function.parameters.is_some());
        
        // Verify parameters schema structure
        let params = function.parameters.as_ref().unwrap();
        assert!(params.is_object());
        let obj = params.as_object().unwrap();
        assert!(obj.contains_key("properties"));
        assert!(obj.contains_key("type"));
    } else {
        panic!("Expected Function tool type");
    }
}

/// Test ToolDefinition structure validation
#[test]
fn test_tool_definition_structure() {
    let tool_def = ToolDefinition {
        name: "test_tool".to_string(),
        description: "Test tool description".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "param1": {"type": "string"},
                "param2": {"type": "number"}
            }
        }),
    };
    
    assert_eq!(tool_def.name, "test_tool");
    assert_eq!(tool_def.description, "Test tool description");
    assert!(tool_def.parameters.is_object());
}

/// Test ToolResult serialization and deserialization
#[test]
fn test_tool_result_serialization() {
    let result = ToolResult {
        success: true,
        content: "test content".to_string(),
        error: None,
    };
    
    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: ToolResult = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.success, true);
    assert_eq!(deserialized.content, "test content");
    assert_eq!(deserialized.error, None);
}

/// Test ToolResult with error serialization
#[test]
fn test_tool_result_with_error_serialization() {
    let result = ToolResult {
        success: false,
        content: String::new(),
        error: Some("Something went wrong".to_string()),
    };
    
    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: ToolResult = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.success, false);
    assert_eq!(deserialized.content, "");
    assert_eq!(deserialized.error, Some("Something went wrong".to_string()));
}

/// Test registry creation with disabled web_fetch tool
#[test]
fn test_registry_with_disabled_web_fetch() {
    let config = BuiltinToolsConfig {
        enabled: true,
        web_fetch: WebFetchConfig {
            enabled: false,  // web_fetch disabled
            max_length: 10000,
            timeout: 10,
        },
    };
    
    let registry = ToolRegistry::new(&config);
    assert!(registry.is_empty());  // Should be empty since web_fetch is the only builtin tool
}

/// Test that multiple registrations work correctly
#[test]
fn test_multiple_tool_registrations() {
    let mut registry = ToolRegistry::new(&BuiltinToolsConfig {
        enabled: false,
        web_fetch: WebFetchConfig::default(),
    });
    
    struct MockTool1;
    struct MockTool2;
    
    impl Tool for MockTool1 {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "mock_tool_1".to_string(),
                description: "Mock tool 1".to_string(),
                parameters: serde_json::json!({}),
            }
        }
        async fn execute(&self, _arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
            Ok(ToolResult::default())
        }
    }
    
    impl Tool for MockTool2 {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "mock_tool_2".to_string(),
                description: "Mock tool 2".to_string(),
                parameters: serde_json::json!({}),
            }
        }
        async fn execute(&self, _arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
            Ok(ToolResult::default())
        }
    }
    
    registry.register(Arc::new(MockTool1));
    registry.register(Arc::new(MockTool2));
    
    assert_eq!(registry.to_openai_tools().len(), 2);
    assert!(!registry.is_empty());
}

/// Test that tool registration overwrites existing tools with same name
#[test]
fn test_tool_registration_overwrites() {
    let mut registry = ToolRegistry::new(&BuiltinToolsConfig {
        enabled: false,
        web_fetch: WebFetchConfig::default(),
    });
    
    struct MockToolA;
    struct MockToolB;
    
    impl Tool for MockToolA {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "same_name".to_string(),
                description: "Tool A".to_string(),
                parameters: serde_json::json!({}),
            }
        }
        async fn execute(&self, _arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
            Ok(ToolResult { success: true, content: "A".to_string(), error: None })
        }
    }
    
    impl Tool for MockToolB {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "same_name".to_string(),
                description: "Tool B".to_string(),
                parameters: serde_json::json!({}),
            }
        }
        async fn execute(&self, _arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
            Ok(ToolResult { success: true, content: "B".to_string(), error: None })
        }
    }
    
    registry.register(Arc::new(MockToolA));
    assert_eq!(registry.to_openai_tools().len(), 1);
    
    // Register second tool with same name - should overwrite
    registry.register(Arc::new(MockToolB));
    assert_eq!(registry.to_openai_tools().len(), 1);
    
    // The description should be from Tool B now
    let tools = registry.to_openai_tools();
    if let ChatCompletionTools::Function(ChatCompletionTool { function }) = &tools[0] {
        assert_eq!(function.description, Some("Tool B".to_string()));
    }
}

/// Test execute_tool method with valid tool
#[tokio::test]
async fn test_execute_valid_tool() {
    let mut registry = ToolRegistry::new(&BuiltinToolsConfig {
        enabled: false,
        web_fetch: WebFetchConfig::default(),
    });
    
    struct ValidMockTool;
    
    impl Tool for ValidMockTool {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "valid_tool".to_string(),
                description: "Valid mock tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"}
                    }
                }),
            }
        }
        
        async fn execute(&self, arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
            // Verify arguments structure
            if let Some(input) = arguments.get("input").and_then(|v| v.as_str()) {
                Ok(ToolResult {
                    success: true,
                    content: format!("Processed: {}", input),
                    error: None,
                })
            } else {
                Ok(ToolResult {
                    success: false,
                    content: String::new(),
                    error: Some("Missing 'input' parameter".to_string()),
                })
            }
        }
    }
    
    registry.register(Arc::new(ValidMockTool));
    
    let arguments = serde_json::json!({"input": "test data"});
    let result = registry.execute_tool("valid_tool", arguments).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.content, "Processed: test data");
    assert_eq!(result.error, None);
}

/// Test execute_tool method with invalid/non-existent tool
#[tokio::test]
async fn test_execute_invalid_tool() {
    let registry = ToolRegistry::new(&BuiltinToolsConfig {
        enabled: false,
        web_fetch: WebFetchConfig::default(),
    });
    
    // Try to execute a tool that doesn't exist
    let arguments = serde_json::json!({});
    let result = registry.execute_tool("non_existent_tool", arguments).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Tool not found"));
}

/// Test execute_tool with invalid arguments
#[tokio::test]
async fn test_execute_tool_with_invalid_arguments() {
    let mut registry = ToolRegistry::new(&BuiltinToolsConfig {
        enabled: false,
        web_fetch: WebFetchConfig::default(),
    });
    
    struct ValidationMockTool;
    
    impl Tool for ValidationMockTool {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "validation_tool".to_string(),
                description: "Tool that validates arguments".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "required_field": {"type": "string"}
                    },
                    "required": ["required_field"]
                }),
            }
        }
        
        async fn execute(&self, arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
            if arguments.get("required_field").is_none() {
                return Err(anyhow::anyhow!("Missing required_field"));
            }
            Ok(ToolResult::default())
        }
    }
    
    registry.register(Arc::new(ValidationMockTool));
    
    // Call with missing required field
    let arguments = serde_json::json!({});
    let result = registry.execute_tool("validation_tool", arguments).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing required_field"));
}

// Implement Default for ToolResult to make tests cleaner
impl Default for ToolResult {
    fn default() -> Self {
        Self {
            success: true,
            content: String::new(),
            error: None,
        }
    }
}