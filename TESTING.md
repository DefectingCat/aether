# Testing Guide

This document describes the testing conventions, structure, and patterns used in the Aether Matrix Bot project.

## Test Organization

### Directory Structure

```
tests/
├── common/                     # Shared test utilities
│   ├── mod.rs                 # Module exports and re-exports
│   ├── mock_client.rs         # Mock Matrix client implementation
│   ├── mock_room.rs           # Mock room and message sender
│   ├── test_helpers.rs        # Stream helpers and test fixtures
│   └── test_utils.rs          # Logging and temp directory utilities
├── mcp/                       # MCP module tests
│   ├── config_tests.rs        # Configuration parsing tests
│   ├── retry_mechanism_tests.rs # Tool execution retry logic
│   ├── server_manager_tests.rs  # Server lifecycle tests
│   ├── tool_registry_tests.rs   # Tool registration tests
│   └── builtin_tools_tests.rs   # Built-in tool tests
├── admin_commands.rs          # Bot admin command tests
├── persona_commands.rs        # Persona management tests
├── mcp_commands.rs            # MCP command tests
├── muyu_commands.rs           # Cyber wooden fish tests
├── database_integration.rs    # Database and migration tests
├── bot_integration.rs         # Bot initialization tests
├── ai_service_integration.rs  # AI service tests
├── event_handler_integration.rs # Event handling tests
└── mcp_integration.rs         # MCP integration tests
```

### Unit vs Integration Tests

- **Unit tests**: Test individual functions, structs, and modules in isolation. Often placed in the same file as the code being tested using `#[cfg(test)]` modules.

- **Integration tests**: Test how multiple components work together. Placed in the `tests/` directory as separate files.

## Naming Conventions

### Test Function Names

Test functions should use `snake_case` and follow the pattern `test_<subject>_<scenario>`:

```rust
// Good: Clear what is being tested
#[test]
fn test_mcp_config_defaults() { }

#[test]
fn test_env_overrides_mcp_enabled_false() { }

#[tokio::test]
async fn test_success_on_first_attempt() { }

// Bad: Unclear what is being tested
#[test]
fn defaults() { }

#[test]
fn test_it_works() { }
```

### Test Module Names

Group related tests in modules with descriptive names:

```rust
#[cfg(test)]
mod basic_tests {
    // Basic functionality tests
}

#[cfg(test)]
mod permission_tests {
    // Permission-related tests
}

#[cfg(test)]
mod store_tests {
    // Database store tests
}
```

### Helper Function Names

Helper functions should be descriptive and indicate their purpose:

```rust
fn create_test_store() -> (PersonaStore, TempDir) { }
fn create_temp_dir() -> TempDir { }
fn init_test_logging() { }
```

## Test Patterns

### Setup/Teardown Pattern

Use RAII types like `TempDir` for automatic cleanup:

```rust
use tempfile::TempDir;

fn create_test_store() -> (PersonaStore, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
    let db = Database::new(&db_path).unwrap();
    let store = PersonaStore::new(db.conn().clone());
    store.init_builtin_personas().unwrap();
    (store, temp_dir) // temp_dir keeps the directory alive
}
// When temp_dir is dropped, the directory is cleaned up
```

### Mock Trait Pattern

Define traits for external dependencies and implement mock versions:

```rust
// Define trait for the abstraction
pub trait MatrixClient {
    fn user_id(&self) -> Option<OwnedUserId>;
    async fn join_room_by_id(&self, room_id: &RoomId) -> Result<()>;
}

// Implement mock for testing
#[derive(Clone)]
pub struct MockClient {
    user_id: Option<OwnedUserId>,
    joined_rooms: Arc<Mutex<Vec<OwnedRoomId>>>,
    join_should_fail: bool,
}

impl MatrixClient for MockClient {
    fn user_id(&self) -> Option<OwnedUserId> {
        self.user_id.clone()
    }

    async fn join_room_by_id(&self, room_id: &RoomId) -> Result<()> {
        if self.join_should_fail {
            anyhow::bail!("Failed to join room");
        }
        self.joined_rooms.lock().await.push(room_id.to_owned());
        Ok(())
    }
}
```

### Message Recording Pattern

Record sent messages for verification:

```rust
pub struct MockRoom {
    pub sent_messages: Arc<Mutex<Vec<(String, Option<OwnedEventId>)>>>,
}

impl MessageSender for MockRoom {
    async fn send(&self, content: &str) -> Result<OwnedEventId> {
        let event_id = self.next_event_id();
        self.sent_messages.lock().await.push((content.to_string(), Some(event_id.clone())));
        Ok(event_id)
    }
}

// In test
let room = MockRoom::new();
handler.execute(&ctx).await.unwrap();
let messages = room.get_messages().await;
assert!(messages[0].0.contains("expected content"));
```

### Configuration Testing Pattern

Test default values, environment variable overrides, and TOML parsing:

```rust
#[test]
fn test_mcp_config_defaults() {
    let config = McpConfig::default();
    assert!(config.enabled);
    assert!(config.builtin_tools.enabled);
    assert_eq!(config.external_servers.len(), 0);
}

#[test]
fn test_env_overrides_mcp_enabled_false() {
    env::set_var("MCP_ENABLED", "false");
    let mut config = McpConfig::default();
    config.apply_env_overrides();
    assert!(!config.enabled);
    env::remove_var("MCP_ENABLED");
}

#[test]
fn test_toml_config_parsing_full_config() {
    let toml_str = r#"
        enabled = false
        [builtin_tools]
        enabled = false
    "#;
    let config: McpConfig = toml::from_str(toml_str).expect("TOML parsing should succeed");
    assert!(!config.enabled);
}
```

### Async Test Pattern

Use `#[tokio::test]` for async tests:

```rust
#[tokio::test]
async fn test_create_custom_persona_works() {
    let (store, _temp_dir) = create_test_store();
    
    let custom_persona = Persona {
        id: "custom-test".to_string(),
        name: "Custom Test".to_string(),
        system_prompt: "Custom test prompt".to_string(),
        avatar_emoji: Some("X".to_string()),
        is_builtin: false,
        created_by: Some("@user:matrix.org".to_string()),
    };
    
    store.create_persona(&custom_persona).unwrap();
    
    let retrieved = store.get_by_id("custom-test").unwrap().unwrap();
    assert_eq!(retrieved.name, "Custom Test");
}
```

### Concurrent Testing Pattern

Test thread safety with multi-threaded access:

```rust
#[tokio::test]
async fn test_multi_threaded_concurrent_access() {
    let temp_dir = create_temp_dir();
    let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
    let db = Arc::new(Database::new(&db_path).unwrap());
    
    let mut handles = vec![];
    for i in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            let conn = db_clone.conn().lock().unwrap();
            conn.execute("INSERT INTO personas ...", [...]).unwrap();
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

### State Machine Testing Pattern

Test state transitions with atomic counters:

```rust
struct FailingMockTool {
    call_count: AtomicU32,
    fail_until: u32,
}

impl Tool for FailingMockTool {
    async fn execute(&self, _args: Value) -> Result<ToolResult> {
        let current_call = self.call_count.fetch_add(1, Ordering::Relaxed) + 1;
        if current_call <= self.fail_until {
            return Err(anyhow!("Tool failed on attempt {}", current_call));
        }
        Ok(ToolResult { success: true, ... })
    }
}
```

### Error Handling Testing Pattern

Test both success and failure paths:

```rust
#[tokio::test]
async fn test_invalid_database_path_error_handling() {
    let invalid_path = "/root/protected/test.db";
    let result = Database::new(invalid_path);
    
    assert!(result.is_err(), "Should fail for invalid path");
    
    match result {
        Err(error) => {
            let error_str = error.to_string();
            assert!(
                error_str.contains("denied") || 
                error_str.contains("Permission") ||
                error_str.contains("No such file"),
                "Error should indicate access issue"
            );
        }
        Ok(_) => panic!("Expected error but got success"),
    }
}
```

### Idempotency Testing Pattern

Test that operations can be safely repeated:

```rust
#[tokio::test]
async fn test_migration_idempotency() {
    let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
    
    // Create database (runs migrations)
    let db1 = Database::new(&db_path).unwrap();
    
    // Create another instance (runs migrations again)
    let db2 = Database::new(&db_path).unwrap();
    
    // Both should work without errors
    let count1: i32 = db1.conn().lock().query_row(...);
    let count2: i32 = db2.conn().lock().query_row(...);
    
    assert_eq!(count1, count2);
}
```

## Test Utilities

### Logging Initialization

```rust
use std::sync::Once;

static INIT: Once = Once::new();

fn init_test_logging() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
            )
            .with_test_writer()
            .init();
    });
}
```

### Temporary Directory

```rust
use tempfile::TempDir;

fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

// Usage
let temp_dir = create_temp_dir();
let db_path = temp_dir.path().join("test.db");
```

### Stream Helpers

```rust
pub fn create_test_stream_with_state(
    chunks: Vec<String>,
    state: Arc<Mutex<StreamingState>>,
) -> Pin<Box<dyn Stream<Item = Result<String>> + Send>> {
    use futures_util::stream;
    Box::pin(stream::iter(chunks).then(move |chunk| {
        let state = state.clone();
        async move {
            state.lock().await.append(&chunk);
            Ok(chunk)
        }
    }))
}
```

## Running Tests

```bash
# Run all tests
make test

# Run specific test file
cargo test --test admin_commands

# Run specific test function
cargo test test_bot_info_handler_name

# Run tests matching a pattern
cargo test persona

# Show test output
cargo test -- --nocapture

# Run tests in parallel with specific threads
cargo test -- --test-threads=4

# Run ignored tests
cargo test -- --ignored
```

## Best Practices

1. **One assertion per concept**: Group related assertions but keep tests focused.

2. **Descriptive assertion messages**: Use assertion messages to clarify what failed.

```rust
assert_eq!(
    result,
    expected,
    "Tool should succeed on attempt {}",
    current_attempt
);
```

3. **Test edge cases**: Zero values, empty strings, maximum values, Unicode.

```rust
#[test]
fn test_web_fetch_zero_values() {
    env::set_var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH", "0");
    // Verify zero is accepted
}

#[test]
fn test_web_fetch_large_values() {
    env::set_var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH", "1000000");
    // Verify large values are accepted
}
```

4. **Clean up environment variables**: Always remove test environment variables.

```rust
env::set_var("TEST_VAR", "value");
// ... test code ...
env::remove_var("TEST_VAR");
```

5. **Use `_` prefix for unused values**: Prevent compiler warnings about unused variables.

```rust
let (store, _temp_dir) = create_test_store(); // temp_dir must not be dropped
```

6. **Test both success and failure paths**: Don't only test the happy path.

7. **Avoid test interdependence**: Each test should be independent and runnable in isolation.

8. **Use `serial_test` for tests that share global state**:

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_modifies_global_state_1() { }

#[test]
#[serial]
fn test_modifies_global_state_2() { }
```

## Common Test Dependencies

```toml
[dev-dependencies]
tokio-test = "0.4"       # Tokio testing utilities
wiremock = "0.6"         # HTTP mocking
mockall = "0.14"         # Mock generation
tempfile = "3"           # Temporary files/directories
lazy_static = "1"        # Static variables (prefer std::sync::LazyLock in new code)
tokio-stream = "0.1"     # Stream utilities
futures = "0.3"          # Future utilities
serial_test = "3"        # Serial test execution
```