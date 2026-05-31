---
quick_id: 260531-k4j
phase: 260531-k4j-move-uncommitted-changes-comment
plan: 01
subsystem: review / working-tree diff
tags: [review-session, working-tree-snapshot, diff-toolbar, comment-affordance]
requirements: [WT-COMMENT-01]
status: tasks-1-3-complete-task-4-uat-pending
key-files:
  modified:
    - src-tauri/src/git/workdir_snapshot.rs
    - src-tauri/src/commands/review.rs
    - src-tauri/src/lib.rs
    - src/lib/diff-anchor.ts
    - src/components/DiffPanel.svelte
    - src/components/diff/HunkView.svelte
    - src/components/diff/SplitView.svelte
    - src/components/diff/FullFileView.svelte
    - src/components/ReviewPanel.svelte
    - src/lib/review-session.svelte.ts
metrics:
  tasks_completed: 3
  task_4_status: blocking human-verify UAT pending
---

# Quick 260531-k4j: Move uncommitted-changes Comment affordance into the diff toolbar — Summary

Working-tree comments now anchor to a get-or-create snapshot commit (never orphan on
mid-review edits), the in-diff "Comment" button is the single discoverable entry point
for unstaged New-side selections, and the v1 Review-panel button is gone. Tasks 1-3
are code-complete and `just check` green; Task 4 (in-app UAT) is a blocking human gate.

## What shipped (Tasks 1-3)

### Task 1 — Backend get-or-create snapshot (commit `dedb188`)
- Factored `workdir_tree_oid(repo) -> Result<git2::Oid, TrunkError>` out of
  `snapshot_working_tree` (the throwaway-index tree build, steps 1-3; still never
  calls `idx.write()`, so the real `.git/index` stays untouched). `snapshot_working_tree`
  now calls it then commits.
- Added pure `decide_snapshot(repo, prior: Option<Oid>) -> (Oid, bool)` mirroring the
  validate_range/compute_range_oids pattern (takes `&Repository`, no Tauri state).
  Reuse decision is **tree-vs-tree**: `workdir_tree_oid(repo)` compared against
  `repo.find_commit(prior)?.tree_id()`. Unchanged workdir → `(prior, false)`;
  changed or `prior=None` → `(new_oid, true)`.
- `set_working_tree_snapshot_rmw` is now **never-orphan**: `apply_add` the new oid and
  set `working_tree_snapshot` to it, but **never** `apply_remove` the prior. Both
  snapshot oids stay in `session.commits` so earlier comments resolve; the field tracks
  the latest. Docstring rewritten from the v1 "REPLACE" wording.
- Repurposed `add_working_tree_review` → `ensure_working_tree_snapshot(path) -> Result<String, String>`
  returning the snapshot OID. Keeps the `no_session` fast-fail precheck; reads the prior
  snapshot oid under a short lock and **drops the guard before any git2 work** (the
  read-prior-then-decide TOCTOU is benign — worst case one redundant snapshot); calls
  `decide_snapshot` inside `spawn_blocking`; then RMW + emit_session_changed.
- Updated lib.rs:141 registration to `ensure_working_tree_snapshot`. Exactly one
  working-tree-snapshot command, get-or-create.
- Tests: rewrote the v1 `working_tree_snapshot_replaces_not_stacks` test as
  `working_tree_snapshot_never_orphans_prior` (asserts BOTH oids remain). Added
  `workdir_tree_oid_is_stable_and_matches_snapshot_tree`, `decide_snapshot_reuses_unchanged_workdir`
  (asserts `!created` AND `oid == prior` — catches the tree-vs-commit mixup),
  `decide_snapshot_creates_on_changed_workdir`, `decide_snapshot_creates_first_snapshot`.
  The 4 existing snapshot tests pass unchanged. The two integration-test ReviewSession
  literals keep `working_tree_snapshot: None` (field KEPT) — confirmed compiling via
  unfiltered `just check`.

### Task 2 — In-diff Comment affordance (commit `31c2a2f`)
- Exported `resolveSide` from diff-anchor.ts (no logic change). Both `buildDiffAnchor`
  and the host Old-side guard now use the same function, so they agree on the mixed
  Add+Delete → New edge case.
- HunkView + SplitView: added a Comment button to the **unstaged** `hasSelection`
  toolbar beside Discard/Stage, reusing the commit-mode accent button markup/classes
  verbatim (`var(--color-accent)` / `accent-btn`; no new color). The commit-mode arm is
  byte-for-byte unchanged.
- FullFileView: widened the Comment gate to `diffKind === 'commit' || diffKind === 'unstaged'`
  (full-file is always New-side by construction, no Old-side guard needed).
- DiffPanel: added `workingTreeSnapshotOid` state; scoped `commitOid` to mode
  (`diffKind === 'unstaged' ? (workingTreeSnapshotOid ?? "") : (commitDetail?.oid ?? "")`)
  so a stale snapshot oid can't leak into staged/commit views. `handleCommentLines`
  guards the unstaged Old-side path via shared `resolveSide` (no-op + toast, NOT an
  Old-side comment), then fetches `ensure_working_tree_snapshot` and sets the oid
  BEFORE opening the composer. `handleCommentFullFile` does the same (no Old guard).
  Snapshot oid cleared in `closeComposer`.

### Task 3 — Remove v1 Review-panel entry point (commit `6349d2d`)
- Removed the "Review uncommitted changes" button, `onReviewWorkingTreeClick` handler,
  dead `.snapshot-button` CSS, and now-unused `FilePlus` import from ReviewPanel.
- Removed `reviewWorkingTree` method + interface declaration + comment from
  review-session.svelte.ts.
- `grep` clean: no `reviewWorkingTree`, `onReviewWorkingTreeClick`, `snapshot-button`,
  or `add_working_tree_review` references remain in `src/`.

## Verification

- `just check` green after every task (cargo fmt, biome, svelte-check, clippy,
  cargo-test UNFILTERED, vitest 584 tests). Never a filtered `cargo test --lib`.
- Rust unit tests prove get-or-create reuse/create/first-snapshot and the RMW
  never-orphan invariant.

## Deviations from Plan

- **Toast kind:** The plan said the Old-side guard shows a toast. `showToast` exposes
  only `"success" | "error"` kinds (no `"info"`). Used `"error"` for the
  "Commenting on removed lines isn't supported yet" notice (closest semantic fit — the
  action was rejected) rather than widening the toast API (out of scope, surgical).
- **HunkView/SplitView button placement:** The plan suggested an `{:else if diffKind === 'unstaged'}`
  arm reusing the commit markup. Since an unstaged arm already exists and the Comment
  button must appear *beside* Stage/Discard within a selection, the button was added
  inside the existing unstaged `hasSelection` block (still reusing the commit-mode
  accent markup/classes verbatim). The commit-mode arm is untouched. Same observable
  result, no duplicated commit path.

## Task 4 — In-app UAT (BLOCKING, pending human)

Not executable by `just check` — it is a GUI interaction across the diff toolbar, the
composer, the snapshot backend, and the rendered review doc. The human runs `just dev`
and verifies the six UAT steps in `260531-k4j-PLAN.md` Task 4:
1. Comment button appears for unstaged New-side selections (HunkView, SplitView, full-file).
2. Click Comment → existing CommentComposer opens (no "session"/"snapshot" wording).
3. Submit → comment appears in the review doc with the correct working-tree excerpt.
4. Edit file mid-review, comment again → both comments resolve (earlier NOT orphaned).
5. (Negative) Select only removed (`-`) lines → no-op + toast, NOT an Old-side comment.
6. Review panel no longer shows the "Review uncommitted changes" button.

Resume signal: type "approved" or describe issues.
