---
phase: 34-line-level-staging
verified: 2026-03-19T01:00:00Z
status: human_needed
score: 14/14 must-haves verified
re_verification:
  previous_status: human_needed
  previous_score: 13/13
  gaps_closed:
    - "Shift+click range selection works without triggering browser text selection"
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Shift+click range selection no longer triggers browser text selection"
    expected: "Click a diff line to select it, then shift+click another line in the same hunk. Only the custom line highlights appear — no browser text selection band or text cursor appears."
    why_human: "Visual absence of browser text selection requires a running app; cannot be asserted from static analysis"
  - test: "Click an add line (green) in unstaged diff"
    expected: "Line highlights with brighter green background and pointer cursor; hunk toolbar switches from 'Discard Hunk / Stage Hunk' to 'Discard Lines (1) / Stage Lines (1)'"
    why_human: "Visual rendering and CSS background change cannot be verified programmatically"
  - test: "Stage Lines end-to-end"
    expected: "Selected lines are staged; unselected add lines remain in unstaged diff; diff panes refresh and selection clears"
    why_human: "Requires live git index state after apply operation"
  - test: "Unstage Lines end-to-end"
    expected: "Click lines in staged diff; toolbar shows 'Unstage Lines (N)'; clicking it moves only those lines back to unstaged"
    why_human: "Requires running app with staged content"
  - test: "Discard Lines confirmation dialog"
    expected: "Clicking 'Discard Lines (N)' shows native dialog with correct title/message; confirming discards only selected lines"
    why_human: "Native Tauri dialog requires runtime; visual wording needs human check"
  - test: "Escape clears selection"
    expected: "With lines selected, pressing Escape clears all highlights and returns hunk-mode buttons"
    why_human: "Keyboard interaction in running app"
  - test: "Cross-hunk click clears previous selection"
    expected: "Clicking a line in hunk 1 after selecting in hunk 0 clears hunk 0 selection and starts new selection in hunk 1"
    why_human: "Multi-hunk interaction flow requires running app"
  - test: "Commit diff read-only"
    expected: "In commit diff view, hovering over add/delete lines shows default cursor, no selection triggers"
    why_human: "Visual cursor state and absence of interaction require manual check"
  - test: "File navigation clears selection"
    expected: "Switching to a different file shows no stale highlights"
    why_human: "State reset on navigation requires running app observation"
---

# Phase 34: Line-Level Staging Verification Report

**Phase Goal:** Line-level staging with click/shift-click selection, stage/unstage/discard operations
**Verified:** 2026-03-19T01:00:00Z
**Status:** human_needed
**Re-verification:** Yes — after UAT gap closure (shift+click text selection fix, Plan 03)

---

## Re-Verification Summary

The previous verification (`2026-03-18T07:30:00Z`) passed all 13 automated must-haves and flagged 9 human tests. UAT was subsequently run and revealed one gap: shift+click range selection also triggered browser native text selection (Test 2, minor severity).

Gap closure plan 34-03 was executed and committed at `5226d4c`. This re-verification confirms the fix is present and correct, bringing the total verified must-haves to 14/14.

---

## Goal Achievement

### Observable Truths

#### Plan 01 (Backend) Truths — carried from initial verification, no regression

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `stage_lines_inner` stages only selected add/delete lines from a single hunk | VERIFIED | `build_partial_patch_text` present and non-stub; 5 unit tests pass |
| 2 | `unstage_lines_inner` removes only selected lines from the index | VERIFIED | Reverse-mode patch wired to `ApplyLocation::Index`; test passes |
| 3 | `discard_lines_inner` reverts only selected lines in the working directory | VERIFIED | Reverse-mode patch wired to `ApplyLocation::WorkDir`; test passes |
| 4 | Stale hunk index returns an error, not a panic | VERIFIED | All three inner functions return `TrunkError::new("stale_hunk_index", ...)` when index out of bounds |
| 5 | Partial patch line counts are correct (`old_lines`, `new_lines` recalculated) | VERIFIED | `build_partial_patch_text` maintains per-line counts in `@@` header |

#### Plan 02 (Frontend) Truths — carried from initial verification, no regression

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 6 | User can click an add/delete line to toggle its selection | VERIFIED | `handleLineClick` wired via `onclick` at DiffPanel.svelte line 558 |
| 7 | User can shift-click to select a range of add/delete lines | VERIFIED | Shift-key branch at lines 161-171; context lines excluded |
| 8 | Context lines are not selectable | VERIFIED | `isSelectable = diffKind !== 'commit' && line.origin !== 'Context'`; guard in `handleLineClick` line 149 |
| 9 | Selecting a line in a different hunk clears the previous selection | VERIFIED | `hunkKey !== selectedHunkKey` branch at lines 154-158 |
| 10 | Toolbar shows line-count buttons when lines are selected, hunk buttons when not | VERIFIED | `{#if hasSelection}` branches render correct button text in template |
| 11 | Escape clears the selection | VERIFIED | `e.key === 'Escape'` handler at DiffPanel.svelte lines 48-51 |
| 12 | After a line operation, selection clears and diff refreshes | VERIFIED | All three handlers call `clearSelection()` in `finally` block |
| 13 | Commit diff is read-only (no selection possible) | VERIFIED | `diffKind === 'commit'` guard in both `isSelectable` derived const and `handleLineClick` |

#### Plan 03 (Gap Closure) Truth — new, verified in this run

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 14 | Shift+click range selection does NOT trigger browser text selection | VERIFIED (automated) / ? visual | `onmousedown={(e) => { if (isSelectable && e.shiftKey) e.preventDefault(); }}` at DiffPanel.svelte line 557; placed before `onclick` on same div so mousedown fires before click; secondary guard `e.preventDefault()` at line 162 remains intact; commit `5226d4c` |

**Score: 14/14 truths verified** (all pass automated checks; visual confirmation for the new fix still needs human)

---

### Required Artifacts

| Artifact | Provides | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/commands/staging.rs` | `build_partial_patch_text`, `stage_lines_inner`, `unstage_lines_inner`, `discard_lines_inner` | VERIFIED | All present and non-stub; no regression |
| `src-tauri/src/lib.rs` | Tauri command registration for `stage_lines`, `unstage_lines`, `discard_lines` | VERIFIED | Lines 43-45 unchanged |
| `src/components/DiffPanel.svelte` | Line selection state, click handlers, toolbar switching, IPC calls, shift+click text-selection prevention | VERIFIED | `onmousedown` handler at line 557 added; all existing logic intact at lines 148-248 |
| `src/app.css` | CSS custom properties for selected line backgrounds | VERIFIED | `--color-diff-add-bg-selected` and `--color-diff-delete-bg-selected` unchanged |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `DiffPanel.svelte onmousedown` | `handleLineClick shift+click branch` | mousedown fires before click, preventing text selection before selection starts | WIRED | `onmousedown` at line 557 checks `isSelectable && e.shiftKey`; sits on same div as `onclick`; secondary guard at line 162 intact |
| `DiffPanel.svelte` | `invoke('stage_lines')` | `safeInvoke` IPC call | WIRED | unchanged from initial verification |
| `DiffPanel.svelte` | `invoke('unstage_lines')` | `safeInvoke` IPC call | WIRED | unchanged |
| `DiffPanel.svelte` | `invoke('discard_lines')` | `safeInvoke` IPC call | WIRED | unchanged |
| `src/app.css` | `DiffPanel.svelte` | CSS custom properties in `lineBackground()` | WIRED | unchanged |
| `staging.rs` | `git2::Repository::apply` | partial patch applied to Index or WorkDir | WIRED | unchanged |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| HUNK-07 | 34-01-PLAN, 34-02-PLAN, 34-03-PLAN | User can select and stage individual lines within a diff hunk | SATISFIED | Backend: `stage_lines_inner` + `build_partial_patch_text`; Frontend: `handleStageLines` IPC call; gap fix: `onmousedown` prevents text selection during shift+click range selection |
| HUNK-08 | 34-01-PLAN, 34-02-PLAN | User can select and unstage individual lines within a diff hunk | SATISFIED | Backend: `unstage_lines_inner`; Frontend: `handleUnstageLines` IPC call |

No orphaned requirements.

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| None | — | — | — |

The gap closure change (Plan 03) adds a single-line `onmousedown` handler. No stubs, no TODOs, no console.log-only implementations, no inline colors introduced.

---

### Human Verification Required

#### 1. Shift+Click Text Selection Fix (UAT Test 2 — previously failed, now fixed)

**Test:** Run `cargo tauri dev`, open a repo with unstaged changes, click an add/delete line to anchor a selection, then Shift+click another line in the same hunk.
**Expected:** Only the custom line highlight band appears. No browser text selection (blue text band, text cursor, clipboard selection) occurs anywhere in the diff view.
**Why human:** Visual absence of browser text selection cannot be asserted from static analysis; only observable in a running Tauri app.

#### 2. Line Click Selection Visual

**Test:** Click on a green (add) diff line in the unstaged diff.
**Expected:** Line background brightens (rgba 0.25 vs 0.10 alpha), cursor is pointer, toolbar switches to "Discard Lines (1) / Stage Lines (1)".
**Why human:** CSS background change and cursor are visual.

#### 3. Stage Lines End-to-End

**Test:** Select a subset of add lines in an unstaged hunk, click "Stage Lines (N)".
**Expected:** Only selected lines appear in the staged diff; unselected add lines remain unstaged; selection clears; both panes refresh.
**Why human:** Requires live git index state after apply operation.

#### 4. Unstage Lines End-to-End

**Test:** Stage a file, view staged diff, click individual lines, click "Unstage Lines (N)".
**Expected:** Only those lines move back to unstaged diff; remaining staged lines stay staged.
**Why human:** Requires live git index state and staging panel refresh.

#### 5. Discard Lines Confirmation Dialog

**Test:** Select lines in unstaged diff, click "Discard Lines (N)".
**Expected:** Native dialog with title "Discard Lines" and body "Discard N selected lines? This cannot be undone." Confirming discards exactly those lines; cancelling leaves them intact.
**Why human:** Native Tauri dialog requires runtime; visual dialog wording needs human confirmation.

#### 6. Escape Key Clears Selection

**Test:** Select lines in any hunk, press Escape.
**Expected:** All line highlights clear and toolbar reverts to "Stage Hunk / Discard Hunk".
**Why human:** Keyboard event interaction in running app.

#### 7. Cross-Hunk Selection Reset

**Test:** Select a line in hunk 0, then click a line in hunk 1.
**Expected:** Hunk 0 highlights clear immediately; hunk 1 shows one selected line and selection-mode toolbar.
**Why human:** Multi-hunk interaction flow requires running app.

#### 8. Commit Diff Read-Only

**Test:** Navigate to a commit diff (history view). Hover over and click add/delete lines.
**Expected:** Cursor is default (not pointer); no lines highlight; no toolbar action buttons.
**Why human:** Visual cursor state and absence of interaction need manual observation.

#### 9. File Navigation Clears Selection

**Test:** Select lines in a file, then click a different file in the file list.
**Expected:** New file's diff shows no stale selection highlights.
**Why human:** State reset on navigation requires running app observation.

---

### Gaps Summary

No gaps remain. The single gap identified during UAT (shift+click text selection, Test 2) was addressed by Plan 03. The fix is verified at `src/components/DiffPanel.svelte` line 557:

```
onmousedown={(e) => { if (isSelectable && e.shiftKey) e.preventDefault(); }}
```

This handler sits on the same div as `onclick`, ensuring `mousedown` fires before `click` and stops the browser from initiating text selection before the range-select logic runs. The secondary guard `e.preventDefault()` inside `handleLineClick`'s shift branch at line 162 remains intact as defense-in-depth. Confirmed in commit `5226d4c`.

All 14 must-have truths pass automated verification. The remaining 9 human tests cover visual and runtime behaviors that cannot be confirmed statically.

---

_Verified: 2026-03-19T01:00:00Z_
_Verifier: Claude (gsd-verifier)_
_Re-verification after: UAT gap closure — Plan 03 shift+click text selection fix_
