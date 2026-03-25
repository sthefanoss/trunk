---
phase: 44
slug: backend-state-scoping
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 44 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + cargo test |
| **Config file** | `src-tauri/Cargo.toml` (dev-dependencies: tempfile 3) |
| **Quick run command** | `cargo test --lib -p trunk` |
| **Full suite command** | `cargo test -p trunk` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib -p trunk`
- **After every plan wave:** Run `cargo test -p trunk`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 44-01-01 | 01 | 1 | BACK-01a | unit | `cargo test -p trunk -- running_op` | ❌ W0 | ⬜ pending |
| 44-01-02 | 01 | 1 | BACK-01b | unit | `cargo test -p trunk -- running_op` | ❌ W0 | ⬜ pending |
| 44-01-03 | 01 | 1 | BACK-01c | unit | `cargo test -p trunk -- cancel_remote` | ❌ W0 | ⬜ pending |
| 44-01-04 | 01 | 1 | BACK-01d | unit | `cargo test -p trunk -- close_repo` | ✅ (extend) | ⬜ pending |
| 44-01-05 | 01 | 1 | BACK-01e | unit | `cargo test -p trunk -- force_close` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Unit tests for new `RunningOp` HashMap behavior (insert/check/remove by path)
- [ ] Unit tests for `cancel_remote_op` with path parameter
- [ ] Unit tests for `force_close_repo` cleanup sequence

*Note: Testing `run_git_remote` with actual subprocess spawning requires integration tests (Tauri AppHandle). Unit tests verify mutex/HashMap logic in isolation.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Concurrent push+fetch across tabs | BACK-01 | Requires two Tauri windows with real repos | Open two repos, push in tab A, fetch in tab B simultaneously |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
