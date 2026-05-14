# Phase 63: Full File View & Display Options - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement the full file view (continuous document, no hunk headers), add display option toggles to the diff toolbar (whitespace ignore, show invisibles, word wrap), disable staging when whitespace ignore is active, and render invisible characters (spaces as dots, tabs as arrows). All preferences persist via LazyStore config file. No context lines UI control in toolbar — config file only.

</domain>

<decisions>
## Implementation Decisions

### Full File View Rendering
- **D-01:** Continuous document — remove hunk headers entirely. Show the whole file as one scrollable block with changed lines highlighted (green/red backgrounds). No section dividers.
- **D-02:** No staging in full file view. It's read-only for reviewing. User switches to hunk view to stage. VIEW-05 (staging in all view modes) is Phase 64's scope.
- **D-03:** Two-column gutter (old + new line numbers), same rules as hunk view. Delete lines show old number with blank new column. Add lines show blank old column with new number. Context lines show both. Consistent across all view modes.
- **D-04:** New files (all Add lines) show complete file with add backgrounds and sequential line numbers in full file view.

### Toolbar Display Options
- **D-05:** Three toggle buttons inline after the view mode segmented control, separated by a visual divider: whitespace ignore (WS), show invisibles, word wrap. Layout: `[Hunk|Full|Split] | WS ¶ ↩ | file.ts | Stage ×`
- **D-06:** Active toggle state indicated by highlighted/filled background — same pattern as the view mode segmented control. Inactive = transparent/ghost.
- **D-07:** No context lines dropdown in toolbar. Context lines is a config-file-only setting (LazyStore `trunk-prefs.json`). A settings page will be added in a future milestone to provide UI for all configurables.
- **D-08:** Context lines preset values (for config file and future settings page): 0 / 3 / 5 / 10 / 25. Default: 3.
- **D-09:** All user-configurable preferences stored in config file (`trunk-prefs.json`). Toolbar toggles read/write from this file. Config file is the canonical source of truth. Future settings page reads/writes the same file.
- **D-10:** Context lines dropdown hidden when view mode is "full" (N/A since no dropdown — but the setting itself is irrelevant in full file mode since backend uses show_full_file=true).

### Invisible Characters (WHSP-03)
- **D-11:** Inline substitution: spaces rendered as `·` (middle dot, U+00B7), tabs rendered as `→` (rightwards arrow, U+2192). Symbols shown in a muted color so they don't compete with code content.
- **D-12:** Trailing whitespace at end of lines gets a subtle warning background highlight (faint red/amber). Makes accidental trailing spaces obvious.
- **D-13:** No line ending markers (CR/LF). Only spaces and tabs are shown as invisible characters.

### Whitespace Ignore Staging Guard (WHSP-02)
- **D-14:** When whitespace ignore is active, all staging/unstaging/discard buttons (hunk-level and line-level) are disabled with a tooltip explaining why. GitHub Desktop pattern — never attempt hunk index remapping on whitespace-ignored diffs.

### Word Wrap (DISP-02)
- **D-15:** Lines wrap at the diff viewer container edge. No hanging indent — continuation starts at column 0 (past the gutter). CSS `white-space: pre-wrap`.
- **D-16:** Word wrap is a global toggle — applies to hunk view, full file view, and split view equally. Single config value in LazyStore.

### Claude's Discretion
- CSS implementation technique for invisible character rendering (span replacement vs pseudo-elements)
- Exact muted color for invisible character markers (CSS custom property)
- Warning background color for trailing whitespace (CSS custom property)
- Lucide icon choices for the three toolbar toggle buttons
- Whether invisibles rendering happens in frontend only or also in Rust backend

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Diff components (modify)
- `src/components/diff/FullFileView.svelte` — Stub to implement as continuous document renderer
- `src/components/diff/DiffToolbar.svelte` — Add three toggle buttons (WS, invisibles, word wrap)
- `src/components/diff/HunkView.svelte` — Reference implementation for line rendering, gutter, spans
- `src/components/diff/DiffViewer.svelte` — View mode router, already dispatches to FullFileView
- `src/components/DiffPanel.svelte` — Thin shell owning state; wire new preferences and pass to children

### State persistence
- `src/lib/store.ts` — LazyStore pattern; add showInvisibles and wordWrap preference pairs

### Types
- `src/lib/types.ts` — DiffLine, FileDiff, DiffHunk, MergedSpan, ViewMode, DiffRequestOptions

### Theme / CSS
- `src/app.css` — CSS custom properties; add variables for invisible char color, trailing whitespace warning

### Requirements
- `.planning/REQUIREMENTS.md` — VIEW-04, WHSP-02, WHSP-03, DISP-02 mapped to this phase

### Prior phase context
- `.planning/phases/59-backend-data-model-diff-options/59-CONTEXT.md` — D-01 (whitespace boolean), D-03/D-04 (show_full_file separate from context_lines), D-06/D-07 (global prefs, LazyStore keys)
- `.planning/phases/62-ui-refactor-component-structure/62-CONTEXT.md` — D-03 (FullFileView stub), D-05 (segmented control pattern), D-12 (display options deferred to Phase 63)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `HunkView.svelte` line rendering: gutter columns, origin symbol, merged span loop, syntax classes, word-diff emphasis — FullFileView reuses this pattern
- `DiffToolbar.svelte` segmented control: flexbox layout with active background state — extend with toggle buttons using same pattern
- `LazyStore` get/set/save pattern in `store.ts`: proven async preference persistence — add two new preference pairs
- `DiffLine.old_lineno` / `DiffLine.new_lineno`: already populated by Rust backend
- `show_full_file` backend parameter: already implemented in Phase 59 (passes large context_lines to git2)

### Established Patterns
- Svelte 5 runes (`$state`, `$derived`, `$effect`) for reactive state
- CSS custom properties for all colors — no inline colors
- `$props()` destructuring for component interfaces
- `white-space: pre` on diff lines (will toggle to `pre-wrap` for word wrap)
- `monospace` font at 12px, line-height 1.5 for diff content
- Dynamic gutter width based on max line number digits

### Integration Points
- `DiffPanel.svelte` loads preferences on mount, passes as props to DiffViewer → FullFileView/HunkView
- `DiffToolbar` emits callbacks for toggle changes, DiffPanel handles persistence
- FullFileView consumes the same `fileDiffs` prop as HunkView — iterates all hunks' lines as one flat list
- Whitespace ignore state needs to propagate to hunk/line action buttons to disable them

</code_context>

<specifics>
## Specific Ideas

- Full file view flattens all hunks' lines into a single continuous list — no hunk headers, no separators
- The three toolbar toggles should use Lucide icons for consistency with the rest of the app
- Trailing whitespace warning background should be subtle enough to not overwhelm diff backgrounds (add/delete)
- Context lines values (0/3/5/10/25) stored in config but no toolbar UI — user edits trunk-prefs.json directly until settings page exists
- All configurables in trunk-prefs.json — this becomes the canonical config store that a future settings page will read/write

</specifics>

<deferred>
## Deferred Ideas

- **Settings/preferences page** — Full UI for all configurables (themes, fonts, tab size, context lines, etc.). Future milestone.
- **Per-hunk context expand buttons** — "Show N more lines" within hunk view. Listed in REQUIREMENTS.md as ADVD-02 (future).

</deferred>

---

*Phase: 63-full-file-view-display-options*
*Context gathered: 2026-03-29*
