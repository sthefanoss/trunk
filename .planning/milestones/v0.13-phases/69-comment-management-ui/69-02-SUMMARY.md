---
phase: 69-comment-management-ui
plan: 02
subsystem: review-commands
tags: [tauri-command, review-session, comment-management, rust, rmw, uuid]

# Dependency graph
requires:
  - phase: 69-comment-management-ui
    plan: 01
    provides: "Comment v2 shape — stable id: String (D-03) + commit_oid: Option<String> (D-01); CURRENT_SCHEMA_VERSION = 2; load-path id backfill"
provides:
  - "add_commit_comment command (ANCH-03): commit-level note tied to commit_oid, anchor None, cached_excerpt None"
  - "edit_comment command (CMT-02): update text by stable id; missing id -> not_found"
  - "delete_comment command (CMT-03): remove by id; missing id -> idempotent no-op"
  - "all three route through mutate_session_rmw and emit session-changed; testable _inner cores"
affects: [69-04 panel rendering, 69-05 per-commit add-note affordance]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Found-flag captured inside the infallible mutate_session_rmw closure, error surfaced after — single critical section, no TOCTOU, helper signature untouched"
    - "Idempotent retain-by-id delete (parity with apply_remove); not_found edit-by-id"

key-files:
  created: []
  modified:
    - src-tauri/src/commands/review.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "edit_comment_inner surfaces not_found without changing mutate_session_rmw's infallible-closure signature: a `found` flag is set inside the closure and the not_found TrunkError is returned after the RMW. Single lock acquisition, no TOCTOU; one harmless identical-bytes re-save on a miss. Avoids a Rule-4-shaped change to all 4 existing RMW callers."
  - "AddCommitCommentRequest is a sibling DTO { commit_oid, text } with NO path field (path lives only on the thin wrapper for canonical resolution); add_comment / AddCommentRequest are untouched (RESEARCH Open Question 2 — sibling, not extension)."
  - "add_commit_comment_inner does NOT clear draft_comment — a commit-level note is independent of the line-anchored diff composer."
  - "No backend empty-text validation — the frontend disables empty submit; the backend stores what it is given (dumb writer, threat T-69-07 accepted: local single-user app, text is data not a sink)."

patterns-established:
  - "Found-flag-after-RMW to surface a domain not_found from inside an infallible mutate closure"
  - "Three sibling mutating comment commands mirroring add_comment's resolve -> _inner -> emit shape"

requirements-completed: [ANCH-03, CMT-02, CMT-03]

# Metrics
duration: ~12min
completed: 2026-05-26
---

# Phase 69 Plan 02: Comment Management Commands (commit-level note, edit, delete) Summary

**Added the write half of comment management — three sibling mutating commands (`add_commit_comment` for ANCH-03, `edit_comment` for CMT-02, `delete_comment` for CMT-03), each a testable `_inner` core plus a thin `#[tauri::command]` wrapper that routes through `mutate_session_rmw` and emits `session-changed`. Edit/delete target by stable uuid `id` (D-03), making them multi-tab-safe; the existing line-anchored `add_comment` wire shape is untouched.**

## Performance

- **Duration:** ~12 min
- **Tasks:** 2 (both TDD)
- **Files modified:** 2

## Accomplishments

- `add_commit_comment_inner` pushes a commit-level `Comment` (`commit_oid: Some`, `anchor: None`, `cached_excerpt: None`, fresh uuid id), distinguishable from line-anchored comments so Plan 04/05 render/jump can branch (D-01). It deliberately leaves `draft_comment` untouched.
- `edit_comment_inner` updates one comment's text by id, leaving all others unchanged; a missing id returns a `not_found` `TrunkError` and mutates nothing (T-69-05).
- `delete_comment_inner` removes by id via `retain`; a missing id is an idempotent no-op returning `Ok` (parity with `apply_remove`).
- All three thin wrappers resolve the canonical path, call their `_inner`, and emit `session-changed` after a successful mutation; none emits on read (mirrors `save_draft_comment`'s no-emit precedent for non-panel-visible writes).
- All three registered in `lib.rs` invoke_handler alongside the existing `commands::review::*` entries.

## Task Commits

Each task followed the TDD RED -> GREEN gate:

1. **Task 1 (RED): failing tests for add_commit_comment + edit_comment cores** - `fc609cb` (test)
2. **Task 1 (GREEN): implement add_commit_comment_inner + edit_comment_inner + AddCommitCommentRequest** - `96a544f` (feat)
3. **Task 2 (RED): failing tests for delete_comment core** - `e5bc3c8` (test)
4. **Task 2 (GREEN): delete_comment_inner + three thin wrappers + lib.rs registration** - `b170763` (feat)
5. **rustfmt on test assertions** - `7801e12` (style)

No REFACTOR commits — both implementations were minimal and clean as written. The `style` commit is rustfmt fallout on the Task-1 test assertions, caught by `just check` (see Deviations).

## Files Created/Modified

- `src-tauri/src/commands/review.rs` - Added `AddCommitCommentRequest { commit_oid, text }` DTO; `add_commit_comment_inner`, `edit_comment_inner`, `delete_comment_inner` cores; the three thin `#[tauri::command]` wrappers (`add_commit_comment`, `edit_comment`, `delete_comment`); 7 new unit tests (commit-level persist + distinguishability + draft-untouched; edit-by-id + missing-id not_found; delete-by-id + missing-id no-op).
- `src-tauri/src/lib.rs` - Registered the three new commands in `invoke_handler`.

## Decisions Made

- **not_found from an infallible closure:** `mutate_session_rmw`'s closure is `FnOnce(&mut ReviewSession)` and cannot return an error. Rather than change that helper's signature (which would touch all 4 existing callers — a Rule-4-shaped change), `edit_comment_inner` captures a `found` flag inside the single critical section and returns the `not_found` `TrunkError` after the RMW returns. One lock acquisition (no TOCTOU vs. a pre-pass read); the only cost is one harmless identical-bytes re-save when the id is missing.
- **Sibling DTO, not an extension:** `AddCommitCommentRequest` carries only `{ commit_oid, text }` — no `path` field. `path` exists solely on the thin wrapper for canonical resolution. `add_comment` / `AddCommentRequest` and the 7 existing line-anchored tests are byte-for-byte unchanged (RESEARCH Open Question 2).
- **No draft clear on commit-level note:** a per-commit note is independent of the diff composer, so `add_commit_comment_inner` does not touch `draft_comment` (unlike `add_comment_inner`).
- **No backend empty-text guard:** the frontend disables empty submit; the backend is a dumb writer and stores what it is given (threat T-69-07 accepted — local single-user desktop, text is data not an injection sink).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] rustfmt reformatting on new test assertions**
- **Found during:** Final `just check` (the project gate; the per-task `cargo test review` does not run rustfmt)
- **Issue:** Several multi-line `assert!(condition, "message")` calls in the Task-1 tests were written single-line; `cargo fmt --check` (run by `just check`'s `fmt` recipe) failed.
- **Fix:** Ran `cargo fmt`, which reformatted only the new test assertions in `review.rs`. No production code or logic changed.
- **Files modified:** src-tauri/src/commands/review.rs (test code only)
- **Verification:** `just check` exits 0 (fmt, biome, svelte-check, clippy, cargo-test, vitest 507 passing).
- **Committed in:** `7801e12` (style)

---

**Total deviations:** 1 auto-fixed (Rule 3 - Blocking, formatting only). No scope creep, no production-logic change.
**Impact on plan:** None — the fix was a mechanical rustfmt pass on test code surfaced by the project gate that the plan's per-task verify command does not exercise.

## Issues Encountered

- The plan's per-task `<verify>` command is `cargo test --manifest-path src-tauri/Cargo.toml review`, which does not run rustfmt/clippy/biome. The project gate `just check` does, and it caught the formatting drift. Lesson (consistent with Plan 01): always run `just check` before finalizing, not just the per-task `cargo test`.

## Threat Model Compliance

All `mitigate` dispositions from the plan's STRIDE register are satisfied:

- **T-69-05 (Tampering — stale-index targeting):** edit/delete target by stable uuid `id`, never by list position; missing id -> not_found (edit) / no-op (delete). Tested.
- **T-69-06 (Spoofing — id collision):** ids are `uuid::Uuid::new_v4()`, collision-free, never content-derived.
- **T-69-08 (Tampering — concurrent multi-tab writes):** all three commands route through `mutate_session_rmw` (mutex held across read->mutate->save), the 50-thread-tested primitive.
- **T-69-07 (DoS — unbounded text):** accepted by design (local single-user app, text is data).

No new threat surface beyond the plan's register — three IPC commands on the already-established frontend->IPC trust boundary, all routing through the proven RMW primitive.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The write half of comment management is in place. Plan 04 (panel rendering) can branch on `anchor.is_none() && commit_oid.is_some()` to render commit-level vs. line-anchored comments; Plan 05 can wire the per-commit "Add note" affordance to `add_commit_comment`, and edit/delete UI to `edit_comment` / `delete_comment`.
- No blockers.

---
*Phase: 69-comment-management-ui*
*Completed: 2026-05-26*

## Self-Check: PASSED

- All modified files present on disk (review.rs, lib.rs, 69-02-SUMMARY.md).
- All five task commits found in git history (fc609cb, 96a544f, e5bc3c8, b170763, 7801e12).
- Symbols verified: `fn add_commit_comment_inner`, `fn edit_comment_inner`, `fn delete_comment_inner` in review.rs; `commands::review::add_commit_comment` registered in lib.rs.
- `just check` exits 0 (fmt, biome, svelte-check, clippy, cargo-test, vitest 507 passing); 41 review lib tests pass.
