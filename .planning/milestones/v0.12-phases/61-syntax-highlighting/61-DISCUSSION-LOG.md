# Phase 61: Syntax Highlighting - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 61-syntax-highlighting
**Areas discussed:** Color palette, Scope-to-CSS mapping, Word-diff + syntax layering

---

## Color Palette

### Base Theme

| Option | Description | Selected |
|--------|-------------|----------|
| VS Code Dark+ | Most familiar to developers. Blue keywords, green strings, grey comments. Matches dark background. | ✓ |
| GitHub Dark | Matches GitHub's diff viewer aesthetic. Red keywords, light blue strings, grey comments. | |
| Custom / You decide | Claude picks colors optimized for readability on diff backgrounds. | |

**User's choice:** VS Code Dark+
**Notes:** None

### Syntax Engine (user-initiated)

| Option | Description | Selected |
|--------|-------------|----------|
| syntect | TextMate grammars, 150+ bundled languages, one-shot highlighting, VS Code Dark+ support built-in. | ✓ |
| Tree-sitter | AST-based parsing, per-language grammars, heavier setup. Better for editors with incremental editing. | |

**User's choice:** syntect (confirmed after asking about tree-sitter)
**Notes:** User asked about tree-sitter as an alternative. Explained trade-offs — tree-sitter's strength (incremental reparsing) isn't relevant for static diff lines. syntect is purpose-built for this use case.

### Desaturation Approach (SYNT-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Reduce opacity | Lower syntax color opacity on add/delete lines (e.g., 0.7 alpha). Simple, preserves hue. | ✓ |
| Desaturate HSL | Reduce saturation of syntax colors on diff backgrounds. More control but harder to tune. | |
| You decide | Claude picks the approach that looks best. | |

**User's choice:** Reduce opacity
**Notes:** None

---

## Scope-to-CSS Mapping

### Granularity

| Option | Description | Selected |
|--------|-------------|----------|
| Broad (6-8 classes) | Map top-level scope prefixes: keyword, string, comment, number, type, function, operator, punctuation. | |
| Fine-grained (15+) | Map specific scopes: keyword.control vs keyword.other, string.quoted vs string.template, etc. | ✓ |
| You decide | Claude picks based on maintainability and visual quality. | |

**User's choice:** Fine-grained (15+)
**Notes:** User wants precise syntax coloring close to a real editor experience.

### Mapping Location

| Option | Description | Selected |
|--------|-------------|----------|
| Rust backend | Backend maps TextMate scope to short CSS class name before serialization. Smaller IPC payload. | ✓ |
| Frontend mapping | Backend sends raw scope strings. Frontend maps to CSS classes. More flexible but larger payload. | |
| You decide | Claude picks best fit. | |

**User's choice:** Rust backend
**Notes:** None

---

## Word-diff + Syntax Layering

### Composition Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Syntax text color + word-diff background | Both render simultaneously. Colored keywords with highlight background on changed words. | ✓ |
| Word-diff overrides syntax | Suppress syntax color on changed segments. Syntax only on unchanged segments. | |
| You decide | Claude picks based on what VS Code / GitHub do. | |

**User's choice:** Syntax text color + word-diff background (both signals layered)
**Notes:** None

### Span Merging

| Option | Description | Selected |
|--------|-------------|----------|
| Merge in Rust | Backend merges syntax_tokens and word_spans into single sorted span array. One loop in frontend. | ✓ |
| Two-pass in frontend | Backend sends separately. Frontend overlays both, splits spans at intersection boundaries. | |
| You decide | Claude picks. | |

**User's choice:** Merge in Rust
**Notes:** None

---

## Claude's Discretion

- Exact syntect API usage (SyntaxSet loading, ThemeSet selection)
- Performance caching strategy for syntax sets
- Exact list of 15+ fine-grained scope-to-class mappings
- Merged span type design (new combined type vs extending existing)
- Fallback for unrecognized file extensions

## Deferred Ideas

None — discussion stayed within phase scope
