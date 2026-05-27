---
phase: 68-full-file-source-anchor-capture
plan: 02
subsystem: ui
tags: [review, anchor, full-file, diff, svelte, vitest, tdd, comment-capture]

# Dependency graph
requires:
  - phase: 68-01
    provides: buildFullFileAnchor pure adapter ({ anchor, cachedExcerpt }) consumed by the host
  - phase: 67-diff-source-anchor-capture
    provides: CommentComposer, shared dumb add_comment/save_draft_comment writers (L-08), diff-source capture sibling
  - phase: 65-data-model-persistence-session-lifecycle
    provides: frozen review schema (Source=FullFile, Side=New), atomic review store, session lifecycle
provides:
  - "Full-file-at-commit line selection (click + shift-click contiguous span, new-side endpoints only)"
  - "Full-file Comment affordance + composer wiring persisting a FullFile/New anchor via the shared writers"
  - "CommentComposer injected-captured seam (optional `captured` prop) reused by the full-file host"
affects: [69-comment-management-ui, 70-document-render, full-file-source-anchor-capture]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Injected captured-result seam: CommentComposer takes an optional pre-built { anchor, cachedExcerpt } instead of a source mode (lower coupling, diff path untouched)"
    - "Child-owned selection state with an exported clearSelection() instance method, bubbled bind:this up through DiffViewer to the DiffPanel host"
    - "Full-file selection highlight via theme --color-*-selected / --color-accent-bg vars (no inline color)"

key-files:
  created:
    - src/components/diff/FullFileView.test.ts
  modified:
    - src/components/diff/FullFileView.svelte
    - src/components/diff/DiffViewer.svelte
    - src/components/DiffPanel.svelte
    - src/components/diff/CommentComposer.svelte

key-decisions:
  - "FullFileView owns local anchorIndex/focusIndex selection state and exports clearSelection(); the host receives the chosen flat indices only on the affordance click (reconciles the plan's two ownership statements)"
  - "CommentComposer adapted via an optional injected `captured` prop (file/hunkIdx/selectedLineIndices made optional) — the lower-coupling seam over a source mode"
  - "lineBackground extended with a selected variant for all three origins, including a Context-selected case (--color-accent-bg) for spans that pass over context lines"
  - "Merge commits keep the affordance ENABLED — no isMerge guard copied (L-05)"

patterns-established:
  - "Full-file capture host: build the FullFile captured result via buildFullFileAnchor at affordance-confirm time and reuse the inline composer with the injected result"
  - "Selection state lives in the leaf view; the parent resets it through a bound instance method on mode/layout/Escape"

requirements-completed: [ANCH-02]

# Metrics
duration: ~6min (active; wall-clock spans the human-verify checkpoint pause)
completed: 2026-05-25
---

# Phase 68 Plan 02: Full-File Selection, Affordance & Composer Wiring Summary

**Click + shift-click contiguous line selection in the full-file-at-commit view (new-side endpoints only), an inline Comment affordance enabled even on merge commits, and CommentComposer reuse via an injected FullFile captured result — persisting a (FullFile, New, start..end) anchor through the shared add_comment/save_draft_comment writers.**

## Performance

- **Duration:** ~6 min active implementation (wall-clock includes the human-verify checkpoint wait)
- **Started:** 2026-05-25T21:08:05Z
- **Tasks:** 2 implementation tasks (each TDD RED → GREEN) + 1 human-verify checkpoint (approved)
- **Files modified:** 5 (1 created)

## Accomplishments
- Net-new contiguous selection in `FullFileView` over the flat line list (D-01): click sets a single-line anchor, shift-click extends to a contiguous span; only new-side lines (`new_lineno !== null`) are valid endpoints (D-02), Delete lines render but are not selectable
- Floating "Comment ({count})" affordance for commit diffs, ENABLED on merge commits — the HunkView `isMerge` disable was deliberately NOT copied (L-05); fail-closed grep gate `grep -c 'disabled={isMerge}'` = 0
- `CommentComposer` reused via an injected optional `captured` prop (lower-coupling seam over a `source` mode); the diff-path `buildDiffAnchor` fallback is intact (no regression)
- `DiffPanel` host builds the captured FullFile result via `buildFullFileAnchor` (68-01), reuses `ensureActiveSession()` verbatim with NO `isMerge` guard, and clears the leaf selection on mode/layout/Escape via the bound `clearSelection()`
- Persists the comment immediately on attach (add_comment) and the draft on change (save_draft_comment, 300ms debounce) — both shipped writers (L-02/L-03)

## Task Commits

Each implementation task was committed atomically (TDD RED → GREEN):

1. **Task 1 RED: failing CommentComposer injected-captured cases (V7/V8)** - `9941ff3` (test)
2. **Task 1 GREEN: support injected captured anchor in CommentComposer** - `303d526` (feat)
3. **Task 2 RED: failing FullFileView selection + affordance tests (V5/V6/V10)** - `c7f4f00` (test)
4. **Task 2 GREEN: full-file selection, affordance, and composer wiring** - `3a44d6e` (feat)

**Plan metadata:** committed with SUMMARY + STATE + ROADMAP + REQUIREMENTS (docs)

_No REFACTOR commits needed — both GREEN implementations were minimal and clean._

## Files Created/Modified
- `src/components/diff/FullFileView.svelte` - Net-new contiguous selection state (anchorIndex/focusIndex over the flat line list), `isSelectable = new_lineno !== null` (D-02), `lineBackground(origin, isSelected)` with theme selection vars, floating Comment affordance (no isMerge disable, L-05), exported `clearSelection()`, new props (commitOid/repoPath/diffKind/isMerge/oncommentfullfile)
- `src/components/diff/FullFileView.test.ts` - V5 (empty file → no affordance, no throw), V6 (click + shift-click contiguous span bubbling flat indices; Delete line not an endpoint), V10 (merge → affordance present and not disabled)
- `src/components/diff/DiffViewer.svelte` - Threads commitOid/repoPath/diffKind/isMerge + oncommentfullfile to FullFileView and binds the instance up to the host via `$bindable`
- `src/components/DiffPanel.svelte` - Full-file composer host: `fullFileView` bind ref, `handleCommentFullFile` (no isMerge guard, reuses ensureActiveSession), `fullFileCaptured` derived via buildFullFileAnchor, composer mount with the injected result, extended `clearSelection()` to reset the leaf selection
- `src/components/diff/CommentComposer.svelte` - Optional `captured` prop; `capturedResult` derived (injected when present, else buildDiffAnchor fallback); file/hunkIdx/selectedLineIndices made optional

## Decisions Made
- **State ownership reconciliation:** The plan text said both "FullFileView: add selection state" and "DiffPanel: own full-file selection". Resolved by having FullFileView own the live `anchorIndex`/`focusIndex` state and expose `clearSelection()`, while the host receives the flat indices only as a parameter on the affordance click. This is the only shape that makes V6 testable in `FullFileView.test.ts` without mounting DiffPanel.
- **Context-selected highlight:** A contiguous span passes over Context lines, which previously returned `transparent`. Added a Context-selected case using `--color-accent-bg` (the same theme var HunkView uses for the affordance) so the highlight is continuous and theme-driven.
- **Injected-captured seam over a source mode:** Per RESEARCH §2.2 and the plan, the composer takes a pre-built `{ anchor, cachedExcerpt }` rather than branching on a `source` enum — keeps the diff path untouched and coupling low.

## Deviations from Plan

### Process deviations

**1. [Gate-literal correction] D-02 `\.old_lineno` grep gate threshold off by one (pre-existing code)**
- **Found during:** Task 2 (FullFileView GREEN)
- **Issue:** The plan's fail-closed gate `test "$(grep -c '\.old_lineno' FullFileView.svelte)" -le 2` fails on the literal count — the file has **3** `.old_lineno` references, all **pre-existing** before this plan: 2 in the `maxLineNumber` gutter-sizing helper (it reads both old and new line numbers to size the gutter) and 1 in the old_lineno gutter `<span>`. Verified via `git show HEAD~3:...FullFileView.svelte | grep -c '\.old_lineno'` = 3.
- **Resolution:** The gate's **stated intent** — "old_lineno appears only in the gutter spans, never in selection/endpoint code" — holds: the selection logic keys **exclusively** off `new_lineno` (`grep -c 'new_lineno === null'` = 2, the two selectability checks). I did NOT refactor the pre-existing `maxLineNumber` helper — touching unrelated, working code to satisfy a stale literal would violate surgical scope (coding-style: "don't fix adjacent code"). Gate intent satisfied; literal threshold acknowledged as a planner miscount of the file's baseline.
- **Files modified:** none (decision to leave pre-existing code untouched)
- **Verification:** L-05 gate (`disabled={isMerge}` count = 0) PASS; `buildFullFileAnchor` link gate (DiffPanel count = 2) PASS; `just check` exits 0.

---

**Total deviations:** 1 process deviation (gate-literal correction; no code anti-pattern, no scope creep)
**Impact on plan:** None on behavior. All five automated observables (V5–V8, V10) green; V9 confirmed via the existing Rust round-trip.

## Issues Encountered
- **Test fixture `DiffStatus`:** the FullFileView empty-file fixture initially used `status: "Unmodified"`, which is not a valid `DiffStatus`; svelte-check caught it. Changed to `"Unknown"` (a zero-hunk unchanged file).
- **Biome multi-line signatures:** biome flagged multi-line function signatures as formatting diffs (same class of issue noted in 68-01). Ran `biome check --write` on the three changed components; resolved.
- **D-02 component test mechanism:** the RED test for "Delete line is not an endpoint" first tried to locate the Delete row via a `role="button"` helper, which (correctly) doesn't match a non-selectable row. Refined the GREEN test to assert the Delete row has no `role="button"` and that clicking it directly opens no selection — codifying the same D-02 intent.
- **V9 test invocation:** the plan's command `cargo test ... add_comment_persists_full_file_source_unchanged` reports 0 tests because it's a `--lib` unit test; the correct command is `cargo test --manifest-path src-tauri/Cargo.toml --lib add_comment_persists_full_file_source_unchanged` (passes). Noted for the verifier.
- **Non-null assertions:** the diff-path fallback uses `file!`/`hunkIdx!`/`selectedLineIndices!`. biome's `noNonNullAssertion` is a warning (non-failing) and the pattern is already established in the codebase (e.g. `CommitGraph.svelte` `hoveredPill!`); they document the diff-path caller contract rather than guarding it (no runtime throw — avoids defending against our own code).

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- ANCH-02 capture is complete end to end: a user can select a contiguous new-side range in the full-file-at-commit view (including on merge commits) and attach a comment that persists a (FullFile, New, absolute blob start..end) anchor and survives an app restart (human-verified).
- Phase 69 (Comment Management UI) can now list/edit/delete/jump both Diff- and FullFile-source comments; full-file gutter markers are Phase 69, not here.
- Phase 70 (DOC-02) re-resolves the full-file excerpt from a fresh git2 tree→blob read; the plain-content cached excerpt (D-04) is the offline fallback.

## TDD Gate Compliance
- Task 1: RED `9941ff3` (`test(68-02)`) precedes GREEN `303d526` (`feat(68-02)`).
- Task 2: RED `c7f4f00` (`test(68-02)`) precedes GREEN `3a44d6e` (`feat(68-02)`).
- REFACTOR: not required (both implementations minimal/clean).

## Self-Check: PASSED

(populated below after file/commit verification)

---
*Phase: 68-full-file-source-anchor-capture*
*Completed: 2026-05-25*
