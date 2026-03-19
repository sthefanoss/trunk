---
phase: 32-hunk-staging-backend
plan: 01
subsystem: staging
tags: [git2, hunk, apply, diff, tdd, rust, tauri]

# Dependency graph
requires: []
provides:
  - "stage_hunk_inner, unstage_hunk_inner, discard_hunk_inner inner functions"
  - "stage_hunk, unstage_hunk, discard_hunk Tauri async command wrappers"
  - "Hunk-level staging/unstaging/discarding via git2 ApplyOptions::hunk_callback"
affects: [33-hunk-staging-ui]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ApplyOptions::hunk_callback for selective hunk application"
    - "DiffOptions::reverse(true) for unstage/discard reversed diffs"
    - "Patch::from_diff for hunk count validation before apply"

key-files:
  created: []
  modified:
    - "src-tauri/src/commands/staging.rs"
    - "src-tauri/src/lib.rs"

key-decisions:
  - "Used ApplyOptions::hunk_callback instead of Patch::from_diff + to_buf roundtrip -- simpler, fewer allocations"
  - "Kept hunk commands in staging.rs rather than new file -- shares helpers, natural grouping"

patterns-established:
  - "Hunk callback pattern: counter-based filtering with target index comparison"
  - "Three-operation pattern: stage=forward+Index, unstage=reversed+Index, discard=reversed+WorkDir"

requirements-completed: [HUNK-01, HUNK-02, HUNK-03, HUNK-05]

# Metrics
duration: 4min
completed: 2026-03-18
---

# Phase 32 Plan 01: Hunk Staging Backend Summary

**Three hunk-level git operations (stage/unstage/discard) via git2 ApplyOptions::hunk_callback with TDD and full error handling**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-18T00:50:07Z
- **Completed:** 2026-03-18T00:54:09Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- Implemented stage_hunk_inner that stages only the target hunk from a multi-hunk file using hunk_callback filtering
- Implemented unstage_hunk_inner that removes only the target hunk from the index using reversed diff + hunk_callback
- Implemented discard_hunk_inner that reverts only the target hunk in the working directory using reversed diff to workdir
- All three return stale_hunk_index error when hunk_index >= num_hunks and file_not_found when no relevant changes exist
- Added 3 async Tauri command wrappers and registered in lib.rs invoke_handler
- Full TDD flow: 6 failing tests written first, then implementation made them all pass

## Task Commits

Each task was committed atomically:

1. **Task 1: RED -- Write multi-hunk test fixture and failing tests** - `cd1fdb0` (test)
2. **Task 2: GREEN -- Implement stage_hunk_inner, unstage_hunk_inner, discard_hunk_inner** - `510c9be` (feat)
3. **Task 3: Wire async command wrappers and register in lib.rs** - `a88b521` (feat)

## Files Created/Modified
- `src-tauri/src/commands/staging.rs` - Added 3 inner functions, 3 async wrappers, test fixture helper, and 6 test functions
- `src-tauri/src/lib.rs` - Registered stage_hunk, unstage_hunk, discard_hunk in invoke_handler

## Decisions Made
- Used ApplyOptions::hunk_callback instead of Patch::from_diff + to_buf roundtrip -- simpler with fewer allocations and libgit2 handles all edge cases
- Kept all hunk commands in staging.rs rather than creating a new hunk.rs -- they share open_repo_from_state, is_head_unborn helpers and are natural staging operations

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 3 hunk commands are registered and working, ready for Phase 33 UI integration
- Frontend can call invoke('stage_hunk', { path, filePath, hunkIndex }), invoke('unstage_hunk', ...), and invoke('discard_hunk', ...)
- Full test suite passes (103 tests, including 18 staging-specific tests)

---
## Self-Check: PASSED

All files verified present. All 3 task commits verified in git log.

---
*Phase: 32-hunk-staging-backend*
*Completed: 2026-03-18*
