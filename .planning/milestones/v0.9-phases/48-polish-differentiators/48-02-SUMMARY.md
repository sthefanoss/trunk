---
phase: 48-polish-differentiators
plan: 02
subsystem: ui
tags: [svelte, tree-view, staging, lucide-icons, directory-actions]

# Dependency graph
requires:
  - phase: 47-tree-view-ui-integration
    provides: "TreeFileList, DirectoryRow, buildTree, flattenTree components and utilities"
provides:
  - "countFiles utility for recursive file counting in tree nodes"
  - "collectFilePaths utility for recursive path collection from tree nodes"
  - "Directory count badges on DirectoryRow"
  - "Hover stage/unstage action button on DirectoryRow"
  - "Directory stage/unstage via ondirectoryaction callback"
  - "Expand All / Collapse All buttons in StagingPanel header"
affects: [staging-panel, tree-view]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Signal-based expand/collapse (increment counter triggers effect)"
    - "Directory staging via flat file path prefix matching + Promise.all"

key-files:
  created: []
  modified:
    - src/lib/build-tree.ts
    - src/components/DirectoryRow.svelte
    - src/components/TreeFileList.svelte
    - src/components/StagingPanel.svelte

key-decisions:
  - "Directory staging uses flat file list prefix matching rather than tree traversal for simplicity"
  - "Expand/collapse uses signal counter pattern (increment to trigger) rather than direct method calls"
  - "DirectoryRow action button uses CSS custom properties (--color-success, --color-danger) per CLAUDE.md rules"

patterns-established:
  - "Signal counter pattern: parent increments $state counter, child $effect detects change and acts"
  - "Directory action threading: StagingPanel -> TreeFileList -> DirectoryRow via callback chain"

requirements-completed: [TREE-08, TREE-09, TREE-10]

# Metrics
duration: 7min
completed: 2026-03-25
---

# Phase 48 Plan 02: Tree View Power Features Summary

**Directory count badges, hover stage/unstage buttons on directories, and Expand All / Collapse All header buttons for tree view**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-25T02:14:42Z
- **Completed:** 2026-03-25T02:21:56Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Directory nodes display recursive file count badges (e.g., "(3)") in muted text
- Directory rows show hover stage/unstage action buttons matching FileRow pattern
- Clicking directory action stages or unstages all files recursively under that directory
- Expand All / Collapse All icon buttons appear in staging panel header when tree mode is active
- All four TreeFileList instances (rebase conflicted, merge conflicted, unstaged, staged) respond to expand/collapse signals

## Task Commits

Each task was committed atomically:

1. **Task 1: Count badge utility, DirectoryRow action button and badge, TreeFileList ondirectoryaction** - `9761995` (feat)
2. **Task 2: Directory staging logic and Expand All / Collapse All buttons in StagingPanel** - `198a2a1` (feat)

## Files Created/Modified
- `src/lib/build-tree.ts` - Added countFiles and collectFilePaths recursive utility functions
- `src/components/DirectoryRow.svelte` - Added count badge, hover action button with Plus/Minus icons, new actionLabel/onaction props
- `src/components/TreeFileList.svelte` - Added ondirectoryaction, expandAllSignal, collapseAllSignal props with effects; passes action props to DirectoryRow
- `src/components/StagingPanel.svelte` - Added stageDirectory/unstageDirectory functions, ChevronsUpDown/ChevronsDownUp buttons, wired all TreeFileList instances

## Decisions Made
- Directory staging uses flat file list prefix matching (filtering status.unstaged/staged by path prefix) rather than building a tree and traversing it -- simpler and avoids redundant tree construction
- Expand/collapse uses a signal counter pattern where the parent increments a number and the child detects the change via $effect -- avoids needing bind:this or imperative method calls
- Action button colors use CSS custom properties (--color-success, --color-danger) per CLAUDE.md rules, unlike the pre-existing FileRow which uses inline hex colors

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Tree view power features complete -- directory badges, actions, and expand/collapse all functional
- Ready for Plan 03 (next plan in phase 48)

## Self-Check: PASSED

All 4 modified files verified on disk. Both task commits (9761995, 198a2a1) verified in git log.

---
*Phase: 48-polish-differentiators*
*Completed: 2026-03-25*
