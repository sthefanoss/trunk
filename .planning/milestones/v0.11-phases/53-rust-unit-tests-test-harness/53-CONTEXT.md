# Phase 53: Rust Unit Tests & Test Harness - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish a GOOS-style test harness architecture for the Rust backend and achieve unit test coverage for all `_inner` functions in `src-tauri/src/commands/`. The harness provides Application Runner (lifecycle), Drivers (intention-revealing methods), and Builders (fixture construction). All existing inline tests are migrated to use the new harness.

</domain>

<decisions>
## Implementation Decisions

### Harness Organization
- **D-01:** Harness lives in a separate integration test crate (`src-tauri/tests/`) with shared `common/` module — not inline `#[cfg(test)]` modules.
- **D-02:** `TestContext` struct is the Application Runner — manages tempdir lifecycle, repo handle, and state_map. One struct orchestrates setup and teardown.
- **D-03:** Domain-level Drivers are methods on `TestContext` that fully wrap `_inner` functions. Tests never call `_inner` functions directly — the driver is the API (e.g., `ctx.diff_unstaged("file.txt")`, `ctx.stage_file("README.md")`).

### Fixture Builder API
- **D-04:** Fluent builder pattern via `TestContext::builder()` with composable methods: `.with_file()`, `.with_commit()`, `.with_branch()`, `.checkout()`, `.merge()`, `.with_conflict()`, `.build()`.
- **D-05:** Full coverage upfront — builder supports binary files (`.with_binary_file()`), stashes (`.with_stash()`), tags (`.with_tag()`), and remote setup (`.with_remote()`) from the start.

### Existing Test Migration
- **D-06:** All existing inline `#[cfg(test)]` tests (14 command files) are migrated to the integration test crate and rewritten to use `TestContext`/Drivers. No dual test styles — one consistent approach.
- **D-07:** After migration, inline `#[cfg(test)]` modules and the old `make_test_repo()` / `make_state_map()` helpers are removed.

### Test Readability
- **D-08:** Descriptive action-result naming convention: `fn modified_file_shows_in_unstaged_diff()`, `fn checkout_dirty_workdir_returns_error()`. Describes scenario and expected outcome in plain English.
- **D-09:** Custom assertion helpers on `TestContext` for domain-specific checks: `ctx.assert_file_staged("file.txt")`, `ctx.assert_branch_exists("feature")`, `ctx.assert_status_clean()`. Provides readable assertions with consistent error messages.

### Claude's Discretion
- Exact Driver method signatures and return types
- Which edge cases to cover beyond the required set (empty repos, merge commits, binary files, conflict states)
- Internal implementation of builder state machine
- Test file organization within `tests/` (one file per command module vs grouped by domain)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — HARN-01..04 (test harness architecture) and UNIT-01 (Rust unit test coverage)

### Existing Code
- `src-tauri/src/git/repository.rs` lines 80-115 — Current `make_test_repo()` implementation (to be replaced by builder)
- `src-tauri/src/commands/` — All 13 command files with `_inner` functions that need test coverage
- `src-tauri/Cargo.toml` — Current dev-dependencies (`tempfile = "3"`)

### Architecture
- `.planning/PROJECT.md` §Key Decisions — "inner-fn pattern for Tauri commands" explains the testability architecture

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `make_test_repo()` in `git/repository.rs` — creates tempdir with merge topology (main + feature branch + merge commit). Logic will be absorbed into the new builder.
- `tempfile = "3"` dev dependency — already available for tempdir-based fixtures.
- Inner-fn pattern across all 13 command files — every Tauri command has a pure `_inner` function that takes path + state_map, making unit testing straightforward.

### Established Patterns
- `HashMap<String, PathBuf>` state_map pattern — used by all `_inner` functions to resolve repo paths. TestContext must provide this.
- `make_state_map()` helper duplicated in every inline test module — will be absorbed into TestContext.
- git2 user config setup (`user.name`, `user.email`) required for commits in tests.

### Integration Points
- `cargo test` CI gate already exists (Phase 50) — new integration tests will automatically run.
- Command files that need driver methods: `branches.rs`, `commit_actions.rs`, `commit.rs`, `diff.rs`, `history.rs`, `interactive_rebase.rs`, `merge_editor.rs`, `operation_state.rs`, `remote.rs`, `repo.rs`, `staging.rs`, `stash.rs`.
- Also: `git/graph.rs` and `git/repository.rs` have existing test modules to migrate.

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 53-rust-unit-tests-test-harness*
*Context gathered: 2026-03-26*
