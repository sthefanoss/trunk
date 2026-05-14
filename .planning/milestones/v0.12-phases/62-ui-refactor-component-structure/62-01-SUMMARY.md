---
phase: 62-ui-refactor-component-structure
plan: 01
subsystem: ui
tags: [svelte5, component-decomposition, diff-viewer, segmented-control, line-numbers]

# Dependency graph
requires:
  - phase: 61-syntax-highlighting
    provides: MergedSpan rendering, syn-* CSS classes, word-diff highlights
provides:
  - ViewMode type and LazyStore persistence (getDiffViewMode/setDiffViewMode)
  - DiffToolbar component with segmented control (Hunk/Full/Split)
  - DiffViewer dispatcher component for view mode routing
  - HunkView component with line number gutter extracted from DiffPanel
  - FullFileView and SplitView stub placeholder components
  - Thin-shell DiffPanel pattern (state owner delegating to child components)
affects: [62-02, 63-full-file-view, 64-split-view]

# Tech tracking
tech-stack:
  added: []
  patterns: [thin-shell-parent, view-mode-dispatcher, segmented-control, line-number-gutter]

key-files:
  created:
    - src/components/diff/DiffToolbar.svelte
    - src/components/diff/DiffViewer.svelte
    - src/components/diff/HunkView.svelte
    - src/components/diff/FullFileView.svelte
    - src/components/diff/SplitView.svelte
  modified:
    - src/lib/types.ts
    - src/lib/store.ts
    - src/components/DiffPanel.svelte

key-decisions:
  - "Graceful .catch() on getDiffViewMode $effect to handle test environment without LazyStore"
  - "diff-line-content wrapper span to maintain getByText test compatibility with gutter columns"
  - "hunkElements declared as $state<Record<string, HTMLDivElement>> for cross-boundary reactivity"

patterns-established:
  - "Thin-shell parent: DiffPanel owns state and handlers, delegates rendering to child components"
  - "View mode dispatcher: DiffViewer routes to HunkView/FullFileView/SplitView based on ViewMode prop"
  - "Segmented control: inline-flex button group with active class styling for mode toggles"
  - "Line number gutter: two fixed-width ch-unit columns (old/new) computed from max line number per file"

requirements-completed: [DISP-01, VIEW-01]

# Metrics
duration: 5min
completed: 2026-03-29
---

# Phase 62 Plan 01: UI Refactor & Component Structure Summary

**Decomposed 667-line DiffPanel monolith into 5 focused diff/ components with view mode segmented control and two-column line number gutter**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-29T13:02:30Z
- **Completed:** 2026-03-29T13:07:19Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Decomposed DiffPanel from 667-line monolith into thin-shell parent (~260 lines) delegating to 5 child components
- Added view mode segmented control (Hunk/Full/Split) with LazyStore persistence in DiffToolbar
- Added two-column line number gutter (old + new) with dynamic ch-unit width sizing in HunkView
- All 16 existing DiffPanel tests pass unchanged through the new component hierarchy
- Full test suite (378 tests across 41 files) passes with zero regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ViewMode type, store functions, and create DiffToolbar + stub components** - `e365971` (feat)
2. **Task 2: Create HunkView, DiffViewer, and refactor DiffPanel into thin shell** - `449fa78` (feat)

## Files Created/Modified
- `src/lib/types.ts` - Added ViewMode = "hunk" | "full" | "split" union type
- `src/lib/store.ts` - Added getDiffViewMode/setDiffViewMode LazyStore persistence
- `src/components/diff/DiffToolbar.svelte` - Toolbar with segmented control, filename, file actions, close button
- `src/components/diff/DiffViewer.svelte` - View mode dispatcher routing to HunkView/FullFileView/SplitView
- `src/components/diff/HunkView.svelte` - Extracted hunk rendering with line number gutter and all CSS classes
- `src/components/diff/FullFileView.svelte` - Stub placeholder for full file view
- `src/components/diff/SplitView.svelte` - Stub placeholder for split view
- `src/components/DiffPanel.svelte` - Refactored to thin shell owning state, staging handlers, and keyboard navigation

## Decisions Made
- Added `.catch(() => {})` on `getDiffViewMode()` $effect to gracefully handle test environment where LazyStore is unavailable (Rule 1 - prevents unhandled rejection errors in test suite)
- Wrapped origin+content in a `diff-line-content` span to ensure `getByText` test compatibility when gutter columns are prepended to each line
- Declared `hunkElements` as `$state<Record<string, HTMLDivElement>>({})` in DiffPanel for cross-boundary reactivity with HunkView's `bind:this`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added .catch() on getDiffViewMode $effect**
- **Found during:** Task 2 (DiffPanel refactor)
- **Issue:** The `$effect` calling `getDiffViewMode()` triggers an unhandled rejection in test environments where LazyStore/Tauri invoke is not available, causing 16 error reports despite all tests passing
- **Fix:** Added `.catch(() => {})` to the promise chain to silently handle store unavailability (default "hunk" mode is already correct)
- **Files modified:** src/components/DiffPanel.svelte
- **Verification:** All 16 DiffPanel tests pass with zero errors
- **Committed in:** 449fa78 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Essential for test compatibility. No scope creep.

## Known Stubs

- `src/components/diff/FullFileView.svelte` - Placeholder "coming soon" text, will be implemented in Phase 63
- `src/components/diff/SplitView.svelte` - Placeholder "coming soon" text, will be implemented in Phase 64

These stubs are intentional per the plan (D-03) and are documented here for tracking. They do not prevent the plan's goal (component decomposition and view mode architecture) from being achieved.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Component architecture in place for Phase 62-02 (additional UI refinements)
- FullFileView stub ready for Phase 63 implementation
- SplitView stub ready for Phase 64 implementation
- ViewMode persistence functional, DiffViewer dispatching correctly

## Self-Check: PASSED

All 8 files verified present. Both task commits (e365971, 449fa78) verified in git log.

---
*Phase: 62-ui-refactor-component-structure*
*Completed: 2026-03-29*
