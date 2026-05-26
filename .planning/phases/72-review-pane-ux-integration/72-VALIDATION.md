---
phase: 72
slug: review-pane-ux-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-27
---

# Phase 72 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (frontend) + cargo test (backend, no new tests this phase) |
| **Config file** | `vite.config.ts` / `vitest.config.ts` (project-managed) |
| **Quick run command** | `bun run vitest run src/components/ReviewPanel.test.ts src/components/Toolbar.test.ts src/lib/review-session.svelte.test.ts` |
| **Full suite command** | `just check` |
| **Estimated runtime** | ~25 seconds (component subset) / ~90 seconds (`just check`) |

---

## Sampling Rate

- **After every task commit:** Run `bun run vitest run <touched file>.test.ts`
- **After every plan wave:** Run `bun run vitest run` + `cargo test`
- **Before `/gsd:verify-work`:** `just check` must exit 0
- **Max feedback latency:** ~25 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| _to be populated by planner_ | | | | | | | | | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

Phase requirements (from RESEARCH.md §Validation Architecture):

| Req ID | Behavior | Test Type | Automated Command |
|--------|----------|-----------|-------------------|
| REQ-72-1a | Toolbar Review button click emits `review-toggle` | unit | `vitest run src/components/Toolbar.test.ts -t "review-toggle"` |
| REQ-72-1b | Cmd+Shift+R triggers the menu item | manual UAT | — |
| REQ-72-1c | View menu "Start/End Code Review" regression | manual UAT | — |
| REQ-72-2 | Toolbar button active styling when `reviewActive === true` | unit | `vitest run src/components/Toolbar.test.ts -t "active state"` |
| REQ-72-3a | Copy click calls `generate` then `writeText` with returned markdown | unit | `vitest run src/components/ReviewPanel.test.ts -t "copy click invokes generate and writeText"` |
| REQ-72-3b | ✓ Copied affordance for 1500ms with re-arm | unit | `vitest run src/components/ReviewPanel.test.ts -t "remains clickable during window"` |
| REQ-72-3c | Error toast on failure with `instanceof Error` narrowing | unit | `vitest run src/components/ReviewPanel.test.ts -t "shows error toast on failure"` |
| REQ-72-3d | Non-`Error` rejection coerced via `String(e)` | unit | `vitest run src/components/ReviewPanel.test.ts -t "coerces non-Error rejection"` |
| REQ-72-3e | Copy button stays in "Copy" on failure | unit | `vitest run src/components/ReviewPanel.test.ts -t "does not flip copied on failure"` |
| REQ-72-4a | `ReviewDocPreview.svelte` deleted | file-absence | `test ! -f src/components/ReviewDocPreview.svelte` |
| REQ-72-4b | `panelMode` / `previewMarkdown` / `showList` / `showPreview` removed | unit | `vitest run src/lib/review-session.svelte.test.ts` |
| REQ-72-4c | `generate(repoPath)` returns markdown string | unit | `vitest run src/lib/review-session.svelte.test.ts -t "generate returns markdown string"` |
| REQ-72-5a | Blue-button header strip removed from RepoView | unit (DOM absence) or manual UAT | `vitest run src/components/RepoView.test.ts -t "no review header strip"` |
| REQ-72-5b | DiffPanel close returns to ReviewPanel (regression) | manual UAT | — |
| REQ-72-6 | `just check` green | smoke | `just check` |
| G-71-A | Copy lives on comments view (not preview pane) | covered by REQ-72-3a + REQ-72-4a | — |
| G-71-B | Smooth entry/exit + no dead button | covered by REQ-72-1a + REQ-72-2 + REQ-72-5a | — |

---

## Wave 0 Requirements

- [ ] Confirm whether `RepoView.test.ts` already asserts on the deleted header strip (lines 813-828). If yes, update; if no, accept manual UAT for REQ-72-5a.
- [ ] No framework install required.
- [ ] No new fixtures required — existing `installReads` dispatcher in `ReviewPanel.test.ts:92-113` already handles `generate_review_doc`.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cmd+Shift+R toggles review mode | REQ-72-1b | OS-level menu accelerator binding — Tauri config only | Open app on a repo tab → press Cmd+Shift+R → toolbar Review button shows active state. Press again → returns to default. |
| View → Start/End Code Review still works | REQ-72-1c | Native macOS menu — regression check post-accelerator | Open View menu → confirm "Start Code Review" item present and toggles correctly. |
| DiffPanel close returns to ReviewPanel | REQ-72-5b | Existing wiring confirmed via `reviewSession.showPanel()` — best validated by interaction | Enter review → jump to a diff from a comment → press close → expect comments view (ReviewPanel) restored. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
