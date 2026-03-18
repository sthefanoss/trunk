---
phase: 34
slug: line-level-staging
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-18
---

# Phase 34 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (frontend) / cargo test (backend) |
| **Config file** | `vite.config.ts` / `src-tauri/Cargo.toml` |
| **Quick run command** | `cargo test --manifest-path src-tauri/Cargo.toml` |
| **Full suite command** | `cargo test --manifest-path src-tauri/Cargo.toml && npx vitest run` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --manifest-path src-tauri/Cargo.toml`
- **After every plan wave:** Run `cargo test --manifest-path src-tauri/Cargo.toml && npx vitest run`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 34-01-01 | 01 | 1 | HUNK-07 | unit | `cargo test partial_patch` | ❌ W0 | ⬜ pending |
| 34-01-02 | 01 | 1 | HUNK-08 | unit | `cargo test partial_patch` | ❌ W0 | ⬜ pending |
| 34-02-01 | 02 | 1 | HUNK-07 | manual | visual inspection | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/commands/staging.rs` — unit tests for partial patch construction
- [ ] Existing test infrastructure covers framework needs

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Click/shift-click line selection | HUNK-07 | Browser interaction | Click add/delete lines, verify highlight; shift-click for range |
| Selection clears after operation | HUNK-07 | UI state after async | Stage lines, verify selection clears and diff refreshes |
| Toolbar swaps to line mode | HUNK-07 | Visual inspection | Select lines, verify buttons show "Stage Lines (N)" |
| Staged diff unstage lines | HUNK-08 | Browser interaction | Switch to staged diff, select lines, click "Unstage Lines" |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
