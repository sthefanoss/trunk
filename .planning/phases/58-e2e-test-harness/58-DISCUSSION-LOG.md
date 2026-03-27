# Phase 58: E2E Test Harness - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-27
**Phase:** 58-e2e-test-harness
**Areas discussed:** Selector strategy, macOS E2E approach, Fixture management, CI workflow design
**Mode:** --auto (all choices auto-selected as recommended defaults)

---

## Selector Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| data-testid attributes | Explicit, stable, decoupled from styling/structure | [auto] |
| CSS class selectors | Fragile with Tailwind utility classes | |
| Text content selectors | Brittle, breaks with copy changes | |

**User's choice:** [auto] data-testid attributes (recommended default)
**Notes:** Standard approach for E2E testing — avoids coupling tests to implementation details.

---

## macOS E2E Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Document as manual validation | Manual checklist covering core workflows | [auto] |
| Experimental WebDriver plugin | Tauri WKWebView WebDriver — unreliable for CI | |

**User's choice:** [auto] Document as manual pre-release validation (recommended default)
**Notes:** E2E-05 requirement explicitly offers this as an acceptable option. Experimental WKWebView WebDriver is not mature enough for reliable CI.

---

## Fixture Management

| Option | Description | Selected |
|--------|-------------|----------|
| Git CLI at runtime | Each test creates fresh repo via git commands | [auto] |
| Checked-in fixture repos | Pre-built repos committed to the test directory | |
| Reuse Rust test fixtures | Share Phase 53 TestContext builders | |

**User's choice:** [auto] Git CLI creates fixture repos at runtime (recommended default)
**Notes:** Mirrors the successful pattern from Phase 53 Rust tests. Fresh isolated repos prevent test interference.

---

## CI Workflow Design

| Option | Description | Selected |
|--------|-------------|----------|
| Separate e2e.yml workflow | Independent workflow, doesn't block fast CI | [auto] |
| Add to ci.yml as gate 3 | Integrated but gates on existing checks | |

**User's choice:** [auto] Separate e2e.yml workflow (recommended default)
**Notes:** E2E tests are long-running (build + launch + test). Keeping them separate avoids blocking the fast feedback loop of ci.yml.

---

## Claude's Discretion

- WebdriverIO configuration details
- tauri-driver launch and lifecycle management
- Exact data-testid naming conventions
- Additional edge-case scenarios beyond core workflows
- Test file organization
- Page Object pattern vs flat helpers

## Deferred Ideas

None — discussion stayed within phase scope
