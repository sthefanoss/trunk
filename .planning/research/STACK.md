# Stack Research: Better Diffs

**Domain:** Git GUI diff viewer enhancements (syntax highlighting, word-level diff, whitespace options, view modes)
**Researched:** 2026-03-28
**Confidence:** HIGH (all libraries verified via official docs and npm/crates.io)

## Existing Stack (DO NOT RE-ADD)

Already validated: Tauri 2, Svelte 5, Vite 6, TypeScript 5.6, Tailwind CSS 4, Rust (git2 0.19, notify 7, tokio 1), Vitest, vendored VirtualList, LazyStore for UI persistence.

---

## Recommended Stack Additions

### 1. Syntax Highlighting: Shiki (Frontend, Fine-Grained Bundle)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `shiki` | `^4.0.2` | Syntax highlighting for diff lines | VS Code-quality TextMate grammar output; fine-grained bundling eliminates WASM dependency; lazy-loads only needed languages |

**Why Shiki over alternatives:**

- **vs Prism.js (11.7 KB gzip):** Prism is smaller and faster (1400-2000 ops/s vs 200-280 for Shiki), but Shiki produces VS Code-quality highlighting that matches what developers expect from a professional Git GUI. Prism's grammar accuracy is noticeably worse for complex languages (TypeScript generics, JSX, Rust lifetimes). A diff viewer highlights one file at a time, so 3-5ms per highlight call is invisible to the user.
- **vs highlight.js (15.6 KB gzip):** Similar quality to Prism but larger. Auto-detection is useful but unnecessary since we already know the file extension from `FileDiff.path`. No advantage over Shiki.
- **vs CodeMirror (GitButler's approach):** CodeMirror is a full editor framework. GitButler uses it because their diff viewer supports inline editing. Trunk's diff viewer is read-only (except the merge editor, which already has its own approach). Importing `@codemirror/lang-*` packages purely for read-only highlighting is significant overkill.
- **vs syntect on Rust side (v5.3.0, 13M downloads):** Could generate HTML via `ClassedHTMLGenerator` on the Rust side, avoiding frontend JS entirely. However: (1) it would serialize pre-rendered HTML strings over IPC for every diff line, bloating payloads; (2) theme changes would require re-invoking Rust commands; (3) the frontend already owns rendering and can cache highlighted tokens across re-renders. Keeping highlighting in the frontend is the right separation of concerns for a Tauri app.
- **vs tree-sitter (Rust crate):** Immature highlight query ecosystem; inconsistent grammar quality across languages (Rust grammar worse than C for variable identification); painful upgrade path when adding grammars. Better suited for editors needing incremental AST parsing, not read-only highlighting.

**Fine-grained bundle strategy (no WASM):**

The key to making Shiki lightweight is using the JavaScript regex engine instead of the default Oniguruma WASM engine. This eliminates the 231 KB WASM blob entirely.

```typescript
import { createHighlighterCore } from 'shiki/core';
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript';

// Import only needed languages (each ~2-17 KB gzip)
import langTypescript from '@shikijs/langs/typescript';
import langRust from '@shikijs/langs/rust';
// ... lazy-load others on demand based on file extension

// Import only one theme (~3 KB gzip)
import themeDarkPlus from '@shikijs/themes/dark-plus';
```

Baseline cost: ~28 KB (core) + ~3 KB (theme) = **~31 KB gzip**. Each language grammar adds 2-17 KB and can be lazy-loaded on demand. This is comparable to highlight.js in practice.

As of Shiki 3.9+, **all built-in languages are supported** by the JavaScript engine. The JS engine uses native `RegExp` with the `v` flag (ES2024), which is available in all Tauri 2 webviews (WKWebView on macOS 14+, WebView2 on Chromium 112+).

Shiki v4.0.0 requires Node.js 20+ and only removed deprecated typo-fix APIs -- no functional breaking changes from v3.

**Integration approach:**
- Create a `highlightLine(code: string, lang: string)` utility using `codeToTokens()` API (not `codeToHtml()`) for maximum rendering control
- Shiki returns `ThemedToken[][]` with color info; render as `<span>` elements in diff lines
- Cache the highlighter instance (singleton pattern matching existing LazyStore pattern)
- Detect language from file extension (already available in `FileDiff.path`)
- Lazy-load language grammars on first use per extension

### 2. Word-Level Diff: jsdiff

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `diff` | `^8.0.4` | Word/character-level intra-line diff highlighting | Myers O(ND) algorithm; native `diffWords` and `diffWordsWithSpace` APIs; ships TypeScript types since v8; ~500 KB unpacked (tree-shakeable); O(n^2) worst case (improved from O(n^3) in v7) |

**Key APIs for our use case:**

| Method | Purpose | Token Boundary |
|--------|---------|----------------|
| `diffWords(old, new)` | Word-level diff (whitespace ignored in comparison but preserved in output) | Words + punctuation |
| `diffWordsWithSpace(old, new)` | Word-level diff where whitespace differences are meaningful | Words + punctuation + whitespace |
| `diffChars(old, new)` | Character-level diff for very similar lines | Individual characters |

**Why jsdiff over alternatives:**

- **vs diff-match-patch:** Google's library works at character level only. Word-level diff requires a pre-processing hack (encode words as single characters, diff, decode). jsdiff has native `diffWords()` that handles this cleanly. diff-match-patch also targets text synchronization (patch/match), which we don't need.
- **vs custom implementation:** Myers diff is well-understood but tricky to implement correctly for word boundaries, especially with punctuation and whitespace edge cases. jsdiff is battle-tested (used by Mocha, Jest, and virtually every JS diff tool).
- **vs Rust-side word diffing (similar crate):** Word-level diff needs to run AFTER syntax highlighting to highlight within changed words. If we diff on the Rust side, we'd need to serialize word boundaries over IPC, then correlate them with frontend-side syntax tokens. Much simpler to diff the text in TypeScript and layer syntax highlighting on top.

**v8 improvements relevant to us:**
- Ships own TypeScript types (no need for `@types/diff`)
- Inner-loop array copy eliminated: worst-case O(n^2) instead of O(n^3)
- Class-based `Diff` API for potential custom word boundary definitions

**Integration approach:**
```typescript
import { diffWords } from 'diff';

// For each pair of corresponding Add/Delete lines in a changed hunk:
const changes = diffWords(oldLineText, newLineText);
// changes = [{ value: "const ", added: false, removed: false },
//            { value: "foo", removed: true },
//            { value: "bar", added: true },
//            { value: " = 1;", added: false, removed: false }]
```

Pair corresponding Add/Delete lines within a hunk, run `diffWords()`, then render with additional background highlight on the changed word spans (overlaid on syntax highlighting colors).

### 3. Whitespace Options and Context Lines: git2 DiffOptions (Already Available)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `git2` | `0.19` (existing) | Whitespace ignore + configurable context in diff generation | Already in Cargo.toml; DiffOptions has all 4 whitespace methods + context_lines + algorithm selection; zero new dependencies |

**No new Rust dependencies needed.** The existing `git2 = "0.19"` provides everything:

| Method | Maps to Git Flag | Purpose |
|--------|-----------------|---------|
| `ignore_whitespace(true)` | `-w` / `--ignore-all-space` | Ignore ALL whitespace differences |
| `ignore_whitespace_change(true)` | `-b` / `--ignore-space-change` | Ignore changes in amount of whitespace |
| `ignore_whitespace_eol(true)` | `--ignore-space-at-eol` | Ignore trailing whitespace only |
| `ignore_blank_lines(true)` | `--ignore-blank-lines` | Ignore blank line insertions/deletions |
| `context_lines(n)` | `-U<n>` | Set context lines around changes (default: 3) |
| `interhunk_lines(n)` | (libgit2 feature) | Merge nearby hunks within N unchanged lines |
| `patience(true)` | `--patience` | Patience diff algorithm (better for some files) |
| `indent_heuristic(true)` | `--indent-heuristic` | Smarter hunk boundaries respecting indentation |
| `minimal(true)` | `--minimal` | Smallest possible diff (slower but cleaner output) |

**Integration approach:**
- Add optional parameters to existing `diff_unstaged`, `diff_staged`, and `diff_commit` Tauri commands: `ignore_whitespace: Option<String>` (enum: "all", "change", "eol"), `ignore_blank_lines: Option<bool>`, `context_lines: Option<u32>`
- Frontend passes these as `invoke` arguments based on toolbar toggle state
- Persist toggle states via existing LazyStore pattern
- Default behavior unchanged (no parameters = current behavior)

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@shikijs/langs/*` | `^4.0.2` | Individual language grammars for Shiki | Lazy-load per file extension. Start with ~16 common languages: typescript, javascript, rust, python, go, json, css, html, svelte, markdown, bash, yaml, toml, sql, java, c, cpp, ruby |
| `@shikijs/themes/dark-plus` | `^4.0.2` | VS Code Dark+ theme for syntax colors | Single theme matching Trunk's forced dark UI. Token colors can be overridden via CSS custom properties if needed |

### No New Development Tools Required

The existing toolchain (Vite 6, vitest, biome, svelte-check, cargo clippy/fmt/test) handles all new code without additions.

---

## Installation

```bash
# Syntax highlighting (frontend)
bun add shiki

# Word-level diff (frontend)
bun add diff
```

```toml
# Cargo.toml -- NO CHANGES NEEDED
# git2 = "0.19" already has all whitespace/context methods
```

**Total new frontend dependencies: 2 packages.**
**Total new Rust dependencies: 0.**

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Shiki (frontend JS) | syntect (Rust-side) | If diff lines were rendered as raw HTML from Rust (static site generator pattern); not appropriate for a reactive Svelte UI |
| Shiki (frontend JS) | Prism.js | If bundle size were the overriding concern and highlighting quality were acceptable at a lower level; Prism at 11.7 KB is smaller than Shiki core+JS engine (~31 KB), but highlighting quality is noticeably worse for TypeScript/Rust |
| Shiki (frontend JS) | CodeMirror | If the diff viewer needed inline editing capability (GitButler's use case); Trunk's diff is read-only |
| Shiki JS engine (no WASM) | Shiki WASM engine | If running custom grammars that need Oniguruma features the JS engine can't handle; as of Shiki 3.9+ all built-in languages work with the JS engine |
| jsdiff `diffWords` | diff-match-patch | If you needed patch application or fuzzy text matching; we only need word-level visual diff |
| jsdiff (frontend) | `similar` crate (Rust) | If word diff needed to happen before IPC serialization; we need it after syntax highlighting in the frontend, so frontend is the correct layer |
| git2 whitespace flags | CLI `git diff -w` | Never; project rule is all git ops through git2, no shelling out |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `prismjs` | Lower quality highlighting for complex syntax (TS generics, Rust lifetimes, JSX); maintenance has slowed significantly | Shiki with JS engine |
| `highlight.js` | No advantage over Shiki in this context; auto-detect unnecessary when we have file paths; larger than Prism with similar quality | Shiki with JS engine |
| `@types/diff` | jsdiff v8 ships its own TypeScript types; `@types/diff` would conflict | Just `diff` package |
| `shiki` full/web bundle imports | Full: 1.2 MB gzip, Web: 695 KB gzip -- both include Oniguruma WASM (231 KB) and bundle all languages; wasteful for a desktop app loading one file at a time | Fine-grained: `shiki/core` + `shiki/engine/javascript` + individual `@shikijs/langs/*` |
| `syntect` (Rust crate) | Would serialize pre-rendered HTML over IPC, can't interact with frontend theme system, theme changes require Rust round-trip | Shiki in frontend |
| `tree-sitter` / `tree-sitter-highlight` (Rust crate) | Immature highlight query ecosystem; inconsistent grammar quality; painful multi-grammar maintenance; overkill for read-only highlighting | Shiki in frontend |
| `@codemirror/lang-*` | Full editor framework; massive dependency graph for read-only syntax highlighting | Shiki |
| `monaco-editor` | Full VS Code editor; 10+ MB; extreme overkill for read-only diff display | Shiki |
| Any npm virtual-scroll library for diff panels | Diff views rarely exceed 1,000 lines per file; virtual scrolling adds complexity that conflicts with line selection, hunk staging, and synchronized scrolling | Native `overflow-y: auto` with `scrollTop` sync (see below) |

---

## Virtual Scrolling: Not Needed for Diff Views

**Key insight: diff panels do NOT need virtual scrolling.** This is different from the commit graph.

The commit graph uses virtual scrolling because it can have 10,000+ rows (one per commit). Diff views are fundamentally different:

- A single file diff rarely exceeds 1,000 lines even in full-file view
- Side-by-side view doubles the horizontal space but NOT the row count (same lines, two columns)
- Syntax highlighting creates ~5-15 `<span>` elements per line; at 1,000 lines = 5,000-15,000 DOM nodes, well within browser limits
- Hunk staging interactions (click-to-select lines, shift-click ranges) rely on stable DOM references that virtual scrolling would complicate significantly
- The existing `selectedLineIndices`, `selectedHunkKey`, and `hunkElements` patterns in DiffPanel assume stable DOM

**Synchronized scrolling for split (side-by-side) view:**

Two `<div>` containers with `overflow-y: auto`, synced via `scrollTop`. No third-party library needed -- approximately 20 lines of code:

```typescript
let isSyncing = false;
function syncScroll(source: HTMLElement, target: HTMLElement) {
  if (isSyncing) return;
  isSyncing = true;
  target.scrollTop = source.scrollTop;
  requestAnimationFrame(() => { isSyncing = false; });
}
```

Both panels render the same number of rows. Deleted lines appear as blank placeholders in the "new" side; added lines appear as blank placeholders in the "old" side. This keeps line counts equal and scroll positions aligned.

**If performance degrades with very large full-file views (5,000+ lines):**

1. First measure: is the bottleneck rendering or highlighting?
2. If rendering: use CSS `content-visibility: auto` on hunk/section containers (native browser virtualization, zero JS)
3. If highlighting: chunk the work with `requestIdleCallback` (highlight visible lines first, rest in idle frames)
4. Only as a last resort consider adapting the vendored VirtualList for diff content

---

## Architecture Decision: Where Each Feature Lives

| Feature | Frontend or Backend | Rationale |
|---------|-------------------|-----------|
| Syntax highlighting | Frontend (Shiki) | Theme integration, token-level rendering control, no IPC payload bloat |
| Word-level diff | Frontend (jsdiff) | Must layer on top of syntax tokens; needs old+new line content already available from existing diff data |
| Whitespace toggle | Backend (git2 DiffOptions) | git2 flags control diff generation at the source; frontend just passes toggle state as invoke params |
| Context lines slider | Backend (git2 DiffOptions) | `context_lines(n)` changes hunk boundaries at generation time |
| Show invisible chars | Frontend (render) | Replace `\t` with visible arrow, trailing spaces with middle dots; pure rendering |
| Word wrap toggle | Frontend (CSS) | `white-space: pre-wrap` vs `white-space: pre` toggle |
| Line numbers in gutter | Frontend (render) | `old_lineno`/`new_lineno` already exist in `DiffLine` from Rust |
| View mode toggle (hunk/full/split) | Frontend (layout) | Same diff data, different rendering layout |
| Scrollbar minimap | Frontend (Canvas) | Render miniature color-coded overview of the diff; pure UI component |

**The only backend changes are:** adding optional parameters to the three existing diff commands (`diff_unstaged`, `diff_staged`, `diff_commit`) for whitespace ignore options and context lines. No new Rust crates, no new commands.

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| `shiki@^4.0.2` | Svelte 5, Vite 6, Tauri 2 webview | ESM-only; fine-grained imports work with Vite's tree-shaking; JS engine uses RegExp `v` flag (Safari 17+ via WKWebView, Chromium 112+ via WebView2) |
| `diff@^8.0.4` | Any JS runtime | Zero dependencies; pure algorithm; ESM + CJS dual export; ships own TypeScript types |
| `git2 = "0.19"` | Existing Cargo.toml | No version bump needed; all whitespace methods verified present in 0.19 docs |
| Shiki JS engine RegExp `v` flag | macOS 14+ (Tauri 2 minimum), Windows 10+ (WebView2/Chromium 112+) | Tauri 2 requires macOS 14+ which ships Safari 17; RegExp `v` flag fully supported |

---

## Sources

- [Shiki best performance guide](https://shiki.style/guide/best-performance) -- fine-grained bundling, JS engine recommendation (HIGH confidence)
- [Shiki bundles reference](https://shiki.style/guide/bundles) -- full: 6.4MB/1.2MB gz, web: 3.8MB/695KB gz (HIGH confidence)
- [Shiki regex engines](https://shiki.style/guide/regex-engines) -- JS engine: all built-in languages supported, no WASM needed (HIGH confidence)
- [Shiki v4 release blog](https://shiki.style/blog/v4) -- Node 20+ requirement, typo-fix breaking changes only (HIGH confidence)
- [Code highlighter benchmark (chsm.dev, Jan 2025)](https://chsm.dev/blog/2025/01/08/comparing-web-code-highlighters) -- Prism 11.7KB/Shiki 279.8KB (full bundle), Prism 0.5ms/Shiki 3.5ms per highlight (MEDIUM confidence, independent benchmark)
- [git2 0.19 DiffOptions docs](https://docs.rs/git2/0.19.0/git2/struct.DiffOptions.html) -- all 4 whitespace methods + context_lines + patience + indent_heuristic confirmed (HIGH confidence)
- [git2 latest DiffOptions docs](https://docs.rs/git2/latest/git2/struct.DiffOptions.html) -- full method list reference (HIGH confidence)
- [jsdiff GitHub repository](https://github.com/kpdecker/jsdiff) -- Myers O(ND) algorithm, diffWords/diffChars/diffWordsWithSpace APIs (HIGH confidence)
- [jsdiff v8 release notes](https://github.com/kpdecker/jsdiff/blob/master/release-notes.md) -- O(n^2) fix, built-in TypeScript types, class-based API (HIGH confidence)
- [npm: shiki@4.0.2](https://www.npmjs.com/package/shiki) -- verified current version (HIGH confidence)
- [npm: diff@8.0.4](https://www.npmjs.com/package/diff) -- verified current version, ~500KB unpacked (HIGH confidence)
- [crates.io: syntect@5.3.0](https://crates.io/crates/syntect) -- 13M downloads, evaluated and rejected for frontend highlighting (HIGH confidence)
- [crates.io: git2@0.20.4](https://crates.io/crates/git2) -- latest available, but staying on 0.19 per project constraint (HIGH confidence)
- [GitButler PR #7915](https://github.com/gitbutlerapp/gitbutler/pull/7915) -- uses CodeMirror for syntax highlighting in Tauri+Svelte Git GUI (MEDIUM confidence, prior art)
- [syntect HTML module docs](https://docs.rs/syntect/latest/syntect/html/index.html) -- ClassedHTMLGenerator for HTML output (HIGH confidence)

---
*Stack research for: Trunk v0.12 Better Diffs*
*Researched: 2026-03-28*
