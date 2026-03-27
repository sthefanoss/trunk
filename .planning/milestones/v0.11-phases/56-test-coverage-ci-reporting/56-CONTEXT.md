# Phase 56: Test Coverage & CI Reporting - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Measure test coverage for both Rust and TypeScript codebases and report results in CI. This is a CI/tooling phase — no application code changes, only build configuration and CI workflow updates.

</domain>

<decisions>
## Implementation Decisions

### Rust Coverage Tool
- **D-01:** Use cargo-llvm-cov (not cargo-tarpaulin) — source-based instrumentation is more accurate than tarpaulin's ptrace approach, faster execution, better maintained, and produces standard lcov output for tooling integration.

### TypeScript Coverage Tool
- **D-02:** Use vitest's built-in coverage with @vitest/coverage-v8 provider — already using vitest for tests, native integration via `vitest run --coverage`, produces lcov/text/html reports.

### Coverage Reporting Format
- **D-03:** Both HTML artifacts and PR comment summary — HTML uploaded as CI artifacts for detailed line-by-line browsing, lightweight text/table summary posted as PR comment for quick visibility without downloading artifacts.

### Coverage Thresholds
- **D-04:** Report-only, no enforcement — this is the first time coverage is being measured. Establish a baseline first. Thresholds can be added in a future phase once the team knows what realistic targets look like.

### Claude's Discretion
- Choice of GitHub Action for PR comment posting (e.g., marocchino/sticky-pull-request-comment or similar)
- lcov merge strategy if combining Rust + TS into a single report
- Whether to show per-file coverage or summary-only in PR comments

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### CI Configuration
- `.github/workflows/ci.yml` — Current CI pipeline with cargo-test and vitest jobs (coverage will be added here)
- `.github/workflows/release.yml` — Release pipeline (not modified, but reference for CI patterns)

### Test Configuration
- `vite.config.ts` — Vitest configuration (test.include, environment, setupFiles — coverage config goes here)
- `src-tauri/Cargo.toml` — Rust dependencies and test configuration
- `package.json` — npm scripts for test runner invocation

### Requirements
- `.planning/REQUIREMENTS.md` §UNIT-04 — "Test coverage metrics are measured and reported in CI"

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- CI pipeline already has `cargo-test` and `vitest` jobs with system deps and caching — coverage can extend these existing jobs rather than creating new ones
- Rust cache via `Swatinem/rust-cache@v2` already configured — llvm-cov benefits from same cache
- Gate 1/Gate 2 pattern in CI (fast checks gate heavy checks) — coverage fits naturally in Gate 2

### Established Patterns
- CI uses ubuntu-latest runners
- Rust toolchain installed via `dtolnay/rust-toolchain@stable`
- Bun used for JS tooling (`oven-sh/setup-bun@v2`, `bun install --frozen-lockfile`)
- System deps installed for webkit/tauri compilation

### Integration Points
- `cargo-test` job in ci.yml — add llvm-cov invocation here
- `vitest` job in ci.yml — add --coverage flag and artifact upload
- `vite.config.ts` test section — add coverage provider and reporter config

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches for coverage tooling.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 56-test-coverage-ci-reporting*
*Context gathered: 2026-03-27*
