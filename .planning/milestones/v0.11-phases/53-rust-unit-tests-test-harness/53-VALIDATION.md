---
phase: 53
slug: rust-unit-tests-test-harness
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 53 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | src-tauri/Cargo.toml |
| **Quick run command** | `cd src-tauri && cargo test` |
| **Full suite command** | `cd src-tauri && cargo test` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test`
- **After every plan wave:** Run `cd src-tauri && cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | HARN-01 | unit | `cd src-tauri && cargo test` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | HARN-02 | unit | `cd src-tauri && cargo test` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | HARN-03 | unit | `cd src-tauri && cargo test` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | HARN-04 | unit | `cd src-tauri && cargo test` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | UNIT-01 | unit | `cd src-tauri && cargo test` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/tests/common/mod.rs` — TestContext, builder, driver stubs
- [ ] `src-tauri/src/lib.rs` — make modules public for integration test access

*Existing `tempfile` dev-dependency and `cargo test` CI gate already in place.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
