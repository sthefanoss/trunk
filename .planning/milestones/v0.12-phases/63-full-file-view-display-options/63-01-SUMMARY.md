---
phase: 63-full-file-view-display-options
plan: 01
subsystem: ui
tags: [svelte, lazystore, css-custom-properties, lucide, diff-toolbar, whitespace, word-wrap]

# Dependency graph
requires:
  - phase: 62-ui-refactor-component-structure
    provides: DiffToolbar, DiffViewer, HunkView component decomposition with Props interfaces
provides:
  - getDiffShowInvisibles/setDiffShowInvisibles LazyStore preference pair
  - getDiffWordWrap/setDiffWordWrap LazyStore preference pair
  - --color-invisible and --color-trailing-ws-bg CSS custom properties
  - Three toolbar toggle buttons (Space, Pilcrow, TextWrap) in DiffToolbar
  - Staging disabled with tooltip when ignoreWhitespace active (WHSP-02)
  - Word wrap CSS toggle on diff lines (DISP-02)
  - ondiffoptionschange callback for diff re-fetch on backend-affecting option changes
affects: [63-02-PLAN, full-file-view, split-view, show-invisibles-rendering]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Toggle button pattern: .toggle-btn with class:active binding and Lucide icon"
    - "stagingDisabled $derived combining multiple disable conditions"
    - "ondiffoptionschange callback for LazyStore-first-then-refetch pattern"

key-files:
  created: []
  modified:
    - src/lib/store.ts
    - src/app.css
    - src/components/diff/DiffToolbar.svelte
    - src/components/DiffPanel.svelte
    - src/components/diff/DiffViewer.svelte
    - src/components/diff/HunkView.svelte
    - src/components/RepoView.svelte
    - src/components/DiffPanel.test.ts

key-decisions:
  - "LazyStore-first-then-callback pattern: DiffPanel persists new value to LazyStore before calling ondiffoptionschange, so RepoView's buildDiffOptions reads updated values"
  - "FullFileView prop pass deferred to Plan 02 since FullFileView is still a stub component"

patterns-established:
  - "Toggle button: .toggle-btn with class:active={prop} and Lucide icon, separated by .toolbar-divider"
  - "Staging guard: stagingDisabled $derived combining hunkOperationInFlight with ignoreWhitespace"
  - "Diff options re-fetch: ondiffoptionschange callback propagated from DiffPanel to RepoView"

requirements-completed: [WHSP-02, DISP-02]

# Metrics
duration: 7min
completed: 2026-03-29
---

# Phase 63 Plan 01: Display Option Infrastructure Summary

**LazyStore preferences, CSS vars, three toolbar toggles (WS/invisibles/wrap), staging guard when whitespace ignored, word wrap toggle, and diff re-fetch callback wired end-to-end**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-29T22:51:12Z
- **Completed:** 2026-03-29T22:58:27Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Added 4 new LazyStore exports (get/set for showInvisibles and wordWrap preferences)
- Added 2 CSS custom properties for invisible character markers and trailing whitespace warning
- Added 3 toggle buttons with Lucide icons (Space, Pilcrow, TextWrap) to DiffToolbar with active/inactive styling
- All 8 staging buttons (hunk + line level) disabled with tooltip when ignoreWhitespace is active
- Word wrap toggle switches diff lines between pre and pre-wrap with flex-start alignment
- View mode change to/from "full" syncs showFullFile preference and triggers diff re-fetch
- ondiffoptionschange callback wired from DiffPanel through to RepoView for re-fetching

## Task Commits

Each task was committed atomically:

1. **Task 1: Store preferences, CSS vars, DiffToolbar toggles** - `14e3a2f` (feat)
2. **Task 2: DiffPanel state wiring, DiffViewer/HunkView prop threading, RepoView re-fetch** - `2f9a49b` (feat)

## Files Created/Modified
- `src/lib/store.ts` - Added getDiffShowInvisibles/setDiffShowInvisibles and getDiffWordWrap/setDiffWordWrap preference pairs
- `src/app.css` - Added --color-invisible and --color-trailing-ws-bg CSS custom properties
- `src/components/diff/DiffToolbar.svelte` - Three toggle buttons with Lucide icons, staging guard on file-level buttons
- `src/components/DiffPanel.svelte` - State owner for ignoreWhitespace/showInvisibles/wordWrap, handler functions, ondiffoptionschange prop
- `src/components/diff/DiffViewer.svelte` - Prop threading for ignoreWhitespace/showInvisibles/wordWrap to HunkView
- `src/components/diff/HunkView.svelte` - stagingDisabled derived, title tooltip, word wrap CSS toggle, align-items: flex-start
- `src/components/RepoView.svelte` - ondiffoptionschange callback triggering refetchFileDiff on staging DiffPanel
- `src/components/DiffPanel.test.ts` - Updated store mock with new function stubs

## Decisions Made
- LazyStore-first-then-callback pattern: DiffPanel persists new value to LazyStore before calling ondiffoptionschange, so RepoView's buildDiffOptions reads the updated values automatically
- FullFileView prop pass deferred to Plan 02 since FullFileView is still a stub component -- no point passing props to a component that ignores them

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated DiffPanel.test.ts store mock with new function stubs**
- **Found during:** Task 2 (DiffPanel state wiring)
- **Issue:** DiffPanel.test.ts mock of store.js was missing the 6 new store functions (getDiffIgnoreWhitespace, setDiffIgnoreWhitespace, getDiffShowFullFile, setDiffShowFullFile, getDiffShowInvisibles, setDiffShowInvisibles, getDiffWordWrap, setDiffWordWrap), causing all 23 DiffPanel tests to fail
- **Fix:** Added all 8 missing mock functions to the vi.mock block
- **Files modified:** src/components/DiffPanel.test.ts
- **Verification:** All 385 tests pass (41 test files)
- **Committed in:** 2f9a49b (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Test mock update was necessary for correctness. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Display option infrastructure complete and wired end-to-end
- Plan 02 can implement FullFileView with virtual scrolling, passing showInvisibles/wordWrap props from DiffViewer
- CSS custom properties (--color-invisible, --color-trailing-ws-bg) ready for show-invisibles rendering
- All existing tests pass (385/385)

---
*Phase: 63-full-file-view-display-options*
*Completed: 2026-03-29*
