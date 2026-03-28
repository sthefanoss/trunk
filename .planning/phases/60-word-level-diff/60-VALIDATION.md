---
phase: 60
slug: word-level-diff
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-28
---

# Phase 60 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Rust Framework** | cargo test (integration tests in src-tauri/tests/) |
| **Rust Config** | src-tauri/Cargo.toml |
| **Rust Quick Command** | `cd src-tauri && cargo test word_span -- --nocapture` |
| **Rust Full Command** | `cd src-tauri && cargo test` |
| **Frontend Framework** | vitest (src/**/*.test.ts) |
| **Frontend Config** | package.json |
| **Frontend Quick Command** | `bun run test -- src/components/DiffPanel.test.ts` |
| **Frontend Full Command** | `bun run test` |
| **Estimated runtime** | ~15 seconds (Rust) + ~5 seconds (Frontend) |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test word_span -- --nocapture && bun run test`
- **After every plan wave:** Run `cd src-tauri && cargo test && bun run test && bun run check`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 20 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 60-01-01 | 01 | 1 | WORD-01 | Rust integration | `cd src-tauri && cargo test word_span -- --nocapture` | ❌ W0 | ⬜ pending |
| 60-01-02 | 01 | 1 | WORD-02 | Rust integration | `cd src-tauri && cargo test word_span_long -- --nocapture` | ❌ W0 | ⬜ pending |
| 60-01-03 | 01 | 1 | WORD-02 | Rust integration | `cd src-tauri && cargo test word_span_dissimilar -- --nocapture` | ❌ W0 | ⬜ pending |
| 60-02-01 | 02 | 2 | WORD-01 | Frontend component | `bun run test -- src/components/DiffPanel.test.ts` | Partial | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/tests/test_diff.rs` — word-span integration tests (paired lines, long lines, dissimilar lines)
- [ ] `src/components/DiffPanel.test.ts` — updated test data with non-empty word_spans, verify span rendering

*Existing infrastructure covers all framework needs — no new installs required.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Word-diff background colors readable on add/delete line backgrounds | WORD-01 | Visual readability is subjective | Open diff with modified lines, verify highlighted words are visible against green/red backgrounds |
| CSS custom properties defined (not inline colors) | WORD-01 | Requires theme inspection | Inspect element in devtools, verify `--color-diff-word-add-bg` and `--color-diff-word-delete-bg` are used |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 20s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
