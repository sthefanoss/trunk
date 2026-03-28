# Pitfalls Research

**Domain:** Adding diff viewer improvements (split view, syntax highlighting, word-level diff, whitespace options, context lines, display options) to an existing Tauri 2 + Svelte 5 + Rust desktop Git GUI
**Researched:** 2026-03-28
**Confidence:** HIGH (verified against existing codebase analysis, libgit2/git2 API docs, Shiki documentation, Zed's split diff post-mortem, GitHub Desktop issues, and established editor patterns)

---

## Context: What Is Changing in v0.12

The existing DiffPanel.svelte renders a unified hunk-based diff with line-level selection and hunk staging. It uses a flat `<div>` per line with inline styles for `+`/`-`/context coloring via CSS custom properties. No syntax highlighting, no split view, no word-level highlighting. The Rust backend produces `Vec<FileDiff>` via git2's `diff_index_to_workdir` / `diff_tree_to_index` / `diff_tree_to_tree` with default 3-line context.

v0.12 adds:
1. **View modes:** hunk view (current), full file view, split (side-by-side)
2. **Syntax highlighting** with auto language detection
3. **Word-level (intra-line) diff** highlighting
4. **Whitespace toggle** (ignore whitespace changes) + show invisible characters
5. **Configurable context lines** with incremental expand
6. **Display options:** word wrap toggle, line numbers in gutter, scrollbar minimap

Each feature has pitfalls on its own. The interactions between them create compounding failure modes.

---

## Critical Pitfalls

### Pitfall 1: Syntax highlighting DOM explosion freezes the webview on large files

**What goes wrong:**
Traditional syntax highlighters (Shiki, Prism, Highlight.js) wrap every token -- keywords, strings, operators, punctuation -- in individual `<span>` elements. A 2,000-line file with syntax highlighting can produce 30,000-50,000 DOM nodes just for the highlighted tokens. The current DiffPanel renders one `<div>` per diff line (cheap). Adding syntax highlighting naively multiplies DOM node count by 10-20x. In Tauri's WKWebView (macOS) or WebKitGTK (Linux), this causes visible jank: layout reflow takes 100ms+, scrolling stutters, and the UI feels frozen for 1-2 seconds when switching files.

The full-file view mode makes this worse because it renders the entire file, not just changed hunks with 3 lines of context.

**Why it happens:**
Developers prototype syntax highlighting on a 50-line test file where performance is invisible. The DOM node count scales linearly with file size but rendering cost scales super-linearly (each reflow touches more nodes, each paint covers more area). The problem only manifests with real-world files: 500+ line source files, minified bundles, generated code.

Shiki's full bundle is 6.4 MB minified (1.2 MB gzip). Even the web bundle is 3.8 MB (695 KB gzip). Loading this at startup adds perceptible delay to tab opening.

**How to avoid:**
1. **Do syntax highlighting in Rust, not JS.** Use `syntect` (the same engine as Sublime Text / bat) in the Rust backend. It is fast (single-threaded, no WASM overhead), produces inline style annotations that map directly to CSS, and adds zero frontend bundle size. Compute highlighting during diff generation so IPC sends pre-highlighted lines.
2. **If doing it in JS:** Use Shiki with fine-grained imports (`shiki/core` + only needed languages) and the JavaScript engine (not Oniguruma/WASM). Create one singleton `Highlighter` instance. Lazy-load grammars per file extension. Use `codeToTokens` (not `codeToHtml`) to get token arrays, then render with your own Svelte template to avoid Shiki generating its own DOM.
3. **Cap highlighting:** Skip syntax highlighting for files over a threshold (e.g., 10,000 lines or 500 KB). Show a "File too large for syntax highlighting" notice. Every professional tool does this.
4. **Virtualize the full-file view.** The current hunk view renders all lines in DOM (acceptable because hunks are small). Full-file view MUST use virtual scrolling -- render only visible lines plus a buffer. The MergeEditor already has a virtualization pattern (`getVisibleRange` + `OVERSCAN`) that should be reused.

**Warning signs:**
- Prototype works on small files but stutters on real projects
- `document.querySelectorAll('span')` returns 20,000+ nodes in the diff panel
- File switching takes more than 200ms perceived delay
- Bundle size grows by more than 200 KB gzip after adding syntax highlighting

**Phase to address:**
Syntax highlighting phase -- MUST decide Rust-side (`syntect`) vs. JS-side (Shiki) in the first plan. The architecture choice cascades through every subsequent phase. If choosing Shiki, fine-grained bundle configuration must be the first implementation task, not an optimization afterthought.

---

### Pitfall 2: Split view sync scroll infinite loop or visible desync

**What goes wrong:**
Split (side-by-side) diff has two scrollable panes. When the user scrolls the left pane, the right pane must follow, and vice versa. The naive implementation -- `leftPane.onscroll = () => rightPane.scrollTop = leftPane.scrollTop` and the mirror -- creates an infinite feedback loop: setting `scrollTop` on the right pane fires its `onscroll`, which sets `scrollTop` on the left pane, which fires its `onscroll`, and so on. Even with a guard flag (`let scrolling = false`), timing issues cause visible oscillation where panes bounce back and forth by 1-2 pixels.

A subtler variant: the two sides have different heights because deleted lines appear only on the left and added lines only on the right. Naive `scrollTop` mirroring causes the shorter side to hit its scroll limit before the longer side, and then they desync permanently for the rest of the scroll range.

**Why it happens:**
The browser's `onscroll` event fires asynchronously after the scroll position is set programmatically. The guard flag (`scrolling = true` before setting, reset after) can be cleared before the programmatic scroll event fires, depending on browser event loop timing. Additionally, `scrollTop` assignment is clamped by the browser to the element's actual scroll range, so if the two sides have different `scrollHeight` values, setting `scrollTop = X` on the shorter side actually sets it to `min(X, maxScroll)`, which does not equal the source scroll position.

**How to avoid:**
1. **Use the MergeEditor pattern already in the codebase.** The existing `MergeEditor.svelte` has a working synchronized scroll implementation: a `scrolling` guard flag combined with `requestAnimationFrame` to defer the guard reset. This pattern works because rAF fires after the browser has processed the programmatic scroll event. Port this pattern directly.
2. **Insert spacer rows for alignment.** When the left side has a deleted block of N lines, insert N blank spacer rows on the right side (and vice versa for additions). This ensures both sides have identical total height, making `scrollTop` mirroring trivially correct. Zed uses this approach ("empty visual regions rendered with a subtle checkerboard"). The MergeEditor's `flattenAligned()` function already implements exactly this pattern -- padding conflict regions to equal height.
3. **Never use percentage-based scroll sync.** Some implementations try to sync scroll position as a percentage (`right.scrollTop = right.scrollHeight * (left.scrollTop / left.scrollHeight)`). This breaks because the correspondence between left and right lines is not uniform -- a large deletion block on the left has no corresponding content on the right, so the percentage mapping creates visual jumps.

**Warning signs:**
- Panes visibly oscillate (jitter) by 1-2 pixels when scrolling slowly
- Scrolling to the bottom of one pane does not reach the bottom of the other
- Fast scrolling causes the two sides to drift apart by several lines
- A `console.warn` or counter on `onscroll` shows 10+ events per frame

**Phase to address:**
Split view phase. The spacer/alignment strategy and scroll sync must be designed together as a single unit, not separately. Reuse the MergeEditor's `flattenAligned()` + scroll sync pattern.

---

### Pitfall 3: Whitespace-ignored diff breaks hunk staging line numbers

**What goes wrong:**
The user toggles "ignore whitespace" to review a diff without noise. They see a clean 3-line hunk showing only the meaningful change. They click "Stage Hunk." The backend receives the hunk index, but the hunk index and line numbers were computed from a whitespace-ignored diff, while `stage_hunk` applies a patch against the actual working tree content (which includes the whitespace changes). The patch fails to apply, or worse, applies to the wrong lines, silently corrupting the staging area.

This is the single most dangerous pitfall in this milestone because it causes data loss (wrong lines staged) rather than just a UI bug.

**Why it happens:**
The current backend computes diffs and applies hunks using the same `DiffOptions`. When you add `opts.ignore_whitespace(true)`, the diff output changes: hunks merge differently, line numbers shift, whitespace-only lines that were `+`/`-` become context lines. But `git2::apply` uses the actual file content, not the whitespace-ignored view. The hunk index from the whitespace-ignored diff does not correspond to the same hunk in the non-ignored diff.

git2's `DiffOptions` exposes `ignore_whitespace()`, `ignore_whitespace_change()`, and `ignore_whitespace_eol()`, but these affect diff generation only. The apply machinery uses the literal patch text, which must match the actual file content.

**How to avoid:**
1. **Whitespace ignore is display-only.** Generate TWO diffs from the backend: one with the user's whitespace settings (for display) and one with default settings (for staging operations). The frontend shows the display diff but sends the staging command with the original hunk index from the non-ignored diff.
2. **Alternatively, disable hunk/line staging when whitespace ignore is active.** Show a tooltip: "Hunk staging is not available when whitespace changes are hidden. Turn off whitespace ignore to stage individual hunks." This is what GitHub Desktop does -- it grays out the staging checkboxes when whitespace settings are non-default.
3. **Never pass whitespace-ignored hunk indices to staging commands.** If you must allow staging with whitespace ignore, you need a mapping layer: for each hunk in the display diff, find the corresponding hunk(s) in the real diff by matching line content. This is complex and error-prone; option 1 or 2 is safer.

**Warning signs:**
- `stage_hunk` error "patch does not apply" after toggling whitespace ignore
- Lines appear in the wrong position after staging with whitespace ignore on
- Staging succeeds but the staged content differs from what was shown in the diff
- No test covers staging with whitespace ignore enabled

**Phase to address:**
Whitespace options phase. The very first design decision must be: does whitespace ignore coexist with staging, or is staging disabled when whitespace ignore is active? This must be decided before writing any whitespace-related code.

---

### Pitfall 4: Syntax highlighting and diff background colors create unreadable visual soup

**What goes wrong:**
Diff lines have background colors: green tint for additions, red tint for deletions, transparent for context. Syntax highlighting adds foreground colors: blue for keywords, green for strings, orange for numbers, etc. When a syntax-highlighted string literal (green foreground) appears on an added line (green background), the text becomes invisible. When a keyword (blue foreground) appears on a deletion line (red background), the contrast ratio drops below WCAG thresholds. Word-level diff highlighting adds a THIRD layer: a brighter background on the specific changed words within a line. Three overlapping color systems create visual chaos.

**Why it happens:**
Each color system is designed independently: diff colors assume plain text, syntax highlighting assumes neutral background, word-level highlighting assumes it is the only background. Nobody tests the three together across all combinations of syntax token types and diff line types.

**How to avoid:**
1. **Design a layered color system upfront.** Define CSS custom properties for all combinations: `--color-diff-add-token-keyword`, `--color-diff-delete-token-string`, etc. This is impractical for every combination. Instead:
2. **Use opacity-based layering.** Diff background is the base layer (set on the line `<div>`). Syntax colors are foreground-only (no background from syntax tokens). Word-level diff is an additional semi-transparent background on `<span>` elements within the line. This means:
   - Line background: `var(--color-diff-add-bg)` or `var(--color-diff-delete-bg)` or `transparent`
   - Token foreground: syntax color (adjusted for readability on both add/delete backgrounds)
   - Word-level highlight: `var(--color-diff-word-add)` or `var(--color-diff-word-delete)` -- a slightly more saturated version of the line background
3. **Desaturate syntax colors on diff lines.** On added/deleted lines, reduce syntax color saturation by 30-40% to prevent clashing with the background tint. GitHub does this: syntax colors on diff lines are muted compared to the same tokens in a non-diff code view.
4. **Test the 6 critical combinations:** (add/delete/context) x (keyword/string) at minimum. Print them out and check contrast ratios.

**Warning signs:**
- Text that is hard to read on colored backgrounds
- Two adjacent colors that look identical (green string on green add background)
- Word-level diff highlights that are invisible against the line background
- Users reporting "I can't see what changed"

**Phase to address:**
Syntax highlighting phase AND word-level diff phase must coordinate. Define the color layering system as a shared design document before implementing either feature. Add the CSS custom properties for all diff+syntax combinations in the first phase that touches colors.

---

### Pitfall 5: Line number alignment breaks in split view with unequal sides

**What goes wrong:**
In split view, the left pane shows old line numbers and the right pane shows new line numbers. Deleted lines appear only on the left (with a blank spacer on the right). Added lines appear only on the right (with a blank spacer on the left). If spacers are not precisely the same height as real lines, the two sides drift out of alignment progressively -- by the bottom of a 500-line diff, the left and right panes can be off by 20+ pixels, making it impossible to visually compare corresponding lines.

A related failure: line numbers in the gutter must track the actual file line numbers, not the visual row index. The left gutter shows the old file's line numbers (skipping numbers for added lines that do not exist in the old file). The right gutter shows the new file's line numbers (skipping numbers for deleted lines). Getting this wrong produces nonsensical line numbers that confuse users.

**Why it happens:**
Spacer rows must have exactly `line-height` height (currently 1.5 * 12px = 18px in the DiffPanel). If a spacer is an empty `<div>` without explicit height, the browser may collapse it to 0px. If a real line wraps (because word wrap is enabled), it becomes taller than 18px, but the spacer on the opposite side stays at 18px. This is the fundamental conflict between word wrap and split view alignment -- and the reason most diff tools disable word wrap in split view mode.

**How to avoid:**
1. **Fixed line height, no word wrap in split view.** Use `overflow-x: auto` (horizontal scroll) on each line in split view. This guarantees every row is exactly one line height. Word wrap is only available in unified view. This is what VS Code, GitHub Desktop, and GitKraken do.
2. **If allowing word wrap in split view:** Each row must be a flex container whose height is determined by the taller side. When the left side wraps to 3 visual lines, the right side's spacer (or non-wrapping line) must also expand to 3 visual lines. This requires measuring each row's rendered height and setting `min-height` on the opposite side -- expensive and complex.
3. **Line numbers:** Track `old_lineno` and `new_lineno` from the `DiffLine` struct (already provided by git2). For spacer rows, show no line number (empty gutter). Never compute line numbers from the visual row index.
4. **Use a CSS grid for the split layout.** Each row is a grid row containing [left-gutter | left-content | right-gutter | right-content]. The grid ensures all four cells in a row have the same height automatically. Do NOT use two independent scrollable containers with separate row heights.

**Warning signs:**
- Lines at the bottom of a long diff are visibly misaligned between left and right
- Line numbers show sequential integers instead of actual file line numbers
- Blank spacer rows appear with 0 height (collapsed)
- Word wrap enabled in split view causes the two sides to drift

**Phase to address:**
Split view phase. The layout strategy (grid vs. two containers) and the word-wrap-in-split-view decision must be made in the first plan. These are architectural choices that cannot be retrofitted.

---

### Pitfall 6: Context line expansion silently invalidates cached hunk indices

**What goes wrong:**
The user expands context lines from 3 to 10 (or clicks "Show 20 more lines"). The diff is re-fetched from the backend with the new `context_lines` parameter. The hunks merge differently: two previously separate hunks may now overlap and become one hunk because the expanded context fills the gap between them. The frontend's hunk indices are now stale. If the user had selected lines in hunk 2, the selection now refers to what used to be hunk 2 but is now part of the merged hunk 0. Clicking "Stage Hunk" stages the wrong content.

**Why it happens:**
git2's `DiffOptions::context_lines()` changes how hunks are split. With `context_lines(3)` (default), two changes separated by 7 unchanged lines produce two hunks. With `context_lines(10)`, they merge into one hunk because the context overlaps. `DiffOptions::interhunk_lines()` also affects merging. The frontend caches hunk indices for selection state, keyboard navigation (`[`/`]`), and staging commands. When the backend returns a different hunk structure, all cached indices are invalid.

**How to avoid:**
1. **Clear all selection and navigation state on context change.** When `context_lines` changes, clear `selectedHunkKey`, `selectedLineIndices`, `focusedHunkIndex`, and `hunkElements`. The existing `$effect` that clears state when `fileDiffs` changes should cover this, but verify it fires on context-line-triggered re-fetches.
2. **Increment context on the frontend, not by re-fetching.** Instead of calling the backend with a different `context_lines` parameter, fetch the full file content once and reconstruct the expanded view on the frontend by inserting additional context lines from the full file into the existing hunk structure. This preserves hunk identity. The "expand context" button adds lines from the surrounding file content, not from a re-generated diff.
3. **If re-fetching from backend:** The re-fetch is a complete replacement. Treat it as if the user selected a different file. Reset all UI state. Do not try to preserve selection across hunk re-generation.

**Warning signs:**
- "Stage Hunk" error after expanding context lines
- Hunk navigation (`[`/`]`) jumps to unexpected positions after context change
- Selected lines visually shift position after context expansion
- Tests pass with default context (3) but fail with expanded context

**Phase to address:**
Context lines phase. The approach (frontend expansion vs. backend re-fetch) must be decided in the first plan. Frontend expansion is more complex but preserves user state; backend re-fetch is simpler but requires full state reset.

---

### Pitfall 7: Word-level diff algorithm quadratic blowup on large changed lines

**What goes wrong:**
Word-level (intra-line) diff compares a deleted line against the subsequent added line to find which specific words or characters changed. The standard approach is a character-level or word-level Myers diff. Myers diff is O(ND) where N is the total characters and D is the edit distance. For two similar lines of 200 characters with 5 changed characters, this is fast (O(200*5) = O(1000)). But for a completely rewritten line (common with reformatted code, moved function arguments, or minified files), D approaches N and the algorithm becomes O(N^2). A single pair of 10,000-character minified lines takes seconds to diff.

When many lines are completely rewritten (e.g., a file was reformatted), word-level diff runs on every line pair and the total time becomes unacceptable.

**Why it happens:**
Developers test word-level diff on typical source code lines (50-120 characters) where it is instant. The quadratic case only triggers on pathological inputs: minified JavaScript, base64-encoded data, long JSON single-line values, or lines where most content changed.

**How to avoid:**
1. **Set line-length and edit-distance thresholds.** If a line exceeds 500 characters, skip word-level diff and show the whole line as changed. If the edit distance exceeds 60% of the line length (i.e., most of the line changed), show the whole line as changed rather than highlighting 60% of it in word-level markers.
2. **Do word-level diff in Rust, not JS.** The `similar` crate provides high-quality word/character diff with good performance characteristics. Running in Rust via `spawn_blocking` prevents blocking the UI thread. The result can be pre-computed alongside the line diff and included in the `DiffLine` struct as optional `inline_changes: Option<Vec<InlineSpan>>`.
3. **Diff words, not characters.** Split each line by whitespace/punctuation into tokens, then diff the token sequences. Token-level diff is O(T*D_t) where T is token count (much smaller than character count). Fall back to character-level only when two adjacent tokens differ and you want sub-token granularity.
4. **Limit the number of word-diffed line pairs per hunk.** If a hunk has 200 changed lines, do not word-diff all 200 pairs. Word-diff the first 50 pairs and mark the rest as "fully changed." Users scrolling through 200 changed lines are not reading word-by-word anyway.

**Warning signs:**
- Diff rendering takes more than 500ms on files with long lines
- The UI freezes when viewing minified files or base64 data
- Word-level highlighting marks 80%+ of a line, providing no useful information
- CPU spikes when switching to a file with heavy reformatting

**Phase to address:**
Word-level diff phase. Threshold configuration must be part of the initial implementation, not added after performance problems are discovered.

---

### Pitfall 8: Bundle size explosion from syntax highlighting grammars

**What goes wrong:**
Shiki's full bundle is 6.4 MB minified (1.2 MB gzip). Even importing the "web" bundle adds 3.8 MB (695 KB gzip). Each TextMate grammar is 50-200 KB. A Git GUI needs to highlight many languages (users open repos containing JS, TS, Python, Rust, Go, C++, Java, Ruby, YAML, JSON, Markdown, CSS, HTML, SQL, Shell, Dockerfile, etc.). Loading 30+ grammars adds 3-6 MB to the bundle.

For a Tauri desktop app, the bundle is embedded in the binary. The current Trunk app is likely under 5 MB. Adding syntax highlighting could double the binary size.

**Why it happens:**
Developers start with a few grammars ("I only need JS and Rust"), then add more as they encounter files without highlighting. Each grammar is small individually, but they accumulate. The WASM Oniguruma engine adds another 500 KB+.

**How to avoid:**
1. **Use `syntect` in Rust (strongest recommendation).** `syntect` is compiled into the Rust binary. Grammar files are embedded at build time. The `syntect` default bundle (all common languages) adds approximately 2 MB to the binary -- less than Shiki's web bundle -- and requires zero JavaScript bundle impact. It also avoids the WASM overhead.
2. **If using Shiki:** Import from `shiki/core` (not the main entry). Use the JavaScript engine (not Oniguruma/WASM). Import only the 10-15 most common languages statically. Lazy-load additional grammars on demand when a file with an uncommon extension is opened. Use `import()` with the specific grammar path: `import('@shikijs/langs/rust')`.
3. **Never import `shiki` or `shiki/bundle/full` or `shiki/bundle/web`.** These include all grammars/themes upfront. Use fine-grained imports exclusively.
4. **Measure after every grammar addition.** Add a CI check or build-time log that reports total bundle size. Set a budget (e.g., 2 MB gzip for syntax highlighting contribution).

**Warning signs:**
- Build output size increases by more than 1 MB after adding syntax highlighting
- `import 'shiki'` appears anywhere in the codebase (should be `shiki/core`)
- More than 20 grammar imports in a single file
- WASM files appear in the build output

**Phase to address:**
Syntax highlighting phase -- the first task. Technology choice (syntect vs. Shiki) determines bundle size strategy for the entire milestone.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Inline styles on diff lines (current pattern) | Quick to implement, dynamic per-line | Cannot use CSS pseudo-elements, harder to theme, verbose markup | Current state is acceptable for v0.12, but syntax highlighting + word-level diff should migrate to CSS classes |
| Rendering all lines in DOM (no virtualization) for full-file view | Simple implementation | UI freezes on files over 1,000 lines | Never for full-file view. The MergeEditor virtualizes; full-file must too. |
| Single diff fetch (no separate display-diff / staging-diff) | Simpler backend, one IPC call | Cannot safely stage hunks when whitespace ignore is active | Acceptable if whitespace ignore disables staging. Not acceptable if staging must work with whitespace ignore. |
| Hardcoded 3-line context (current behavior) | No UI for context configuration needed | Users cannot see more context without opening the file separately | Acceptable for v0.12 initial phases, but context configuration should be added before milestone close. |
| Word-level diff computed on the frontend in JS | No backend changes needed | Blocks the main thread on large diffs, cannot be cached | Acceptable for prototype. Must move to Rust backend for production. |
| Loading all syntax grammars at startup | Simple initialization, no lazy-loading code | 1-3 second startup delay, wasted memory for unused grammars | Never. Lazy-load grammars by file extension. |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| git2 `DiffOptions` + whitespace ignore | Passing whitespace-ignored hunk indices to `stage_hunk` / `stage_lines` | Generate display diff (with ignore) and staging diff (without ignore) separately. Or disable staging when whitespace ignore is on. |
| git2 `context_lines` + hunk staging | Assuming hunk indices are stable across context-line changes | Clear all selection state when context lines change. Hunk indices from different context settings are not comparable. |
| Syntax highlighting + diff line rendering | Using Shiki's `codeToHtml` which generates its own DOM structure | Use `codeToTokens` to get token arrays. Render tokens inside existing diff line structure to preserve click handlers, selection, and line-level styling. |
| Split view + existing hunk keyboard navigation | `[`/`]` navigation assumes a single scrollable container | Update keyboard navigation to work within the active pane of a split view. Track which pane has focus. |
| Word wrap + line numbers | Line numbers rendered per visual line instead of per source line | Line numbers must be per source line only (one number per actual line, not per wrapped segment). Use CSS `position: sticky` or a fixed-width gutter column. |
| Minimap + virtual scrolling | Minimap tries to render all lines for the overview but diff panel virtualizes | Minimap must render from the data model (line count + change types), not from the DOM. Use a canvas element, not DOM nodes. |
| `syntect` (Rust) + Tauri IPC | Sending fully highlighted HTML over IPC (large payload) | Send token spans with style indices, not HTML strings. Frontend maps indices to CSS classes. Reduces IPC payload by 60-80%. |
| Full-file view + hunk actions | Full-file view loses hunk boundaries, so "Stage Hunk" has no target | In full-file view, show hunk boundaries as visual separators. Map each line back to its original hunk index for staging. Or disable hunk staging in full-file view and only allow line staging. |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Syntax highlighting all lines on every diff re-render | 200ms+ delay when switching files, visible flash of unstyled text | Cache highlighted tokens per file path + content hash. Only re-highlight when file content changes. | At 500+ line files |
| Word-level diff on every changed line without thresholds | UI freezes on files with many changes or long lines | Skip lines over 500 chars. Skip if edit distance exceeds 60% of line length. Cap at 50 word-diffed line pairs per hunk. | At files with 100+ changed lines or any line over 1,000 chars |
| Split view rendering both sides' full DOM | Double the DOM nodes, double the layout/paint cost | Virtual-scroll both panes from a single shared scroll position. Render only visible rows. | At 500+ line diffs |
| Canvas minimap re-rendering on every scroll event | Visible jank during fast scrolling, high CPU usage | Render minimap once on diff load. Only update the viewport indicator overlay on scroll. Re-render full minimap only when diff data changes. | Immediately on any non-trivial file |
| Re-generating diff on every context-line increment | Multiple sequential IPC round-trips as user clicks "show more context" | Debounce context expansion requests (300ms). Or fetch the full file once and expand context on the frontend. | When user clicks expand 3+ times quickly |
| Shiki highlighter created per file | Highlighter instantiation is expensive (100ms+); creating per file adds up | Create one singleton highlighter. Load grammars lazily. Reuse across all files and tabs. | At 3+ files opened in sequence |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Word-level diff highlights 90% of a reformatted line | User sees almost the entire line highlighted, providing no useful information about what actually changed | If word-level diff covers more than 60% of a line, fall back to showing the entire line as changed (no word-level markers). Show word-level only when it narrows down to a small portion. |
| Invisible whitespace characters shown by default | Visual noise overwhelms the actual diff content; users cannot focus on meaningful changes | Whitespace visualization must be off by default. Toggle it on explicitly. Even when on, only show trailing whitespace and mixed tabs/spaces, not every single space. |
| Context expansion has no visual boundary | After expanding context, users lose track of where the original hunk boundaries were | Keep a subtle visual marker (thin line, background change) at the original hunk boundary positions even after context expansion. |
| No keyboard shortcut to toggle between view modes | Users must reach for the mouse to switch between unified/split/full-file view | Assign keyboard shortcuts: Cmd+Shift+1/2/3 or similar. Show in tooltip on the toggle buttons. |
| Split view makes each side too narrow on smaller screens | At 1280px width, each split pane is ~500px after gutters -- barely enough for 80-char lines | Set a minimum window width for split view (e.g., 1400px). Below that, show a notice suggesting unified view. Or allow independent horizontal scrolling per pane. |
| Word wrap toggle has no visual indicator of current state | User does not know whether word wrap is on or off without looking carefully at line behavior | Use a toggle button with pressed/unpressed state and a tooltip showing the current mode. |

## "Looks Done But Isn't" Checklist

- [ ] **Syntax highlighting:** Handles files with no recognized language (shows plain text, no crash, no missing colors)
- [ ] **Syntax highlighting:** Works with binary detection -- binary files still show "Binary file -- no diff available" (not garbled highlighted bytes)
- [ ] **Split view:** Spacer rows are exactly the same height as content rows across ALL zoom levels (Cmd+/Cmd-)
- [ ] **Split view:** Horizontal scroll works independently per pane (scrolling right in left pane does not scroll right pane)
- [ ] **Split view:** The three-way state (unified/split/full-file) persists across file switches within the same session
- [ ] **Word-level diff:** Handles no-newline-at-EOF marker lines (should not word-diff the "\ No newline at end of file" git marker)
- [ ] **Word-level diff:** Adjacent add/delete line pairing handles cases with unequal counts (3 deletions, 5 additions -- which pairs to word-diff?)
- [ ] **Whitespace toggle:** Does not break the diff when the file contains only whitespace changes (all hunks disappear, UI shows "No changes" not an empty broken panel)
- [ ] **Context lines:** Expanding to maximum context shows the full file content without gaps or duplicated lines
- [ ] **Context lines:** Hunk staging still works after context expansion (indices are valid)
- [ ] **Line numbers:** Gutter numbers match `git diff` output exactly for a given file (verify manually)
- [ ] **Line numbers:** New file (all additions) shows line numbers starting at 1, not 0
- [ ] **Minimap:** Renders correctly for files with 1-2 changed lines (not just large diffs)
- [ ] **Minimap:** Click-to-scroll on minimap navigates to the correct position in the diff
- [ ] **View mode toggle:** Switching from split to unified and back preserves scroll position (approximately)
- [ ] **Full-file view:** Does not re-fetch the entire file content when already available from the diff data

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| DOM explosion from syntax highlighting | LOW | Add virtualization to the affected view. If using Shiki, switch to `codeToTokens` + custom rendering. If architectural, switch to `syntect` in Rust. |
| Sync scroll infinite loop | LOW | Copy the MergeEditor's `scrolling` + `requestAnimationFrame` pattern. Takes 30 minutes to fix. |
| Whitespace ignore corrupts staging | HIGH | Audit all staged changes for correctness. If data was silently corrupted, users may have committed wrong content. Add a dual-diff architecture (display vs. staging). |
| Syntax + diff color conflict | MEDIUM | Define a CSS custom property layer. Requires touching all diff line rendering but is purely visual (no data changes). |
| Split view line misalignment | MEDIUM | Retrofit spacer row insertion. Requires refactoring split view layout from two containers to a single grid. |
| Context expansion breaks staging | MEDIUM | Add state-clearing logic on context change. If using frontend expansion, must build a line-to-hunk mapping. |
| Word-level diff quadratic blowup | LOW | Add threshold checks (line length, edit distance). Takes 1 hour to add guards. |
| Bundle size explosion | MEDIUM | If caught early: switch to fine-grained imports. If architectural (committed to Shiki full bundle): migration to `syntect` requires backend work. |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Syntax highlighting DOM explosion | Syntax highlighting (first plan: technology choice) | Open a 2,000-line file; rendering completes in under 200ms. DOM node count under 5,000. |
| Split view sync scroll bugs | Split view (reuse MergeEditor pattern) | Scroll both panes rapidly; no jitter, no desync, panes stay aligned to bottom. |
| Whitespace ignore breaks staging | Whitespace options (first decision: staging compatibility) | Stage a hunk with whitespace ignore on; verify staged content matches display. Or verify staging is disabled with whitespace ignore. |
| Syntax + diff color conflict | Syntax highlighting + word-level diff (shared color design) | Screenshot all 6 combinations (add/delete/context x keyword/string). All text readable, contrast ratio above 4.5:1. |
| Split view line number alignment | Split view (layout architecture decision) | Scroll to bottom of a 500-line diff; left and right panes are aligned within 1px. Line numbers match `git diff` output. |
| Context expansion invalidates hunks | Context lines (approach decision: frontend vs. backend) | Expand context from 3 to 10; verify hunk staging still works or verify selection is cleared. |
| Word-level diff quadratic blowup | Word-level diff (threshold implementation) | Open a minified 10,000-char file; word-level diff completes or falls back within 100ms. |
| Bundle size explosion | Syntax highlighting (first task: technology + bundle strategy) | Build output size increase is under 2 MB after syntax highlighting is added. |

## Sources

- [Shiki Best Performance Practices](https://shiki.style/guide/best-performance) -- singleton pattern, fine-grained bundles, worker offloading, JS vs Oniguruma engine
- [Shiki Bundle Guide](https://shiki.style/guide/bundles) -- full bundle 6.4 MB, web bundle 3.8 MB, fine-grained import patterns
- [Split Diffs in Zed](https://zed.dev/blog/split-diffs) -- spacer approach for alignment, block map architecture, cascading misalignment risk, performance optimization discoveries
- [GitHub Desktop Split Diffs announcement](https://github.blog/news-insights/product-news/introducing-split-diffs-in-github-desktop/) -- split view implementation in a desktop Git GUI
- [GitHub Desktop diff scroll jumping issue #17776](https://github.com/desktop/desktop/issues/17776) -- real-world sync scroll bugs in production
- [GitLab split view alignment bug #450248](https://gitlab.com/gitlab-org/gitlab/-/issues/450248) -- side-by-side alignment failures
- [VS Code manual alignment issue #113357](https://github.com/microsoft/vscode/issues/113357) -- split diff alignment challenges
- [libgit2 DiffOptions v0.19](https://libgit2.org/docs/reference/v0.19.0/diff/git_diff_option_t.html) -- whitespace ignore flags, context line configuration
- [git2 crate DiffOptions](https://docs.rs/git2/latest/git2/struct.DiffOptions.html) -- `context_lines()`, `interhunk_lines()`, `ignore_whitespace()` methods
- [Git diff-options documentation](https://git-scm.com/docs/diff-options) -- context expansion, inter-hunk lines, whitespace options
- [GitHub Blog: Expanding context in diffs](https://github.com/blog/1705-expanding-context-in-diffs) -- UI pattern for context expansion
- [CSS Highlights API for syntax highlighting](https://pavi2410.com/blog/high-performance-syntax-highlighting-with-css-highlights-api/) -- DOM-free highlighting alternative
- [Shiki diff syntax + CSS variables issue #697](https://github.com/shikijs/shiki/issues/697) -- known conflict between diff highlighting and CSS variable themes
- [Tauri webview freezing issue #13498](https://github.com/tauri-apps/tauri/issues/13498) -- WKWebView freezes under heavy DOM load
- [codediff.nvim](https://github.com/esmuellert/codediff.nvim) -- VSCode-style two-tier highlighting (line + character level) implementation reference
- Existing codebase analysis: DiffPanel.svelte (line rendering, selection, staging), MergeEditor.svelte (sync scroll pattern, spacer alignment, virtualization), diff.rs (git2 diff generation), staging.rs (hunk/line staging via git2 apply)

---
*Pitfalls research for: Trunk v0.12 Better Diffs -- split view, syntax highlighting, word-level diff, whitespace options, context lines, display options*
*Researched: 2026-03-28*
