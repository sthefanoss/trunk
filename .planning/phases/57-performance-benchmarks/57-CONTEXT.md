# Phase 57: Performance Benchmarks - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish Criterion benchmarks for critical Rust backend operations with CI regression detection. Benchmarks run on every push to main and fail the workflow if any benchmark regresses beyond a configured threshold. Frontend benchmarks (IPC round-trips, startup time) are deferred to Phase 58 where the E2E harness will provide the necessary infrastructure.

</domain>

<decisions>
## Implementation Decisions

### Benchmark Scope
- **D-01:** Rust-only this phase — Criterion benchmarks for `_inner` functions. BENCH-03 (frontend IPC round-trips) and BENCH-04 (application startup time) are deferred to Phase 58 where tauri-driver infrastructure will already exist.
- **D-02:** Required benchmark targets: `walk_commits` (graph.rs) at 100, 1k, 10k commit scales; `list_refs_inner`, `diff_unstaged_inner`, `stage_hunk_inner`.
- **D-03:** Additional benchmark targets at Claude's discretion based on call frequency and complexity (e.g., `get_status_inner`, `search_commits_inner`).

### Regression Detection
- **D-04:** Use `benchmark-action/github-action-benchmark` for automated regression detection. Compares each run against previous stored results.
- **D-05:** Alert threshold of 130% (30% regression tolerance) — sweet spot for GitHub Actions shared runners. Catches real regressions while ignoring CI noise.
- **D-06:** `fail-on-alert: true` — workflow fails if any benchmark exceeds the threshold.
- **D-07:** Store baselines in CI cache (`actions/cache`), not gh-pages branch.

### Fixture Generation
- **D-08:** Use git2's `treebuilder()` + `blob()` to create benchmark repos entirely in-memory (no filesystem writes). Fast even at 10k commits.
- **D-09:** Generate fixtures once per benchmark group via `OnceLock`. Fixture creation runs outside the timed section.

### CI Integration
- **D-10:** Separate `benchmarks.yml` workflow — triggers on push to main only. Runs `cargo bench`, compares via benchmark-action, fails on regression.
- **D-11:** Add `cargo test --benches` to existing `ci.yml` as a compile-check only (ensures benchmarks build without running them).
- **D-12:** Use Criterion 0.8 with `html_reports` feature. Each benchmark file gets a `[[bench]]` entry with `harness = false`.

### Claude's Discretion
- Which additional `_inner` functions to benchmark beyond the required set (D-03)
- Exact benchmark group structure and file organization in `benches/`
- Sample size and measurement time configuration per benchmark
- Whether to add branch topology (merges, forks) to benchmark fixtures or keep them linear
- `warm_up_time` and `measurement_time` tuning per benchmark scale

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Benchmark Infrastructure
- `src-tauri/Cargo.toml` — Add Criterion dev-dependency and `[[bench]]` entries here
- `src-tauri/src/git/graph.rs` — Contains `walk_commits` function (primary benchmark target)
- `src-tauri/src/commands/branches.rs` — Contains `list_refs_inner` (BENCH-02)
- `src-tauri/src/commands/diff.rs` — Contains `diff_unstaged_inner` (BENCH-02)
- `src-tauri/src/commands/staging.rs` — Contains `stage_hunk_inner` (BENCH-02)

### CI Configuration
- `.github/workflows/ci.yml` — Add `cargo test --benches` compile-check to existing cargo-test job
- `.github/workflows/release.yml` — Reference for workflow patterns (not modified)

### Test Harness Reference
- `src-tauri/tests/common/` — Existing test harness with TestContext builder (pattern reference, not reused directly for benchmarks)

### Requirements
- `.planning/REQUIREMENTS.md` §BENCH-01..05 — Performance benchmark requirements

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `walk_commits` in `src-tauri/src/git/graph.rs` — Public function taking `&mut Repository, offset, limit`. Primary benchmark target.
- `_inner` functions across `src-tauri/src/commands/` — 50+ inner functions, all take `path: &str` + `state_map` pattern. Benchmark candidates.
- `tempfile` crate already in dev-dependencies — used for test fixtures, reusable for benchmark repo generation.

### Established Patterns
- Library crate `trunk_lib` with `crate-type = ["staticlib", "cdylib", "rlib"]` — benchmarks access pub items through `rlib`.
- `_inner` function pattern separates business logic from Tauri command wrappers — benchmarks can call `_inner` directly.
- Gate 1 (fast) / Gate 2 (heavy) CI pattern — benchmarks get their own workflow, compile-check goes in Gate 2.

### Integration Points
- `src-tauri/Cargo.toml` — Criterion dependency and `[[bench]]` entries
- `.github/workflows/` — New `benchmarks.yml` workflow
- `.github/workflows/ci.yml` — Add `cargo test --benches` to cargo-test job

</code_context>

<specifics>
## Specific Ideas

- User's first time setting up benchmarks — research-backed decisions, not guesswork
- Direct-to-main workflow (no PRs) — benchmarks run on push to main, not on PRs
- 130% threshold chosen based on Criterion community best practice for shared CI runners
- CI cache chosen over gh-pages for baseline storage — simpler, no extra branch

</specifics>

<deferred>
## Deferred Ideas

- BENCH-03 (Frontend IPC round-trip benchmarks) — deferred to Phase 58 where tauri-driver infrastructure exists
- BENCH-04 (Application startup time measurement) — deferred to Phase 58 for same reason
- Visual benchmark dashboard (gh-pages) — can be added later if historical tracking is needed
- iai-callgrind instruction counting — Linux-only, deterministic but doesn't measure wall-clock time. Consider if false positives become a problem.

</deferred>

---

*Phase: 57-performance-benchmarks*
*Context gathered: 2026-03-27*
