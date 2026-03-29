# Phase 61: Syntax Highlighting - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Language-aware syntax coloring for all diff lines using syntect. Auto-detects language from file extension. Syntax colors are desaturated on add/delete backgrounds so diff context remains dominant. Populates the existing `syntax_tokens` field on DiffLine (added empty in Phase 59) and merges with word_spans into a unified span array for frontend rendering.

</domain>

<decisions>
## Implementation Decisions

### Syntax Engine
- **D-01:** syntect (Rust, TextMate grammars) — confirmed. 150+ bundled languages, one-shot highlighting, VS Code Dark+ theme support built-in. Tree-sitter considered and rejected (AST-based incremental parsing is overkill for static diff line highlighting).

### Color Palette
- **D-02:** VS Code Dark+ base theme. Blue keywords (#569cd6), orange strings (#ce9178), green comments (#6a9955), light green numbers (#b5cea8), teal types (#4ec9b0), yellow functions (#dcdcaa), light grey operators (#d4d4d4).
- **D-03:** All syntax colors defined as CSS custom properties (e.g., `--color-syn-keyword`, `--color-syn-string`). Never inline styles.

### Desaturation on Diff Backgrounds (SYNT-03)
- **D-04:** Reduce opacity of syntax-colored text on add/delete line backgrounds (e.g., 0.7 alpha). Context lines get full-color syntax. This keeps the green/red line background as the dominant visual signal.

### Scope-to-CSS Mapping
- **D-05:** Fine-grained mapping (15+ CSS classes) for precise syntax coloring close to a real editor experience. Map specific TextMate scopes to `.syn-*` CSS classes.
- **D-06:** Mapping happens in Rust backend. `SyntaxToken.scope` field carries the mapped CSS class name (not the raw TextMate scope string). Smaller IPC payload, simpler frontend rendering.

### Word-Diff + Syntax Layering
- **D-07:** Syntax tokens set text color (via CSS class). Word-diff spans set background color. Both render simultaneously on changed lines — colored keywords with highlight background on changed words.
- **D-08:** Backend merges syntax_tokens and word_spans into a single sorted span array per line. Each span carries both a CSS class (syntax) and an emphasized flag (word-diff). Frontend renders one loop — no overlap resolution in JS.

### Claude's Discretion
- Exact syntect API usage (SyntaxSet loading strategy, ThemeSet selection)
- Performance caching for syntax sets (per-request vs singleton)
- Exact list of 15+ fine-grained scope-to-class mappings
- How to handle the merged span type (new combined type vs extending SyntaxToken)
- Fallback behavior for unrecognized file extensions (no highlighting, plain text)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Rust types and diff pipeline
- `src-tauri/src/git/types.rs` — SyntaxToken struct (start: u32, end: u32, scope: String), DiffLine with syntax_tokens field, WordSpan struct
- `src-tauri/src/commands/diff.rs` — `walk_diff_into_file_diffs` central converter where syntax_tokens must be populated (line 203 currently sets `syntax_tokens: vec![]`)

### Frontend types and rendering
- `src/lib/types.ts` — TypeScript SyntaxToken and WordSpan interface mirrors
- `src/components/DiffPanel.svelte` — Current diff line rendering with word-span highlight rendering (Phase 60)

### Theme and CSS
- `src/app.css` — CSS custom properties for diff colors (lines 16-25), word-diff highlights (lines 24-25)

### Requirements
- `.planning/REQUIREMENTS.md` — SYNT-01 (language-aware coloring), SYNT-02 (auto-detect from extension), SYNT-03 (desaturated on diff backgrounds)

### Prior phase context
- `.planning/phases/59-backend-data-model-diff-options/59-CONTEXT.md` — D-05 (enrichment field structure, byte offset ranges)
- `.planning/phases/60-word-level-diff/60-CONTEXT.md` — D-07/D-08 (word-diff CSS custom properties, background span rendering)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `SyntaxToken { start: u32, end: u32, scope: String }` (types.rs:148): Already defined — scope field will carry CSS class name after mapping
- `WordSpan { start: u32, end: u32, emphasized: bool }` (types.rs:141): Word-diff spans that must be merged with syntax tokens
- `walk_diff_into_file_diffs` (diff.rs): Central function that builds DiffLine structs — syntax token population hooks in here
- `DiffPanel.svelte` word-span rendering: Existing `<span>` rendering loop for word_spans — will be replaced by unified merged span rendering

### Established Patterns
- Byte offset ranges (u32 start/end) for enrichment fields — Phase 59/60 pattern, frontend slices `content` string
- CSS custom properties for all colors — `--color-diff-*` pattern in app.css
- All computation in Rust, frontend only renders
- syntect is NOT yet in Cargo.toml — needs to be added as dependency

### Integration Points
- `walk_diff_into_file_diffs` → after building DiffLine, run syntect highlighting and populate syntax_tokens
- Merge syntax_tokens + word_spans into unified span array before serialization
- `DiffPanel.svelte` line rendering → render merged spans with CSS class + emphasized flag
- `app.css` → add `--color-syn-*` custom properties for all syntax token types
- `Cargo.toml` → add `syntect = "5"` dependency

</code_context>

<specifics>
## Specific Ideas

- VS Code Dark+ colors as the base — most familiar to developers, matches the dark #0d1117 background
- Opacity reduction (0.7) on add/delete lines rather than HSL desaturation — simpler, preserves hue identity
- The merged span approach means `DiffLine` may need a new combined type (or the existing arrays are replaced by a single merged array) — Claude has discretion on the exact type design
- Frontend rendering becomes a single loop over merged spans: apply `.syn-*` CSS class for text color, apply word-diff background class when emphasized=true

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 61-syntax-highlighting*
*Context gathered: 2026-03-28*
