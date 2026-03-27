# Phase 55: Integration Tests - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

End-to-end validation of the Tauri IPC bridge (serialization round-trips), multi-step git operation sequences (workflows + state transitions), and filesystem watcher integration (real fs events with debounce timing). These tests validate that components work together as integrated systems, layering on top of Phase 53 (individual Rust command unit tests) and Phase 54 (frontend unit tests).

</domain>

<decisions>
## Implementation Decisions

### IPC Test Approach
- **D-01:** Serialization round-trip tests — verify that Rust command return types serialize to JSON and deserialize correctly at the serde boundary. No Tauri runtime needed; runs in `cargo test`.
- **D-02:** All registered Tauri commands get round-trip tests (~30 commands). Comprehensive coverage catches any serde mismatch regardless of type complexity.

### Git Operation Sequences
- **D-03:** Both multi-step workflows AND state transition chains. Validates realistic user flows (init→commit→branch→merge, branch→commit→rebase→conflict→resolve, stash→checkout→pop) and operation state consistency across transitions (normal→merging→resolved→committed, normal→rebasing→conflict→skip→done).
- **D-04:** Tests compose existing Phase 53 drivers into multi-step sequences. Individual commands already validated; integration tests verify correct composition.

### Watcher Test Strategy
- **D-05:** Real filesystem events with generous timeouts (e.g. 2s) to test the actual notify crate + debouncer integration. Flakiness mitigated by generous timeouts.
- **D-06:** Use `tauri::test` module (`tauri::test::mock_builder()`) to create test AppHandle instances. Tests the exact production path with real AppHandle mock rather than extracting watcher logic behind a trait.

### Test Organization
- **D-07:** Separate integration directory (`src-tauri/tests/integration/`) distinct from Phase 53 unit tests in `src-tauri/tests/`.
- **D-08:** Reuse existing `common/` module (TestContext, builders, drivers) from the shared test infrastructure. Integration tests compose existing drivers into multi-step workflows without duplicating helpers.

### Claude's Discretion
- Exact Cargo configuration for the separate integration test directory (may need separate test binary or `[[test]]` entries in Cargo.toml)
- Which specific multi-step workflow scenarios to include beyond the examples above
- Serialization round-trip test implementation details (snapshot testing vs explicit field assertions)
- Watcher test timeout values and retry strategy for CI reliability
- How to structure `tauri::test` mock builder setup for watcher tests

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — INTG-01 (IPC bridge testing), INTG-02 (git operation integration), INTG-03 (filesystem watcher integration)

### Prior Phase Context
- `.planning/phases/53-rust-unit-tests-test-harness/53-CONTEXT.md` — GOOS-style harness architecture (TestContext, drivers, builders) that integration tests build upon
- `.planning/phases/54-frontend-unit-tests/54-CONTEXT.md` — Frontend test patterns and invoke mocking approach

### Existing Test Infrastructure
- `src-tauri/tests/common/context.rs` — TestContext struct (Application Runner) managing tempdir lifecycle
- `src-tauri/tests/common/builder.rs` — Fluent builder for test fixtures
- `src-tauri/tests/common/drivers/` — Domain-level drivers for all commands (staging, diff, commit, branches, etc.)
- `src-tauri/tests/common/assertions.rs` — Custom assertion helpers

### Code Under Test
- `src-tauri/src/lib.rs` lines 67-110 — All registered Tauri commands (invoke_handler)
- `src-tauri/src/watcher.rs` — Filesystem watcher implementation (notify-debouncer-mini, 300ms debounce, repo-changed event)
- `src-tauri/src/commands/` — All command files with `_inner` functions
- `src/lib/invoke.ts` — Frontend safeInvoke<T> wrapper and TrunkError type

### Configuration
- `src-tauri/Cargo.toml` — Test dependencies and build configuration

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TestContext` in `tests/common/context.rs` — manages tempdir, state_map, cache_map. Integration tests reuse this directly.
- `TestContextBuilder` in `tests/common/builder.rs` — fluent API for fixture construction (.with_file(), .with_commit(), .with_branch(), etc.)
- Domain drivers in `tests/common/drivers/` — intention-revealing methods for all commands (e.g., `ctx.stage_file()`, `ctx.create_commit()`, `ctx.checkout()`)
- `tempfile = "3"` dev dependency — already available for tempdir-based fixtures

### Established Patterns
- Inner-fn pattern: every Tauri command has `_inner(path, &state_map, ...)` — pure git logic separated from Tauri state
- `HashMap<String, PathBuf>` state_map — used by all `_inner` functions to resolve repo paths
- All commands registered via `tauri::generate_handler![]` in lib.rs
- `safeInvoke<T>` frontend wrapper parses JSON error strings into `TrunkError { code, message }`

### Integration Points
- `cargo test` CI gate (Phase 50) — new integration tests automatically run
- Watcher emits `repo-changed` Tauri event via `app.emit()` — requires AppHandle for testing
- `notify-debouncer-mini` with 300ms debounce — timing-sensitive for test assertions

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

*Phase: 55-integration-tests*
*Context gathered: 2026-03-26*
