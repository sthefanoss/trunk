---
phase: 69-comment-management-ui
plan: 04
subsystem: ui
tags: [svelte5, runes, typescript, dto, review-session, frontend-contracts]

# Dependency graph
requires:
  - phase: 69-comment-management-ui
    plan: 01
    provides: "Comment v2 wire shape — id: String (D-03) + commit_oid: Option<String> (D-01); the serde PascalCase-enum / snake_case-field convention these TS DTOs mirror"
provides:
  - "TS Comment DTO v2: Comment.id + Comment.commit_oid mirroring the Rust wire shape string-for-string"
  - "OrphanReason union + CommentResolution interface mirroring the Rust resolution DTOs (land backend in 69-03)"
  - "review-session.svelte.ts rune factory createReviewSession() owning center-pane Review-mode state (rightPaneMode panel|diff, reviewActive) and a decoupled jumpTo(comment, deps) action (D-07/D-08)"
affects: [69-05 panel rendering + RepoView wiring (binds jumpTo's JumpDeps callbacks)]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Rune factory in a .svelte.ts module: createXxxState() returning { state, ...methods } with $state declared inside (mirrors createUndoRedoState)"
    - "Dependency-injected navigation seams (JumpDeps): the rune composes injected callbacks so it stays decoupled from RepoView internals; the component binds them in a later plan"

key-files:
  created:
    - src/lib/review-session.svelte.ts
  modified:
    - src/lib/types.ts

key-decisions:
  - "commit_oid?: string | null uses the optional `?` per the plan/must-haves, even though sibling fields (anchor, cached_excerpt) use plain `| null` — followed the plan literally rather than normalizing to the neighbors"
  - "jumpTo deps are typed `() => void | Promise<void>` (sync-or-async) since RepoView's handleCommitSelect/handleCommitFileSelect are async; await handles both"
  - "jumpTo early-returns on `comment.anchor === null` (strict, matches the Anchor | null type) — commit-level and orphaned comments have no line target (D-08)"
  - "RightPaneMode extracted as a named type alias so the literal union ('panel' | 'diff') reads once and reuses on the $state init cast"

patterns-established:
  - "DI navigation seams via a typed deps object keep a rune module decoupled from the component that owns the concrete machinery"

requirements-completed: [CMT-01, CMT-04]

# Metrics
duration: ~6min
completed: 2026-05-26
---

# Phase 69 Plan 04: Frontend Contracts (v2 DTOs + review-session rune) Summary

**Defined the frontend interface layer the panel + wiring (Plan 05) consume: extended the TS Comment DTO to the v2 wire shape (`id`, `commit_oid?`), added the `OrphanReason`/`CommentResolution` mirror types, and created the `review-session.svelte.ts` rune factory that owns center-pane Review-mode state (`rightPaneMode` panel|diff, `reviewActive`) and a RepoView-decoupled `jumpTo(comment, deps)` action (D-07/D-08).**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-05-26T03:00Z (approx)
- **Completed:** 2026-05-26T03:02Z (approx)
- **Tasks:** 2 (both type="auto", non-TDD interface definition)
- **Files modified:** 2 (1 created, 1 modified)

## Accomplishments

- TS `Comment` carries `id: string` and `commit_oid?: string | null`, string-for-string with the Rust v2 wire shape (snake_case field, `| null` for `Option`).
- `OrphanReason = "CommitGone" | "FileGone" | "LineOutOfRange"` and `CommentResolution { id; resolvable; reason }` mirror the Rust resolution DTOs (whose backend lands in 69-03), following the existing PascalCase-enum-string convention.
- New `review-session.svelte.ts` rune factory `createReviewSession()` returns `{ state, ...methods }` with `$state` declared inside, owning `reviewActive` + `rightPaneMode` ("panel" | "diff").
- `jumpTo(comment, deps)` composes injected `JumpDeps` callbacks (`selectCommit`/`selectFile`/`scrollToRange`); it returns early on a null anchor (commit-level/orphaned, D-08) and otherwise selects commit+file, swaps the center pane to "diff", and scrolls to the anchor range — all without importing RepoView.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend TS DTOs to v2 + add resolution types** - `4696a86` (feat)
2. **Task 2: Create review-session.svelte.ts rune module** - `9e6bb0e` (feat)

**Plan metadata:** docs commit (this SUMMARY + STATE + ROADMAP).

## Files Created/Modified

- `src/lib/types.ts` - Added `id` + `commit_oid?` to `Comment`; added `OrphanReason` union and `CommentResolution` interface mirroring the Rust resolution DTOs.
- `src/lib/review-session.svelte.ts` - NEW. `createReviewSession()` factory owning center-pane Review-mode state (`rightPaneMode`/`reviewActive`) and the decoupled `jumpTo` action; exports `RightPaneMode`, `ReviewSessionState`, `JumpDeps`, `ReviewSessionManager`.

## Decisions Made

- **`commit_oid?` keeps the optional `?`** despite sibling `Comment` fields using plain `| null` — the plan and must-haves explicitly specify `?`, so it was followed literally rather than normalized to the neighbors.
- **`JumpDeps` callbacks typed sync-or-async** (`() => void | Promise<void>`): RepoView's selection handlers are `async`, and `await` transparently handles both; this keeps Plan 05's binding flexible.
- **Strict null check in `jumpTo`** (`comment.anchor === null`) matches the `Anchor | null` type exactly; commit-level and orphaned comments have no jump target (D-08).
- **`RightPaneMode` named alias** so the `"panel" | "diff"` literal union is written once and reused on the `$state` initializer cast.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. Both tasks were pure interface definitions; `svelte-check` reported 0 errors after each change (no frontend code constructs a `Comment` literal — they all come from IPC — so adding the required `id` field broke no consumers). The full `just check` gate (fmt, biome, svelte-check, clippy, cargo-test, vitest 507 passing) passed before finalizing, avoiding the weaker-per-task-verify trap that bit Plans 01 and 02.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The frontend contract layer is complete and `just check`-clean. Plan 05 can:
  - render line-anchored vs commit-level comments by branching on `comment.anchor === null && comment.commit_oid != null`;
  - badge orphans using `CommentResolution.reason` (`OrphanReason`);
  - bind the rune's `JumpDeps` (`selectCommit`/`selectFile`/`scrollToRange`) to RepoView's `handleCommitSelect`/`handleCommitFileSelect`/scroll machinery and gate the center pane on `reviewSession.state.rightPaneMode`.
- Backend reads `list_session_comments` / `resolve_session_comments` (referenced by name only here) land in Plan 03 — no backend was implemented in this plan, per scope.
- No blockers.

---
*Phase: 69-comment-management-ui*
*Completed: 2026-05-26*

## Self-Check: PASSED

- All files present on disk (types.ts, review-session.svelte.ts, 69-04-SUMMARY.md).
- Both task commits found in git history (4696a86, 9e6bb0e).
- Acceptance criteria verified by grep: `id: string` + `commit_oid?: string | null` + `OrphanReason` + `CommentResolution` in types.ts; `createReviewSession` export, `"panel" | "diff"` union, `comment.anchor === null` early return, `rightPaneMode = "diff"`, and NO RepoView import (only JSDoc mentions) in review-session.svelte.ts.
- `just check` exits 0 (fmt, biome, svelte-check 0 errors, clippy, cargo-test, vitest 507 passing).
