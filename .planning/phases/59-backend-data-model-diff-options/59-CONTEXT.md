# Phase 59: Backend Data Model & Diff Options - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Extend DiffLine with enrichment fields (word_spans, syntax_tokens) and wire DiffRequestOptions (context lines, whitespace ignore, show full file) through all diff commands. Persist diff display preferences via LazyStore.

</domain>

<decisions>
## Implementation Decisions

### Whitespace Handling
- **D-01:** Single boolean toggle using git2's `ignore_whitespace_change` option. No multi-level dropdown (ignore all, ignore EOL, etc.) — one on/off toggle like GitHub Desktop and VS Code.

### Context Lines
- **D-02:** Context lines is a bounded integer (0-10 range). No "All" option in the context lines setting.
- **D-03:** Separate `show_full_file: bool` toggle on DiffRequestOptions. When true, backend passes a large context_lines value to git2 to return the entire file. When false, the user's context_lines preference applies.
- **D-04:** Context lines and show_full_file are independent settings — context_lines is only relevant when show_full_file is off.

### Enrichment Fields
- **D-05:** DiffLine gets `word_spans` and `syntax_tokens` fields (empty vecs in this phase, populated in Phases 60-61).

### Claude's Discretion
- Enrichment field structure (byte offset ranges vs pre-segmented strings) — Claude picks what aligns best with `similar` crate's `iter_inline_changes()` output and `syntect`'s token ranges.

### Preference Scope
- **D-06:** All diff display preferences are global (shared across tabs), not per-tab. Uses existing LazyStore pattern with dedicated keys in `trunk-prefs.json`.
- **D-07:** Preferences persisted: context_lines, ignore_whitespace, show_full_file (plus word_wrap, show_invisibles, view_mode for later phases).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing diff implementation
- `src-tauri/src/commands/diff.rs` — All diff commands and `walk_diff_into_file_diffs` central function
- `src-tauri/src/git/types.rs` — DiffLine, DiffHunk, FileDiff, DiffOrigin Rust types
- `src/lib/types.ts` — TypeScript mirrors of all Rust types (must stay in sync)

### Persistence pattern
- `src/lib/store.ts` — LazyStore get/set pattern for trunk-prefs.json

### Frontend diff consumption
- `src/components/DiffPanel.svelte` — Current diff rendering component (will be refactored in Phase 62)
- `src/components/RepoView.svelte` — Calls diff commands via safeInvoke (lines 217, 249, 306, 447)

### Requirements
- `.planning/REQUIREMENTS.md` — CTXL-01, CTXL-02, WHSP-01, DISP-03 mapped to this phase

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `walk_diff_into_file_diffs` (diff.rs:27): Central diff-to-DTO converter — needs to accept options and populate enrichment fields
- `LazyStore` pattern (store.ts): Established get/set/save per key pattern — extend for diff preferences
- `safeInvoke<T>` (RepoView.svelte): IPC wrapper — callers need updated to pass options
- Inner-fn pattern: All diff commands have `_inner` variants for testability — new options flow through these

### Established Patterns
- All Rust types in `git/types.rs` use owned types (String, Vec, etc.), no git2 lifetimes
- TypeScript types mirror Rust 1:1 in `lib/types.ts`
- git2::DiffOptions used in `diff_unstaged_inner` and `diff_staged_inner` already — extend with `.context_lines()` and `.ignore_whitespace_change()`
- `diff_commit_inner` currently passes `None` for DiffOptions — needs to create and pass options

### Integration Points
- Frontend callers in RepoView.svelte pass `{ path, file_path }` or `{ path, oid }` — need to add options parameter
- DiffPanel.svelte reads FileDiff[] — needs to handle new DiffLine fields (word_spans, syntax_tokens) gracefully when empty
- LazyStore keys in trunk-prefs.json — add diff preference keys alongside existing ones

</code_context>

<specifics>
## Specific Ideas

- Context line presets for the dropdown (later phases): 0/3/5/10 — not 25 or All
- The "All"/"full file" concept from CTXL-02 is handled by a separate `show_full_file` boolean, not a context_lines value
- WHSP-01 requirement ("re-fetches diff with ignore_whitespace_change") maps directly to the git2 DiffOptions method of the same name

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 59-backend-data-model-diff-options*
*Context gathered: 2026-03-28*
