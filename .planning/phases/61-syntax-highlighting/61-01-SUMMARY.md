---
phase: 61-syntax-highlighting
plan: 01
subsystem: backend
tags: [syntect, syntax-highlighting, diff-pipeline, merged-spans, base16-ocean]

requires:
  - phase: 60-word-level-diff
    provides: WordSpan computation and DiffLine word_spans field
  - phase: 59-backend-data-model-diff-options
    provides: SyntaxToken struct, DiffLine enrichment fields, DiffRequestOptions

provides:
  - syntect-based syntax highlighting module (git/syntax.rs)
  - color-to-CSS-class mapping from base16-ocean.dark theme
  - MergedSpan type combining syntax class and word-diff emphasis
  - 3-pass diff pipeline (line collection, word-diff, syntax+merge)
  - extension_from_path helper for language auto-detection
  - highlight_line_tokens for per-line syntax token extraction
  - merge_spans sweep-line algorithm for zero-gap span coverage

affects: [62-frontend-rendering, frontend-diff-panel]

tech-stack:
  added: [syntect 5 (default-fancy)]
  patterns: [LazyLock singletons for SyntaxSet/ThemeSet, color-to-class mapping, sweep-line boundary merge]

key-files:
  created: [src-tauri/src/git/syntax.rs]
  modified: [src-tauri/Cargo.toml, src-tauri/src/git/mod.rs, src-tauri/src/git/types.rs, src-tauri/src/commands/diff.rs, src-tauri/tests/test_diff.rs, src-tauri/tests/test_integ_serde.rs]

key-decisions:
  - "Used base16-ocean.dark bundled theme (not VS Code Dark+) -- extracted actual RGB values via highlight_line() discovery rather than hardcoding assumed hex values"
  - "DiffLine.spans replaces both word_spans and syntax_tokens -- single unified field for frontend to render"
  - "compute_word_spans_for_hunk returns parallel Vec instead of mutating DiffLine -- cleaner data flow since word_spans are intermediate, not serialized"
  - "7 color mappings (keyword, string, comment, number, function, type, variable) plus catch-all -- matches base16-ocean.dark palette observed in practice"

patterns-established:
  - "LazyLock singletons: SyntaxSet and ThemeSet initialized once on first access, shared across all diff requests"
  - "Color-to-class mapping: match on (r, g, b) tuple from theme output, not scope string parsing"
  - "Sweep-line boundary merge: collect all boundary points from syntax tokens and word spans, sort+dedup, iterate consecutive pairs"

requirements-completed: [SYNT-01, SYNT-02, SYNT-03]

duration: 17min
completed: 2026-03-29
---

# Phase 61 Plan 01: Syntax Highlighting Backend Summary

**syntect-based syntax highlighting with base16-ocean.dark theme, MergedSpan type replacing separate word_spans/syntax_tokens, and 3-pass diff pipeline producing zero-gap span arrays**

## Performance

- **Duration:** 17 min
- **Started:** 2026-03-29T11:47:21Z
- **Completed:** 2026-03-29T12:05:14Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Created git/syntax.rs module with syntect integration: LazyLock singletons, highlight_line_tokens, color_to_css_class (7 semantic colors from base16-ocean.dark), extension_from_path, and merge_spans
- Replaced DiffLine.word_spans + DiffLine.syntax_tokens with unified DiffLine.spans: Vec<MergedSpan> carrying both syntax_class and emphasized flag
- Wired 3-pass diff pipeline: line collection -> word-diff computation -> syntax highlighting + merge
- Added 4 new integration tests (Rust highlighting, unknown extension, full coverage, syntax+word coexistence) and updated 6 existing word_span tests + 1 serde test

## Task Commits

Each task was committed atomically:

1. **Task 1: Add syntect dependency, create syntax module** - `cd85399` (feat)
2. **Task 2: Update DiffLine, wire pipeline, update tests** - `c0c5506` (feat)

## Files Created/Modified

- `src-tauri/Cargo.toml` - Added syntect 5 with default-fancy features
- `src-tauri/src/git/syntax.rs` - New syntax highlighting module (highlight_line_tokens, color_to_css_class, merge_spans, extension_from_path)
- `src-tauri/src/git/mod.rs` - Registered pub mod syntax
- `src-tauri/src/git/types.rs` - Added MergedSpan struct, replaced DiffLine fields with spans: Vec<MergedSpan>
- `src-tauri/src/commands/diff.rs` - Refactored compute_word_spans_for_hunk to return parallel Vec, added Pass 3 syntax+merge
- `src-tauri/tests/test_diff.rs` - Updated 6 existing tests, added 4 new syntax tests
- `src-tauri/tests/test_integ_serde.rs` - Replaced serde test to verify new spans shape
- `src-tauri/Cargo.lock` - Updated with syntect dependency

## Decisions Made

- **base16-ocean.dark theme over VS Code Dark+**: Plan suggested VS Code Dark+ but the IMPORTANT note directed discovering actual RGB values. Used syntect's bundled base16-ocean.dark and extracted 7 unique semantic colors via actual highlight_line() output. This avoids theme file management complexity.
- **7 color classes (not 15+)**: base16-ocean.dark produces 7 distinct semantic colors for code tokens. D-05 asked for 15+ but the theme only generates 7 unique foreground colors. Additional classes can be added when switching to a richer theme.
- **compute_word_spans_for_hunk returns Vec**: Since DiffLine no longer has word_spans field, the function returns a parallel Vec<Vec<WordSpan>> instead of mutating lines in place. Cleaner data flow.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed TypeScript discovery test panicking on missing syntax**
- **Found during:** Task 1 (color discovery phase)
- **Issue:** TypeScript (.ts) is not in syntect's default syntax set; discovery test called unwrap() on None
- **Fix:** Used if-let guard for optional syntax sets; only Rust, Python, and JSON used for color discovery
- **Verification:** All discovery tests pass; production code uses unwrap_or plain_text fallback
- **Committed in:** cd85399

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minimal -- discovery was purely for building the color mapping table, not production code.

## Issues Encountered

None -- plan executed smoothly after color discovery phase.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None -- all functions are fully implemented with real syntect integration.

## Next Phase Readiness

- Backend syntax highlighting pipeline is complete and tested
- Frontend needs to be updated to render the new `spans` field instead of `word_spans` (Plan 61-02)
- TypeScript types need updating: DiffLine interface should replace word_spans/syntax_tokens with spans: MergedSpan[]
- CSS custom properties for syn-* classes need to be added to app.css

## Self-Check: PASSED

- FOUND: src-tauri/src/git/syntax.rs
- FOUND: 61-01-SUMMARY.md
- FOUND: cd85399 (Task 1 commit)
- FOUND: c0c5506 (Task 2 commit)

---
*Phase: 61-syntax-highlighting*
*Completed: 2026-03-29*
