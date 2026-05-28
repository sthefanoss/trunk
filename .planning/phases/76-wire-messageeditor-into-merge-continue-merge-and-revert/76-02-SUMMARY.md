---
phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert
plan: 02
subsystem: api
tags: [tauri, git, git2, subprocess, serde, revert, rust, tdd]

# Dependency graph
requires:
  - phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert
    plan: 01
    provides: "shared lib.rs invoke-handler list + the two-step begin/continue pattern this plan mirrors for revert"
provides:
  - "RevertBeginResult struct ({ graph, message: Option<String> }) — the contract Plan 03's frontend reads message off of"
  - "revert_commit_begin two-step command (git revert --no-commit -> verbatim MERGE_MSG default; emits repo-changed; conflict -> Err(conflict_state))"
  - "revert_continue finish command (git commit -m --cleanup=strip; clears REVERT_HEAD)"
  - "revert_abort command (git revert --abort) — the MSG-06 recovery path for revert (NEW; did not exist before)"
  - "#[cfg(test)] temp-repo suite in commit_actions.rs (Wave-0 gap closed)"
affects: [76-03 frontend merge/revert wiring, 76-04 UAT checkpoint]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "two-step begin/continue mirrored from 76-01 merge backend; begin wrapper emits repo-changed (REVERT_HEAD set before the editor opens)"
    - "plain 2-field struct (RevertBeginResult) instead of a tagged enum — a single editor outcome, conflict is Err not a variant"
    - "temp-repo TDD harness in commit_actions.rs (git2 + tempfile, real git subprocess, no mocks)"

key-files:
  created: []
  modified:
    - src-tauri/src/commands/commit_actions.rs
    - src-tauri/src/lib.rs
    - src-tauri/tests/common/drivers/commit_actions.rs
    - src-tauri/tests/test_commit_actions.rs

key-decisions:
  - "revert_commit_inner renamed to revert_commit_begin_inner (not added alongside) — the plan's 'replaces revert_commit' decision + coding-style no-dead-code; mirrors 76-01's merge_branch -> merge_branch_begin rename"
  - "RevertBeginResult is a plain struct, NOT a serde-tagged enum like MergeBeginResult — there is one editor outcome (clean), and conflict stays Err(conflict_state)"
  - "Integration suite (tests/) migrated to two-step semantics — the rename broke its compile; the filtered cargo test --lib gate cannot catch this, only just check (wave-1 lesson)"

# Backend slice of MSG-03/MSG-06 (this plan's scope). The user-facing
# requirements are NOT fully Complete until the frontend wiring (Plan 03) and
# UAT (Plan 04) land — REQUIREMENTS.md traceability stays In Progress.
requirements-completed: [MSG-03, MSG-06]

# Metrics
duration: 18min
completed: 2026-05-29
---

# Phase 76 Plan 02: Revert-side backend (begin/continue/abort) Summary

**Two-step `revert_commit_begin` (`git revert --no-commit` -> verbatim `.git/MERGE_MSG` default with the full 40-char OID, emits `repo-changed`), a `revert_continue` finish (`git commit -m --cleanup=strip`, clears `REVERT_HEAD`), and a NEW `revert_abort` (`git revert --abort`) that makes MSG-06 satisfiable for revert — all proven against temp git repos, with the `--no-edit` bypass removed.**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-05-29
- **Completed:** 2026-05-29
- **Tasks:** 2 (TDD: RED + GREEN)
- **Files modified:** 4

## Accomplishments
- `RevertBeginResult { graph, message: Option<String> }` — the frozen contract Plan 03's frontend reads the default message off of (a plain struct, not a tagged enum: there is one editor outcome).
- `revert_commit_begin`: `git revert --no-commit <oid>` stages the revert and returns the verbatim `.git/MERGE_MSG` default (`Revert "<subject>"` + `This reverts commit <full-40-char-oid>.`). The `conflict_state` `Err` branch is preserved verbatim — a conflicted revert never opens the editor.
- The `revert_commit_begin` async wrapper emits `repo-changed` on the clean (Ok) path before returning, because `REVERT_HEAD` is set before the editor opens — a later cancel must still surface the in-progress banner (RESEARCH Pitfall 2/4).
- `revert_continue`: `git commit -m <msg> --cleanup=strip` drops git's `# Conflicts:` comment block (MSG-03 fidelity for conflicted reverts) and clears `REVERT_HEAD`.
- `revert_abort`: `git revert --abort` clears `REVERT_HEAD` and restores a clean tree — the MSG-06 recovery path that did not exist before (RESEARCH finding 4: a cancelled revert previously trapped the user).
- First `#[cfg(test)]` temp-repo suite in `commit_actions.rs` (5 tests), closing the Wave-0 gap.
- `--no-edit` removed from the revert path (D-01, OQ-4); `editor.rs` left intentionally unused (D-01a) — not imported, not deleted.

## Task Commits

1. **Task 0: RED — failing tests for revert begin/continue/abort** — `f8291c6` (test)
2. **Task 1: GREEN — implement revert_commit_begin/continue/abort, cleanup=strip, recovery path; register in lib.rs; migrate integration suite** — `c192b3e` (feat)

_TDD plan: the RED commit established the failing compile gate (missing `revert_commit_begin_inner` / `revert_continue_inner` / `revert_abort_inner` / `RevertBeginResult`); the GREEN commit made all 5 unit tests pass. The integration-suite migration is folded into the GREEN commit (ownership.md: the rename broke its compile — caused-by-this-change, fixed before done)._

**Plan metadata:** (this docs commit)

## Files Created/Modified
- `src-tauri/src/commands/commit_actions.rs` — `RevertBeginResult` struct; `revert_commit_inner` -> `revert_commit_begin_inner` (`--no-commit`, returns the struct, keeps `conflict_state`); NEW `revert_continue_inner` + `revert_abort_inner`; `revert_commit` wrapper -> `revert_commit_begin` (struct return, emit) + NEW `revert_continue` / `revert_abort` wrappers; `#[cfg(test)]` temp-repo suite.
- `src-tauri/src/lib.rs` — registered `revert_commit_begin`, `revert_continue`, `revert_abort`; removed `revert_commit`.
- `src-tauri/tests/common/drivers/commit_actions.rs` — driver migrated from `revert_commit` to `revert_commit_begin` / `revert_continue` / `revert_abort`.
- `src-tauri/tests/test_commit_actions.rs` — `revert_commit_removes_changes` replaced by three two-step tests (begin stages removal + carries default message; continue commits + clears `REVERT_HEAD`; abort restores the file).

## Decisions Made
- **Renamed `revert_commit_inner` -> `revert_commit_begin_inner` rather than adding alongside.** The plan's `<interfaces>` block says `revert_commit_begin` *replaces* `revert_commit`, Task 1 step 5 removes the old registration, and 76-01 set the precedent (`merge_branch` -> `merge_branch_begin`). Adding alongside would leave dead code (coding-style: delete the path you control).
- **`RevertBeginResult` is a plain 2-field struct, not a serde-tagged enum.** Revert has a single editor outcome (clean -> `Ready`-equivalent); a conflict is `Err(conflict_state)`, not a third variant. The plan's `<interfaces>` block recommends exactly this; mirroring `MergeBeginResult`'s tagged enum would over-model.
- **Integration suite migrated in the GREEN commit.** The rename breaks `tests/common/drivers/commit_actions.rs` and `tests/test_commit_actions.rs` at compile time. `cargo test --lib commit_actions` (the per-task gate) does NOT compile the `tests/` integration binary, so it would go green while the integration suite is broken — the exact wave-1 lesson. `just check` is the gate that catches it.

## Deviations from Plan

None affecting scope — the plan was executed as written. One within-task test-logic fix (Rule 1) applied to a unit test, not the production contract:

### Auto-fixed Issues

**1. [Rule 1 - Bug] Unit test asserted exact body equality against git's trailing-newline normalization**
- **Found during:** Task 1 (GREEN — `revert_continue_clears_revert_head_and_commits_edited_body`)
- **Issue:** The test asserted `head_body == edited` exactly, but `git commit -m` appends a trailing newline to the stored message (`"...edited body\n"` vs `"...edited body"`). The production behavior was correct (`--cleanup=strip` applied, `REVERT_HEAD` cleared, no `#` lines) — the assertion was too strict.
- **Fix:** Assert `body.trim_end() == edited` (the meaningful behavior: the edited text is the body, modulo git's trailing newline). The `no # lines` assertion is unchanged. This matches how 76-01's merge `--cleanup=strip` test asserts (`!lines().any(starts_with('#'))`), not raw equality.
- **Files modified:** src-tauri/src/commands/commit_actions.rs (test module only)
- **Verification:** All 5 unit tests pass; `just check` green.
- **Committed in:** `c192b3e` (Task 1 GREEN commit)

---

**Total deviations:** 1 auto-fixed (Rule 1, test harness only). The production command signatures match the plan's `<interfaces>` block exactly. The integration-suite migration was required by ownership.md (the rename broke its compile), not a scope change.
**Impact on plan:** No scope creep; no contract drift.

## Wave-ordering Note (expected, not a defect)

`lib.rs` no longer registers `revert_commit`, but the frontend still calls `invoke("revert_commit", …)` at `CommitGraph.svelte:544`. That site is exactly Plan 03's edit target (per 76-RESEARCH); it repoints to `revert_commit_begin`. Between this plan and Plan 03 the frontend revert button is wired to an unregistered command — this is the planned wave seam (PLAN Task 1 step 5 makes the removal an explicit decision), identical to 76-01's `merge_branch` seam, not a regression introduced here.

## Issues Encountered
- The one test-logic bug above (trailing-newline equality). Resolved within Task 1's GREEN cycle; no fix-attempt limit reached.

## Known Stubs
None — this plan ships complete backend behavior. (`editor.rs` remains intentionally unused per D-01a — not imported, not deleted, confirmed by grep: no `editor::prepare`/`EditorHandle` reference in `commit_actions.rs`.)

## Acceptance Gate Results
- `cargo test --lib commit_actions` — 5 passed, 0 failed.
- `grep no-edit commit_actions.rs` — none (D-01, OQ-4).
- `grep -c cleanup=strip commit_actions.rs` — 4 (commit arg + test assertions).
- `grep '"revert", "--abort"' commit_actions.rs` — matches (`revert_abort` exists).
- `grep -c conflict_state commit_actions.rs` — 5 (conflict branch preserved).
- `grep 'editor::prepare\|EditorHandle' commit_actions.rs` — none (D-01a).
- `cargo build` — succeeds (no orphan `revert_commit` reference).
- **`just check` — fully green** (cargo fmt, biome, svelte-check, clippy, all cargo tests incl. the `tests/` integration binary — `test_commit_actions` 13 passed — and 566 vitest tests). Exit 0.

## Next Phase Readiness
- `RevertBeginResult` is the contract for Plan 03's frontend (`result.message` -> pre-fill the MessageEditor; `null`/empty -> no `revert_continue`, recoverable via the begin emit).
- `revert_abort` is ready for the OperationBanner Revert Abort button (Plan 03/04).
- The `repo-changed`-on-begin emit is structural; its visible effect (banner-after-cancel) is verified by manual UAT in 76-04, not a Rust unit test.

## Self-Check: PASSED
- Commits `f8291c6` (RED) and `c192b3e` (GREEN) exist in git history.
- `RevertBeginResult`, `revert_commit_begin_inner`, `revert_continue_inner`, `revert_abort_inner` present in commit_actions.rs.
- `lib.rs` registers `revert_commit_begin`, `revert_continue`, `revert_abort`; `revert_commit` removed.
- SUMMARY file present on disk.

---
*Phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert*
*Completed: 2026-05-29*
