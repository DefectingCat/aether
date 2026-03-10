
## 2025-03-10: Admin Command Handlers Documentation

### Pattern Applied
For command handler documentation, follow this structure:
1. **Struct documentation**: Brief description → Detailed description → `# 权限` section → `# Example` block
2. **Method documentation**: One-line description + `# Arguments` / `# Returns` sections for `execute()`
3. **Example blocks**: Use ` ```ignore ` to prevent compilation attempts

### Key Insight
The `BotInfoHandler` is a multi-subcommand handler where the base `permission()` returns `Anyone`, but individual subcommands have their own runtime permission checks. Document this clearly in the struct-level `# 权限` section.

### Template Adherence
- Chinese for user-facing descriptions (description, usage)
- `# 权限` section required for non-Anyone commands (BotLeaveHandler uses RoomMod)
- All trait methods get doc comments even if trivial
- Private helper methods (`handle_info`, `handle_name`, etc.) NOT documented per task requirements

### Verification
`cargo doc --no-deps` compiles successfully - pre-existing warnings about private links are unrelated to documentation changes.

## 2025-03-10: MCP Command Handlers Documentation

### Pattern Applied
Same pattern as admin handlers, with multi-subcommand documentation:
1. **Struct documentation**: Brief description → Detailed description → `# 子命令` table → `# 权限` section → `# Example` block
2. **Permission note**: Since `reload` requires BotOwner but base `permission()` returns Anyone, document this clearly in both the table and the `# 权限` section
3. **Method documentation**: All trait methods documented, `execute()` has detailed dispatch description

### Key Insight
For multi-subcommand handlers with mixed permissions:
- Use `# 子命令` table to list all subcommands with their individual permissions
- Document runtime permission checks in `# 权限` section and `permission()` doc
- Keep base `permission()` returning the lowest common denominator

### Template Adherence
- Chinese for user-facing descriptions
- `# 权限` section required (even for Anyone, to explain subcommand permissions)
- ` ```ignore ` for example blocks
- Private helper methods NOT documented (handle_list, handle_servers, etc.)

### Verification
`cargo doc --no-deps` compiles successfully. Pre-existing warnings about private links in other files are unrelated to documentation changes.

## 2025-03-10: Persona Command Handler Documentation

### Pattern Applied
Same pattern as admin handlers, but with important distinction:
- `PersonaHandler` is a multi-subcommand handler
- Base `permission()` returns `Anyone`, but individual subcommands have runtime checks
- Document the subcommand list clearly in struct documentation
- Note which subcommands require elevated permissions in `# 权限` section

### Multi-Subcommand Handler Pattern
For handlers with runtime permission checks:
1. Struct-level `permission()` returns least restrictive level (`Anyone`)
2. Individual handlers (`handle_set`, `handle_create`, etc.) check permissions explicitly
3. Document this clearly: "基础命令任何房间成员都可以执行。管理命令需要房间管理员权限。"

### Documentation Structure
- Struct: brief → detailed (subcommand list) → `# 权限` → `# Example`
- `new()`: Arguments section for the `store` parameter
- `execute()`: Lists all subcommand routes + Arguments + Returns sections
- Other trait methods: brief one-liner descriptions

### Verification
`cargo doc --no-deps` compiles successfully with only pre-existing warnings about private link references.
