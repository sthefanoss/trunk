# Phase 53: Rust Unit Tests & Test Harness - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-26
**Phase:** 53-rust-unit-tests-test-harness
**Areas discussed:** Harness layering, Fixture builder API, Existing test migration, Test readability

---

## Harness Layering

### Organization

| Option | Description | Selected |
|--------|-------------|----------|
| Shared test module | src-tauri/src/test_harness/ with #[cfg(test)] gating. Importable from inline modules. | |
| Separate test crate | src-tauri/tests/common/ integration-style crate. More isolated. | ✓ |
| Single file | One test_utils.rs file. Simple but may grow unwieldy. | |

**User's choice:** Separate test crate
**Notes:** Harness lives in tests/common/, inline #[cfg(test)] modules stay lightweight.

### Driver Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Drivers wrap everything | Tests call ctx.method() which internally calls _inner(). Tests never import _inner directly. | ✓ |
| Drivers for setup, raw for assertions | Use drivers for repo setup but call _inner directly for assertions. | |

**User's choice:** Drivers wrap everything
**Notes:** Tests interact purely through TestContext methods.

---

## Fixture Builder API

### Builder Style

| Option | Description | Selected |
|--------|-------------|----------|
| Fluent builder | TestContext::builder().with_file().with_commit().with_branch().build() | ✓ |
| Preset functions | empty_repo(), linear_repo(), merge_repo(), conflict_repo() | |
| Both | Presets for common shapes + builder for edge cases | |

**User's choice:** Fluent builder
**Notes:** Composable, expressive, flexible enough for any topology.

### Builder Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Core only | Branches, commits, files, merges, conflicts. Add rest later. | |
| Full coverage upfront | Include binary files, stashes, tags, and remote setup from the start. | ✓ |

**User's choice:** Full coverage upfront

---

## Existing Test Migration

| Option | Description | Selected |
|--------|-------------|----------|
| Migrate all | Move all existing tests to integration test crate, rewritten with TestContext. | ✓ |
| Leave existing, harness for new | Keep inline tests as-is, only new tests use harness. | |
| Gradual migration | Start with harness for new, migrate existing over time. | |

**User's choice:** Migrate all
**Notes:** One consistent style. Removes duplicated make_state_map() boilerplate.

---

## Test Readability

### Naming Convention

| Option | Description | Selected |
|--------|-------------|----------|
| Descriptive action-result | fn modified_file_shows_in_unstaged_diff() | ✓ |
| Given-when-then | fn given_modified_file_when_diffing_unstaged_then_returns_hunks() | |
| Module-scoped short names | mod diff_unstaged { fn returns_hunks() } | |

**User's choice:** Descriptive action-result
**Notes:** Clear and reads naturally without being verbose.

### Assertion Style

| Option | Description | Selected |
|--------|-------------|----------|
| Custom helpers | ctx.assert_file_staged("file.txt"), ctx.assert_branch_exists("feature") | ✓ |
| Standard assert! only | Use assert!, assert_eq! from std | |

**User's choice:** Custom helpers
**Notes:** Domain-specific assertions on TestContext for readable checks with consistent error messages.

---

## Claude's Discretion

- Driver method signatures and return types
- Edge case selection beyond required set
- Builder state machine internals
- Test file organization within tests/

## Deferred Ideas

None — discussion stayed within phase scope.
