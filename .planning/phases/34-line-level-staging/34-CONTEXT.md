# Phase 34: Line-Level Staging - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Enable selecting and staging/unstaging individual lines within a diff hunk, constructing partial patches from line selections. Requirements: HUNK-07, HUNK-08. Also includes line-level discard (symmetry with existing Discard Hunk).

</domain>

<decisions>
## Implementation Decisions

### Line selection model
- **Click to toggle** individual add/delete lines — each click adds/removes a line from the selection
- **Shift+click** to select a range of lines between last-clicked and shift-clicked line
- **Context lines are not selectable** — only add and delete lines respond to clicks
- **Selection scoped to a single hunk** — clicking a line in a different hunk clears the previous selection
- **Symmetric for staged diffs** — same click-to-select interaction works in staged diffs for "Unstage Lines"
- **Commit diffs** remain read-only (no selection)

### Selection visuals
- Selected lines get a **brighter/more saturated version** of their add/delete background color (unselected lines stay at current muted level)
- **Pointer cursor** on add/delete lines to signal clickability; context lines keep default cursor
- Selection clears **after operation completes** (diff re-fetches anyway) and **when navigating** to a different file or clicking a line in another hunk

### Stage Lines button placement
- "Stage Lines (N)" appears **in the hunk toolbar row alongside "Stage Hunk"** when lines are selected in that hunk
- Button label **includes count** of selected lines: "Stage Lines (3)"
- In staged diffs: "Unstage Lines (N)" appears alongside "Unstage Hunk" when lines are selected
- No explicit "Clear selection" button — **Escape key clears** selection, as does clicking in another hunk

### Discard lines
- When lines are selected in an unstaged diff, "Discard Lines (N)" replaces "Discard Hunk" (or appears alongside)
- Confirmation dialog includes count: **"Discard 3 selected lines? This cannot be undone."**
- Uses same `ask()` from `@tauri-apps/plugin-dialog` — consistent with Discard Hunk pattern
- Backend constructs reversed partial patch and applies to working directory

### Claude's Discretion
- Partial patch construction algorithm (how to build a valid git patch from selected lines within a hunk)
- CSS custom properties for selected-line highlight colors (brighter variants of existing diff colors)
- Backend command signatures (`stage_lines`, `unstage_lines`, `discard_lines`) and parameter design
- Whether to send line indices or line content to the backend
- Escape key handler integration with existing keyboard shortcut system

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### DiffPanel (primary integration point)
- `src/components/DiffPanel.svelte` — Current hunk toolbar, diff line rendering, keyboard navigation, hunkOperationInFlight pattern. Lines are rendered as divs with origin-based background colors. This is where click-to-select and visual highlighting will be added.

### Backend hunk commands (Phase 32 output)
- `src-tauri/src/commands/staging.rs` — `stage_hunk_inner`, `unstage_hunk_inner`, `discard_hunk_inner` functions. Uses `ApplyOptions::hunk_callback` for single-hunk extraction. New line-level commands will follow similar pattern with partial patch construction.

### Diff type definitions
- `src-tauri/src/git/types.rs` — Rust `DiffHunk`, `DiffLine`, `FileDiff` structs. `DiffLine` has `origin`, `content`, `old_lineno`, `new_lineno`.
- `src/lib/types.ts` — TypeScript mirror types. `DiffLine` has `origin: DiffOrigin`, `content`, `old_lineno`, `new_lineno`.

### IPC & error handling
- `src/lib/invoke.ts` — `safeInvoke<T>` with `TrunkError` parsing
- `src/lib/toast.svelte.ts` — `showToast(message, kind)` for feedback

### Existing patterns
- `src/components/App.svelte` — `handleFileSelect(path, kind)`, keyboard shortcut pattern via global `$effect`
- `src/components/StagingPanel.svelte` — `ask()` for destructive confirmations, `loadStatus()` refresh

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `hunkOperationInFlight`: Boolean flag disabling all hunk buttons during operations — extend for line operations
- `safeInvoke<T>`: IPC wrapper — use for new line-level commands
- `showToast`: Toast feedback — use for line operation results
- `ask()` from `@tauri-apps/plugin-dialog`: Confirmation dialog — use for discard lines
- `lineBackground()` / `lineColor()`: Existing line styling functions — extend with selected variants
- `ApplyOptions::hunk_callback`: Used in Phase 32 for single-hunk extraction — similar approach for partial patch

### Established Patterns
- Diff lines rendered as individual `<div>` elements with inline styles — click handlers can be added per-div
- `DiffLine` has `origin` field ('Add' | 'Delete' | 'Context') — use to determine selectability
- `DiffLine` has `old_lineno` and `new_lineno` — can be used to identify lines for backend
- `focusedHunkIndex` + `hunkElements` pattern for hunk tracking — extend for line selection state
- Escape key not currently bound — available for clear-selection

### Integration Points
- New backend commands register in `src-tauri/src/lib.rs` invoke_handler
- Line selection state lives in DiffPanel component (`$state` for selected line indices per hunk)
- After line operation: re-fetch diff (existing `onhunkaction` callback) + clear selection
- CSS custom properties for highlight colors go in theme file (per feedback: never inline colors)

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 34-line-level-staging*
*Context gathered: 2026-03-18*
