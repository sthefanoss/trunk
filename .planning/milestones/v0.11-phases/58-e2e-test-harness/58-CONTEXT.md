# Phase 58: E2E Test Harness - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

WebdriverIO + tauri-driver E2E tests covering core user workflows (browsing commits, staging files, creating commits, branch operations) running against a debug build on Linux CI. macOS E2E is documented as manual pre-release validation.

</domain>

<decisions>
## Implementation Decisions

### Selector Strategy
- **D-01:** Use `data-testid` attributes on key interactive elements and verification targets. E2E tests select elements via `[data-testid="..."]` — explicit, stable, decoupled from styling and DOM structure.
- **D-02:** Add `data-testid` attributes incrementally — only on elements needed by E2E test scenarios, not exhaustively across the entire UI.

### macOS E2E Approach
- **D-03:** macOS E2E documented as manual pre-release validation (E2E-05). The experimental Tauri WebDriver plugin for WKWebView is not reliable enough for CI. A manual validation checklist covers the same core workflows tested on Linux.

### Fixture Management
- **D-04:** Each E2E test creates its own fixture repository at runtime using git CLI commands (init, add, commit, branch, etc.). Tests get fresh isolated repos — mirrors the Rust test pattern from Phase 53.
- **D-05:** A helper/utility module provides fixture builders for common repo shapes (linear history, branches, conflicts) to keep test code DRY.

### CI Workflow Design
- **D-06:** Separate `e2e.yml` GitHub Actions workflow — E2E tests are long-running and should not block the fast CI gates in `ci.yml`.
- **D-07:** Linux CI uses Xvfb (virtual framebuffer) for headless display. The workflow installs webkit2gtk system deps, builds a debug Tauri binary, starts tauri-driver, and runs WebdriverIO tests.
- **D-08:** E2E workflow triggers on push to main and pull requests (same as ci.yml), but runs in parallel rather than gating on ci.yml completion.

### Claude's Discretion
- WebdriverIO configuration details (wdio.conf.ts structure, reporters, timeouts)
- tauri-driver launch and lifecycle management
- Exact data-testid naming conventions
- Which additional edge-case scenarios to cover beyond the required core workflows
- Test file organization within the E2E test directory
- Whether to use Page Object pattern or flat test helpers

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — E2E-01 (WebdriverIO + tauri-driver on Linux CI), E2E-02 (browse commit history), E2E-03 (staging workflow), E2E-04 (branch operations), E2E-05 (macOS experimental/manual)

### Prior Phase Context
- `.planning/phases/53-rust-unit-tests-test-harness/53-CONTEXT.md` — GOOS-style harness architecture (TestContext, builders, drivers) — pattern reference for E2E test organization
- `.planning/phases/55-integration-tests/55-CONTEXT.md` — Integration test patterns, test directory separation, shared infrastructure reuse

### Existing CI
- `.github/workflows/ci.yml` — Current CI workflow structure (gate 1: fast checks, gate 2: heavy checks) — E2E workflow follows similar patterns
- `.github/workflows/benchmarks.yml` — Separate workflow pattern reference

### Tauri Configuration
- `src-tauri/tauri.conf.json` — App identifier (`com.joaofnds.trunk`), window config, build commands

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src-tauri/tests/common/` — Shared test infrastructure (TestContext, builders, drivers) from Phase 53. Pattern reference for E2E fixture helpers.
- `.github/workflows/ci.yml` — System dependency installation (webkit2gtk, libssl, etc.) can be reused in E2E workflow.

### Established Patterns
- Inner-fn pattern for testable Tauri commands — E2E tests exercise the full stack (UI → IPC → Rust → git2)
- Gate-based CI pipeline — fast checks gate heavy checks; E2E as separate workflow follows this philosophy
- Descriptive test naming — `modified_file_shows_in_unstaged_diff()` style from Phase 53

### Integration Points
- Debug build binary (`cargo build --manifest-path src-tauri/Cargo.toml`) — E2E tests launch this binary via tauri-driver
- Svelte components — will need `data-testid` attributes added to key interactive elements (sidebar, commit list, staging panel, diff view, commit form)
- Tauri window — `titleBarStyle: "Overlay"` with hidden title may affect element targeting

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 58-e2e-test-harness*
*Context gathered: 2026-03-27*
