# Architecture Research: Better Diffs (v0.12)

**Domain:** Diff viewer enhancements for desktop Git GUI
**Researched:** 2026-03-28
**Confidence:** HIGH

## System Overview: Current vs. Target

### Current Data Flow

```
StagingPanel (file click)
    |
RepoView.svelte (orchestrator)
    |  invoke("diff_unstaged" | "diff_staged" | "diff_commit")
    v
Rust diff.rs  -->  git2::DiffOptions (defaults)  -->  walk_diff_into_file_diffs()
    |
    v  IPC: Vec<FileDiff>
DiffPanel.svelte (hunk view, line selection, staging actions)
```

### Target Data Flow (v0.12)

```
DiffToolbar.svelte (view mode, whitespace, context, display toggles)
    |  user settings
    v
RepoView.svelte (orchestrator, passes DiffOptions to backend)
    |  invoke("diff_unstaged", { ..., contextLines, ignoreWhitespace })
    v
Rust diff.rs  -->  git2::DiffOptions (whitespace flags, context_lines)
    |                -->  walk_diff_into_file_diffs()
    |                -->  similar::TextDiff (word-level inline diff per changed line pair)
    |                -->  syntect::HighlightLines (syntax tokens per line)
    v  IPC: Vec<FileDiff> (enriched with word spans + syntax tokens)
    |
DiffViewer.svelte (dispatcher)
    |--- HunkView.svelte       (existing, enhanced with line numbers + word highlights)
    |--- FullFileView.svelte   (new: all lines with diff decorations)
    |--- SplitView.svelte      (new: two synced scroll panels)
```

## Component Responsibilities

### New Components

| Component | Responsibility | Why New |
|-----------|----------------|---------|
| `DiffToolbar.svelte` | View mode selector, whitespace toggle, context lines slider, word wrap toggle, line numbers toggle | Centralizes all diff display controls; too much to bolt onto DiffPanel toolbar |
| `DiffViewer.svelte` | Dispatches to HunkView/FullFileView/SplitView based on mode | Replaces current DiffPanel as the diff rendering surface; DiffPanel remains as the outer shell (toolbar + file list + close button) |
| `FullFileView.svelte` | Renders complete file with diff decorations (add/delete/context lines marked) | New view mode; uses high context_lines to get full file from diff |
| `SplitView.svelte` | Two synchronized scroll panels showing old (left) and new (right) | New view mode with synced scroll |
| `DiffLine.svelte` | Single diff line renderer: gutter (line numbers) + syntax-highlighted content + word-level diff spans | Extracts rendering logic from DiffPanel's inline `{#each}` into a reusable component |

### Modified Components

| Component | Change | Why |
|-----------|--------|-----|
| `DiffPanel.svelte` | Becomes a shell: toolbar + `DiffViewer` child; moves line rendering out | Current 632-line monolith mixes toolbar, hunk iteration, line rendering, and staging logic |
| `RepoView.svelte` | Passes diff options (contextLines, ignoreWhitespace) to invoke calls | Backend needs options to generate different diffs |
| `src/lib/types.ts` | Add `DiffOptions`, `WordSpan`, `SyntaxToken` types; extend `DiffLine` | New data needs TS type coverage |
| `src-tauri/src/git/types.rs` | Add `WordSpan`, `SyntaxToken` to DiffLine; add `DiffRequestOptions` | Rust side of the enriched data model |
| `src-tauri/src/commands/diff.rs` | Accept options param; apply whitespace/context flags; run word-diff and syntax highlighting | Core backend changes |
| `src/app.css` | Add `--color-diff-word-add-bg`, `--color-diff-word-delete-bg`, syntax theme variables | New visual treatments need CSS custom properties |

### Unchanged Components

Staging operations (`stage_hunk`, `unstage_hunk`, `stage_lines`, etc.) remain unchanged in their Rust implementation. They operate on git2's own diff with default options regardless of display options -- this is critical because staging must always use the real diff, not a whitespace-ignored or context-adjusted view.

## Architectural Decisions

### Decision 1: Syntax Highlighting in Rust (syntect)

**Recommendation:** Use `syntect` in the Rust backend, not Shiki/Prism in the frontend.

**Rationale:**

| Factor | syntect (Rust) | Shiki (TypeScript) |
|--------|----------------|-------------------|
| IPC payload | Tokens serialized with line data -- one round-trip | Raw lines sent, then JS processes -- same round-trip but adds ~200KB WASM load |
| Performance | ~16,000 lines/sec, runs on Rust thread pool | Comparable speed but blocks JS main thread or needs Web Worker |
| Language coverage | Sublime Text syntax definitions (~200 languages) | TextMate grammars (~200 languages, same VS Code set) |
| Bundle impact | Compiled into Rust binary, zero frontend cost | ~200KB+ per language/theme loaded, adds to webview memory |
| Theme integration | Returns `(Style, &str)` tuples -- convert to CSS class tokens | Returns inline-styled HTML -- conflicts with CSS custom properties rule |
| Caching | SyntaxSet is Send+Sync, can be cached in Tauri managed state | Per-highlighter instance in JS, no cross-tab sharing |
| Incremental | HighlightLines supports line-by-line with state caching | Full re-highlight on change |

**Critical factor:** The project rule "never inline colors -- always use CSS custom properties" makes Shiki's inline-style output a poor fit. Syntect can output scope-based tokens that map to CSS classes, keeping color control in `app.css`.

**Implementation:** syntect returns `Vec<SyntaxToken>` per line where each token has `(start_byte, end_byte, scope_name)`. Frontend maps scope names to CSS classes. A single `SyntaxSet` + `ThemeSet` lives in Tauri managed state, loaded once at startup (~50ms), shared across all tabs.

**Confidence:** HIGH -- syntect is the standard choice for Rust syntax highlighting. Used by bat, delta, xi-editor, helix.

### Decision 2: Word-Level Diffing in Rust (similar crate)

**Recommendation:** Use `similar` crate with the `inline` feature in Rust, not a JavaScript diff library.

**Rationale:**

The word-level diff runs per changed line pair (adjacent delete+add). For a typical file diff with 50 changed lines, that is 25 word-diff operations. Running this in Rust avoids:
1. Sending raw line text to JS, diffing there, then mapping back to line indices for staging
2. Re-running word diff on every Svelte reactivity cycle
3. Duplicating diff logic across Rust (for staging) and JS (for display)

`similar` crate with `features = ["inline", "unicode"]`:
- `TextDiff::from_words(old_line, new_line)` produces word-level ops
- `iter_inline_changes()` yields `InlineChange` items with `(emphasized: bool, value: &str)` spans
- Hardcoded 500ms deadline prevents pathological cases from blocking

**IPC payload impact:** Each `DiffLine` gains a `Vec<WordSpan>` field where `WordSpan { start: u32, end: u32, kind: "equal" | "insert" | "delete" }`. For a typical line of 80 chars with 3 word changes, this adds ~5 spans x 12 bytes = 60 bytes per changed line. Negligible compared to the line content itself.

**Confidence:** HIGH -- `similar` is the most-used Rust diffing library, authored by mitsuhiko (Flask/Sentry creator). The `inline` feature is specifically designed for this use case.

### Decision 3: Whitespace and Context Lines via git2 DiffOptions (Native)

**Recommendation:** Pass whitespace and context line settings directly to `git2::DiffOptions`. No custom implementation needed.

**git2::DiffOptions natively supports:**

| Method | Maps to git flag | Purpose |
|--------|-----------------|---------|
| `ignore_whitespace(true)` | `-w` | Ignore all whitespace |
| `ignore_whitespace_change(true)` | `-b` | Ignore whitespace amount changes |
| `ignore_whitespace_eol(true)` | `--ignore-space-at-eol` | Ignore trailing whitespace |
| `context_lines(n)` | `-U<n>` | Number of context lines (default 3) |

**Implementation:** Add a `DiffRequestOptions` struct passed from frontend:

```rust
#[derive(Debug, Deserialize)]
pub struct DiffRequestOptions {
    pub context_lines: Option<u32>,          // default 3
    pub ignore_whitespace: Option<String>,   // "none" | "all" | "change" | "eol"
}
```

Apply these to `git2::DiffOptions` before generating the diff. The existing `walk_diff_into_file_diffs` function needs zero changes -- it already processes whatever diff git2 produces.

**Critical subtlety:** Staging operations MUST NOT use display options. The user sees a whitespace-ignored diff but stages against the real diff. This means:
- `diff_unstaged`/`diff_staged`/`diff_commit` accept `DiffRequestOptions` for display
- `stage_hunk`/`unstage_hunk`/`stage_lines`/etc. continue using default `DiffOptions`
- When whitespace ignore is active, hunk indices from the display diff will NOT match the real diff. **Hunk staging must be disabled or re-mapped when whitespace ignore is on.**

**Confidence:** HIGH -- verified from git2 0.19 docs.rs documentation.

### Decision 4: Split View Architecture -- Two Synced Scroll Panels

**Recommendation:** Two separate scroll containers with synchronized scroll positions via `scrollTop` event listeners.

**Why not a single merged render:**
- A merged table with left/right columns forces both sides to share row heights, which breaks when lines wrap differently
- Line selection for staging maps to one side only (you stage from the "new" side or unstage from the "old" side) -- separate panels make click targets unambiguous
- Delete-only and add-only lines create gaps on the opposite side. With two panels, gaps are filled with empty placeholder rows. With a merged render, you need complex rowspan logic.

**Implementation pattern:**

```svelte
<!-- SplitView.svelte -->
<div class="split-container">
  <div class="split-left" bind:this={leftPanel} onscroll={syncScroll}>
    {#each alignedLines as line}
      <!-- old line or empty placeholder -->
    {/each}
  </div>
  <div class="split-right" bind:this={rightPanel} onscroll={syncScroll}>
    {#each alignedLines as line}
      <!-- new line or empty placeholder -->
    {/each}
  </div>
</div>
```

The key data structure is an `AlignedLine[]` array computed from the diff hunks:
- Context line: both sides show the same content
- Delete line: left shows content, right shows empty placeholder
- Add line: left shows empty placeholder, right shows content
- Replaced line pair: left shows old, right shows new (word-diff highlights both)

Scroll sync: on `scroll` event from either panel, set the other panel's `scrollTop` to match. Use a `syncing` flag to prevent infinite loops.

**Confidence:** HIGH -- this is the standard approach used by GitHub, GitKraken, and VS Code's diff editor.

### Decision 5: Full-File View via Expanded Context

**Recommendation:** Request the diff with `context_lines` set to `u32::MAX` (or a large number like 999999), not by fetching and merging separate file content.

**Why:** git2 with `context_lines(999999)` returns the entire file content as context lines around the diff hunks. This means:
- The existing `DiffHunk` data model works unchanged
- Line numbers are already computed by git2
- No separate "get file content" command needed
- Diff decorations (add/delete/context) are already classified

The full-file view renderer is then simply the hunk view but with all hunks rendered contiguously (no hunk separators) and context lines styled normally instead of dimmed.

**Confidence:** HIGH -- this is how `git diff -U999999` works and git2 supports it natively.

### Decision 6: Staging in Non-Default View Modes

**Hunk view (default):** Staging works exactly as today. No changes.

**Full-file view:** Staging still works because hunks are preserved in the data model. The UI just renders them contiguously. Hunk action buttons appear in the gutter or as floating controls when hovering a hunk boundary. Line selection works identically -- the `hunkIndex` and `lineIndex` are still available.

**Split view:** Staging maps to the right panel (new side) for stage operations and left panel (old side) for unstage operations. When the user selects lines in the right panel, `stage_lines` is called. When selecting in the left panel, `unstage_lines` is called. Hunk boundaries are marked in both panels.

**Whitespace-ignored mode:** Staging MUST be disabled. The hunk indices from a whitespace-ignored diff do not correspond to the real diff hunks. Display a tooltip: "Staging disabled while whitespace changes are hidden." This is the same approach GitHub uses.

**Confidence:** HIGH for hunk/full-file/split. MEDIUM for whitespace-ignored staging disable (could alternatively re-map indices, but the complexity is not worth it for v0.12).

## Data Model Changes

### Rust Types (src-tauri/src/git/types.rs)

```rust
// NEW: Request options from frontend
#[derive(Debug, Deserialize, Default)]
pub struct DiffRequestOptions {
    pub context_lines: Option<u32>,
    pub ignore_whitespace: Option<String>,  // "none" | "all" | "change" | "eol"
}

// NEW: Word-level diff span within a line
#[derive(Debug, Serialize, Clone)]
pub struct WordSpan {
    pub start: u32,        // byte offset in line content
    pub end: u32,          // byte offset end (exclusive)
    pub kind: WordSpanKind,
}

#[derive(Debug, Serialize, Clone)]
pub enum WordSpanKind {
    Equal,
    Insert,
    Delete,
}

// NEW: Syntax highlighting token
#[derive(Debug, Serialize, Clone)]
pub struct SyntaxToken {
    pub start: u32,        // byte offset
    pub end: u32,          // byte offset end (exclusive)
    pub scope: String,     // e.g., "keyword.control", "string.quoted"
}

// MODIFIED: DiffLine gains optional enrichment fields
pub struct DiffLine {
    pub origin: DiffOrigin,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub word_spans: Vec<WordSpan>,         // NEW: empty for context lines
    pub syntax_tokens: Vec<SyntaxToken>,   // NEW: empty if highlighting disabled/failed
}
```

### TypeScript Types (src/lib/types.ts)

```typescript
// NEW
export interface DiffRequestOptions {
    contextLines?: number;
    ignoreWhitespace?: "none" | "all" | "change" | "eol";
}

export type WordSpanKind = "Equal" | "Insert" | "Delete";

export interface WordSpan {
    start: number;
    end: number;
    kind: WordSpanKind;
}

export interface SyntaxToken {
    start: number;
    end: number;
    scope: string;
}

// MODIFIED: DiffLine gains enrichment
export interface DiffLine {
    origin: DiffOrigin;
    content: string;
    old_lineno: number | null;
    new_lineno: number | null;
    word_spans: WordSpan[];        // NEW
    syntax_tokens: SyntaxToken[];  // NEW
}
```

### IPC Payload Size Analysis

For a typical 500-line diff with 100 changed lines:
- **Current:** ~50KB (line content + hunk headers)
- **With word spans:** ~56KB (+6KB for ~500 spans on changed lines)
- **With syntax tokens:** ~80KB (+24KB for ~2000 tokens across all lines)
- **Total:** ~80KB -- well within IPC comfort zone (Tauri handles megabytes without issue)

For pathological case (10,000-line diff, entire file changed):
- **With all enrichment:** ~1.5MB -- still fine for IPC, but should consider lazy loading syntax tokens (only for visible lines) if this becomes a real bottleneck.

## Patterns to Follow

### Pattern 1: Display Options as Separate Concern from Staging

**What:** Diff display options (whitespace, context, view mode) affect only the *display* diff, never the staging diff.

**Why:** Staging hunk/line indices must correspond to the actual diff. If the user hides whitespace changes, hunk 3 in the display diff might be hunk 5 in the real diff.

**Implementation:**
```typescript
// RepoView.svelte
const displayOptions: DiffRequestOptions = $state({
    contextLines: 3,
    ignoreWhitespace: "none",
});

// Display diff uses options
const displayDiff = await safeInvoke<FileDiff[]>("diff_unstaged", {
    path: repoPath,
    filePath: path,
    options: displayOptions,
});

// Staging always uses defaults (no options param)
await safeInvoke("stage_hunk", { path: repoPath, filePath, hunkIndex });
```

### Pattern 2: SyntaxSet as Managed Tauri State

**What:** Load syntect's `SyntaxSet` and theme once, store in Tauri managed state, share across all commands.

**Why:** Loading syntax definitions takes ~50ms. Doing this per-diff-request would add noticeable latency.

```rust
pub struct SyntaxState {
    pub syntax_set: SyntaxSet,
    pub theme: Theme,
}

// In main.rs setup:
let ss = SyntaxSet::load_defaults_newlines();
let ts = ThemeSet::load_defaults();
let theme = ts.themes["base16-ocean.dark"].clone();
app.manage(SyntaxState { syntax_set: ss, theme });
```

### Pattern 3: Aligned Line Array for Split View

**What:** Transform `DiffHunk[]` into a flat `AlignedLine[]` where each entry has an old-side and new-side, either of which may be null (placeholder).

**Why:** Split view needs both panels to have the same number of rows for scroll sync.

```typescript
interface AlignedLine {
    left: DiffLine | null;    // old content (or null for add-only lines)
    right: DiffLine | null;   // new content (or null for delete-only lines)
    type: "context" | "add" | "delete" | "modify";
    hunkIndex: number;        // for staging operations
    lineIndex: number;        // within the hunk, for line staging
}
```

This transformation happens purely in TypeScript from the existing `FileDiff` data. No backend changes needed.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Frontend Word Diffing

**What people do:** Send raw lines to frontend, run a JS diff library (diff-match-patch, jsdiff) per line pair.
**Why it's wrong:** Duplicates diff logic. Must re-run on every Svelte reactivity cycle. Adds ~50KB to frontend bundle.
**Do this instead:** Compute word spans in Rust alongside the line diff. Send once via IPC. Frontend only renders.

### Anti-Pattern 2: Separate "Get File Content" Command for Full-File View

**What people do:** Fetch full file content separately, then overlay diff decorations by matching line numbers.
**Why it's wrong:** Requires a new backend command, complex line-number alignment between file content and diff output, and breaks for files that don't exist on one side (added/deleted files).
**Do this instead:** Use `context_lines(999999)` to get the full file as diff output. All lines already have origin markers and line numbers.

### Anti-Pattern 3: Inline Color Styles for Syntax Highlighting

**What people do:** Use Shiki or syntect's HTML output which produces `<span style="color: #ff0000">`.
**Why it's wrong:** Violates project rule "never inline colors." Makes theme switching impossible without re-highlighting.
**Do this instead:** Map syntect scopes to CSS classes (`scope-keyword`, `scope-string`, etc.) defined in `app.css` with CSS custom properties.

### Anti-Pattern 4: Re-running Display Diff for Staging Operations

**What people do:** After user changes display options, use the display diff's hunk indices for staging.
**Why it's wrong:** Whitespace ignore changes hunk boundaries. Context line count changes hunk count (hunks may merge or split). Staging with wrong indices corrupts the index.
**Do this instead:** Always stage against the default diff. Disable staging when whitespace ignore is active.

## Integration Points

### Backend Changes

| File | Change | Dependency |
|------|--------|------------|
| `Cargo.toml` | Add `syntect = "5"`, `similar = { version = "2", features = ["inline", "unicode"] }` | None |
| `src-tauri/src/state.rs` | Add `SyntaxState` to managed state | syntect dependency |
| `src-tauri/src/git/types.rs` | Add `WordSpan`, `SyntaxToken`, `DiffRequestOptions`; extend `DiffLine` | None |
| `src-tauri/src/commands/diff.rs` | Accept `DiffRequestOptions`; apply to `git2::DiffOptions`; run word-diff + syntax highlighting in `walk_diff_into_file_diffs` | syntect + similar deps, types changes |
| `src-tauri/src/main.rs` | Register `SyntaxState` in managed state | state.rs changes |

### Frontend Changes

| File | Change | Dependency |
|------|--------|------------|
| `src/lib/types.ts` | Add `DiffRequestOptions`, `WordSpan`, `SyntaxToken`; extend `DiffLine` | None |
| `src/app.css` | Add `--color-diff-word-add-bg`, `--color-diff-word-delete-bg`, syntax scope color variables | None |
| `src/components/DiffPanel.svelte` | Extract line rendering; add toolbar controls; dispatch to view components | New components |
| `src/components/DiffToolbar.svelte` (new) | View mode selector, all toggle controls | types.ts |
| `src/components/DiffViewer.svelte` (new) | View mode dispatcher | DiffToolbar, HunkView, FullFileView, SplitView |
| `src/components/FullFileView.svelte` (new) | Full file rendering with diff decorations | DiffLine.svelte |
| `src/components/SplitView.svelte` (new) | Two-panel synced scroll split view | DiffLine.svelte, aligned-lines.ts |
| `src/lib/aligned-lines.ts` (new) | Transform FileDiff hunks into AlignedLine[] for split view | types.ts |
| `src/components/DiffLine.svelte` (new) | Line renderer with gutter, syntax, word-diff | types.ts, CSS |
| `src/components/RepoView.svelte` | Pass DiffRequestOptions to diff invoke calls; manage display state | types.ts |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Display options state | `$state` rune in RepoView, passed as props to DiffToolbar and diff invoke | Consider a `diff-options.svelte.ts` shared state module if other components need it |
| DiffPanel <-> View modes | Props: `fileDiffs`, `viewMode`, `displayOptions` | DiffPanel remains outer shell, inner view is swapped |
| Staging <-> Display diff | Completely separate paths | Display options NEVER affect staging commands |
| SyntaxState <-> diff commands | Tauri managed state `State<'_, SyntaxState>` | Loaded once in main.rs, shared across commands |

## Suggested Build Order

Dependencies flow bottom-up. Build in this order:

### Phase 1: Backend Options + Data Model Extension
1. Add `similar` and `syntect` to `Cargo.toml`
2. Add new types to `types.rs` (both Rust and TypeScript)
3. Extend `DiffLine` with empty `word_spans` and `syntax_tokens` vecs (backward compatible -- existing UI ignores them)
4. Add `DiffRequestOptions` parameter to diff commands
5. Wire `context_lines` and `ignore_whitespace` flags into `git2::DiffOptions`
6. **Tests:** Verify whitespace flags and context lines work via existing GOOS test harness

### Phase 2: Word-Level Diff
1. Implement word-diff in `walk_diff_into_file_diffs`: for adjacent delete+add line pairs, run `similar::TextDiff::from_words` and populate `word_spans`
2. **Frontend:** Render `word_spans` as inline `<span>` with word-diff background colors
3. Add CSS variables `--color-diff-word-add-bg`, `--color-diff-word-delete-bg`
4. **Tests:** Rust tests for word span generation; frontend component tests for rendering

### Phase 3: Syntax Highlighting
1. Add `SyntaxState` managed state with `SyntaxSet::load_defaults_newlines()`
2. In diff commands, detect language from file extension via `SyntaxSet::find_syntax_for_file`
3. Run `HighlightLines::highlight_line` per line, convert to `SyntaxToken` vec
4. **Frontend:** Apply syntax tokens as CSS classes on `<span>` elements within each line
5. Add syntax theme CSS variables to `app.css`
6. **Tests:** Rust tests for token generation; visual verification

### Phase 4: UI Refactor + View Modes
1. Extract `DiffLine.svelte` from DiffPanel's inline rendering
2. Create `DiffToolbar.svelte` with view mode toggle (hunk/full/split)
3. Create `DiffViewer.svelte` as dispatcher
4. Refactor DiffPanel to use DiffViewer
5. **Tests:** Verify existing hunk view still works after refactor

### Phase 5: Full-File View
1. Implement `FullFileView.svelte` -- renders all hunks contiguously with context lines styled normally
2. When user switches to full-file mode, re-request diff with `context_lines: 999999`
3. Support hunk/line staging in full-file mode (hunk boundaries are still in the data)
4. **Tests:** Component tests for full-file rendering

### Phase 6: Split View
1. Implement `aligned-lines.ts` transformation
2. Implement `SplitView.svelte` with two scroll containers
3. Add scroll sync logic
4. Map line clicks in left panel to unstage, right panel to stage
5. **Tests:** Unit tests for aligned-lines; component tests for split view

### Phase 7: Display Options
1. Whitespace toggle in toolbar (triggers re-fetch with whitespace flag)
2. Context lines control (dropdown or slider, triggers re-fetch)
3. Show invisible characters toggle (CSS `white-space: pre` + visible whitespace rendering)
4. Word wrap toggle (CSS `white-space: pre-wrap` vs `pre`)
5. Line numbers in gutter (already available from `old_lineno`/`new_lineno`)
6. Disable staging controls when whitespace ignore is active
7. **Tests:** Verify staging disabled when whitespace ignored; verify context line changes

### Phase 8: Scrollbar Minimap (Stretch)
1. Canvas-based minimap rendering alongside scrollbar
2. Color-coded change regions (green add, red delete)
3. Click-to-navigate
4. **Tests:** Visual verification only -- minimap is cosmetic

**Rationale for ordering:**
- Phases 1-3 are backend-heavy with minimal UI risk. They extend the data model in backward-compatible ways.
- Phase 4 (refactor) happens before new view modes so the new code goes into the new component structure from the start.
- Phases 5-6 depend on Phase 4's component structure.
- Phase 7 (display options) depends on Phase 1's backend options and Phase 4's toolbar component.
- Phase 8 (minimap) is independent and lowest priority.

## Sources

- [git2::DiffOptions docs](https://docs.rs/git2/latest/git2/struct.DiffOptions.html) -- whitespace flags and context_lines confirmed (HIGH confidence)
- [syntect crate](https://crates.io/crates/syntect) -- HighlightLines API returns `Vec<(Style, &str)>` (HIGH confidence)
- [syntect HighlightLines docs](https://docs.rs/syntect/latest/syntect/easy/struct.HighlightLines.html) -- line-by-line API (HIGH confidence)
- [similar crate](https://github.com/mitsuhiko/similar) -- inline feature for word-level diffs (HIGH confidence)
- [similar TextDiff docs](https://docs.rs/similar/latest/similar/struct.TextDiff.html) -- iter_inline_changes method (HIGH confidence)
- [Zed split diffs blog post](https://zed.dev/blog/split-diffs) -- two-panel synced scroll approach (MEDIUM confidence, Zed's architecture differs)

---
*Architecture research for: v0.12 Better Diffs*
*Researched: 2026-03-28*
