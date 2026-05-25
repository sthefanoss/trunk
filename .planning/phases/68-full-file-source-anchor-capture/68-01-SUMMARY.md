---
phase: 68-full-file-source-anchor-capture
plan: 01
subsystem: ui
tags: [review, anchor, full-file, diff, typescript, vitest, tdd]

# Dependency graph
requires:
  - phase: 65-data-model-persistence-session-lifecycle
    provides: frozen review schema (Anchor, Source=FullFile, Side=New) the adapter targets
  - phase: 67-diff-source-anchor-capture
    provides: buildDiffAnchor pure-adapter pattern and shared dumb add_comment writer (L-08)
provides:
  - "buildFullFileAnchor — pure capture-time adapter for the full-file-at-commit view"
  - "FullFileAnchorResult type ({ anchor, cachedExcerpt }) for the 68-02 UI plan to consume"
affects: [68-02, full-file-source-anchor-capture, comment-capture, FullFileView]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Sibling pure adapter (full-file-anchor.ts) rather than conditionalizing the shipped diff adapter"
    - "Flat-line-list selection model (file.hunks.flatMap) for the full-file view"

key-files:
  created:
    - src/lib/full-file-anchor.ts
    - src/lib/full-file-anchor.test.ts
  modified: []

key-decisions:
  - "Separate module src/lib/full-file-anchor.ts (sibling of diff-anchor.ts), not an extension — the two diverge on D-02/D-04 and share almost no logic once side/source are constants"
  - "Selection input is Set<number> of indices into file.hunks.flatMap(h => h.lines), mirroring buildDiffAnchor's positional shape"
  - "Gap marker count N = next.new_lineno - prev.new_lineno - 1, derived from new-side line numbers (not index arithmetic)"

patterns-established:
  - "Full-file capture: side=New / source=FullFile constants, new-side coordinates only, plain-content excerpt, gap markers"

requirements-completed: [ANCH-02]

# Metrics
duration: ~6min
completed: 2026-05-25
---

# Phase 68 Plan 01: Full-File Capture Adapter Summary

**Pure TDD-built `buildFullFileAnchor` — the sibling of `buildDiffAnchor` for the full-file-at-commit view — translating a flat line selection into a `(source=FullFile, side=New, start..end)` anchor plus a plain-content cached excerpt with `… N lines unchanged …` gap markers.**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-05-25T22:56:00Z
- **Completed:** 2026-05-25T22:59:00Z
- **Tasks:** 2 (RED + GREEN)
- **Files modified:** 2 (both created)

## Accomplishments
- Pure, fully-tested full-file capture adapter carrying the core net-new risk of Phase 68 (D-01..D-04, L-01, L-04)
- New-side-only range/excerpt: removed (Delete) lines excluded from both (D-02)
- Plain-content excerpt with no diff prefixes, ready for Phase 70's language-fenced render (D-04)
- Gap-crossing selections keep a correct monotonic blob range and insert a `… N lines unchanged …` marker (D-03)
- All purity/divergence grep gates hold (no `old_lineno`, no `prefixLine`, no `resolveSide`); `FullFile` is the literal constant

## Task Commits

Each task was committed atomically (TDD RED → GREEN):

1. **Task 1: RED — failing V1–V4 unit tests** - `207b335` (test)
2. **Task 2: GREEN — implement buildFullFileAnchor** - `41bdf85` (feat)

**Plan metadata:** committed with SUMMARY + STATE + ROADMAP (docs)

_No REFACTOR commit needed — GREEN implementation was already clean and minimal._

## Files Created/Modified
- `src/lib/full-file-anchor.ts` - Pure adapter `buildFullFileAnchor(commitOid, file, selectedIndices) -> { anchor, cachedExcerpt }`; exports `buildFullFileAnchor` and `FullFileAnchorResult`
- `src/lib/full-file-anchor.test.ts` - V1–V4 unit coverage (cloned fixture builders from `diff-anchor.test.ts`)

## Decisions Made
- **Separate sibling module** rather than extending `diff-anchor.ts`: once D-02 (delete exclusion) and D-04 (plain content) diverge, the two adapters share almost no logic (no `resolveSide`, no `prefixLine`, flat list vs. single hunk). Keeps the shipped diff adapter untouched.
- **Selection input `Set<number>`** of flat indices into `file.hunks.flatMap(h => h.lines)`, mirroring `buildDiffAnchor`'s positional shape; sorted + deduped via `Set` + `sort`.
- **Gap N from new-side line numbers:** `N = curr.new_lineno - prev.new_lineno - 1` between consecutive survivors (index arithmetic would be wrong — flat-list indices are contiguous, only line numbers skip).
- **No defensive guard for an all-Delete selection** (`Math.min(...[])` → Infinity): mirrors `diff-anchor.ts`, which doesn't guard either. The caller (68-02 UI) prevents Delete lines from being selection endpoints (D-02), so the contract is enforced upstream — adding a fallback for a case we control would be defending against our own code.

## Deviations from Plan

None - plan executed exactly as written. The RED/GREEN gate sequence is satisfied (`test(68-01)` precedes `feat(68-01)`).

## Issues Encountered
- Biome flagged a multi-line `expect(...).toBe(...)` in the V4 test as a formatting diff during `just check`. Collapsed it to a single line (formatting-only, no behavior change), then `just check` passed end-to-end.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- The pure adapter is ready for plan 68-02 (full-file selection state + Comment affordance + `CommentComposer` wiring) to consume a trusted, deterministic `{ anchor, cachedExcerpt }`.
- 68-02 must thread `commitOid`/`repoPath` to the composer, NOT copy HunkView's `isMerge` disable (L-05), and add selection-highlight CSS using a theme `--color-*` var.

## TDD Gate Compliance
- RED gate: `207b335` (`test(68-01)`) — tests written first, confirmed failing (module not found).
- GREEN gate: `41bdf85` (`feat(68-01)`) — implementation after, all 5 cases pass.
- REFACTOR gate: not required (implementation already minimal/clean).

## Self-Check: PASSED

- `src/lib/full-file-anchor.ts` — FOUND
- `src/lib/full-file-anchor.test.ts` — FOUND
- `68-01-SUMMARY.md` — FOUND
- Commit `207b335` (RED) — FOUND
- Commit `41bdf85` (GREEN) — FOUND

---
*Phase: 68-full-file-source-anchor-capture*
*Completed: 2026-05-25*
