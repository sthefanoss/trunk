# Project Research Summary

**Project:** Trunk v0.12 — Better Diffs
**Domain:** Git GUI diff viewer enhancements (syntax highlighting, word-level diff, view modes, whitespace options, context lines, display options)
**Researched:** 2026-03-28
**Confidence:** HIGH

## Executive Summary

Trunk v0.12 upgrades the diff viewer from a plain hunk display to a professional-grade experience matching GitKraken, Sublime Merge, and VS Code. The six table-stakes features — syntax highlighting, split view, word-level diff, whitespace toggle, configurable context lines, and line number gutters — are all achievable with the existing stack plus two minimal additions (`syntect` and `similar` Rust crates). The architecture research and pitfalls research agree on one decisive point: syntax highlighting and word-level diffing belong in Rust, not JavaScript, because the project rule against inline colors makes Shiki's output a poor fit, and frontend diffing blocks the main thread without solving the caching problem. STACK.md independently arrived at the opposite conclusion (Shiki in the frontend), but ARCHITECTURE.md and PITFALLS.md are recommended here because they respect the CSS custom properties rule and are safer for large files.

The most dangerous risk in this milestone is a data-integrity pitfall, not a performance one: whitespace-ignored diffs produce different hunk indices than real diffs, and staging against a whitespace-ignored view silently corrupts the staging area. The correct mitigation is to disable hunk and line staging when whitespace ignore is active — the approach GitHub Desktop uses — not to attempt index remapping. A second structural risk is split view scroll sync, which has a known-good pattern already implemented in the existing `MergeEditor.svelte` (guard flag + `requestAnimationFrame`). That component's `flattenAligned()` spacer row pattern also solves split view line alignment, making these risks largely pre-solved by existing codebase patterns.

The suggested build order (backend data model, word-level diff, syntax highlighting, UI refactor, full-file view + display options, split view) respects all feature dependencies. Each phase is backward compatible — the enriched `DiffLine` fields are optional vecs, so existing rendering continues to work throughout the build.

## Key Findings

### Recommended Stack

The existing stack needs two Rust crates and zero new frontend dependencies for the core milestone: `syntect = "5"` for syntax highlighting and `similar = { version = "2", features = ["inline", "unicode"] }` for word-level diff. Both are compiled into the binary with no webview bundle impact. All whitespace and context-line behavior is already available in `git2 = "0.19"` (currently in Cargo.toml) via `DiffOptions` — no version bump needed.

The STACK.md researcher recommended Shiki (frontend JS) because it avoids IPC payload bloat and matches the frontend ownership pattern. ARCHITECTURE.md and PITFALLS.md both recommend `syntect` (Rust backend) because: (1) Shiki outputs inline styles which violates the project's CSS custom properties rule; (2) `syntect` tokens can be emitted as scope-named spans mapped to CSS classes in `app.css`; (3) the `SyntaxSet` can be loaded once into Tauri managed state and shared across all commands (~50ms startup cost, amortized). The `syntect` path is the correct choice.

**Core technologies:**
- `syntect = "5"`: Syntax highlighting — Sublime Text grammars, used by bat/delta/helix, runs on Rust thread pool, outputs scope tokens not inline styles, zero frontend bundle impact
- `similar = { version = "2", features = ["inline", "unicode"] }`: Word-level diff — authored by mitsuhiko, `iter_inline_changes()` is purpose-built for intra-line diffing, prevents main-thread blocking
- `git2 = "0.19"` (existing): Whitespace and context options — `ignore_whitespace_change()`, `context_lines(n)`, `interhunk_lines(n)` all present; no version bump needed
- `diff = "^8.0.4"` (npm): Word-level diff frontend library — only needed if word-level diff moves to the frontend; not recommended for v0.12 core but available if the approach changes

### Expected Features

**Must have (table stakes — defines "better diffs"):**
- Syntax highlighting — every competitor ships this; absence makes the GUI feel like a terminal
- Split (side-by-side) view — expected by developers reviewing large changes; requires alignment algorithm and phantom line insertion
- Word-level (intra-line) diff — highlights specific changed words within add/delete lines; reduces cognitive load dramatically
- Whitespace toggle — essential when reviewing reformatted code; must disable staging when active
- Configurable context lines (global dropdown: 3/5/10/25/All) — "All" is full-file view via `context_lines(999999)`
- Line numbers in gutter — data already present in `DiffLine.old_lineno` / `DiffLine.new_lineno`; trivial to render

**Should have (add during polish pass):**
- Word wrap toggle — low complexity; interacts with split view row heights, so add after split view is stable
- Show invisible characters — renders tabs/trailing spaces as visible symbols; off by default; niche but valued
- Per-hunk context expand buttons — better UX than global slider, but more complex; defer within v0.12 if time is tight

**Defer to v0.13+:**
- Scrollbar minimap — high complexity (Canvas rendering); genuine differentiator but not essential for launch
- Image diff — separate feature domain entirely
- Blame integration in diff view — belongs in its own dedicated view

### Architecture Approach

The architecture cleanly extends the existing data model: `DiffLine` gains two optional vec fields (`word_spans: Vec<WordSpan>` and `syntax_tokens: Vec<SyntaxToken>`), which are empty for context lines or when highlighting is disabled. A new `DiffRequestOptions` struct flows from the frontend toolbar through `invoke()` into `git2::DiffOptions` parameters — but critically, staging commands never receive these options and always use the default diff. The `DiffPanel.svelte` monolith (currently 632 lines) gets refactored into `DiffPanel` (outer shell) → `DiffToolbar` → `DiffViewer` (dispatcher) → `HunkView` / `FullFileView` / `SplitView`, each using a new `DiffLine.svelte` component. IPC payload grows from ~50 KB to ~80 KB for a typical 500-line diff with enrichment — well within Tauri's comfort zone.

**Major components:**
1. `DiffToolbar.svelte` (new) — view mode selector, whitespace toggle, context lines dropdown, word wrap / show invisibles toggles
2. `DiffViewer.svelte` (new) — dispatches to HunkView / FullFileView / SplitView based on active mode
3. `DiffLine.svelte` (new) — single line renderer with gutter (line numbers), syntax token spans, word-diff background overlays
4. `SplitView.svelte` (new) — two scroll containers synced via guard flag + rAF (port from MergeEditor pattern); uses `AlignedLine[]` transformation
5. `FullFileView.svelte` (new) — all hunks rendered contiguously; achieved by requesting `context_lines: 999999` from backend
6. `SyntaxState` (new Tauri managed state) — `SyntaxSet` + theme loaded once at startup, shared across all diff commands
7. `diff.rs` (modified) — accepts `DiffRequestOptions`; runs `syntect` for tokens and `similar` for word spans inside `walk_diff_into_file_diffs()`

### Critical Pitfalls

1. **Whitespace-ignored diff corrupts staging** — When `ignore_whitespace` is active, hunk indices in the display diff do not correspond to the same hunks in the real diff. Staging with wrong indices silently corrupts the staging area. Mitigation: disable hunk and line staging when any whitespace option is active; show tooltip "Staging disabled while whitespace changes are hidden." Never attempt to remap indices.

2. **Syntax highlighting + diff background colors = unreadable visual soup** — Three color layers (line background tint, syntax foreground, word-diff background highlight) conflict unless designed together. Mitigation: syntax tokens provide foreground colors only; word-diff provides semi-transparent backgrounds via CSS custom properties; define `--color-diff-word-add-bg` and `--color-diff-word-delete-bg` before implementing either feature; desaturate syntax colors on add/delete lines by 30-40%.

3. **Split view scroll sync infinite loop or desync** — The naive `onscroll` handler creates feedback loops; differing scroll heights between the two panels causes permanent desync. Mitigation: port the `MergeEditor.svelte` guard flag + `requestAnimationFrame` pattern; use spacer rows (phantom lines) to equalize both panels' total height, making `scrollTop` mirroring trivially correct.

4. **Context line change silently invalidates hunk indices** — Expanding context merges previously separate hunks; any cached selection or keyboard navigation index becomes stale. Mitigation: clear all selection state (`selectedHunkKey`, `selectedLineIndices`, `focusedHunkIndex`) when context lines change. Treat a context re-fetch as equivalent to switching to a new file.

5. **Word-level diff quadratic blowup on long or rewritten lines** — Myers diff at character level is O(N^2) worst case. Long minified lines or completely rewritten lines can freeze the backend for seconds. Mitigation: skip word-level diff for lines over 500 chars; skip when edit distance exceeds 60% of line length; cap at 50 word-diffed line pairs per hunk. Implement thresholds in the initial Rust code, not as a later optimization.

## Implications for Roadmap

Based on research, the build order from ARCHITECTURE.md is the right structure. Dependencies flow bottom-up; the data model must be extended before any UI can consume the new fields. Each phase is backward compatible.

### Phase 1: Backend Data Model and Options

**Rationale:** All subsequent phases depend on the enriched `DiffLine` type and the ability to pass `DiffRequestOptions` to diff commands. This phase has zero frontend risk and establishes the foundation. Adding empty vecs to `DiffLine` is backward compatible — existing UI ignores them.
**Delivers:** Extended `DiffLine` (with `word_spans`, `syntax_tokens` fields), `DiffRequestOptions` accepted by all three diff commands, `context_lines` and `ignore_whitespace` flags wired into `git2::DiffOptions`, Rust and TypeScript types in sync.
**Addresses:** Context lines and whitespace toggle backend foundations (FEATURES.md table stakes).
**Avoids:** Context-line-invalidates-hunk-indices pitfall — verify selection state clears on context change from day one.

### Phase 2: Word-Level Diff

**Rationale:** Depends only on Phase 1's extended `DiffLine`. No new Rust crates beyond `similar`. Frontend rendering is simple (background spans). Verifiable in isolation before syntax highlighting is added.
**Delivers:** `WordSpan` vecs populated by `similar::TextDiff` in `walk_diff_into_file_diffs()`; frontend renders word-diff background highlights via CSS custom properties `--color-diff-word-add-bg` / `--color-diff-word-delete-bg`; line-length and edit-distance thresholds implemented from the start.
**Addresses:** Word-level diff (FEATURES.md table stakes).
**Avoids:** Quadratic blowup pitfall (thresholds required at initial implementation); begins the color layering system that Phase 3 builds on.

### Phase 3: Syntax Highlighting

**Rationale:** Depends on Phase 1's `SyntaxToken` in `DiffLine`. Introduces `syntect` crate and `SyntaxState` managed state. Must coordinate with Phase 2's color system — both phases define CSS custom properties that interact. Technology choice (syntect) is the single most consequential decision in this milestone and must be locked in here.
**Delivers:** `SyntaxToken` vecs computed by `syntect::HighlightLines` in `walk_diff_into_file_diffs()`; `SyntaxState` (SyntaxSet + theme) loaded at startup in Tauri managed state; frontend maps scope names to CSS classes; syntax theme variables in `app.css`; desaturated syntax colors on add/delete lines.
**Addresses:** Syntax highlighting (FEATURES.md highest user-value item).
**Avoids:** Inline color pitfall (scope tokens → CSS classes, never inline styles); DOM explosion pitfall (Rust-side computation, no WASM, no JS bundle bloat); color layering pitfall (designed together with Phase 2).

### Phase 4: UI Refactor and Component Structure

**Rationale:** Refactor before building new view modes. New view mode code should go into the new component structure from the start, not bolted onto the 632-line monolith. This phase delivers no new user-facing features but creates the structural foundation for Phases 5 and 6.
**Delivers:** `DiffLine.svelte` extracted from DiffPanel; `DiffToolbar.svelte` with view mode toggle (Hunk/Full/Split); `DiffViewer.svelte` as dispatcher; `DiffPanel.svelte` reduced to outer shell; existing hunk view works identically after refactor.
**Addresses:** Sets up architecture for Full File View, Split View, and all display toggles.
**Avoids:** Accumulating tech debt from building view modes into the monolith.

### Phase 5: Full File View and Display Options

**Rationale:** Full file view is simply `context_lines: 999999` — no new backend logic. Display options (whitespace toggle, context lines dropdown, word wrap, show invisibles, line numbers) all connect to the `DiffToolbar` and `DiffRequestOptions` already built in Phases 1 and 4. The staging-disabled-on-whitespace-ignore logic belongs here.
**Delivers:** `FullFileView.svelte`; `DiffToolbar.svelte` fully wired (whitespace toggle with staging disabled, context dropdown, word wrap toggle, show invisibles toggle); line numbers rendered in gutter; LazyStore persistence for all toggle state.
**Addresses:** Full file view (FEATURES.md differentiator), whitespace toggle (table stakes), context lines (table stakes), line numbers (table stakes), word wrap (P2), show invisibles (P2).
**Avoids:** Whitespace-corrupts-staging pitfall — staging explicitly disabled when whitespace option is non-default.

### Phase 6: Split View

**Rationale:** The most complex feature. Depends on Phase 4's component structure and Phase 1's `AlignedLine` data. Building last among core features ensures the component architecture is stable before adding the alignment and scroll sync complexity.
**Delivers:** `aligned-lines.ts` transformation (`DiffHunk[]` → `AlignedLine[]` with phantom rows); `SplitView.svelte` with two scroll containers; scroll sync ported from `MergeEditor.svelte` (guard flag + rAF); line click → stage (right panel) / unstage (left panel) mapping; horizontal scroll independent per pane.
**Addresses:** Split view (FEATURES.md highest-priority differentiator alongside syntax highlighting).
**Avoids:** Scroll sync infinite loop (MergeEditor pattern); line number alignment drift (spacer rows equalize heights; fixed line height; word wrap disabled in split mode).

### Phase Ordering Rationale

- **Data model first:** All features consume `WordSpan` and `SyntaxToken` from IPC; types must be defined and tested before any UI builds on them.
- **Rust-heavy phases before UI phases:** Phases 1-3 are pure backend additions with backward-compatible IPC. Frontend risk is deferred until the data is reliable.
- **Refactor before new view modes:** Phase 4 ensures Phases 5 and 6 build into a clean structure rather than extending a monolith.
- **Full file view before split view:** Full file view is simple (context=MAX) and validates the `DiffToolbar` + `DiffViewer` dispatcher pattern before the complex split view is built.
- **Color system designed across Phases 2 and 3:** Word-diff CSS vars (Phase 2) and syntax token CSS vars (Phase 3) must be designed together to prevent the three-layer color conflict pitfall.

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 3 (Syntax Highlighting):** The exact `syntect` API to use (`ClassedHTMLGenerator` vs `HighlightLines`), the scope-to-CSS-class mapping strategy, and the CSS variable naming scheme all need a focused spike. ARCHITECTURE.md is detailed but this is the first time `syntect` is used in this codebase.
- **Phase 6 (Split View):** The `AlignedLine` pairing algorithm for unequal add/delete runs (e.g., 3 deletions paired with 5 additions — which lines get word-diff, which are shown as pure add/delete) needs explicit design before implementation. PITFALLS.md and ARCHITECTURE.md describe the approach but leave pairing heuristics open.

Phases with standard, well-documented patterns (research-phase can be skipped):

- **Phase 1 (Data Model):** All types and git2 APIs are verified and documented. Standard Rust/Tauri patterns throughout.
- **Phase 2 (Word-Level Diff):** The `similar` crate's `iter_inline_changes()` API is purpose-built for this use case. No unknowns.
- **Phase 4 (UI Refactor):** Pure Svelte 5 component extraction — established patterns in the codebase.
- **Phase 5 (Display Options):** All backend options verified via git2 docs. LazyStore persistence is an existing pattern.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All library APIs verified against official docs and crates.io; only disagreement is Shiki vs. syntect — ARCHITECTURE.md position is recommended |
| Features | HIGH | Feature landscape verified against GitKraken, Fork, Sublime Merge, VS Code, GitHub Desktop documentation; competitor analysis is thorough |
| Architecture | HIGH | Data model types, IPC payload estimates, component boundaries, and build order are all specified in detail; MergeEditor precedent in codebase validates key patterns |
| Pitfalls | HIGH | Pitfalls verified against real bugs (GitHub Desktop issue #17776, GitLab issue #450248, VS Code issue #113357) and existing codebase analysis; not speculative |

**Overall confidence:** HIGH

### Gaps to Address

- **Shiki vs. syntect decision:** STACK.md and ARCHITECTURE.md reached opposite conclusions. This must be resolved in Phase 3 planning before any implementation begins. Recommendation is `syntect` (CSS custom properties compliance, no WASM, Tauri managed state pattern). If the team has strong reasons to prefer Shiki, Phase 3 planning should include a spike on the fine-grained bundle approach.
- **AlignedLine pairing heuristic:** How to pair N deletions with M additions when N ≠ M is described qualitatively but not algorithmically. Phase 6 planning must define the exact pairing rule before writing split view code.
- **syntect scope-to-CSS-class mapping:** The set of scope names syntect emits and the CSS custom properties to map them to need to be enumerated during Phase 3 planning. The approach is clear but the mapping table does not yet exist.
- **Full-file view virtualization:** PITFALLS.md flags that full-file view must use virtual scrolling for large files, citing the MergeEditor pattern. ARCHITECTURE.md does not include virtualization in `FullFileView.svelte`. Phase 5 planning should decide whether to include virtual scrolling from the start or add it reactively if performance issues arise on real-world files.

## Sources

### Primary (HIGH confidence)
- [git2 0.19 DiffOptions docs](https://docs.rs/git2/0.19.0/git2/struct.DiffOptions.html) — all whitespace flags and `context_lines()` verified
- [syntect crate docs](https://docs.rs/syntect/latest/syntect/) — `HighlightLines`, `SyntaxSet`, `ClassedHTMLGenerator` APIs
- [similar crate docs](https://docs.rs/similar/latest/similar/) — `TextDiff`, `iter_inline_changes()`, inline feature
- [Shiki best performance guide](https://shiki.style/guide/best-performance) — fine-grained bundling, JS engine, singleton pattern (evaluated; syntect chosen instead)
- [jsdiff v8 release notes](https://github.com/kpdecker/jsdiff/blob/master/release-notes.md) — built-in TypeScript types, O(n^2) fix (evaluated; similar crate chosen instead)
- Existing codebase: DiffPanel.svelte, MergeEditor.svelte, diff.rs, staging.rs — patterns and constraints confirmed

### Secondary (MEDIUM confidence)
- [Zed split diffs blog](https://zed.dev/blog/split-diffs) — spacer row alignment, block map architecture, scroll sync lessons
- [GitHub Desktop syntax highlighting docs](https://github.com/desktop/desktop/blob/development/docs/technical/syntax-highlighting.md) — tokenize both file versions, stitch onto diff lines pattern
- [Code highlighter benchmark (chsm.dev, Jan 2025)](https://chsm.dev/blog/2025/01/08/comparing-web-code-highlighters) — Prism vs. Shiki performance comparison
- [GitButler PR #7915](https://github.com/gitbutlerapp/gitbutler/pull/7915) — CodeMirror for syntax highlighting in Tauri+Svelte (evaluated; rejected for Trunk)

### Tertiary (reference)
- [GitHub Desktop diff scroll jumping issue #17776](https://github.com/desktop/desktop/issues/17776) — real-world sync scroll bugs
- [GitLab split view alignment bug #450248](https://gitlab.com/gitlab-org/gitlab/-/issues/450248) — alignment failure patterns
- [VS Code minimap implementation](https://github.com/microsoft/vscode/blob/main/src/vs/editor/browser/viewParts/minimap/minimap.ts) — Canvas-based minimap reference for future P3 work

---
*Research completed: 2026-03-28*
*Ready for roadmap: yes*
