---
phase: 44-backend-state-scoping
plan: 01
subsystem: backend
tags: [state-management, per-repo, multi-tab, force-close]
dependency_graph:
  requires: []
  provides: [per-repo-running-op, force-close-repo-command]
  affects: [remote-operations, tab-lifecycle]
tech_stack:
  added: []
  patterns: [per-repo-hashmap-mutex, sigterm-cancel-on-force-close]
key_files:
  created: []
  modified:
    - src-tauri/src/state.rs
    - src-tauri/src/commands/remote.rs
    - src-tauri/src/commands/repo.rs
    - src-tauri/src/lib.rs
decisions:
  - RunningOp uses HashMap<String, u32> keyed by repo path instead of global Option<u32>
  - cancel_remote_op takes path parameter to target specific repo
  - force_close_repo cancels running op via SIGTERM before cleaning state
  - close_repo intentionally does NOT touch RunningOp (D-02 graceful behavior)
metrics:
  duration: ~8min
  completed: 2026-03-23
  tasks: 2/2
  tests_added: 7
  tests_total: 149
---

# Phase 44 Plan 01: Per-repo RunningOp and force_close_repo Summary

Per-repo RunningOp HashMap replacing global Option<u32>, enabling independent remote operations per repository with force_close_repo for tab lifecycle management.

## What Was Done

### Task 1: Convert RunningOp to per-repo HashMap

- **state.rs**: Changed `RunningOp(Mutex<Option<u32>>)` to `RunningOp(Mutex<HashMap<String, u32>>)` with updated doc comments
- **remote.rs run_git_remote**: Updated signature to `&Mutex<HashMap<String, u32>>`, mutual exclusion uses `contains_key(repo_path)`, PID stored via `insert(repo_path.to_owned(), pid)`, cleared via `remove(repo_path)`
- **remote.rs cancel_remote_op**: Added `path: String` parameter, uses `guard.remove(&path)` instead of `guard.take()`
- **remote.rs callers** (git_fetch, git_pull, git_push): No changes needed -- `&running.0` type propagates automatically
- **D-04 confirmed**: remote-progress event already includes repo path in payload
- **4 new unit tests**: running_op_allows_different_repos, running_op_blocks_same_repo, running_op_remove_one_keeps_other, cancel_removes_only_target_repo

### Task 2: Add force_close_repo command

- **repo.rs**: Added `force_close_repo` command that cancels running op via SIGTERM then cleans all state (RepoState, CommitCache, WatcherState)
- **repo.rs**: `close_repo` intentionally does NOT touch RunningOp (D-02: graceful close leaves ops running)
- **lib.rs**: Registered `force_close_repo` in Tauri invoke_handler
- **3 new unit tests**: force_close_removes_running_op, force_close_no_running_op_still_succeeds, close_does_not_touch_running_op

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| HashMap<String, u32> keyed by repo path | Natural extension of existing RepoState/CommitCache pattern; enables per-repo remote op isolation |
| cancel_remote_op takes explicit path parameter | Required for multi-tab -- frontend must specify which repo's operation to cancel |
| force_close_repo SIGTERM then cleanup | D-03: force close must kill subprocess before releasing state to avoid orphaned git processes |
| close_repo does not touch RunningOp | D-02: graceful close should not kill a running fetch/push, only remove UI state |

## Verification Results

1. `cargo test --lib -p trunk` -- 149 tests pass (142 existing + 7 new)
2. `cargo check -p trunk` -- Clean compilation, no warnings
3. `cargo test -p trunk` -- Full test suite passes
4. No `Mutex<Option<u32>>` remains in state.rs (old global pattern removed)
5. `force_close_repo` registered in lib.rs invoke_handler
6. D-04 confirmed: `"path": repo_path` in remote-progress event payload

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 (RED) | a9d28bf | test(44-01): add failing tests for per-repo RunningOp |
| 1 (GREEN) | 7da4541 | feat(44-01): convert RunningOp to per-repo HashMap |
| 2 (RED) | 4688bb8 | test(44-01): add tests for force_close_repo and close_repo isolation |
| 2 (GREEN) | 6a18940 | feat(44-01): add force_close_repo command with SIGTERM cancel |

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

None -- all functionality is fully wired.

## Self-Check: PASSED

All files exist, all 4 commits verified.
