---
phase: 61
slug: syntax-highlighting
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-28
---

# Phase 61 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust: built-in #[test] with cargo test; Frontend: vitest 3.x |
| **Config file** | Cargo.toml (Rust); vite.config.ts (vitest) |
| **Quick run command** | `cargo test -p trunk --lib -- syntax && bun run test` |
| **Full suite command** | `cargo test -p trunk && bun run test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p trunk --lib -- syntax && bun run test`
- **After every plan wave:** Run `cargo test -p trunk && bun run test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 61-01-01 | 01 | 1 | SYNT-01 | unit | `cargo test -p trunk --lib -- syntax::tests` | ❌ W0 | ⬜ pending |
| 61-01-02 | 01 | 1 | SYNT-02 | unit | `cargo test -p trunk --lib -- syntax::tests::extension_detection` | ❌ W0 | ⬜ pending |
| 61-01-03 | 01 | 1 | SYNT-01 | unit | `cargo test -p trunk --lib -- syntax::tests::merged_spans` | ❌ W0 | ⬜ pending |
| 61-02-01 | 02 | 2 | SYNT-03 | unit | `bun run test -- DiffPanel` | Partially | ⬜ pending |
| 61-02-02 | 02 | 2 | SYNT-01 | integration | `cargo test -p trunk -- test_integ` | Partially | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/git/syntax.rs` — new module needs unit tests for color-to-class mapping, extension detection, and merged span generation
- [ ] `src/components/DiffPanel.test.ts` — add merged span rendering tests with syntax classes

*Existing infrastructure covers framework setup. Only new test files/cases needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Syntax colors visually desaturated on add/delete backgrounds | SYNT-03 | Visual appearance requires human judgment | Open a diff with mixed add/delete/context lines; verify syntax colors on add/delete lines appear muted compared to context lines |
| Syntax coloring matches VS Code Dark+ palette | SYNT-01 | Color accuracy is visual | Compare a `.rs` file diff with the same file open in VS Code with Dark+ theme |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
