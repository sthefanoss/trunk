# Phase 59: Backend Data Model & Diff Options - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 59-backend-data-model-diff-options
**Areas discussed:** Whitespace granularity, Enrichment field shape, Preference scope, Context "All" semantic

---

## Whitespace Granularity

| Option | Description | Selected |
|--------|-------------|----------|
| Single toggle | One on/off toggle using ignore_whitespace_change. What GitHub Desktop and VS Code do. | ✓ |
| Three-level dropdown | Off / Ignore Changes / Ignore All Whitespace. More control but more UI complexity. | |
| You decide | Claude picks based on codebase and downstream phases | |

**User's choice:** Single toggle (Recommended)
**Notes:** Straightforward pick, aligns with established tools.

---

## Enrichment Field Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Byte offset ranges | word_spans: Vec<{start, end, kind}>. syntax_tokens: Vec<{start, end, scope}>. Frontend slices by range. | |
| Pre-segmented strings | Vec<{text, class}>. Backend pre-splits content. Simpler frontend. | |
| You decide | Claude picks structure that works with similar + syntect output formats | ✓ |

**User's choice:** You decide
**Notes:** Claude has discretion to pick the structure that aligns best with similar crate's iter_inline_changes() and syntect's token output.

---

## Preference Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Global | All tabs share the same diff settings. Matches VS Code and GitHub Desktop. | ✓ |
| Per-tab | Each tab has its own diff settings. More state to manage. | |
| You decide | Claude picks based on tab architecture | |

**User's choice:** Global (Recommended)
**Notes:** Simpler UX, uses existing LazyStore pattern directly.

---

## Context "All" Semantic

| Option | Description | Selected |
|--------|-------------|----------|
| Large sentinel value | Pass context_lines = u32::MAX to git2. No special backend logic. | |
| Separate boolean flag | Add show_full_file: bool alongside context_lines. | |
| You decide | Claude picks simplest approach | |

**User's choice:** Other (free text)
**Notes:** User specified: No "All" option in context lines. Context lines should be bounded (0-10). Separate `show_full_file: bool` toggle that is independent — context_lines only applies when show_full_file is off. When show_full_file is on, backend passes a large value to git2 internally.

---

## Claude's Discretion

- Enrichment field structure (word_spans, syntax_tokens) — Claude picks based on similar/syntect output formats

## Deferred Ideas

None — discussion stayed within phase scope
