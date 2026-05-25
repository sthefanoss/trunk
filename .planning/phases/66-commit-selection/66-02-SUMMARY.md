---
phase: 66-commit-selection
plan: 02
subsystem: api
tags: [tauri, git2, rust, review-session, concurrency, mutex]

# Dependency graph
requires:
  - phase: 65-data-model-persistence
    provides: review_store atomic save/load, canonical_repo_path, ReviewSessionsState mutex, session-changed emit, _inner+thin command shape
  - phase: 66-commit-selection (plan 01)
    provides: validate_range, compute_range_oids, apply_add, apply_remove, union_dedup, intersect_graph_order, SessionCommit
provides:
  - "seed_review_range / add_review_commit / remove_review_commit / list_session_commits #[tauri::command]s"
  - "mutex-serialized read-modify-write of the persisted review session (no lost writes on rapid clicks)"
  - "no_session vs not_open error distinction the frontend can branch on"
  - "invoke_handler registration for the four selection commands"
affects: [66-03, 66-04, 67-anchor-capture, 69-review-panel]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "RMW serialization: hold ReviewSessionsState mutex across read→mutate→save_session→map-write (diverges from Phase 65 create/delete which read no prior state)"
    - "Dual path-keying in one command: canonical key for review_store/sessions map, raw path for CommitCache"
    - "git2 work in spawn_blocking on a cloned RepoState snapshot; locks live only in the outer async fn, never held across .await"
    - "Generic mutate_session_rmw(closure) free function over &Mutex<..> so concurrency is unit-testable without a Tauri runtime"

key-files:
  created: []
  modified:
    - src-tauri/src/commands/review.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "Extracted a generic mutate_session_rmw free function taking the raw &Mutex<..> so selection_rmw_serialized can spawn real threads against an Arc<Mutex<..>>; the thin commands are wrappers passing &sessions.0"
  - "save_session runs while the sessions mutex guard is held (get_mut mutates in place), so disk and memory can never diverge and there is no clone-and-reinsert race window"
  - "list_session_commits drops the AppHandle param entirely — it never emits (no mutation), so taking it would be a dead Tauri injection"
  - "no_session (missing in-memory session) is distinct from not_open (repo closed) so Plan 04 can branch session-active vs error"

patterns-established:
  - "RMW serialization under ReviewSessionsState mutex for every mutating selection command"
  - "Range seed = one walk / one save / one emit per gesture (never N adds)"

requirements-completed: [SEL-01, SEL-02, SEL-03, SEL-04]

# Metrics
duration: ~28min
completed: 2026-05-25
---

# Phase 66 Plan 02: Selection Commands with Mutex-Serialized RMW Summary

**Four Tauri commands (seed/add/remove/list) wrapping the Plan 01 set helpers, serializing every read-modify-write of the persisted review session under the ReviewSessionsState mutex so rapid concurrent clicks never lose a write.**

## Performance

- **Duration:** ~28 min
- **Started:** 2026-05-25T12:33Z (approx)
- **Completed:** 2026-05-25T13:01Z
- **Tasks:** 2 (Task 1 was TDD: RED → GREEN)
- **Files modified:** 2

## Accomplishments
- `mutate_session_rmw` holds the `ReviewSessionsState` mutex across read→mutate→`save_session`→map-write, mitigating the lost-write race (Pitfall 2 / threat T-66-02), proven by `selection_rmw_serialized` (50 concurrent adds + a remove → no lost write in memory or on disk).
- `seed_review_range` validates `[base..tip]` and walks the range in `spawn_blocking` (git2 off the lock), then unions the result in one save / one emit per gesture (D-03).
- `add_review_commit` / `remove_review_commit` perform idempotent set mutations under the same serialized RMW path.
- `list_session_commits` reads the session by canonical key and the graph by raw path (Pitfall 3), returns graph-ordered `SessionCommit`s via `intersect_graph_order`, and distinguishes `no_session` from `not_open`. No mutation, no emit.
- All four commands registered in `generate_handler!`; merges are selectable everywhere (no `is_merge` gate, D-08).

## Task Commits

1. **Task 1 (TDD RED): failing selection_rmw_serialized test** - `c0d7633` (test)
2. **Task 1 (TDD GREEN): selection commands with mutex-serialized RMW** - `f47f405` (feat)
3. **Task 2: register the four selection commands in invoke_handler** - `c80ada4` (feat)

_Task 1 followed TDD: RED commit (test compiles-fails because the RMW helpers don't exist) → GREEN commit (helpers + four commands)._

## Files Created/Modified
- `src-tauri/src/commands/review.rs` - Added `mutate_session_rmw` + `seed_review_range_rmw` / `add_review_commit_rmw` / `remove_review_commit_rmw` free functions, the four `#[tauri::command]` wrappers, and the `selection_rmw_serialized` + `rmw_missing_session_is_no_session_error` tests.
- `src-tauri/src/lib.rs` - Registered the four commands in the `generate_handler!` block after the existing `commands::review::*` lifecycle commands.

## Decisions Made
- **Generic `mutate_session_rmw(closure)` over `&Mutex<..>`**: the advisor flagged that `#[tauri::command]` signatures (`State`, `AppHandle`) aren't constructable in a unit test, so the RMW core was extracted as a free function taking the raw mutex. The test drives it with a real `Arc<Mutex<..>>` and N threads; the commands are thin wrappers passing `&sessions.0`. This proves production lock discipline rather than duplicating it in the test.
- **`save_session` inside the guard via `get_mut`**: rather than clone-mutate-reinsert (which has a race window between read and write), the helper holds the guard, mutates `session.commits` in place, and saves — the strongest serialization.
- **Dropped `AppHandle` from `list_session_commits`**: it never emits, so the injected handle would be dead. (Caught the unused-variable warning during GREEN and removed the param rather than `_`-prefixing it.)

## Deviations from Plan

None - plan executed exactly as written. The only adjustment was a design refinement explicitly invited by the plan ("Planner: decide whether to keep a testable `_inner`…") and the advisor: the RMW core is a generic free function rather than three near-duplicate helpers, which keeps lock discipline in one place and makes `selection_rmw_serialized` exercise the real production path.

## Issues Encountered
- **`just check` vitest stage fails (environmental, out of scope).** All 44 frontend test files fail with `Cannot find module '@testing-library/svelte/src/vitest.js'` because this git worktree's `node_modules` is empty/incomplete (it is not symlinked to the main repo's installed deps). This plan touches **zero** frontend files (Rust-only: `review.rs` + `lib.rs`), so the failure is unrelated to the changes and cannot be fixed by editing plan files. The Rust gates of `just check` all pass: `cargo fmt --check` (clean), `cargo clippy -- -D warnings` (clean on the lib target — the recipe does not lint tests/benches), and `cargo test` (36 lib tests + integration pass). Frontend tests should be re-run after the worktree is merged or `node_modules` is installed in-tree.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- SEL-01..04 are reachable over IPC: seed/add/remove mutate and persist; list returns graph-ordered `SessionCommit`s. Plans 03/04 (frontend graph markers, context-menu toggle, panel list) can now `invoke` these commands and listen for `session-changed`.
- Frontend can branch on `no_session` vs `not_open` for the D-06 toggle gating (Pitfall 4) and the panel empty state.
- Concurrency is proven safe (T-66-02 mitigated and tested), so rapid graph clicks are correct.

---
*Phase: 66-commit-selection*
*Completed: 2026-05-25*
