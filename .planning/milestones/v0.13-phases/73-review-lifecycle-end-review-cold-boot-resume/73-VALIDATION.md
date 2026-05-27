---
phase: 73
slug: review-lifecycle-end-review-cold-boot-resume
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-27
---

# Phase 73 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest 2.x (frontend) + cargo test (backend) |
| **Config file** | `vitest.config.ts`, `src-tauri/Cargo.toml` |
| **Quick run command** | `pnpm vitest run src/components/ReviewPanel.test.ts` |
| **Full suite command** | `just check` |
| **Estimated runtime** | quick ~5s; full ~60-90s |

---

## Sampling Rate

- **After every task commit:** Run `pnpm vitest run src/components/ReviewPanel.test.ts`
- **After every plan wave:** Run `just check`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~5 seconds (file-scoped vitest)

---

## Per-Task Verification Map

> Populated by the planner from PLAN.md task IDs. See 73-RESEARCH.md `## Validation Architecture` for the representative-case matrix.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | Bug-3 / End-review | — | N/A (no auth/secret surface) | unit | `pnpm vitest run src/components/ReviewPanel.test.ts` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Existing `src/components/ReviewPanel.test.ts` extended with cold-boot resume cases (no new file; reuses `vi.useFakeTimers` + IPC mock harness already present)
- [ ] No new test infrastructure required — `vitest` + `vi.mock("@tauri-apps/api/core")` already wire `invoke` mocks across `Copy`/`reload` cases

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cold-boot resume across actual app restart | Bug 3 (Phase 72 SC) | Vitest mocks IPC; only a real app boot exercises the persisted-state → first-open path end-to-end | `pnpm tauri dev`, open repo with prior session, observe comments appear without mutation |
| Multi-tab End → empty state in other window | D-09 | Requires two real Tauri windows talking through the actual `session-changed` event bus | Open two windows on same repo, end review in A, observe B re-render |
| Visual treatment of two-step End button danger color | D-05 | CSS custom property + Lucide icon contrast — eye check, not assertion | Click End once, verify "Click again to confirm" + danger color + ~3s auto-revert |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s for quick run
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
