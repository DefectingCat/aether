# Chat History Functionality Issues

## Issue: Missing Chat History Implementation

**Date**: Tue Mar 10 2026

**Description**: 
The database schema includes a `chat_history` table (defined in `migrations/20260305000000_init.sql`), but there is no actual implementation that uses this table for persistent chat history storage.

**Current State**:
- ✅ `chat_history` table exists in database schema
- ✅ Table has proper structure with indexes (`room_id`, `created_at`)
- ❌ No methods in `Database` struct to interact with `chat_history`
- ❌ Conversation history is managed entirely in memory via `ConversationManager`
- ❌ No integration between conversation management and database persistence

**Evidence**:
- `src/store/database.rs` contains only connection and migration logic, no chat history methods
- `src/conversation.rs` manages all conversation state in memory using `HashMap<String, Vec<ChatCompletionRequestMessage>>`
- `src/ai_service.rs` uses only the in-memory `ConversationManager`
- No SQL queries found that interact with `chat_history` table
- Grep search confirms no usage of `chat_history` outside of schema definition and test verification

**Impact**:
- Chat history is lost when the bot restarts
- No persistent conversation history across sessions
- Database table is unused (wasted storage)

**Recommendation**:
Implement chat history persistence by:
1. Adding methods to `Database` struct for saving/loading chat history
2. Integrating database persistence into `ConversationManager` 
3. Creating comprehensive tests for chat history functionality
---

## Issue: cargo-tarpaulin Not Available for Test Coverage

**Date**: Tue Mar 10 2026

**Description**:
`cargo-tarpaulin` (Rust code coverage tool) is not installed and cannot be used to generate test coverage reports.

**Current State**:
- ❌ `cargo tarpaulin` command not found
- ❌ No alternative coverage tool configured
- ❌ No coverage reporting scripts available
- ❌ No `.codecov.yml` or similar configuration

**Impact**:
- Cannot measure test coverage percentage
- Cannot identify untested code paths
- Cannot track coverage trends over time
- No visibility into testing gaps

**Attempted Resolution**:
```bash
cargo tarpaulin --version
# Result: error: no such command: `tarpaulin`
```

**Recommended Actions**:
1. Install cargo-tarpaulin: `cargo install cargo-tarpaulin`
2. Create coverage script: `scripts/coverage.sh`
3. Add coverage configuration to project root
4. Consider CI integration (GitHub Actions, Codecov)

**Platform Note**:
cargo-tarpaulin requires Linux and may not work on macOS. Alternative tools:
- `cargo-llvm-cov` (cross-platform, LLVM-based)
- `tarpaulin` in Docker/Linux CI only
