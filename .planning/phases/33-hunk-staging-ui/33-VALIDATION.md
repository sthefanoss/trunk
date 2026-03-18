---
phase: 33
slug: hunk-staging-ui
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 33 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest 4.1.0 (frontend), cargo test (backend) |
| **Config file** | `vite.config.ts` (test section) |
| **Quick run command** | `npx vitest run --reporter=verbose` |
| **Full suite command** | `npx vitest run && cd src-tauri && cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx vitest run --reporter=verbose`
- **After every plan wave:** Run `npx vitest run && cd src-tauri && cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 33-01-01 | 01 | 1 | HUNK-04 | manual-only | Visual verification in app | N/A | ⬜ pending |
| 33-01-02 | 01 | 1 | HUNK-06 | manual-only | Visual verification with binary file | N/A | ⬜ pending |
| 33-01-03 | 01 | 1 | HUNK-09 | manual-only | Interactive keyboard testing | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. Backend hunk commands are already tested (Tests 13-18 in `staging.rs`). Frontend requirements are UI-interaction behaviors verified through manual testing since the Vitest setup uses `environment: "node"` (not jsdom).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Context-appropriate buttons per diffKind | HUNK-04 | UI rendering requires running Tauri app with real git repo | 1. Open app with a multi-hunk file modified. 2. Verify unstaged diff shows "Stage Hunk" + "Discard Hunk" buttons. 3. Stage a hunk. 4. Verify staged diff shows "Unstage Hunk" only. 5. View a commit diff — verify no hunk buttons. |
| Binary files show no hunk buttons | HUNK-06 | Binary file rendering in actual app context | 1. Add/modify a binary file (e.g., PNG). 2. Open its diff. 3. Verify no hunk action buttons appear. |
| [/] keyboard navigation between hunks | HUNK-09 | Interactive keyboard + DOM scrolling behavior | 1. Open a diff with 3+ hunks. 2. Press `]` — verify scrolls to next hunk with highlight. 3. Press `[` — verify scrolls to previous hunk. 4. At last hunk press `]` — verify no action. 5. At first hunk press `[` — verify no action. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
