# Phase 57: Performance Benchmarks - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-27
**Phase:** 57-performance-benchmarks
**Areas discussed:** Benchmark scope, Regression detection, Fixture generation, CI integration

---

## Benchmark Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Rust-only this phase | Focus on Criterion benchmarks for _inner functions. Defer BENCH-03/04 to Phase 58. | ✓ |
| Include frontend benchmarks | Add Xvfb + tauri-driver for IPC round-trips and startup time. | |
| Lightweight IPC proxy | Benchmark IPC via Tauri test feature without real WebView. | |

**User's choice:** Rust-only this phase
**Notes:** BENCH-03 (IPC round-trips) and BENCH-04 (startup time) require running Tauri app — better fit for Phase 58 E2E harness.

### Additional functions to benchmark

| Option | Description | Selected |
|--------|-------------|----------|
| Required set only | Only the 4 functions from requirements. | |
| Add hot-path functions | Also benchmark get_status_inner, search_commits_inner, checkout_branch_inner. | |
| You decide | Claude picks based on call frequency and complexity. | ✓ |

**User's choice:** You decide
**Notes:** Claude has discretion on additional benchmark targets.

---

## Regression Detection

**User provided freeform context:** "We're not putting up PRs for this repository, we're doing main branch development. Let's just add a GitHub workflow that runs on every commit and fail that workflow if any given benchmark is slower."

**Research conducted:** User requested research on how benchmarks are typically done for Rust projects. Agent researched Criterion best practices, CI regression detection patterns, baseline storage, and real-world examples (rustls, gitoxide, criterion.rs, ripgrep).

### Regression threshold

| Option | Description | Selected |
|--------|-------------|----------|
| 30% (130% ratio) | Sweet spot for GitHub Actions shared runners. Used by criterion.rs project. | ✓ |
| 20% | Tighter, may get false positives from CI runner variance. | |
| 50% | Very permissive, only catches major regressions. | |

**User's choice:** 30% (130% ratio)

### Baseline storage

| Option | Description | Selected |
|--------|-------------|----------|
| gh-pages branch | Persistent history, visual dashboard, native benchmark-action integration. | |
| CI cache only | Simpler, no extra branch. Risk of cache eviction. | ✓ |
| You decide | Claude picks. | |

**User's choice:** CI cache only

---

## Fixture Generation

| Option | Description | Selected |
|--------|-------------|----------|
| In-memory treebuilder | git2's treebuilder() + blob(), no filesystem writes. OnceLock for one-time generation. | ✓ |
| Reuse TestContext builder | Extend existing test harness. Simpler reuse but slower at large scale. | |
| You decide | Claude picks per scale. | |

**User's choice:** In-memory treebuilder

---

## CI Integration

| Option | Description | Selected |
|--------|-------------|----------|
| Separate workflow | New benchmarks.yml on push to main. ci.yml gets compile-check only. | ✓ |
| Extend existing ci.yml | Add Gate 3 job for benchmarks. Everything in one workflow. | |
| You decide | Claude picks. | |

**User's choice:** Separate workflow

---

## Claude's Discretion

- Additional benchmark targets beyond required set (D-03)
- Benchmark group structure and file organization
- Sample size and measurement time configuration
- Branch topology in fixtures
- warm_up_time and measurement_time tuning

## Deferred Ideas

- BENCH-03/04 (frontend benchmarks) — Phase 58
- gh-pages visual dashboard — add later if needed
- iai-callgrind instruction counting — consider if false positives become a problem
