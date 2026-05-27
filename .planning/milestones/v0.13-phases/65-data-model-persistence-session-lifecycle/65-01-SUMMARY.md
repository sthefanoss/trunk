---
phase: 65-data-model-persistence-session-lifecycle
plan: 01
subsystem: database
tags: [serde, dto, review-session, schema, rust, typescript]

# Dependency graph
requires: []
provides:
  - "ReviewSession / Comment / Anchor / DraftComment Rust DTOs (Serialize + Deserialize) in git/types.rs"
  - "Source / Side PascalCase enums (no rename_all) serializing as \"Diff\"/\"FullFile\"/\"Old\"/\"New\""
  - "String-for-string TypeScript mirror (ReviewSession, Comment, Anchor, DraftComment, Source, Side) in src/lib/types.ts"
  - "serde-shape + round-trip guardrail tests asserting schema_version=1 and absence of forbidden anchor/comment fields"
affects: [66-commit-selection, 67-diff-source-capture, 68-full-file-capture, 70-render, review_store, review-commands]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Round-trippable DTO: derive Deserialize alongside Serialize for on-disk types (mirrors DiffStatus)"
    - "PascalCase enum serialization with NO rename_all (mirrors RefType); snake_case struct fields"
    - "serde_json::to_value structural-equality round-trip test (avoids forcing PartialEq onto production structs)"

key-files:
  created:
    - .planning/phases/65-data-model-persistence-session-lifecycle/65-01-SUMMARY.md
  modified:
    - src-tauri/src/git/types.rs
    - src/lib/types.ts
    - src-tauri/tests/test_integ_serde.rs

key-decisions:
  - "Round-trip test compares via serde_json::to_value structural equality instead of adding PartialEq to ReviewSession/Comment/Anchor/DraftComment — the test must not drive an unneeded derive onto production structs."
  - "Added a third test (session_serializes_draft_comment_when_present) to cover the DraftComment serialization path, since the locked schema's anchored-session fixture only exercises draft_comment = None."

patterns-established:
  - "On-disk DTO pattern: Debug + Serialize + Deserialize + Clone, snake_case fields, PascalCase enum variants with no rename_all."
  - "Migration-guardrail test: assert forbidden fields .is_null() on the serialized JSON so a future regression that re-introduces a position-based anchor fails CI."

requirements-completed: [SESS-01, SESS-02, SESS-03]

# Metrics
duration: ~15min
completed: 2026-05-25
---

# Phase 65 Plan 01: Review-Session Data Model Summary

**The full keystone review-session schema (session + comment + anchor + draft + Source/Side enums) is defined in Rust and mirrored string-for-string in TypeScript, serializing PascalCase enums / snake_case fields, round-tripping through serde, and provably excluding hunk_index/line_index/diff-option fields.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-25 (PLAN_START at execution)
- **Completed:** 2026-05-25
- **Tasks:** 2 (TDD RED + GREEN)
- **Files modified:** 3

## Accomplishments
- Froze the on-wire and on-disk review-session shape (D-01..D-07, DP-02) so downstream phases (66/67/68/70) write into it without reshaping.
- Locked the migration guardrail: a serde-shape test fails CI if any forbidden anchor field (hunk_index, line_index, context_lines, ignore_whitespace) or comment metadata field (timestamp, author, severity, status) is ever re-introduced.
- Mirrored the Rust shape string-for-string in TypeScript and verified it compiles via svelte-check.

## Task Commits

Each task was committed atomically:

1. **Task 1: Serde-shape test (RED)** - `04f4fa1` (test)
2. **Task 2: Rust DTOs + TS mirror (GREEN)** - `f7e2b84` (feat)

**Plan metadata:** (this docs commit)

_TDD gate sequence: test (RED) → feat (GREEN). No REFACTOR commit — types are minimal and the test reads cleanly._

## Files Created/Modified
- `src-tauri/src/git/types.rs` - Added Source/Side enums and Anchor/Comment/DraftComment/ReviewSession structs (all derive Deserialize for disk read-back).
- `src/lib/types.ts` - Added the string-for-string TypeScript mirror (Source/Side unions + interfaces).
- `src-tauri/tests/test_integ_serde.rs` - Added session_serde_shape, session_round_trips, session_serializes_draft_comment_when_present.

## Decisions Made
- **serde_json::to_value round-trip comparison:** The plan allowed "Clone + PartialEq on the comparison path, OR field-by-field assert." Since the locked schema only puts PartialEq on Source/Side (not on the structs), comparing serialized `Value`s gives structural equality for free without adding a PartialEq derive that production doesn't need. Test should not drive an unneeded derive onto production code.
- **Extra DraftComment test:** Added `session_serializes_draft_comment_when_present` so the DraftComment serialization path is exercised — the canonical anchored-session fixture uses `draft_comment: None`, which only proves the null path.

## Deviations from Plan
None - plan executed exactly as written. (The extra round-trip-via-Value approach and the added DraftComment test are within the plan's explicitly-permitted latitude; no deviation rules triggered.)

## Issues Encountered
- `cargo test` accepts only a single substring filter, so `cargo test ... session_serde_shape session_round_trips` errored on the second arg. Resolved by running with the shared `session_` prefix via `--test test_integ_serde`. No code impact.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- The schema is frozen. Phase 66 (commit selection) can populate `ReviewSession.commits`; Phase 67/68 (capture) can populate `comments` and `draft_comment`; the `review_store` persistence primitive (Plan 02) can rely on every type deriving `Deserialize`.
- No blockers. The forbidden-field guardrail is in CI as of this plan.

## TDD Gate Compliance
- RED gate: `04f4fa1` (test) — failing compile, types undefined.
- GREEN gate: `f7e2b84` (feat) — tests pass.
- Sequence verified: test commit precedes feat commit.

## Self-Check: PASSED
- Files verified present: src-tauri/src/git/types.rs, src/lib/types.ts, src-tauri/tests/test_integ_serde.rs, 65-01-SUMMARY.md
- Commits verified in git log: 04f4fa1 (RED test), f7e2b84 (GREEN feat)
- Symbols verified: `pub struct ReviewSession` (Rust), `export interface ReviewSession` (TS), `fn session_serde_shape` (test)

---
*Phase: 65-data-model-persistence-session-lifecycle*
*Completed: 2026-05-25*
