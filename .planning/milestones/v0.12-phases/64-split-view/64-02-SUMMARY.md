---
phase: 64-split-view
plan: 02
subsystem: ui
tags: [svelte, typescript, diff-viewer, split-view, scroll-sync, resizable-divider]

requires:
  - phase: 64-split-view
    provides: ContentMode and LayoutMode types, 2D dispatch in DiffViewer, SplitView stub with full prop interface, --color-diff-phantom-bg CSS variable
provides:
  - SplitView.svelte with paired-row alignment, phantom spacers, synchronized scrolling, resizable divider
  - pairLines() utility function and PairedRow interface in diff-utils.ts
  - Split+hunk mode with hunk headers and staging buttons
  - Split+full mode with continuous document rendering
affects: [64-03-PLAN]

tech-stack:
  added: []
  patterns:
    - "Paired-row alignment: pairLines() transforms DiffLine[] into PairedRow[] with phantom spacers"
    - "Scroll sync: boolean syncing guard prevents infinite scroll event recursion"
    - "Resizable split: mousedown/mousemove/mouseup pattern with 20-80% clamp"

key-files:
  created: []
  modified:
    - src/lib/diff-utils.ts
    - src/components/diff/SplitView.svelte
    - src/components/DiffPanel.test.ts

key-decisions:
  - "Two independent scroll panels with syncScroll for vertical sync, independent horizontal scroll per panel"
  - "Hunk headers rendered in both panels: left shows header text, right shows staging buttons"
  - "Line selection only on right panel Add lines per D-15, using original lineIdx from PairedRow"
  - "No origin symbols in split view -- color backgrounds indicate change type per D-10"

patterns-established:
  - "pairLines() row pairing: context on both sides, consecutive deletes paired with adds, phantom rows for mismatches"
  - "syncScroll with boolean guard: set syncing=true before assigning scrollTop, check at handler entry"

requirements-completed: [VIEW-02, VIEW-03]

duration: 10min
completed: 2026-03-30
---

# Phase 64 Plan 02: SplitView Component Summary

**Side-by-side diff renderer with pairLines() row alignment, phantom spacers, synchronized vertical scrolling, resizable divider, and split+hunk/split+full mode support**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-30T15:45:08Z
- **Completed:** 2026-03-30T15:55:11Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Implemented full SplitView.svelte with two scrollable panels, resizable divider, and paired-row alignment
- Added pairLines() utility to diff-utils.ts that transforms DiffLine[] into PairedRow[] with phantom spacer rows
- Split+hunk mode renders hunk headers with staging buttons; split+full mode renders continuous document
- 9 new tests: 6 pairLines unit tests covering all pairing scenarios + 3 VIEW-02 integration tests
- All 49 DiffPanel tests pass, svelte-check reports 0 errors

## Task Commits

Each task was committed atomically:

1. **Task 1: Add pairLines() utility and implement SplitView component** - `905f059` (feat)
2. **Task 2: Add VIEW-02 and VIEW-03 tests for split view** - `ed9d16a` (test)

## Files Created/Modified
- `src/lib/diff-utils.ts` - Added PairedRow interface and pairLines() function for split view row alignment
- `src/components/diff/SplitView.svelte` - Full side-by-side diff renderer replacing stub (paired rows, phantom spacers, scroll sync, resizable divider, staging support, syntax highlighting, word-diff)
- `src/components/DiffPanel.test.ts` - 6 pairLines unit tests + 3 VIEW-02 integration tests, updated split view stub test

## Decisions Made
- Two independent scroll panels with syncScroll for vertical sync -- matches VS Code behavior and allows independent horizontal scroll per panel
- Hunk headers rendered in both panels: left shows hunk header text, right shows staging buttons -- keeps scroll containers independent
- Line selection only on right panel Add lines using original lineIdx from PairedRow -- correct staging callback indices
- No origin symbols (+/-/space) in split view per D-10 -- color backgrounds suffice for change type indication

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Biome lint errors in SplitView and test imports**
- **Found during:** Task 2 (test verification)
- **Issue:** Biome flagged non-null assertion in startResize, incorrect import ordering in SplitView.svelte and DiffPanel.test.ts, unused PairedRow type import in test file
- **Fix:** Replaced non-null assertion with null guard, reordered imports per Biome organizeImports, removed unused import, ran Biome format
- **Files modified:** src/components/diff/SplitView.svelte, src/components/DiffPanel.test.ts
- **Verification:** `npx @biomejs/biome check .` passes on both files
- **Committed in:** 0174130 (separate fix commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Fix necessary for Biome compliance. No scope creep.

## Issues Encountered
- Pre-existing TabBar.test.ts failure ("calls onactivate when tab clicked") confirmed unrelated to this plan's changes -- out of scope per deviation rules (documented in Plan 01 SUMMARY as well)

## Known Stubs
None -- SplitView is now fully implemented replacing the "coming soon" stub from Plan 01.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- SplitView renders paired rows with all styling (syntax highlighting, word-diff, invisible chars, trailing whitespace)
- Staging interactions (hunk-level and line-level) wired through split view
- Ready for Plan 03 integration and end-to-end verification

## Self-Check: PASSED

All 3 modified files verified present. All 3 task commits (905f059, ed9d16a, 0174130) verified in git log.

---
*Phase: 64-split-view*
*Completed: 2026-03-30*
