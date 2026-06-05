# Snapshot of a clean working tree adds a changeless commit to the review session

> **RESOLUTION — OBSOLETE (2026-06-05).** The flow this describes no longer
> exists. `add_working_tree_review` was removed in the 260531-l02 refactor; the
> only snapshot entry point is now `ensure_review_snapshot`
> (`commands/review.rs`), invoked solely at **comment-submit** time from
> `resolveCommentCommitOid` (`DiffPanel.svelte`). You can only reach it by
> commenting on a real diff hunk, so the tree is never clean on that path — the
> "changeless entry in the commit list" symptom can't occur as described. The
> only residual is a narrow TOCTOU (revert the change while the composer is open,
> then submit), which is a degenerate action and not worth speculative defensive
> code (YAGNI). That submit path already toasts any error
> (`DiffPanel.svelte:260`), so if a `nothing_to_review` guard is ever added it
> would surface automatically. Closing as obsolete; reopen if the TOCTOU guard is
> wanted as defense-in-depth.

**Filed:** 2026-05-31 (quick task 260531-4kk)
**Severity:** low (UX wart, not a crash) — now OBSOLETE, see resolution above

## Problem

`snapshot_working_tree` (src-tauri/src/git/workdir_snapshot.rs) has no
empty-diff guard. When the working tree is clean, the snapshot tree equals
HEAD's tree, so `add_working_tree_review` creates a dangling commit whose diff
against its parent (HEAD) is empty, and adds it to the session. It then renders
in the review commit list as an "Uncommitted changes — …" entry with no diff —
confusing, since there is nothing to review.

Not a crash: the pipeline handles it gracefully (the commit lists via the
`intersect_graph_order` fallback; its diff is just empty). Edge case — reviewing
"uncommitted changes" with a clean tree is unusual.

## Proposed fix

In `snapshot_working_tree` (or in `add_working_tree_review`), compare the
snapshot tree OID to HEAD's tree OID. If equal (clean tree), return a distinct
signal — e.g. a `nothing_to_review` TrunkError code — and have
`add_working_tree_review` surface it so the frontend shows a "nothing to review"
toast instead of adding a changeless commit. Touches: workdir_snapshot.rs,
commands/review.rs (handle the new arm), review-session.svelte.ts /
ReviewPanel.svelte (surface the message). ~3 files, small.

## Also worth a test

No frontend test currently locks the snapshot-button gate
(`sessionState === "active"`, fixed in 8dc2e7b). A ReviewPanel component test
asserting the button is hidden in `resume-available` and shown in `active`
would prevent regression. Medium — needs the panel test fixture
(installReads/dispatcher pattern from Phase 73 tests).
