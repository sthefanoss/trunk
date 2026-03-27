---
phase: 54-frontend-unit-tests
plan: 04
subsystem: testing
tags: [vitest, svelte, testing-library, jsdom, component-tests, tauri-mock]

# Dependency graph
requires:
  - phase: 54-01
    provides: "vitest jsdom+svelteTesting environment, shared factories and Tauri mock"
provides:
  - "Unit tests for 6 largest Svelte components: BranchSidebar, MergeEditor, RebaseEditor, StagingPanel, CommitGraph, RepoView"
  - "Patterns for testing components with deep Tauri integration, SortableJS, VirtualList, and OffscreenCanvas"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Class-based MockLazyStore for constructable vi.mock (vi.fn().mockImplementation is not constructable)"
    - "Sortable.create static method mock for SortableJS"
    - "OffscreenCanvas stub for jsdom (text-measure.ts uses canvas for text width)"
    - "Element.scrollTo stub for jsdom (VirtualList scrollTo)"
    - "Local vi.mock declarations in test files for proper hoisting (side-effect import hoisting unreliable)"

key-files:
  created:
    - src/components/BranchSidebar.test.ts
    - src/components/MergeEditor.test.ts
    - src/components/RebaseEditor.test.ts
    - src/components/StagingPanel.test.ts
    - src/components/CommitGraph.test.ts
    - src/components/RepoView.test.ts
  modified: []

key-decisions:
  - "Declared all vi.mock calls locally in each test file instead of relying on side-effect import from tauri-mock.ts — vi.mock hoisting from imported modules is unreliable in vitest"
  - "Used class-based MockLazyStore instead of vi.fn().mockImplementation() — the latter is not constructable with new"
  - "Added OffscreenCanvas and Element.scrollTo stubs directly in test files needing them — avoids modifying shared vitest-setup.ts during parallel execution"
  - "Focused on smoke tests and primary rendering verification — exhaustive interaction testing for 1800-line components is impractical in jsdom"

patterns-established:
  - "Complex component test pattern: local vi.mock for all Tauri modules + class-based LazyStore + OffscreenCanvas/scrollTo stubs"
  - "Mock Sortable.create static factory method for SortableJS (not just constructor)"

requirements-completed: [UNIT-03]

# Metrics
duration: 14min
completed: 2026-03-26
---

# Phase 54 Plan 04: Very Complex Component Tests Summary

**35 tests across 6 largest Svelte components (764-1826 lines each) with local Tauri mocks, class-based LazyStore, and jsdom polyfills for OffscreenCanvas/scrollTo**

## Performance

- **Duration:** 14 min
- **Started:** 2026-03-26T21:22:56Z
- **Completed:** 2026-03-26T21:37:26Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Created tests for all 6 very complex components: BranchSidebar (5 tests), MergeEditor (6 tests), RebaseEditor (9 tests), StagingPanel (6 tests), CommitGraph (5 tests), RepoView (4 tests)
- Each component has a "renders without crashing" smoke test plus rendering verification and primary interaction tests
- Established reliable mock patterns for components with deep Tauri integration, SortableJS drag-and-drop, VirtualList scroll virtualization, and Canvas text measurement
- All 35 tests pass via `bun run test`

## Task Commits

Each task was committed atomically:

1. **Task 1: BranchSidebar, MergeEditor, and RebaseEditor tests** - `7e39901` (feat)
2. **Task 2: StagingPanel, CommitGraph, and RepoView tests** - `5b1a633` (feat)
3. **Type fix: Sortable mock cast** - `3b54003` (fix)

## Files Created/Modified
- `src/components/BranchSidebar.test.ts` - 5 tests: render, section header, branch names, multiple branches, invoke call
- `src/components/MergeEditor.test.ts` - 6 tests: render, loading state, panel headers, save button, close button, onclose callback
- `src/components/RebaseEditor.test.ts` - 9 tests: render, title, branch pills, summaries, OIDs, buttons, onclose, action dropdowns, column headers
- `src/components/StagingPanel.test.ts` - 6 tests: render, file count header, unstaged/staged section headers, branch name, invoke call
- `src/components/CommitGraph.test.ts` - 5 tests: render, column headers, data loading verification, listbox role, stash loading
- `src/components/RepoView.test.ts` - 4 tests: render, BranchSidebar presence, get_dirty_counts call, collapsed left pane

## Decisions Made
- Declared all vi.mock calls locally in each test file: the side-effect import pattern from `tauri-mock.ts` does not reliably hoist vi.mock to the test file's scope, causing the real LazyStore constructor to execute
- Used class-based MockLazyStore: `vi.fn().mockImplementation(() => ({...}))` is not constructable with `new` in vitest (same issue discovered in plan 01)
- Added OffscreenCanvas and Element.scrollTo stubs per-file rather than modifying shared `vitest-setup.ts` to avoid conflicts with parallel agent execution
- Mocked Sortable with `.create()` static method: RebaseEditor uses `Sortable.create(el, opts)` not `new Sortable(el, opts)`
- RepoView tests mock child component dependencies (SortableJS for RebaseEditor) even though RebaseEditor is conditionally rendered — ensures no errors during prop-change reactivity
- CommitGraph "renders commit summaries" test verifies the invoke call rather than DOM text due to VirtualList not rendering rows in jsdom without scroll events

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] vi.mock hoisting from side-effect import unreliable**
- **Found during:** Task 1 (all 3 components)
- **Issue:** `import "../__tests__/helpers/tauri-mock"` side-effect import's vi.mock calls were not properly hoisted to the test file scope, causing `invoke` to not be a mock function
- **Fix:** Declared all vi.mock calls locally in each test file, removing dependency on tauri-mock.ts for vi.mock hoisting
- **Files modified:** All 6 test files
- **Verification:** All tests pass with local mocks
- **Committed in:** 7e39901, 5b1a633

**2. [Rule 1 - Bug] Class-based MockLazyStore needed for constructability**
- **Found during:** Task 1 (RebaseEditor)
- **Issue:** `vi.fn().mockImplementation(() => ({...}))` is not constructable with `new` — LazyStore in store.ts calls `new LazyStore(...)`
- **Fix:** Used class-based MockLazyStore with get/set/save methods
- **Files modified:** RebaseEditor.test.ts, StagingPanel.test.ts, CommitGraph.test.ts, RepoView.test.ts
- **Verification:** All components importing from store.ts now render correctly
- **Committed in:** 7e39901, 5b1a633

**3. [Rule 3 - Blocking] OffscreenCanvas not available in jsdom**
- **Found during:** Task 1 (RebaseEditor)
- **Issue:** text-measure.ts creates OffscreenCanvas at module level; jsdom does not implement it
- **Fix:** Added OffscreenCanvas stub globally in test files that need it (RebaseEditor, CommitGraph, RepoView)
- **Files modified:** RebaseEditor.test.ts, CommitGraph.test.ts, RepoView.test.ts
- **Verification:** measureTextWidth returns stub value (50px), no runtime errors
- **Committed in:** 7e39901, 5b1a633

**4. [Rule 3 - Blocking] Element.scrollTo not available in jsdom**
- **Found during:** Task 2 (CommitGraph)
- **Issue:** VirtualList calls `viewport.scrollTo()` which jsdom does not implement, causing unhandled rejection errors
- **Fix:** Added `Element.prototype.scrollTo = function() {}` stub in CommitGraph and RepoView test files
- **Files modified:** CommitGraph.test.ts, RepoView.test.ts
- **Verification:** No unhandled rejection errors, all tests pass cleanly
- **Committed in:** 5b1a633

**5. [Rule 1 - Bug] Sortable.create static method not mocked**
- **Found during:** Task 1 (RebaseEditor)
- **Issue:** Mock only covered `new Sortable()` constructor but RebaseEditor uses `Sortable.create()` static factory
- **Fix:** Added `.create` static method to the mock Sortable function
- **Files modified:** RebaseEditor.test.ts, RepoView.test.ts
- **Verification:** RebaseEditor renders without "Sortable.create is not a function" error
- **Committed in:** 7e39901, 5b1a633

**6. [Rule 1 - Bug] BranchSection renders "Local (1)" not "Local"**
- **Found during:** Task 1 (BranchSidebar)
- **Issue:** Plan assumed section header text was "Local" but BranchSection template renders "{label} ({count})"
- **Fix:** Changed assertion to match "Local (1)" with count included
- **Files modified:** BranchSidebar.test.ts
- **Verification:** Test passes with correct text assertion
- **Committed in:** 7e39901

---

**Total deviations:** 6 auto-fixed (2 bugs, 4 blocking)
**Impact on plan:** All auto-fixes necessary for test execution in jsdom environment. No scope creep — the component tests cover exactly what was planned.

## Issues Encountered
- svelte-check reports `toBeInTheDocument` type errors across ALL component test files (not just plan 04) — this is a pre-existing project-wide issue where the jest-dom type augmentation from vitest-setup.ts is not picked up by the TypeScript checker. Tests work correctly at runtime.

## User Setup Required
None - no external service configuration required.

## Known Stubs
None - all tests exercise real rendering paths with mocked external dependencies.

## Next Phase Readiness
- All 6 very complex components now have test files with smoke test coverage
- Combined with plans 02 and 03, all 26 Svelte components in src/components/ should have test files
- Test patterns established for future expansion: class-based LazyStore mock, Sortable.create mock, OffscreenCanvas/scrollTo stubs

## Self-Check: PASSED

- All 6 test files exist
- All 3 commits verified (7e39901, 5b1a633, 3b54003)

---
*Phase: 54-frontend-unit-tests*
*Completed: 2026-03-26*
