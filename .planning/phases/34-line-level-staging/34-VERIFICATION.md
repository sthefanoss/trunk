---
phase: 34-line-level-staging
verified: 2026-03-18T07:30:00Z
status: human_needed
score: 13/13 must-haves verified
human_verification:
  - test: "Click an add line (green) in unstaged diff"
    expected: "Line highlights with brighter green background and pointer cursor appears; hunk toolbar switches from 'Discard Hunk / Stage Hunk' to 'Discard Lines (1) / Stage Lines (1)'"
    why_human: "Visual rendering and CSS background change cannot be verified programmatically"
  - test: "Click 'Stage Lines (N)' button"
    expected: "Selected lines are staged, diff refreshes to reflect the partial staging, selection clears"
    why_human: "End-to-end git operation result and UI refresh require a running app"
  - test: "Unstage Lines from staged diff"
    expected: "Click lines in staged diff view; toolbar shows 'Unstage Lines (N)'; clicking it moves only those lines back to unstaged"
    why_human: "Requires running app with staged content"
  - test: "Discard Lines confirmation dialog"
    expected: "Clicking 'Discard Lines (N)' shows native dialog 'Discard N selected lines? This cannot be undone.'; confirming discards only selected lines"
    why_human: "Native dialog requires Tauri runtime; visual appearance and wording need human check"
  - test: "Escape clears selection"
    expected: "With lines selected, pressing Escape clears selection and returns hunk-mode buttons"
    why_human: "Keyboard event behaviour in running app"
  - test: "Shift+click range selection"
    expected: "Shift+clicking a line selects all add/delete lines between the last clicked and the new click; context lines in the range are skipped"
    why_human: "Mouse interaction and visual range highlight cannot be verified statically"
  - test: "Cross-hunk click clears previous selection"
    expected: "Clicking a line in hunk 1 after selecting in hunk 0 clears hunk 0 selection and starts new selection in hunk 1"
    why_human: "Interaction flow requires running app"
  - test: "Commit diff read-only"
    expected: "In commit diff view, hovering over add/delete lines shows default cursor (not pointer), and clicking does not trigger selection"
    why_human: "Visual cursor state and absence of interaction require manual check"
  - test: "File navigation clears selection"
    expected: "Switching to a different file resets selection state; no stale highlights appear"
    why_human: "State reset on navigation requires running app observation"
---

# Phase 34: Line-Level Staging Verification Report

**Phase Goal:** Enable selecting and staging/unstaging individual lines within a diff hunk, constructing partial patches from line selections.
**Verified:** 2026-03-18T07:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

#### Plan 01 (Backend) Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `stage_lines_inner` stages only selected add/delete lines from a single hunk | VERIFIED | `build_partial_patch_text` skips unselected adds and converts unselected deletes to context; 5 tests pass including `stage_lines_stages_selected_adds`, `stage_lines_stages_selected_deletes`, `stage_lines_mixed_selection` |
| 2 | `unstage_lines_inner` removes only selected lines from the index | VERIFIED | Reverse-mode patch construction wires correctly to `ApplyLocation::Index`; `unstage_lines_unstages_selected` test passes |
| 3 | `discard_lines_inner` reverts only selected lines in the working directory | VERIFIED | Reverse-mode patch applies to `ApplyLocation::WorkDir`; `discard_lines_discards_selected` test passes |
| 4 | Stale hunk index returns an error, not a panic | VERIFIED | `stage_lines_inner` line 813-818, `unstage_lines_inner` line 869, `discard_lines_inner` line 918 all return `TrunkError::new("stale_hunk_index", ...)` when `hunk_index >= patch.num_hunks()`; `stage_lines_stale_index` test asserts `err.code == "stale_hunk_index"` |
| 5 | Partial patch line counts are correct (`old_lines`, `new_lines` recalculated) | VERIFIED | `build_partial_patch_text` maintains `old_count`/`new_count` per-line based on origin and selection; counts used in `@@ -{old_start},{old_count} +{new_start},{new_count} @@` header (staging.rs lines 675-783) |

#### Plan 02 (Frontend) Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 6 | User can click an add/delete line to toggle its selection | VERIFIED (automated) / ? visual | `handleLineClick` wired via `onclick` on every line div (DiffPanel.svelte line 555); toggle logic at lines 171-179 |
| 7 | User can shift-click to select a range of add/delete lines | VERIFIED (automated) / ? visual | Shift-key branch at DiffPanel.svelte lines 161-170; context lines excluded via `hunkLines[i].origin !== 'Context'` check |
| 8 | Context lines are not selectable | VERIFIED | `isSelectable = diffKind !== 'commit' && line.origin !== 'Context'` (line 540); guard in `handleLineClick` (`if (origin === 'Context') return` at line 149) |
| 9 | Selecting a line in a different hunk clears the previous selection | VERIFIED | `handleLineClick` resets `selectedHunkKey`, `selectedLineIndices`, `lastClickedIndex` when `hunkKey !== selectedHunkKey` (lines 153-158) |
| 10 | When lines are selected, toolbar shows 'Stage Lines (N)' / 'Discard Lines (N)' for unstaged or 'Unstage Lines (N)' for staged | VERIFIED | Template renders `Stage Lines ({selectedCount})`, `Discard Lines ({selectedCount})`, `Unstage Lines ({selectedCount})` inside `{#if hasSelection}` branches (lines 435, 453, 513); visual confirmation requires human |
| 11 | When no lines are selected, toolbar shows original hunk-level buttons | VERIFIED | `{:else}` branches render `Stage Hunk`, `Discard Hunk`, `Unstage Hunk` when `hasSelection` is false |
| 12 | Escape clears the selection | VERIFIED | `if (e.key === 'Escape' && selectedCount > 0)` calls `clearSelection()` (DiffPanel.svelte line 48-51) |
| 13 | After a line operation completes, selection clears and diff refreshes | VERIFIED | All three handlers call `clearSelection()` in `finally` block and `await onhunkaction?.(filePath)` on success |

**Score: 13/13 truths verified** (9 fully automated, 4 also needing human visual confirmation)

---

### Required Artifacts

| Artifact | Provides | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/commands/staging.rs` | `build_partial_patch_text`, `stage_lines_inner`, `unstage_lines_inner`, `discard_lines_inner` + 3 async Tauri command wrappers | VERIFIED | All functions present and non-stub; `build_partial_patch_text` at line 662 (126 lines of real implementation); no `todo!()` in file |
| `src-tauri/src/lib.rs` | Tauri command registration for `stage_lines`, `unstage_lines`, `discard_lines` | VERIFIED | Lines 43-45 register all three commands in `invoke_handler` |
| `src/components/DiffPanel.svelte` | Line selection state, click handlers, toolbar mode switching, line-level IPC calls | VERIFIED | `selectedLineIndices` at line 23, `handleLineClick` at line 148, `handleStageLines`/`handleUnstageLines`/`handleDiscardLines` at lines 183-245, toolbar mode switching in template |
| `src/app.css` | CSS custom properties for selected line backgrounds | VERIFIED | `--color-diff-add-bg-selected: rgba(74, 222, 128, 0.25)` at line 18; `--color-diff-delete-bg-selected: rgba(248, 113, 113, 0.25)` at line 19 |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src-tauri/src/commands/staging.rs` | `git2::Diff::from_buffer` | partial patch text parsed into Diff | WIRED | Lines 826, 881, 930 all call `git2::Diff::from_buffer(patch_text.as_bytes())` |
| `src-tauri/src/commands/staging.rs` | `git2::Repository::apply` | apply partial diff to Index or WorkDir | WIRED | Lines 829 (`ApplyLocation::Index` for stage), 884 (`ApplyLocation::Index` for unstage), 933 (`ApplyLocation::WorkDir` for discard) |
| `src/components/DiffPanel.svelte` | `invoke('stage_lines')` | `safeInvoke` IPC call | WIRED | `safeInvoke('stage_lines', {...})` at line 186, passing `lineIndices: Array.from(selectedLineIndices)` |
| `src/components/DiffPanel.svelte` | `invoke('unstage_lines')` | `safeInvoke` IPC call | WIRED | `safeInvoke('unstage_lines', {...})` at line 205 |
| `src/components/DiffPanel.svelte` | `invoke('discard_lines')` | `safeInvoke` IPC call | WIRED | `safeInvoke('discard_lines', {...})` at line 232 |
| `src/app.css` | `src/components/DiffPanel.svelte` | CSS custom properties in `lineBackground()` | WIRED | `lineBackground()` returns `var(--color-diff-add-bg-selected)` and `var(--color-diff-delete-bg-selected)` at DiffPanel.svelte lines 250-251 |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| HUNK-07 | 34-01-PLAN, 34-02-PLAN | User can select and stage individual lines within a diff hunk | SATISFIED | Backend: `stage_lines_inner` + `build_partial_patch_text`; Frontend: `handleStageLines` calls `safeInvoke('stage_lines', ...)`; 3 backend tests verify partial staging; toolbar shows "Stage Lines (N)" |
| HUNK-08 | 34-01-PLAN, 34-02-PLAN | User can select and unstage individual lines within a diff hunk | SATISFIED | Backend: `unstage_lines_inner` with reverse patch construction; Frontend: `handleUnstageLines` calls `safeInvoke('unstage_lines', ...)`; `unstage_lines_unstages_selected` test passes; toolbar shows "Unstage Lines (N)" |

No orphaned requirements — both HUNK-07 and HUNK-08 are claimed in both plan frontmatter fields and implemented.

Note: `discard_lines` (partial discard of working directory changes by line) is implemented beyond what the two requirements specify. It is wired end-to-end and is not a stub.

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| None | — | — | — |

No `todo!()`, `FIXME`, `TODO`, `placeholder`, `return null`, or inline colors found in the modified files. Frontend build (`vite build`) succeeds with 0 errors. All 6 backend tests pass with 0 failures.

---

### Human Verification Required

#### 1. Line Click Selection Visual

**Test:** Run `cargo tauri dev`, open a repo with unstaged changes, click on a green (add) diff line.
**Expected:** The line background brightens (rgba 0.25 vs 0.10 alpha), cursor changes to pointer, and the hunk toolbar switches to "Discard Lines (1) / Stage Lines (1)".
**Why human:** CSS background change and cursor are visual; cannot be asserted from static analysis.

#### 2. Stage Lines End-to-End

**Test:** Select one or more add lines in the unstaged diff, click "Stage Lines (N)".
**Expected:** Only those lines appear in the staged diff. Unselected add lines remain in the unstaged diff. Selection clears and both diff panes refresh.
**Why human:** Requires live git index state after the apply operation.

#### 3. Unstage Lines End-to-End

**Test:** Stage a file, switch to staged diff, click individual lines, click "Unstage Lines (N)".
**Expected:** Only those lines move back to the unstaged diff; remaining staged lines stay staged.
**Why human:** Requires live git index state and staging panel refresh.

#### 4. Discard Lines Confirmation Dialog

**Test:** Select lines in unstaged diff, click "Discard Lines (N)".
**Expected:** Native dialog appears with text "Discard N selected lines? This cannot be undone." and title "Discard Lines". Confirming discards exactly those lines from the working file; cancelling leaves them intact.
**Why human:** Native Tauri dialog requires a running app; visual dialog text needs human confirmation.

#### 5. Escape Key Clears Selection

**Test:** Select lines in any hunk, press Escape.
**Expected:** All line highlights clear and toolbar reverts to "Stage Hunk / Discard Hunk".
**Why human:** Keyboard interaction in running app.

#### 6. Shift+Click Range Selection

**Test:** Click line 2 in a hunk, then Shift+click line 6 (which includes context lines in the range).
**Expected:** Lines 2-6 that are add/delete are selected; context lines in range are skipped (no highlight, still default cursor).
**Why human:** Multi-step mouse interaction and visual range highlight.

#### 7. Cross-Hunk Selection Reset

**Test:** Select a line in hunk 0, then click a line in hunk 1.
**Expected:** Hunk 0 selection clears immediately; hunk 1 shows one selected line with selection-mode toolbar.
**Why human:** Multi-hunk interaction flow in running app.

#### 8. Commit Diff Read-Only

**Test:** Navigate to a commit diff (history view). Hover over and click add/delete lines.
**Expected:** Cursor remains default (not pointer); no lines highlight; toolbar shows no action buttons (commit view has no stage/discard).
**Why human:** Visual cursor appearance requires manual observation; absence of interaction cannot be asserted without running app.

#### 9. File Navigation Clears Selection

**Test:** Select lines in a file, then click a different file in the file list.
**Expected:** No stale selection highlights appear in the newly displayed diff.
**Why human:** Navigation flow and state reset require running app.

---

### Summary

All 13 must-have truths are verified at the code level. The backend (Plan 01) is fully automated-verifiable: 6 unit tests cover selected adds, selected deletes, mixed selection, stale hunk index, unstage, and discard — all pass. The Rust code is clean with no stubs. The frontend (Plan 02) passes static analysis: all state variables, handlers, toolbar mode switching, IPC calls, and CSS property references are present and wired correctly. The build compiles with zero errors.

The only open items are the 9 human tests listed above — these verify visual appearance, end-to-end git operations in a live repo, native dialog rendering, and keyboard/mouse interaction flows that cannot be confirmed from static analysis alone.

---

_Verified: 2026-03-18T07:30:00Z_
_Verifier: Claude (gsd-verifier)_
