# Phase 56: Test Coverage & CI Reporting - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-27
**Phase:** 56-test-coverage-ci-reporting
**Areas discussed:** Rust coverage tool, Coverage reporting format, Coverage thresholds
**Mode:** auto (all decisions auto-selected)

---

## Rust Coverage Tool

| Option | Description | Selected |
|--------|-------------|----------|
| cargo-llvm-cov | Source-based instrumentation, accurate, fast, standard lcov output | ✓ |
| cargo-tarpaulin | ptrace-based, older approach, Linux-only, slower | |

**User's choice:** [auto] cargo-llvm-cov (recommended default)
**Notes:** Source-based coverage is more accurate and produces standard lcov format for downstream tooling.

---

## Coverage Reporting Format

| Option | Description | Selected |
|--------|-------------|----------|
| HTML artifacts + PR comment | Detailed HTML for browsing + summary in PR for visibility | ✓ |
| HTML artifacts only | Detailed but requires downloading artifact | |
| PR comment only | Quick visibility but no line-level detail | |

**User's choice:** [auto] Both HTML artifacts and PR comment summary (recommended default)
**Notes:** Combines detailed browsing capability with quick PR-level visibility.

---

## Coverage Thresholds

| Option | Description | Selected |
|--------|-------------|----------|
| Report-only | Measure and report, no enforcement | ✓ |
| Enforce minimum | Fail CI below threshold | |

**User's choice:** [auto] Report-only (recommended default)
**Notes:** First-time coverage measurement — establish baseline before setting enforcement thresholds.

---

## Claude's Discretion

- GitHub Action for PR comments
- lcov merge strategy
- Per-file vs summary-only in PR comments

## Deferred Ideas

None
