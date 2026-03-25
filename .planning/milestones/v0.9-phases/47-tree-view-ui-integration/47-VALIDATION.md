---
phase: 47
slug: tree-view-ui-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-24
---

# Phase 47 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest |
| **Config file** | vitest.config.ts |
| **Quick run command** | `bun run test` |
| **Full suite command** | `bun run test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bun run test`
- **After every plan wave:** Run `bun run test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 47-01-01 | 01 | 1 | TREE-01 | unit + manual | `bun run test` | ❌ W0 | ⬜ pending |
| 47-01-02 | 01 | 1 | TREE-02 | unit + manual | `bun run test` | ❌ W0 | ⬜ pending |
| 47-01-03 | 01 | 1 | TREE-03 | unit | `bun run test` | ❌ W0 | ⬜ pending |
| 47-01-04 | 01 | 1 | TREE-04 | manual | N/A | N/A | ⬜ pending |
| 47-01-05 | 01 | 1 | TREE-05 | unit | `bun run test` | ❌ W0 | ⬜ pending |
| 47-01-06 | 01 | 1 | TREE-06 | manual | N/A | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Tree view component tests — stubs for TREE-01 through TREE-05
- [ ] Expand/collapse state persistence tests — stubs for TREE-03

*Existing test infrastructure (vitest) covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Toggle between flat/tree view in staging panel, commit diffs, merge editor | TREE-01 | Visual integration across multiple UI contexts | Open app, stage files, verify toggle works in each panel |
| Keyboard navigation (arrow keys) | TREE-04 | Focus management and key event handling | Navigate tree with arrow keys, verify up/down/left/right behavior |
| View mode persists across sessions | TREE-05 | Requires app restart | Set tree view, restart app, verify mode restored |
| Chevron indicators render correctly | TREE-06 | Visual verification | Expand/collapse directories, verify chevron rotation |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
