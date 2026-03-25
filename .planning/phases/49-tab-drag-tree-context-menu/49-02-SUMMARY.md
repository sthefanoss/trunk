---
phase: 49-tab-drag-tree-context-menu
plan: 02
subsystem: ui
tags: [svelte, context-menu, tauri-menu, tree-view, staging]

requires:
  - phase: 48-polish-differentiators
    provides: directory staging, tree view, expand/collapse signals
provides:
  - directory right-click context menus for bulk stage/unstage/discard/resolve
  - oncontextmenu prop on DirectoryRow for right-click handling
  - ondirectorycontextmenu prop on TreeFileList threaded to DirectoryRow
affects: []

tech-stack:
  added: []
  patterns:
    - "Directory context menu uses same dynamic import pattern as file context menus (@tauri-apps/api/menu)"
    - "Discard directory shows confirmation via @tauri-apps/plugin-dialog ask()"

key-files:
  created: []
  modified:
    - src/components/DirectoryRow.svelte
    - src/components/TreeFileList.svelte
    - src/components/StagingPanel.svelte

key-decisions:
  - "Directory context menus use native Tauri menus matching existing file context menu pattern"
  - "Menu item text includes file count for user clarity (e.g. 'Stage All (5)')"

patterns-established:
  - "ondirectorycontextmenu prop pattern: TreeFileList threads (e, dirPath) to parent, matching onfilecontextmenu pattern"

requirements-completed: [TREE-11]

duration: 2min
completed: 2026-03-25
---

# Phase 49 Plan 02: Tree Directory Context Menus Summary

**Right-click context menus on directory nodes for bulk stage/unstage/discard/resolve using native Tauri menus with file count display and discard confirmation**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-25T03:19:39Z
- **Completed:** 2026-03-25T03:22:13Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- DirectoryRow accepts oncontextmenu prop with e.preventDefault() for right-click handling on directory nodes only
- TreeFileList accepts ondirectorycontextmenu and threads it to DirectoryRow with directory path
- StagingPanel has directory context menu handlers for all three sections: unstaged (Stage All + Discard All), staged (Unstage All), conflicted (Resolve All + Unresolve All)
- Discard All shows native confirmation dialog before executing
- All context menus display file count in menu item text

## Task Commits

Each task was committed atomically:

1. **Task 1: Add oncontextmenu prop to DirectoryRow and ondirectorycontextmenu to TreeFileList** - `007405e` (feat)
2. **Task 2: Create directory context menu handlers in StagingPanel for all three sections** - `1017488` (feat)

## Files Created/Modified
- `src/components/DirectoryRow.svelte` - Added oncontextmenu prop and handler on outer div
- `src/components/TreeFileList.svelte` - Added ondirectorycontextmenu prop, threaded to DirectoryRow
- `src/components/StagingPanel.svelte` - Added handleDiscardDirectory, resolveDirectory, unresolveDirectory, showUnstagedDirContextMenu, showStagedDirContextMenu, showConflictedDirContextMenu; wired ondirectorycontextmenu to all 4 TreeFileList instances

## Decisions Made
- Directory context menus use native Tauri menus (dynamic import pattern) matching existing file context menu approach
- Menu item text includes file count for clarity (e.g. "Stage All (5)")
- Discard directory uses Promise.all for parallel discard_file invocations matching stageDirectory pattern

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Directory context menus complete for all three staging panel sections
- All tree view features for Phase 49 are now implemented

## Self-Check: PASSED

- All 3 modified source files exist
- SUMMARY.md created
- Both task commits verified (007405e, 1017488)

---
*Phase: 49-tab-drag-tree-context-menu*
*Completed: 2026-03-25*
