# Feature Research

**Domain:** Diff viewer enhancement for desktop Git GUI -- view modes, syntax highlighting, word-level diff, whitespace options, context lines, display options
**Researched:** 2026-03-28
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features every professional Git GUI (GitKraken, Fork, Sublime Merge, VS Code, GitHub Desktop) ships. Missing these makes the diff viewer feel incomplete compared to competitors. Trunk already has hunk view with stage/unstage/discard, line-level staging, and keyboard nav -- these features build on that foundation.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Split (side-by-side) view | Every competitor (GitKraken, Fork, Sublime Merge, VS Code, GitHub Desktop) offers this. Developers reviewing large changes expect to see old and new code aligned vertically. Without it, comparing changes requires mental mapping between `+` and `-` lines. | HIGH | Requires transforming unified diff into two parallel columns. Added lines appear on the right with a "phantom" (blank spacer) on the left; deleted lines appear on the left with a phantom on the right. Modified line pairs are aligned horizontally. Synchronized scrolling between panels. Main complexity: alignment algorithm and phantom line insertion. Must coexist with existing hunk stage/unstage/discard actions. |
| Syntax highlighting in diffs | GitKraken, Sublime Merge, GitHub Desktop, and VS Code all syntax-highlight diff content. Without it, reading code changes is significantly harder -- developers rely on color to parse structure. Raw monochrome diffs feel like terminal output, not a GUI. | HIGH | Recommended approach: use Shiki (TextMate grammar engine, same as VS Code) running in the frontend. Tokenize both the old and new file versions separately, then map tokens onto diff lines. Context lines pull tokens from either version; added lines from new; deleted lines from old. Shiki's `codeToTokens` API + `grammarState` enable incremental tokenization. Bundle size concern: Shiki core is 34KB gzipped but grammars load on demand. Language auto-detection from file extension. |
| Word-level (intra-line) diff | GitHub, GitKraken, Sublime Merge ("character diffs"), and VS Code all highlight which words/characters changed within a modified line. Without this, the entire line shows as red/green even if only one variable name changed. | MEDIUM | Run a secondary diff algorithm (Myers) at the word/character level on paired add/delete lines. Render changed tokens with a brighter/saturated background highlight within the already-tinted line. Library options: `diff` (jsdiff) npm package provides `diffWords` and `diffChars` -- well-maintained, 15M weekly downloads. The pairing step identifies which deleted line corresponds to which added line using similarity scoring. |
| Whitespace toggle | Fork, GitHub, and `git diff` all support ignoring whitespace changes. Essential when reviewing reformatted code or indentation changes that obscure real logic changes. | LOW | git2's `DiffOptions` exposes three levels: `ignore_whitespace()` (all), `ignore_whitespace_change()` (amount only, still distinguishes presence vs absence), `ignore_whitespace_eol()` (trailing only). Implement as a toolbar toggle that re-fetches the diff with the option set. The toggle should use `ignore_whitespace_change` by default (matches `git diff -b`), which is the most useful: `test  123` and `test 123` are equal but `test123` and `test 123` still differ. Whitespace-only hunks disappear entirely. |
| Configurable context lines | Git default is 3 lines. Every GUI lets users see more context. GitHub's "unfold" pattern (click to expand) is the standard UX. Sublime Merge adds drag-to-expand on hunk edges. Developers reviewing unfamiliar code need more context. | MEDIUM | git2's `DiffOptions::context_lines()` sets the count. Two UX approaches: (1) global slider/dropdown in toolbar (3/5/10/25/all), (2) per-hunk "expand" buttons to load more context incrementally. Approach (2) requires re-fetching with higher context or fetching the full file and computing visible ranges. Recommended: start with a global dropdown (simplest), add per-hunk expand later. Also leverage `interhunk_lines` to merge nearby hunks when context is large. |
| Line numbers in gutter | VS Code, Sublime Merge, Fork, and GitHub all show line numbers. Essential for cross-referencing with an editor, discussing code in reviews, and orienting within a file. | LOW | Already available in the data: `DiffLine` has `old_lineno` and `new_lineno` fields. Render a gutter column showing old line number (left) and new line number (right). In split view, each side shows its own line numbers. Context lines show both; added lines show only new; deleted lines show only old. Gutter should have a distinct background color (use CSS custom property). |

### Differentiators (Competitive Advantage)

Features that go beyond baseline expectations. Not all competitors have these, and they signal polish.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Full file view (diff-with-full-context) | Sublime Merge's signature feature: toggle between hunk view and full file with diff highlighting. Shows the complete file with changed lines highlighted, making it easy to understand changes in full context. GitKraken has "Inline View" which is similar. Most tools only offer hunk view + split view. | MEDIUM | Implemented by setting `context_lines` to a very large number (e.g., u32::MAX) so the entire file appears as one hunk. Changed lines retain their add/delete background coloring. Works naturally with syntax highlighting since the full file is tokenized. Toggle button in toolbar alongside hunk/split view selector. |
| Scrollbar minimap | GitKraken has a "file mini-map" in diffs. VS Code's minimap is iconic. Shows a compressed overview of the entire diff with colored markers for changes, letting users jump to distant changes quickly. | HIGH | Requires rendering a scaled-down representation of the diff content. Two approaches: (1) Canvas-based minimap rendering (VS Code approach -- fast, flexible), (2) CSS-scaled clone of the content. Canvas is better for performance. The minimap shows red/green strips for deleted/added regions. Clicking jumps to that position. This is a genuine differentiator -- most Git GUIs lack it. |
| Word wrap toggle | GitKraken has word wrap in diffs. Sublime Merge users have requested it. Useful for prose-heavy files (markdown, comments) and minified code. Without it, long lines require horizontal scrolling. | LOW | Toggle `white-space: pre` to `white-space: pre-wrap; word-break: break-all` on diff line elements. In split view, both panels must wrap independently. Line numbers should remain stable (wrapped continuation lines don't get new numbers). The gutter needs to handle variable-height rows. Persist preference via LazyStore. |
| Show invisible characters | VS Code and Sublime Text offer this. Helps debug whitespace issues (tabs vs spaces, trailing whitespace, mixed line endings). Niche but valuable when you need it. | LOW | Render whitespace characters as visible symbols: space as `\u00B7` (middle dot), tab as `\u2192` (right arrow), CRLF as `\u23CE`. Toggle in toolbar. Only render when enabled to avoid visual noise. Apply via CSS or character replacement in rendering. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem appealing but create complexity without proportional value for this project.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Editable diff (edit-in-diff) | VS Code and Zed let you edit the right side of a split diff. Feels natural to fix a typo while reviewing. | Massive complexity: requires bidirectional sync between diff view and working tree, conflict resolution when the file changes externally, undo/redo integration, and invalidation of all hunk boundaries. Trunk is a Git GUI, not an editor. | Double-click a file path to open in the user's configured editor. This is what Fork and Sublime Merge do. |
| Three-way diff view for non-conflict files | Some tools show base/ours/theirs for any diff, not just conflicts. | Trunk already has a full three-panel merge editor for conflicts (v0.8). Adding three-way for regular diffs adds UI complexity for a rarely needed feature. Standard two-way (old vs new) covers 99% of review needs. | The existing merge editor already handles the conflict case. For commit diffs, two-way is sufficient. |
| Image diff (side-by-side/overlay/onion-skin) | Fork and GitHub show image diffs with visual comparison modes. | Requires loading and decoding images, building overlay/slider UI, handling many formats (PNG, JPEG, SVG, WebP). Binary file detection already exists; this is a separate feature domain. | Show image dimensions and file size change. Defer visual image diff to a future milestone. |
| Structural/AST diff (difftastic-style) | Difftastic parses code into ASTs and shows structural changes. Much better diffs for refactors. | Requires language-specific parsers for every language, fundamentally different diff algorithm, and a completely different rendering model. Extremely high complexity for marginal benefit in a general-purpose Git GUI. | Word-level diff within lines covers 80% of the value. Syntax highlighting provides structure awareness. |
| Blame integration in diff | Show git blame annotations alongside diff lines. | Mixes two different views (who changed it vs what changed). Clutters the diff view. Blame is useful but belongs in its own dedicated view. | Build a separate blame view in a future milestone. |

## Feature Dependencies

```
[Syntax Highlighting]
    (standalone, no dependencies on other new features)

[Line Numbers]
    (standalone, no dependencies)

[Word-Level Diff]
    └──enhances──> [Syntax Highlighting]
        (word-level spans must not break syntax token spans;
         highlighting should be applied first, then word-diff
         overlays background colors on top)

[Split View]
    └──requires──> [Line Numbers]
        (each panel needs its own gutter with line numbers)
    └──enhanced-by──> [Syntax Highlighting]
    └──enhanced-by──> [Word-Level Diff]

[Full File View]
    └──enhanced-by──> [Syntax Highlighting]
    └──enhanced-by──> [Line Numbers]
        (full file without line numbers is disorienting)

[Whitespace Toggle]
    (standalone backend change, re-fetches diff with options)
    └──interacts-with──> [Word-Level Diff]
        (when whitespace is hidden, word diff should also ignore whitespace)

[Context Lines]
    └──enables──> [Full File View]
        (full file is context_lines=MAX)
    └──interacts-with──> [Split View]
        (more context = more phantom lines to align)

[Word Wrap]
    └──interacts-with──> [Split View]
        (both panels wrap independently, variable row heights)
    └──interacts-with──> [Line Numbers]
        (wrapped lines share a single line number)

[Minimap]
    └──requires──> [Syntax Highlighting]
        (minimap shows colored code representation)
    └──requires──> one of [Full File View] or large context
        (minimap is pointless for 3-line hunks)

[Show Invisible Characters]
    (standalone display option)
    └──interacts-with──> [Whitespace Toggle]
        (showing invisibles while hiding whitespace changes is contradictory;
         UI should prevent or warn about this combination)
```

### Dependency Notes

- **Word-Level Diff enhances Syntax Highlighting:** Both operate on the same text. Syntax tokens provide foreground colors; word-diff provides background highlights. They compose naturally as layers: syntax highlighting tokens first, then word-diff background overlays on changed segments. If applied in wrong order, syntax spans could be split by word-diff boundaries.
- **Split View requires Line Numbers:** A side-by-side view without line numbers is unusable -- users cannot orient themselves. The gutter is also where phantom/spacer lines indicate alignment gaps.
- **Context Lines enables Full File View:** Full file view is simply context_lines set to maximum. No separate code path needed. The toolbar toggle switches between 3 (default), custom value, and "all" (full file).
- **Minimap requires Syntax Highlighting:** A minimap that only shows red/green change markers provides some value, but a syntax-colored minimap (like VS Code) provides much more. Build minimap after syntax highlighting is working.

## MVP Definition

### Launch With (v0.12 core)

Minimum features to call the milestone complete. Ordered by dependency and impact.

- [x] **Line numbers in gutter** -- Data already exists in DiffLine. Render old_lineno/new_lineno in a fixed-width gutter. Foundation for split view and full file view.
- [x] **Syntax highlighting** -- Use Shiki with codeToTokens. Detect language from file extension. Tokenize old+new file versions, map tokens to diff lines. Single biggest visual improvement.
- [x] **Word-level diff** -- Use jsdiff's `diffWords` on paired add/delete lines. Render as background highlight spans within syntax-highlighted lines.
- [x] **Whitespace toggle** -- Pass `ignore_whitespace_change` to git2 DiffOptions. Toolbar toggle button. Re-fetches diff on toggle.
- [x] **Context lines control** -- Toolbar dropdown: 3/5/10/25/All. Passes value to git2 DiffOptions::context_lines(). "All" = full file view.
- [x] **Split (side-by-side) view** -- Transform unified diff into two-column layout with phantom lines. Synchronized scrolling. View mode toggle in toolbar: Hunk | Split.

### Add After Validation (v0.12 polish)

Features to add once the core six are working and tested.

- [ ] **Word wrap toggle** -- Low complexity but interacts with split view row heights. Add after split view is stable.
- [ ] **Show invisible characters** -- Low complexity, standalone. Nice polish feature.
- [ ] **Per-hunk context expand** -- "Show N more lines" buttons at hunk boundaries. More complex than global context slider but better UX for large files.

### Future Consideration (v0.13+)

Features to defer until after this milestone.

- [ ] **Scrollbar minimap** -- High complexity, requires Canvas rendering. Significant value but not essential for a good diff viewer. Defer to a dedicated milestone or add to a future polish pass.
- [ ] **Image diff** -- Entirely different feature domain. Defer.
- [ ] **Blame integration** -- Separate view, not a diff enhancement.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Syntax highlighting | HIGH | HIGH | P1 |
| Split (side-by-side) view | HIGH | HIGH | P1 |
| Word-level diff | HIGH | MEDIUM | P1 |
| Line numbers | MEDIUM | LOW | P1 |
| Whitespace toggle | MEDIUM | LOW | P1 |
| Context lines control | MEDIUM | MEDIUM | P1 |
| Word wrap toggle | MEDIUM | LOW | P2 |
| Show invisible chars | LOW | LOW | P2 |
| Per-hunk expand | MEDIUM | MEDIUM | P2 |
| Scrollbar minimap | MEDIUM | HIGH | P3 |

**Priority key:**
- P1: Must have for v0.12 launch -- these define "better diffs"
- P2: Should have, add during polish phase if time allows
- P3: Nice to have, defer to future milestone

## Competitor Feature Analysis

| Feature | GitKraken | Fork | Sublime Merge | VS Code | GitHub Desktop | Our Approach |
|---------|-----------|------|---------------|---------|----------------|--------------|
| View modes | Hunk, Inline (full file), Split | Unified, Side-by-side, Full file | Hunk, Full file, Side-by-side | Inline, Side-by-side | Unified, Split | Hunk (existing) + Split + Full file (via context=all) |
| Syntax highlighting | Yes (built-in) | Yes | Yes (TextMate, 40+ langs) | Yes (TextMate) | Yes (CodeMirror modes) | Shiki (TextMate, 200+ langs, same engine as VS Code) |
| Word-level diff | "Word diffing" toggle | Yes (less aggressive) | "Character diffs" | Yes (decorations) | Yes | jsdiff diffWords on paired lines, background highlight |
| Whitespace toggle | Not documented | Yes (ignore whitespace control) | Not documented | Yes (settings) | Not documented | git2 DiffOptions ignore_whitespace_change, toolbar toggle |
| Context lines | Not documented | "Show entire file" option | Drag hunk edges to expand; full file toggle on hunk header | Setting (default 3) | Fixed 3 lines (expand requested) | Global dropdown (3/5/10/25/All), All = full file |
| Line numbers | Yes | Yes | Yes | Yes (gutter) | Yes | Gutter with old_lineno + new_lineno from DiffLine |
| Word wrap | Yes (toggle in toolbar) | Yes (toggle above diff) | Requested, not built-in | Yes (setting) | No | Toggle, persisted via LazyStore |
| Minimap | Yes ("file mini-map") | No | No (requested for scrollbar) | Yes (iconic feature) | No | Defer to P3 / future milestone |
| Show invisibles | Not in diff | Not documented | Via Sublime Text engine | Yes (renderWhitespace) | No | Toggle, render middle dots / arrows |

### Key UX Patterns from Competitors

**Split View Alignment:**
- Paired lines (one deleted, one added) sit on the same row. The left shows the old version, the right shows the new version.
- When one side has more lines (e.g., 3 lines deleted, 5 lines added), the shorter side gets phantom/spacer lines. These are typically rendered with a subtle pattern (Zed uses checkerboard; GitHub uses solid muted color) to distinguish them from blank code lines.
- Context lines appear on both sides identically.
- Synchronized scrolling: both panels scroll together. Some tools (VS Code) allow independent scrolling with a lock/unlock toggle; most keep them locked.

**Full File View:**
- Not a separate "view mode" in most tools -- it's the inline/unified diff with context set to show the entire file. Changed lines retain their green/red background. Unchanged lines have no special styling.
- Sublime Merge's approach is best: a toggle on each hunk header switches between hunk context and full file context. This is exactly `context_lines = u32::MAX`.

**Word-Level Diff Rendering:**
- Changed words/characters get a brighter or more saturated background within the already-tinted line. For example, a green added line with bright green backgrounds on the specific changed words.
- Pairing algorithm: for consecutive runs of deletions followed by additions, pair them 1:1 by position. If counts differ, leftover lines are unpaired (shown as pure add or pure delete without word highlighting).
- Word boundaries: split on whitespace and punctuation. Some tools (delta) use Levenshtein distance for similarity; most use Myers diff at the word level.

**Whitespace Toggle:**
- `git diff -b` semantics (ignore amount of whitespace change) is the standard. Hunks that are whitespace-only disappear entirely.
- Some tools offer multiple levels: ignore trailing only, ignore amount, ignore all. Fork exposes this granularity. Recommended: start with a single toggle using `ignore_whitespace_change`, add granularity later if needed.

**Context Lines:**
- Git default: 3 lines. All tools respect this.
- GitHub web: "unfold" arrows at hunk boundaries to progressively reveal more context (20 lines per click).
- Sublime Merge: drag hunk edges to add/remove context lines. Double-click expands by a few lines.
- Most desktop GUIs: global setting in preferences (3/5/10/25), not per-hunk expand.
- Recommended: global dropdown is simpler and covers the common case. Per-hunk expand is a nice-to-have.

## Sources

- [GitKraken Diff Documentation](https://help.gitkraken.com/gitkraken-desktop/diff/) -- Hunk/Inline/Split views, word wrap, minimap, word diffing, syntax highlighting
- [GitKraken Preferences](https://help.gitkraken.com/gitkraken-desktop/preferences/) -- Editor preferences for syntax highlighting, line count, word wrap
- [Sublime Merge Diff Context](https://www.sublimemerge.com/docs/diff_context) -- Context dragging, full file toggle on hunk header
- [Zed Blog: Split Diffs](https://zed.dev/blog/split-diffs) -- Block map for alignment, spacer/phantom lines with checkerboard, dual multibuffers, synchronized scrolling
- [GitHub Desktop Syntax Highlighting](https://github.com/desktop/desktop/blob/development/docs/technical/syntax-highlighting.md) -- Tokenize both file versions via CodeMirror, stitch tokens onto diff lines, 256KB limit per version, web workers for async tokenization
- [GitHub Blog: Introducing Split Diffs in GitHub Desktop](https://github.blog/2020-11-17-introducing-split-diffs-in-github-desktop/) -- Side-by-side view with syntax highlighting
- [GitHub Blog: Expanding Context in Diffs](https://github.blog/2013-12-02-expanding-context-in-diffs/) -- Unfold button UX pattern for progressive context reveal
- [GitHub Blog: Better Word Highlighting in Diffs](https://github.blog/2014-09-04-better-word-highlighting-in-diffs/) -- Word-level instead of section-level highlighting
- [git2 DiffOptions](https://docs.rs/git2/latest/git2/struct.DiffOptions.html) -- ignore_whitespace, ignore_whitespace_change, ignore_whitespace_eol, context_lines (default 3), interhunk_lines
- [Shiki Syntax Highlighter](https://shiki.style/guide/) -- TextMate grammars, 200+ languages, codeToTokens API, grammarState for incremental tokenization, 34KB gzipped core
- [jsdiff (diff npm package)](https://www.npmjs.com/package/diff) -- diffWords, diffChars, diffWordsWithSpace; Myers O(ND) algorithm; 15M weekly downloads
- [Fork Git Client](https://git-fork.com/releasenotes) -- Side-by-side diff, whitespace ignore, word wrap, full file toggle
- [Baeldung: Git Whitespace Options](https://www.baeldung.com/ops/git-control-ignore-whitespace) -- --ignore-space-change vs --ignore-all-space behavior
- [VS Code Minimap Implementation](https://github.com/microsoft/vscode/blob/main/src/vs/editor/browser/viewParts/minimap/minimap.ts) -- Canvas-based rendering

---
*Feature research for: Diff viewer enhancement (v0.12 Better Diffs)*
*Researched: 2026-03-28*
