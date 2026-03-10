# Documentation Improvement Plan

## TL;DR

> **Quick Summary**: Improve code documentation across the Aether Matrix Bot project by adding comprehensive rustdoc comments, examples, and doctests to poorly documented files while following established patterns from well-documented core files.
> 
> **Deliverables**: 
> - Complete documentation for command handlers (`modules/*/handlers.rs`)
> - Comprehensive documentation for data structures (`store/persona_store.rs`, `store/database.rs`)
> - Unified documentation templates for consistent command handler documentation
> - Runnable doctests for all new documentation
> 
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Core documentation templates → Command handlers → Data structures → Integration tests

---

## Context

### Original Request
Analyze current project comments and provide a comprehensive documentation improvement plan.

### Interview Summary
**Key Discussions**:
- Project has excellent documentation in core files (main.rs, lib.rs, bot.rs, ai_service.rs, traits.rs)
- Module-level documentation using `//!` is well-established
- Function documentation follows consistent patterns with Arguments/Returns/Errors sections
- Major gaps exist in modules/admin/handlers.rs, store/persona_store.rs, and new mcp modules
- Project uses standard Rust test framework with extensive doctests (43 tests)

**Research Findings**:
- Well-documented files: `main.rs`, `lib.rs`, `bot.rs`, `ai_service.rs`, `traits.rs`, `config.rs`, `command/*`
- Poorly documented files: `modules/admin/handlers.rs`, `modules/persona/handlers.rs`, `store/persona_store.rs`, `store/database.rs`, `modules/mcp/handlers.rs`
- Established patterns include module-level docs, struct/enum docs, method docs with standardized sections, and doctests

**Metis Review**
**Identified Gaps** (addressed):
- Prioritized command handlers first (highest user impact)
- Created unified templates based on existing well-documented files
- Ensured all acceptance criteria are executable by agents
- Set clear scope boundaries to prevent scope creep

---

## Work Objectives

### Core Objective
Improve code documentation across the Aether Matrix Bot project by adding comprehensive rustdoc comments, examples, and doctests to poorly documented files while following established patterns from well-documented core files.

### Concrete Deliverables
- Complete documentation for `modules/admin/handlers.rs`
- Complete documentation for `modules/persona/handlers.rs`  
- Complete documentation for `modules/mcp/handlers.rs`
- Comprehensive documentation for `store/persona_store.rs`
- Comprehensive documentation for `store/database.rs`
- Unified documentation templates for command handlers
- Runnable doctests for all new documentation

### Definition of Done
- [ ] All target files have comprehensive rustdoc documentation following established patterns
- [ ] All public structs, enums, traits, and methods have proper documentation
- [ ] All documentation includes `# Arguments`, `# Returns`, `# Errors` sections where applicable
- [ ] All documentation includes runnable doctests or `no_run` examples
- [ ] `cargo test --doc` passes without errors
- [ ] `cargo doc --no-deps` compiles without warnings

### Must Have
- Follow documentation patterns from `[src/config.rs:29-389]`, `[src/ai_service.rs:1-43]`, and `[src/command/registry.rs:33-102]`
- Document all public structs, enums, traits, and impl methods in target files
- Include `# Arguments`, `# Returns`, `# Errors` sections for all public methods
- Add runnable doctests where possible, use `no_run` for external dependencies
- Focus on public APIs first, add detailed examples incrementally

### Must NOT Have (Guardrails)
- Invent new documentation patterns when existing ones work
- Add documentation to private/internal items unless they're critical for understanding
- Create documentation that requires human review for validation
- Add documentation that doesn't follow the established Arguments/Returns/Errors pattern
- Include vague or unverifiable acceptance criteria

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.
> Acceptance criteria requiring "user manually tests/confirms" are FORBIDDEN.

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: TDD
- **Framework**: cargo test (rustdoc)
- **If TDD**: Each task follows RED (failing test) → GREEN (minimal impl) → REFACTOR

### QA Policy
Every task MUST include agent-executed QA scenarios (see TODO template below).
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Library/Module**: Use Bash (cargo commands) — Compile, run doctests, verify output
- **All tasks**: Use Bash to verify documentation compilation and test execution

---

## Execution Strategy

### Parallel Execution Waves

> Maximize throughput by grouping independent tasks into parallel waves.
> Each wave completes before the next begins.
> Target: 5-8 tasks per wave. Fewer than 3 per wave (except final) = under-splitting.

```
Wave 1 (Start Immediately — foundation + templates):
├── Task 1: Create unified documentation templates for command handlers
├── Task 2: Create struct documentation template based on config.rs
├── Task 3: Create method documentation template based on ai_service.rs
└── Task 4: Verify templates compile with cargo doc

Wave 2 (After Wave 1 — command handlers):
├── Task 5: Document admin command handlers (BotInfoHandler, BotLeaveHandler, BotPingHandler)
├── Task 6: Document persona command handlers
├── Task 7: Document MCP command handlers
└── Task 8: Add doctests for all command handler documentation

Wave 3 (After Wave 2 — data structures):
├── Task 9: Document PersonaStore and Persona struct
├── Task 10: Document Database connection management
├── Task 11: Add doctests for data structure documentation
└── Task 12: Verify all documentation compiles

Wave 4 (After Wave 3 — integration + verification):
├── Task 13: Run comprehensive doctest suite
├── Task 14: Verify documentation renders correctly
├── Task 15: Final integration test
└── Task 16: Git cleanup + tagging

Wave FINAL (After ALL tasks — independent review, 4 parallel):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)

Critical Path: Task 1 → Task 5 → Task 9 → Task 13 → F1-F4
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 4 (Waves 1-3)
```

### Dependency Matrix (abbreviated — show ALL tasks in your generated plan)

- **1-4**: — — 5-8, 1
- **5-8**: 1-4 — 9-12, 2
- **9-12**: 5-8 — 13-16, 3
- **13-16**: 9-12 — F1-F4, 4

### Agent Dispatch Summary

- **1**: **4** — T1-T4 → `writing`
- **2**: **4** — T5-T8 → `writing`
- **3**: **4** — T9-T12 → `writing`
- **4**: **4** — T13-T16 → `quick`
- **FINAL**: **4** — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

> Implementation + Test = ONE Task. Never separate.
> EVERY task MUST have: Recommended Agent Profile + Parallelization info + QA Scenarios.
> **A task WITHOUT QA Scenarios is INCOMPLETE. No exceptions.**

- [x] 1. Create unified documentation templates for command handlers

  **What to do**:
  - Create a unified template for command handler documentation based on existing patterns from `command/registry.rs`
  - Template should include: struct documentation, name/description/usage methods, execute method with Arguments/Returns/Errors sections
  - Include example usage with `# Example` section showing command syntax
  - Save as a reference template for Tasks 5-7

  **Must NOT do**:
  - Invent new documentation patterns not found in existing well-documented files
  - Add documentation for private/internal methods

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `writing`
    - Reason: This task involves creating documentation templates and writing clear, consistent documentation patterns
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Need to follow Rust documentation best practices and existing project patterns

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3, 4)
  - **Blocks**: Tasks 5, 6, 7 (command handler documentation)
  - **Blocked By**: None (can start immediately)

  **References** (CRITICAL - Be Exhaustive):

  **Pattern References** (existing code to follow):
  - `src/command/registry.rs:33-102` - CommandHandler trait documentation pattern with Required/Optional methods
  - `src/command/mod.rs:1-72` - Module-level documentation with architecture diagram and examples
  - `src/modules/admin/handlers.rs:10-15` - Basic struct documentation that needs to be expanded

  **WHY Each Reference Matters**:
  - `command/registry.rs` shows the established pattern for documenting traits with clear Required vs Optional method sections
  - `command/mod.rs` demonstrates how to provide comprehensive module-level documentation with examples
  - `admin/handlers.rs` shows the current minimal state that needs to be improved

  **Acceptance Criteria**:

  **If TDD (tests enabled):**
  - [ ] Template file created with proper rustdoc syntax
  - [ ] cargo doc --no-deps compiles without warnings

  **QA Scenarios (MANDATORY — task is INCOMPLETE without these):**

  ```
  Scenario: Template compiles correctly with cargo doc
    Tool: Bash
    Preconditions: Documentation template file exists
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo doc --no-deps --document-private-items
    Expected Result: Command completes successfully with exit code 0
    Failure Indicators: Compilation errors, rustdoc warnings
    Evidence: .sisyphus/evidence/task-1-template-compile.txt

  Scenario: Template follows established patterns
    Tool: Bash
    Preconditions: Template file exists
    Steps:
      1. grep -A 10 -B 5 "CommandHandler" src/command/registry.rs
      2. Compare template structure with established pattern
    Expected Result: Template includes all required sections: struct docs, method docs, Examples
    Evidence: .sisyphus/evidence/task-1-pattern-compliance.txt
  ```

  **Evidence to Capture**:
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Terminal output for compilation and pattern verification

  **Commit**: NO (groups with Task 4)
  - Message: docs(command): create unified documentation template
  - Files: template file
  - Pre-commit: cargo doc --no-deps

- [x] 2. Create struct documentation template based on config.rs

  **What to do**:
  - Analyze `config.rs` documentation pattern for structs
  - Create a template for documenting data structures like `Persona`, `PersonaStore`, etc.
  - Template should include: field-level documentation, constructor documentation, usage examples
  - Save as reference template for Task 9

  **Must NOT do**:
  - Add documentation for private fields unless critical for understanding
  - Deviate from the established `config.rs` pattern

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Creating documentation templates requires clear writing and following established patterns
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Ensure adherence to Rust documentation standards

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3, 4)
  - **Blocks**: Task 9 (PersonaStore documentation)
  - **Blocked By**: None (can start immediately)

  **References**:
  - `src/config.rs:29-389` - Comprehensive struct documentation with field-level comments and examples
  - `src/store/persona_store.rs:33-479` - Current minimal documentation that needs improvement

  **WHY Each Reference Matters**:
  - `config.rs` demonstrates the gold standard for struct documentation in this project
  - `persona_store.rs` shows the target file that needs to be improved using this template

  **Acceptance Criteria**:
  - [ ] Template file created with proper rustdoc syntax for structs
  - [ ] cargo doc --no-deps compiles without warnings

  **QA Scenarios**:
  ```
  Scenario: Struct template compiles correctly
    Tool: Bash
    Preconditions: Struct template file exists
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo doc --no-deps --document-private-items
    Expected Result: Command completes successfully with exit code 0
    Failure Indicators: Compilation errors, rustdoc warnings about struct documentation
    Evidence: .sisyphus/evidence/task-2-struct-template-compile.txt
  ```

  **Commit**: NO (groups with Task 4)

- [x] 3. Create method documentation template based on ai_service.rs

  **What to do**:
  - Analyze `ai_service.rs` documentation pattern for methods
  - Create a template for documenting complex methods with multiple parameters
  - Template should include: `# Arguments`, `# Returns`, `# Errors`, `# Example` sections
  - Save as reference template for Tasks 5-7 and 9-10

  **Must NOT do**:
  - Skip any of the standard sections (Arguments/Returns/Errors)
  - Use vague parameter descriptions

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Creating method documentation templates requires precise technical writing
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Ensure proper Rust documentation conventions

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 4)
  - **Blocks**: Tasks 5-7, 9-10 (all method documentation)
  - **Blocked By**: None (can start immediately)

  **References**:
  - `src/ai_service.rs:112-164` - Method documentation with Arguments/Returns/Errors sections
  - `src/ai_service.rs:166-204` - Another example of comprehensive method documentation
  - `src/bot.rs:98-100` - Bot::new method documentation with error conditions

  **WHY Each Reference Matters**:
  - `ai_service.rs` shows the established pattern for documenting methods with multiple parameters and complex error conditions
  - `bot.rs` demonstrates how to document error conditions clearly

  **Acceptance Criteria**:
  - [ ] Template file created with proper method documentation sections
  - [ ] cargo doc --no-deps compiles without warnings

  **QA Scenarios**:
  ```
  Scenario: Method template compiles correctly
    Tool: Bash
    Preconditions: Method template file exists
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo doc --no-deps --document-private-items
    Expected Result: Command completes successfully with exit code 0
    Failure Indicators: Compilation errors, rustdoc warnings about method documentation
    Evidence: .sisyphus/evidence/task-3-method-template-compile.txt
  ```

  **Commit**: NO (groups with Task 4)

- [x] 4. Verify templates compile with cargo doc

  **What to do**:
  - Combine all templates into actual documentation in the target files
  - Run `cargo doc --no-deps` to verify all templates compile correctly
  - Fix any compilation errors or warnings
  - Ensure templates can be applied to actual code

  **Must NOT do**:
  - Apply templates to actual code yet (save for later tasks)
  - Ignore compilation warnings

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: This is a verification task that should be quick once templates are created
  - **Skills**: []
    - No special skills needed for basic compilation verification

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 1 (sequential after Tasks 1-3)
  - **Blocks**: All subsequent tasks (5-16)
  - **Blocked By**: Tasks 1, 2, 3

  **References**:
  - All template files from Tasks 1-3
  - `src/modules/admin/handlers.rs` - Target file for template application

  **Acceptance Criteria**:
  - [ ] All templates compile successfully with `cargo doc --no-deps`
  - [ ] No rustdoc warnings or errors

  **QA Scenarios**:
  ```
  Scenario: All templates compile without errors
    Tool: Bash
    Preconditions: All template files exist
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo doc --no-deps --document-private-items
    Expected Result: Command completes successfully with exit code 0, no warnings
    Failure Indicators: Any compilation errors or warnings
    Evidence: .sisyphus/evidence/task-4-all-templates-compile.txt
  ```

  **Commit**: YES
  - Message: docs(templates): create unified documentation templates
  - Files: template files
  - Pre-commit: cargo doc --no-deps

- [x] 5. Document admin command handlers (BotInfoHandler, BotLeaveHandler, BotPingHandler)

  **What to do**:
  - Apply the unified command handler template to `BotInfoHandler` struct
  - Document all methods: name(), description(), usage(), permission(), execute()
  - Add comprehensive documentation for each handler method with Arguments/Returns/Errors
  - Include usage examples showing command syntax and expected responses
  - Apply same pattern to `BotLeaveHandler` and `BotPingHandler`

  **Must NOT do**:
  - Add documentation for private helper functions like `handle_info()`
  - Deviate from the established template pattern

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Requires detailed technical writing following established patterns
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Ensure consistency with existing documentation standards

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7, 8)
  - **Blocks**: Task 13 (comprehensive doctest suite)
  - **Blocked By**: Tasks 1, 2, 3, 4 (templates verified)

  **References**:
  - `src/modules/admin/handlers.rs:10-364` - Current file with minimal documentation
  - Template from Task 1 - Unified command handler documentation template
  - `src/command/registry.rs:33-102` - CommandHandler trait documentation pattern

  **WHY Each Reference Matters**:
  - `admin/handlers.rs` is the target file that needs comprehensive documentation
  - Template from Task 1 ensures consistency across all command handlers
  - `command/registry.rs` provides the established pattern for CommandHandler trait

  **Acceptance Criteria**:
  - [ ] All public structs (`BotInfoHandler`, `BotLeaveHandler`, `BotPingHandler`) have proper documentation
  - [ ] All public methods have Arguments/Returns/Errors sections
  - [ ] `cargo doc --no-deps` compiles without warnings
  - [ ] All documentation includes usage examples

  **QA Scenarios**:
  ```
  Scenario: Admin handlers documentation compiles
    Tool: Bash
    Preconditions: Admin handlers documentation added
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo doc --no-deps --document-private-items
    Expected Result: Command completes successfully with exit code 0
    Failure Indicators: Compilation errors, rustdoc warnings
    Evidence: .sisyphus/evidence/task-5-admin-docs-compile.txt

  Scenario: Admin handlers follow template pattern
    Tool: Bash
    Preconditions: Admin handlers documentation exists
    Steps:
      1. grep -A 20 "pub struct BotInfoHandler" src/modules/admin/handlers.rs
      2. Verify all required sections are present
    Expected Result: Documentation includes struct docs, method docs, Examples
    Evidence: .sisyphus/evidence/task-5-admin-template-compliance.txt
  ```

  **Commit**: NO (groups with Task 8)

- [x] 6. Document persona command handlers

  **What to do**:
  - Apply the unified command handler template to all persona command handlers in `modules/persona/handlers.rs`
  - Document structs like `PersonaListHandler`, `PersonaSetHandler`, etc.
  - Document all methods with Arguments/Returns/Errors sections
  - Include usage examples for persona commands

  **Must NOT do**:
  - Add documentation for private internal functions
  - Skip any public command handler structs

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Consistent technical writing following established patterns
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Maintain consistency with project documentation standards

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 7, 8)
  - **Blocks**: Task 13 (comprehensive doctest suite)
  - **Blocked By**: Tasks 1, 2, 3, 4 (templates verified)

  **References**:
  - `src/modules/persona/handlers.rs:1-308` - Current file with minimal documentation
  - Template from Task 1 - Unified command handler documentation template

  **Acceptance Criteria**:
  - [ ] All public persona command handler structs have proper documentation
  - [ ] All public methods have complete documentation sections
  - [ ] `cargo doc --no-deps` compiles without warnings

  **QA Scenarios**:
  ```
  Scenario: Persona handlers documentation compiles
    Tool: Bash
    Preconditions: Persona handlers documentation added
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo doc --no-deps --document-private-items
    Expected Result: Command completes successfully with exit code 0
    Failure Indicators: Compilation errors, rustdoc warnings
    Evidence: .sisyphus/evidence/task-6-persona-docs-compile.txt
  ```

  **Commit**: NO (groups with Task 8)

- [x] 7. Document MCP command handlers

  **What to do**:
  - Apply the unified command handler template to MCP command handlers in `modules/mcp/handlers.rs`
  - Document all public structs and methods
  - Add comprehensive documentation with usage examples for MCP commands
  - Ensure consistency with other command handler documentation

  **Must NOT do**:
  - Add redundant documentation for already well-documented parts
  - Skip any public command handler structs

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Technical writing for new MCP functionality
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Ensure MCP documentation follows project standards

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 8)
  - **Blocks**: Task 13 (comprehensive doctest suite)
  - **Blocked By**: Tasks 1, 2, 3, 4 (templates verified)

  **References**:
  - `src/modules/mcp/handlers.rs` - Current MCP handlers file
  - Template from Task 1 - Unified command handler documentation template

  **Acceptance Criteria**:
  - [ ] All public MCP command handler structs have proper documentation
  - [ ] All public methods have complete documentation sections
  - [ ] `cargo doc --no-deps` compiles without warnings

  **QA Scenarios**:
  ```
  Scenario: MCP handlers documentation compiles
    Tool: Bash
    Preconditions: MCP handlers documentation added
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo doc --no-deps --document-private-items
    Expected Result: Command completes successfully with exit code 0
    Failure Indicators: Compilation errors, rustdoc warnings
    Evidence: .sisyphus/evidence/task-7-mcp-docs-compile.txt
  ```

  **Commit**: NO (groups with Task 8)

- [x] 8. Add doctests for all command handler documentation

  **What to do**:
  - Add runnable doctests for all documented command handlers
  - For complex handlers that require external dependencies, use `no_run` or `ignore` examples
  - Ensure all examples are syntactically correct and follow established patterns
  - Test that all doctests pass with `cargo test --doc`

  **Must NOT do**:
  - Add doctests that require external services or complex setup
  - Skip doctests for any public API

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Writing executable documentation tests
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Ensure doctests follow Rust documentation best practices

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2 (sequential after Tasks 5, 6, 7)
  - **Blocks**: Task 13 (comprehensive doctest suite)
  - **Blocked By**: Tasks 5, 6, 7 (handlers documented)

  **References**:
  - `src/config.rs` - Examples of good doctests in the project
  - `src/conversation.rs` - More examples of comprehensive doctests
  - All command handler files updated in Tasks 5-7

  **Acceptance Criteria**:
  - [ ] All command handler documentation includes appropriate doctests
  - [ ] `cargo test --doc` passes without failures
  - [ ] Doctests use `no_run` or `ignore` for complex external dependencies

  **QA Scenarios**:
  ```
  Scenario: Command handler doctests pass
    Tool: Bash
    Preconditions: Doctests added to command handlers
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo test --doc
    Expected Result: All doctests pass, no failures
    Failure Indicators: Any doctest failures
    Evidence: .sisyphus/evidence/task-8-handler-doctests-pass.txt
  ```

  **Commit**: YES
  - Message: docs(handlers): document admin, persona, and mcp command handlers
  - Files: src/modules/*/handlers.rs
  - Pre-commit: cargo test --doc

- [x] 9. Document PersonaStore and Persona struct

  **What to do**:
  - Apply the struct documentation template to `Persona` struct in `store/persona_store.rs`
  - Document all fields with clear descriptions
  - Apply the struct documentation template to `PersonaStore` struct
  - Document all public methods with Arguments/Returns/Errors sections using the method template
  - Include usage examples showing how to use the PersonaStore

  **Must NOT do**:
  - Add documentation for private/internal methods
  - Skip field-level documentation for public structs

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Comprehensive struct and method documentation
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Follow struct documentation patterns from config.rs

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 10, 11, 12)
  - **Blocks**: Task 13 (comprehensive doctest suite)
  - **Blocked By**: Tasks 1, 2, 3, 4 (templates verified)

  **References**:
  - `src/store/persona_store.rs:33-479` - Current file with minimal documentation
  - Template from Task 2 - Struct documentation template based on config.rs
  - Template from Task 3 - Method documentation template based on ai_service.rs

  **WHY Each Reference Matters**:
  - `persona_store.rs` is the target file that needs comprehensive documentation
  - Struct template ensures consistency with other well-documented structs like Config
  - Method template ensures consistent method documentation across the codebase

  **Acceptance Criteria**:
  - [ ] `Persona` struct has field-level documentation
  - [ ] `PersonaStore` struct has proper documentation
  - [ ] All public methods have Arguments/Returns/Errors sections
  - [ ] `cargo doc --no-deps` compiles without warnings

  **QA Scenarios**:
  ```
  Scenario: PersonaStore documentation compiles
    Tool: Bash
    Preconditions: PersonaStore documentation added
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo doc --no-deps --document-private-items
    Expected Result: Command completes successfully with exit code 0
    Failure Indicators: Compilation errors, rustdoc warnings about struct documentation
    Evidence: .sisyphus/evidence/task-9-persona-store-compile.txt
  ```

  **Commit**: NO (groups with Task 11)

- [x] 10. Document Database connection management

  **What to do**:
  - Apply the struct documentation template to `Database` struct in `store/database.rs`
  - Document all public methods with Arguments/Returns/Errors sections
  - Include usage examples showing database connection management
  - Ensure consistency with existing documentation patterns

  **Must NOT do**:
  - Add documentation for private/internal methods
  - Skip any public API methods

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Technical writing for database connection documentation
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Ensure database documentation follows project standards

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 9, 11, 12)
  - **Blocks**: Task 13 (comprehensive doctest suite)
  - **Blocked By**: Tasks 1, 2, 3, 4 (templates verified)

  **References**:
  - `src/store/database.rs` - Current database store file
  - Template from Task 2 - Struct documentation template
  - Template from Task 3 - Method documentation template

  **Acceptance Criteria**:
  - [ ] `Database` struct has proper documentation
  - [ ] All public methods have complete documentation sections
  - [ ] `cargo doc --no-deps` compiles without warnings

  **QA Scenarios**:
  ```
  Scenario: Database documentation compiles
    Tool: Bash
    Preconditions: Database documentation added
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo doc --no-deps --document-private-items
    Expected Result: Command completes successfully with exit code 0
    Failure Indicators: Compilation errors, rustdoc warnings
    Evidence: .sisyphus/evidence/task-10-database-compile.txt
  ```

  **Commit**: NO (groups with Task 11)

- [x] 11. Add doctests for data structure documentation

  **What to do**:
  - Add runnable doctests for all documented data structures (PersonaStore, Database, etc.)
  - Use `no_run` or `ignore` for examples that require external dependencies
  - Ensure all examples are syntactically correct and follow established patterns
  - Test that all doctests pass with `cargo test --doc`

  **Must NOT do**:
  - Add doctests that require external services or complex setup
  - Skip doctests for any public API

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Writing executable documentation tests for data structures
  - **Skills**: [`coding-guidelines`]
    - `coding-guidelines`: Ensure doctests follow Rust documentation best practices

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (sequential after Tasks 9, 10)
  - **Blocks**: Task 13 (comprehensive doctest suite)
  - **Blocked By**: Tasks 9, 10 (data structures documented)

  **References**:
  - `src/config.rs` - Examples of good doctests for data structures
  - `src/conversation.rs` - More examples of comprehensive doctests
  - All data structure files updated in Tasks 9-10

  **Acceptance Criteria**:
  - [ ] All data structure documentation includes appropriate doctests
  - [ ] `cargo test --doc` passes without failures
  - [ ] Doctests use `no_run` or `ignore` for complex external dependencies

  **QA Scenarios**:
  ```
  Scenario: Data structure doctests pass
    Tool: Bash
    Preconditions: Doctests added to data structures
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo test --doc
    Expected Result: All doctests pass, no failures
    Failure Indicators: Any doctest failures
    Evidence: .sisyphus/evidence/task-11-data-doctests-pass.txt
  ```

  **Commit**: YES
  - Message: docs(store): document persona store and database modules
  - Files: src/store/*.rs
  - Pre-commit: cargo test --doc

- [x] 12. Verify all documentation compiles

  **What to do**:
  - Run `cargo doc --no-deps` to verify all documentation compiles correctly
  - Fix any compilation errors or warnings
  - Ensure all templates and documentation are applied consistently
  - Prepare for comprehensive testing in Wave 4

  **Must NOT do**:
  - Ignore compilation warnings
  - Skip verification of any documentation files

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Quick verification task that should be fast
  - **Skills**: []
    - No special skills needed for basic compilation verification

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (sequential after Tasks 11)
  - **Blocks**: Task 13 (comprehensive doctest suite)
  - **Blocked By**: Tasks 11 (doctests added)

  **References**:
  - All documentation files updated in Tasks 1-11

  **Acceptance Criteria**:
  - [ ] `cargo doc --no-deps` compiles without warnings
  - [ ] All documentation renders correctly

  **QA Scenarios**:
  ```
  Scenario: All documentation compiles without errors
    Tool: Bash
    Preconditions: All documentation completed
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo doc --no-deps --document-private-items
    Expected Result: Command completes successfully with exit code 0, no warnings
    Failure Indicators: Any compilation errors or warnings
    Evidence: .sisyphus/evidence/task-12-all-docs-compile.txt
  ```

  **Commit**: NO (groups with Task 16)

- [x] 13. Run comprehensive doctest suite

  **What to do**:
  - Run `cargo test --doc` to execute all doctests across the entire project
  - Verify that all existing doctests still pass (43 existing + new ones)
  - Fix any failing doctests
  - Ensure comprehensive test coverage

  **Must NOT do**:
  - Skip any doctests
  - Ignore failing tests

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Automated test execution
  - **Skills**: []
    - No special skills needed for test execution

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 14, 15, 16)
  - **Blocks**: None (final verification wave)
  - **Blocked By**: Tasks 12 (all documentation verified)

  **References**:
  - All doctests in the project (existing 43 + new ones)

  **Acceptance Criteria**:
  - [ ] `cargo test --doc` passes all tests
  - [ ] No doctest failures

  **QA Scenarios**:
  ```
  Scenario: Comprehensive doctest suite passes
    Tool: Bash
    Preconditions: All documentation and doctests completed
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo test --doc
    Expected Result: All doctests pass, including existing 43 and new ones
    Failure Indicators: Any doctest failures
    Evidence: .sisyphus/evidence/task-13-comprehensive-doctests-pass.txt
  ```

  **Commit**: NO (groups with Task 16)

- [x] 14. Verify documentation renders correctly

  **What to do**:
  - Generate HTML documentation with `cargo doc --no-deps`
  - Verify that all new documentation renders correctly in the generated HTML
  - Check that examples, code blocks, and links render properly
  - Ensure navigation and cross-references work correctly

  **Must NOT do**:
  - Skip visual verification of rendered documentation
  - Ignore rendering issues

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Documentation rendering verification
  - **Skills**: []
    - Basic documentation rendering verification

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 13, 15, 16)
  - **Blocks**: None (final verification wave)
  - **Blocked By**: Tasks 12 (all documentation verified)

  **References**:
  - Generated HTML documentation in `target/doc/`

  **Acceptance Criteria**:
  - [ ] HTML documentation generates without errors
  - [ ] All new documentation renders correctly with proper formatting

  **QA Scenarios**:
  ```
  Scenario: Documentation renders correctly in HTML
    Tool: Bash
    Preconditions: All documentation completed
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo doc --no-deps
      3. Check target/doc/ for generated files
    Expected Result: HTML documentation generated successfully, files exist
    Failure Indicators: Generation errors, missing files
    Evidence: .sisyphus/evidence/task-14-html-render-success.txt
  ```

  **Commit**: NO (groups with Task 16)

- [x] 15. Final integration test

  **What to do**:
  - Run full test suite including unit tests and integration tests
  - Verify that documentation improvements don't break existing functionality
  - Ensure all tests pass: `cargo test`
  - Confirm that the bot still functions correctly with new documentation

  **Must NOT do**:
  - Skip integration tests
  - Assume documentation changes don't affect functionality

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Integration testing
  - **Skills**: []
    - Standard integration testing

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 13, 14, 16)
  - **Blocks**: None (final verification wave)
  - **Blocked By**: Tasks 12 (all documentation verified)

  **References**:
  - All test files in `tests/` directory
  - Integration tests for bot functionality

  **Acceptance Criteria**:
  - [ ] `cargo test` passes all tests
  - [ ] Integration tests pass
  - [ ] No regressions in functionality

  **QA Scenarios**:
  ```
  Scenario: Full test suite passes
    Tool: Bash
    Preconditions: All documentation completed
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. cargo test
    Expected Result: All tests pass, including unit and integration tests
    Failure Indicators: Any test failures
    Evidence: .sisyphus/evidence/task-15-full-test-suite-pass.txt
  ```

  **Commit**: NO (groups with Task 16)

- [x] 16. Git cleanup + tagging

  **What to do**:
  - Clean up any temporary files or artifacts
  - Ensure all changes are properly committed
  - Tag the documentation improvement milestone if appropriate
  - Prepare final summary of changes

  **Must NOT do**:
  - Leave temporary files in the repository
  - Skip final cleanup steps

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Cleanup and finalization tasks
  - **Skills**: []
    - Basic git operations

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 13, 14, 15)
  - **Blocks**: None (final task)
  - **Blocked By**: Tasks 15 (integration test passed)

  **References**:
  - Git repository state
  - All documentation changes

  **Acceptance Criteria**:
  - [ ] Repository is clean with no temporary files
  - [ ] All documentation changes are committed
  - [ ] Final verification complete

  **QA Scenarios**:
  ```
  Scenario: Repository is clean after documentation changes
    Tool: Bash
    Preconditions: All documentation and tests completed
    Steps:
      1. cd /Users/xfy/Developer/aether
      2. git status
    Expected Result: Working tree clean, all changes committed
    Failure Indicators: Uncommitted changes, untracked files
    Evidence: .sisyphus/evidence/task-16-repo-clean.txt
  ```

  **Commit**: YES
  - Message: docs(tests): add comprehensive doctests and verify all documentation
  - Files: all documentation files
  - Pre-commit: cargo test --doc && cargo doc --no-deps

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Rejection → fix → re-run.

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, cargo doc, cargo test). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo doc --no-deps` + `cargo test --doc`. Review all changed files for: missing documentation sections, inconsistent patterns, vague descriptions. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Doc Build [PASS/FAIL] | Doc Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [x] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration (templates applied consistently across files). Test edge cases: missing sections, wrong patterns.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [x] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **1**: `docs(templates): create unified documentation templates` — template files, cargo doc --no-deps
- **2**: `docs(handlers): document admin, persona, and mcp command handlers` — handler files, cargo test --doc
- **3**: `docs(store): document persona store and database modules` — store files, cargo test --doc  
- **4**: `docs(tests): add comprehensive doctests and verify all documentation` — all files, cargo test --doc && cargo doc --no-deps

---

## Success Criteria

### Verification Commands
```bash
cargo test --doc  # Expected: all doctests pass
cargo doc --no-deps  # Expected: compiles without warnings
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All doctests pass
- [ ] All documentation compiles without warnings