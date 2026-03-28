# Phase 60: Word-Level Diff - Research

**Researched:** 2026-03-28
**Domain:** Rust word-level diff computation with `similar` crate, frontend inline span rendering
**Confidence:** HIGH

## Summary

This phase populates the `word_spans` field on `DiffLine` (added empty in Phase 59) by pairing adjacent Delete/Add lines within hunks and running `similar::TextDiff` to compute word-level differences. The `similar` crate v2.7.0 provides `iter_inline_changes()` which yields `InlineChange` values containing `(emphasized: bool, value: &str)` tuples -- a near-exact match for the existing `WordSpan { start: u32, end: u32, emphasized: bool }` type. The backend computes byte offset ranges from these tuples and populates `word_spans` per line. The frontend renders spans with CSS class-based highlighting.

The performance guard (WORD-02) uses two checks: line length > 500 characters and `TextDiff::ratio()` < 0.4 (which corresponds to >60% edit distance since ratio = similarity, not distance). Both checks run in Rust before word-diff computation, short-circuiting to empty `word_spans`.

**Primary recommendation:** Use `similar` v2.7.0 with `inline` feature. Compute word-spans in a post-processing pass over each hunk's DiffLine vec in `walk_diff_into_file_diffs`, after git2's diff callback completes. Frontend renders `<span>` elements with `.word-add` / `.word-delete` CSS classes.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Word-level tokenization -- split lines on whitespace and punctuation boundaries for comparison. Character-level is too noisy. Language-aware belongs in Phase 61.
- **D-02:** Use `similar` crate's `TextDiff` with word-level splitting via `ChangeTag` iteration on paired lines.
- **D-03:** Sequential pairing within hunks -- pair consecutive Delete/Add runs by position (first Delete with first Add, etc.). Unpaired lines get empty `word_spans`.
- **D-04:** Pairing happens per-hunk, not across hunks.
- **D-05:** Both threshold checks run in Rust backend before populating `word_spans`. Line > 500 chars or paired lines > 60% edit distance leaves `word_spans` empty.
- **D-06:** Frontend simply checks if `word_spans` is non-empty. No frontend-side threshold logic.
- **D-07:** CSS custom properties: `--color-diff-word-add-bg` and `--color-diff-word-delete-bg`. Never inline colors.
- **D-08:** Word-diff highlights render as background-color spans within line text, overlaid on existing line-level add/delete background.

### Claude's Discretion
- Exact `similar` API usage (Algorithm choice: Patience vs Myers vs LCS)
- How to compute edit distance ratio for the 60% threshold
- Whether to use `iter_inline_changes()` or manual word splitting + diff
- Frontend rendering approach (inline `<span>` elements vs CSS ranges)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WORD-01 | Changed words/characters within modified lines are highlighted with a distinct background | `similar` crate `iter_inline_changes()` provides `(emphasized, value)` tuples -> convert to `WordSpan` byte offsets -> frontend renders `<span>` with CSS class |
| WORD-02 | Word-level diff is skipped for lines over 500 chars or with >60% edit distance (performance guard) | Length check via `content.len() > 500`; edit distance via `TextDiff::ratio() < 0.4` (ratio is similarity, so <0.4 means >60% different) |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| similar | 2.7.0 | Word-level diff computation | Locked decision D-02. Rust-native, purpose-built `iter_inline_changes()` API, used by insta (snapshot testing) |

### Feature Flags Required
| Crate | Feature | Purpose |
|-------|---------|---------|
| similar | `inline` | Enables `iter_inline_changes()` and `iter_inline_changes_deadline()` methods on `TextDiff`. NOT in default features -- must be explicitly enabled |
| similar | `text` | Default feature, enables `TextDiff::from_lines()`, `from_words()`, etc. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `iter_inline_changes()` | Manual `TextDiff::from_words()` + `iter_all_changes()` | Manual approach gives more control over tokenization but requires hand-tracking byte offsets. `iter_inline_changes()` does the two-level diff automatically (line-level then word-level within replacements). Recommend `iter_inline_changes()`. |
| Algorithm::Myers (default) | Algorithm::Patience | Patience produces "more human-readable" diffs for code but Myers is faster. For short within-line diffs, performance difference is negligible. Recommend keeping default (Myers). |

**Installation (Cargo.toml):**
```toml
similar = { version = "2.7", features = ["inline"] }
```

## Architecture Patterns

### Implementation Flow

```
walk_diff_into_file_diffs(diff)
  |
  v
git2 diff.foreach() callbacks build Vec<FileDiff> with empty word_spans
  |
  v
Post-processing: for each hunk in each FileDiff:
  1. Collect Delete/Add line runs
  2. Pair sequentially (D-03)
  3. For each pair, check thresholds (D-05):
     a. Either line.content.len() > 500 -> skip
     b. TextDiff::from_words(&del_content, &add_content).ratio() < 0.4 -> skip
  4. If pass: run TextDiff::from_lines (from_words not needed --
     iter_inline_changes does word-level internally)
     Actually: use TextDiff::from_lines on the two single lines,
     then iter_inline_changes gives word-level spans
  5. Convert (emphasized, value) tuples to WordSpan byte offsets
  6. Assign to respective DiffLine.word_spans
```

### Recommended Architecture: Two-Pass in `walk_diff_into_file_diffs`

**Pass 1 (existing):** git2 `diff.foreach()` callbacks populate `Vec<FileDiff>` with `DiffLine` structs where `word_spans` is empty.

**Pass 2 (new):** After `diff.foreach()` completes, iterate over all hunks and compute word spans. This is necessary because git2's callback API delivers lines one at a time -- you cannot see the full Delete/Add run until all callbacks have fired.

```
fn walk_diff_into_file_diffs(diff: git2::Diff<'_>) -> Result<Vec<FileDiff>, TrunkError> {
    // ... existing Pass 1 (unchanged) ...
    let mut file_diffs = file_diffs.into_inner();

    // Pass 2: word-level diff enrichment
    for fd in &mut file_diffs {
        for hunk in &mut fd.hunks {
            compute_word_spans_for_hunk(&mut hunk.lines);
        }
    }

    Ok(file_diffs)
}
```

### Line Pairing Algorithm (D-03, D-04)

Within a single hunk, scan lines and collect consecutive runs of Delete lines followed by Add lines. Pair them positionally:

```
Lines: [Context, Delete, Delete, Add, Add, Add, Context]
Runs:  [Delete(0), Delete(1)] + [Add(0), Add(1), Add(2)]
Pairs: (Delete(0), Add(0)), (Delete(1), Add(1))
Unpaired: Add(2) -- gets empty word_spans
```

Edge cases:
- Delete-only run (no following Adds): all Deletes get empty word_spans
- Add-only run (no preceding Deletes): all Adds get empty word_spans
- Interleaved Context lines break runs: each Delete/Add block is independent

### Byte Offset Computation from `iter_inline_changes`

`InlineChange::iter_strings_lossy()` yields `(bool, Cow<str>)` tuples where:
- `bool` = emphasized (true = changed, false = unchanged)
- `Cow<str>` = the text fragment

To convert to `WordSpan { start: u32, end: u32, emphasized: bool }`:

```rust
let mut offset: u32 = 0;
let mut spans = Vec::new();
for (emphasized, value) in change.iter_strings_lossy() {
    let len = value.len() as u32;
    if len > 0 {
        spans.push(WordSpan {
            start: offset,
            end: offset + len,
            emphasized,
        });
    }
    offset += len;
}
```

**Important:** The spans cover the ENTIRE line content (both emphasized and non-emphasized segments). The frontend uses `emphasized: true` to apply highlight styling and `emphasized: false` for normal rendering. This means the content string can be fully reconstructed from spans.

### Anti-Patterns to Avoid
- **Computing word diff inside git2 callbacks:** The callbacks fire line-by-line and hold a borrow on the diff. You cannot do expensive work or accumulate cross-line state inside them. Always post-process.
- **Using `TextDiff::from_words()` directly on line pairs:** This gives you `Change` values without emphasis information. Use `TextDiff::from_lines()` on the two single-line strings and then `iter_inline_changes()` to get the two-level diff with emphasis.
- **Assuming content byte offsets match character offsets:** Content is UTF-8. Multi-byte characters mean byte offset != character index. WordSpan uses byte offsets, and frontend `String.slice()` in JavaScript slices by UTF-16 code units, not bytes. Need to handle this correctly (see Pitfalls section).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Word-level diff algorithm | Custom tokenizer + LCS | `similar::TextDiff` with `iter_inline_changes()` | Handles word boundary detection, emphasis tracking, and algorithm timeout automatically |
| Edit distance ratio | Manual Levenshtein on strings | `TextDiff::ratio()` | Already computed from the same diff ops, zero extra cost |
| Line pairing | Complex graph-matching of deleted/added lines | Sequential positional pairing per hunk | Locked decision D-03. Simple, predictable, matches user mental model |

**Key insight:** `iter_inline_changes()` is purpose-built for exactly this use case. It performs a line-level diff first, then for adjacent Replace operations (delete+add pairs), it runs a secondary word-level diff and marks which portions are emphasized. This two-level approach is precisely what the phase needs.

## Common Pitfalls

### Pitfall 1: UTF-8 Byte Offsets vs JavaScript String Indices
**What goes wrong:** Rust `WordSpan` byte offsets are UTF-8 byte positions. JavaScript `String.slice()` uses UTF-16 code unit indices. For ASCII-only content they match, but for emoji, CJK, or accented characters, they diverge. A 4-byte emoji in Rust is 2 code units in JS.
**Why it happens:** The two languages use different string encodings internally.
**How to avoid:** Convert byte offsets to character (code point) offsets in Rust before serialization, OR use a `TextEncoder`/byte-aware slicing utility in the frontend. Since the existing Phase 59 decision (D-05 from 59-CONTEXT.md) established byte offset ranges for WordSpan, the simplest approach is to convert in the frontend using `TextEncoder().encode(content)` to get a UTF-8 byte array, then slice that. However, since most code is ASCII, the pragmatic path is: compute byte offsets in Rust (natural for `similar` output), and if the content is all ASCII (common case), JS `slice()` works directly. For non-ASCII, use a small utility function.
**Warning signs:** Highlights appearing shifted or wrong length on lines containing non-ASCII characters.

### Pitfall 2: `iter_inline_changes` Requires Line-Level TextDiff
**What goes wrong:** Calling `iter_inline_changes` on a `TextDiff::from_words()` result does not produce meaningful output. It expects a line-level diff so it can find Replace operations to drill into.
**Why it happens:** `iter_inline_changes` is designed for a two-level diff: line-level first, then word-level within replacements.
**How to avoid:** Create the TextDiff with `from_lines()` (passing the two single lines as separate "documents" each terminated with newline). Then call `iter_inline_changes()` on each `DiffOp` from `ops()`.
**Warning signs:** Empty or trivial emphasis results.

### Pitfall 3: Missing `inline` Feature Flag
**What goes wrong:** `iter_inline_changes()` method does not exist, compilation fails.
**Why it happens:** The `inline` feature is NOT part of `similar`'s default features.
**How to avoid:** Add `features = ["inline"]` to the Cargo.toml dependency declaration.
**Warning signs:** "no method named `iter_inline_changes` found" compiler error.

### Pitfall 4: Word-Diff Highlights Invisible Against Line Background
**What goes wrong:** The word-diff background color is too similar to the line-level add/delete background, making highlights invisible.
**Why it happens:** Both are tints of the same hue (green for add, red for delete).
**How to avoid:** Word-diff backgrounds should be significantly more opaque/saturated than line backgrounds. Current line backgrounds use alpha 0.1. Word-diff backgrounds should use alpha 0.3-0.4 to create visible contrast.
**Warning signs:** Users cannot see which specific words changed within a highlighted line.

### Pitfall 5: Newline Handling in Single-Line Diffs
**What goes wrong:** When creating a `TextDiff::from_lines()` from two single lines, the line might or might not have a trailing newline. If both lines lack newlines, `similar` may treat them as one "line" and not produce a Replace op.
**Why it happens:** `from_lines` splits on newlines. A string without a trailing newline is one line. Two such strings diffed produce a single Replace of that one line, which `iter_inline_changes` can process.
**How to avoid:** Ensure both lines are newline-terminated before passing to `from_lines`. The `DiffLine.content` from git2 may or may not include a trailing newline -- normalize by appending `\n` if absent.
**Warning signs:** Some paired lines produce empty word_spans when they should have highlights.

## Code Examples

### Example 1: Adding `similar` to Cargo.toml
```toml
# In src-tauri/Cargo.toml [dependencies]
similar = { version = "2.7", features = ["inline"] }
```

### Example 2: Core Word-Span Computation Function
```rust
// Source: similar docs (iter_inline_changes) + InlineChange::iter_strings_lossy()
use similar::{ChangeTag, TextDiff};
use crate::git::types::WordSpan;

/// Compute word spans for a paired delete/add line.
/// Returns (delete_spans, add_spans).
fn compute_word_spans_for_pair(
    old_content: &str,
    new_content: &str,
) -> (Vec<WordSpan>, Vec<WordSpan>) {
    // Normalize: ensure newline-terminated for from_lines
    let old = if old_content.ends_with('\n') {
        old_content.to_string()
    } else {
        format!("{}\n", old_content)
    };
    let new = if new_content.ends_with('\n') {
        new_content.to_string()
    } else {
        format!("{}\n", new_content)
    };

    let diff = TextDiff::from_lines(&old, &new);
    let mut del_spans = Vec::new();
    let mut add_spans = Vec::new();

    for op in diff.ops() {
        for change in diff.iter_inline_changes(op) {
            let mut offset: u32 = 0;
            let mut spans = Vec::new();
            for (emphasized, value) in change.iter_strings_lossy() {
                let len = value.len() as u32;
                if len > 0 {
                    spans.push(WordSpan {
                        start: offset,
                        end: offset + len,
                        emphasized,
                    });
                }
                offset += len;
            }
            match change.tag() {
                ChangeTag::Delete => del_spans = spans,
                ChangeTag::Insert => add_spans = spans,
                ChangeTag::Equal => {} // context within inline changes
            }
        }
    }

    (del_spans, add_spans)
}
```

### Example 3: Line Pairing Within a Hunk
```rust
use crate::git::types::{DiffLine, DiffOrigin};

fn compute_word_spans_for_hunk(lines: &mut [DiffLine]) {
    let mut i = 0;
    while i < lines.len() {
        // Find start of a Delete run
        if !matches!(lines[i].origin, DiffOrigin::Delete) {
            i += 1;
            continue;
        }

        // Collect consecutive Deletes
        let del_start = i;
        while i < lines.len() && matches!(lines[i].origin, DiffOrigin::Delete) {
            i += 1;
        }
        let del_end = i;

        // Collect consecutive Adds following the Deletes
        let add_start = i;
        while i < lines.len() && matches!(lines[i].origin, DiffOrigin::Add) {
            i += 1;
        }
        let add_end = i;

        // Pair positionally
        let pairs = (del_end - del_start).min(add_end - add_start);
        for p in 0..pairs {
            let del_idx = del_start + p;
            let add_idx = add_start + p;

            let del_content = &lines[del_idx].content;
            let add_content = &lines[add_idx].content;

            // Threshold checks (WORD-02)
            if del_content.len() > 500 || add_content.len() > 500 {
                continue; // leave word_spans empty
            }

            // Edit distance check: ratio < 0.4 means >60% different
            let check_diff = TextDiff::from_chars(del_content, add_content);
            if check_diff.ratio() < 0.4 {
                continue; // leave word_spans empty
            }

            let (del_spans, add_spans) = compute_word_spans_for_pair(del_content, add_content);
            lines[del_idx].word_spans = del_spans;
            lines[add_idx].word_spans = add_spans;
        }
    }
}
```

### Example 4: Frontend Rendering with Word Spans
```svelte
<!-- In DiffPanel.svelte line rendering -->
{#if line.word_spans.length > 0}
  {#each line.word_spans as span}
    <span class={span.emphasized
      ? (line.origin === 'Add' ? 'word-add' : 'word-delete')
      : ''}
    >{line.content.slice(span.start, span.end)}</span>
  {/each}
{:else}
  {originSymbol(line.origin)}{line.content}
{/if}
```

### Example 5: CSS Custom Properties
```css
/* In src/app.css :root block, after existing diff colors */
--color-diff-word-add-bg: rgba(74, 222, 128, 0.35);
--color-diff-word-delete-bg: rgba(248, 113, 113, 0.35);
```

```css
/* Component styles */
.word-add {
  background-color: var(--color-diff-word-add-bg);
  border-radius: 2px;
}
.word-delete {
  background-color: var(--color-diff-word-delete-bg);
  border-radius: 2px;
}
```

### Example 6: Edit Distance Ratio Interpretation
```rust
// similar::TextDiff::ratio() returns SIMILARITY (not distance)
// ratio = 2.0 * matching / total
// ratio = 1.0 means identical
// ratio = 0.0 means completely different
//
// "60% edit distance" = "40% similar" = ratio < 0.4
//
// Use from_chars for ratio check (character-level granularity for accurate similarity)
let check = TextDiff::from_chars("hello world", "goodbye mars");
let ratio = check.ratio(); // ~0.27 (very different)
// ratio < 0.4 -> skip word-level diff
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `difference` crate | `similar` crate | 2020+ | `similar` is actively maintained (v2.7.0), used by insta/cargo; `difference` is unmaintained |
| Manual word splitting + LCS | `iter_inline_changes()` | similar v2.2+ | Two-level diff (line then word) in one API call |
| Unbounded diff computation | Deadline-based termination | similar v2.2+ | `iter_inline_changes` has hardcoded 500ms deadline; `iter_inline_changes_deadline()` allows custom |

**Deprecated/outdated:**
- `difference` crate: Unmaintained, replaced by `similar`
- Manual `TextDiff::from_words()` approach: `iter_inline_changes()` is more ergonomic for line-pair word diffs

## Open Questions

1. **UTF-8 byte offset handling in frontend**
   - What we know: WordSpan uses byte offsets (u32). JavaScript `String.slice()` uses UTF-16 code unit indices. For ASCII they match.
   - What's unclear: How common non-ASCII content is in typical diffs. Whether to convert in Rust or JS.
   - Recommendation: Start with byte offsets from Rust. Add a small JS utility `sliceByBytes(content, start, end)` that handles UTF-8 correctly. For ASCII strings (the common case), `String.slice()` works as-is. The utility only needs to activate when content contains multi-byte characters. This can be a follow-up refinement if issues appear in testing.

2. **Origin symbol (+/-/space) in word-span rendering**
   - What we know: Current rendering prepends `originSymbol(line.origin)` before content. With word spans, the origin symbol must be rendered separately, outside the span loop.
   - What's unclear: Whether the origin symbol should be considered part of the content or rendered as a separate gutter element.
   - Recommendation: Render origin symbol as a separate element before the span loop. Word spans refer to `content` only (which does not include the origin character).

## Project Constraints (from CLAUDE.md)

- **Never inline colors** -- all colors must use CSS custom properties from theme (enforced by D-07)
- **Never fight layout with positioning hacks** -- use grid/flexbox for natural flow
- **All git operations through git2** -- word-diff is a post-processing step on git2 output, no shelling out
- **Frontend -> Backend via invoke()** -- no new commands needed, word_spans populated in existing diff flow
- **Run all 6 checks before push** -- fmt, clippy, test, vitest, svelte-check, biome

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Rust framework | cargo test (integration tests in src-tauri/tests/) |
| Rust config | src-tauri/Cargo.toml (dev-dependencies: tempfile, tauri test) |
| Rust quick command | `cd src-tauri && cargo test test_diff -- --nocapture` |
| Rust full command | `cd src-tauri && cargo test` |
| Frontend framework | vitest (src/**/*.test.ts) |
| Frontend config | package.json `"test": "vitest run"` |
| Frontend quick command | `bun run test -- --reporter=verbose src/components/DiffPanel.test.ts` |
| Frontend full command | `bun run test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WORD-01 | Paired delete/add lines get non-empty word_spans with correct byte offsets | Rust integration | `cd src-tauri && cargo test word_span -- --nocapture` | No -- Wave 0 |
| WORD-01 | Frontend renders `<span>` elements with word-add/word-delete classes when word_spans present | Frontend component | `bun run test -- src/components/DiffPanel.test.ts` | Partially (DiffPanel.test.ts exists but no word-span tests) |
| WORD-02 | Lines > 500 chars get empty word_spans | Rust integration | `cd src-tauri && cargo test word_span_long -- --nocapture` | No -- Wave 0 |
| WORD-02 | Pairs with > 60% edit distance get empty word_spans | Rust integration | `cd src-tauri && cargo test word_span_dissimilar -- --nocapture` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test test_diff word_span -- --nocapture && bun run test`
- **Per wave merge:** Full suite (`cd src-tauri && cargo test && bun run test && bun run check`)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] New Rust integration tests for word-span computation (in `src-tauri/tests/test_diff.rs`)
- [ ] Updated `DiffPanel.test.ts` test data to include non-empty `word_spans` and verify span rendering
- [ ] No new test framework config needed -- existing Rust and vitest infrastructure covers all needs

## Sources

### Primary (HIGH confidence)
- [similar crate docs.rs](https://docs.rs/similar/latest/similar/) - TextDiff API, InlineChange struct, ratio() method, iter_inline_changes
- [similar TextDiff docs](https://docs.rs/similar/latest/similar/struct.TextDiff.html) - Constructor methods, ratio(), iter_inline_changes signatures
- [similar InlineChange docs](https://docs.rs/similar/latest/similar/struct.InlineChange.html) - values(), iter_strings_lossy(), tag()
- [similar GitHub repo](https://github.com/mitsuhiko/similar) - examples/terminal-inline.rs showing complete iter_inline_changes usage
- [similar Cargo.toml features](https://github.com/mitsuhiko/similar/blob/main/Cargo.toml) - `inline` feature required, not in defaults
- Existing codebase: `src-tauri/src/git/types.rs` (WordSpan struct), `src-tauri/src/commands/diff.rs` (walk_diff_into_file_diffs), `src/lib/types.ts` (TS mirror), `src/components/DiffPanel.svelte` (line rendering), `src/app.css` (theme variables)

### Secondary (MEDIUM confidence)
- [similar crate on crates.io](https://crates.io/crates/similar) - Confirmed v2.7.0 is latest (verified via `cargo search`)
- [similar DiffOp docs](https://docs.rs/similar/latest/similar/enum.DiffOp.html) - Replace/Delete/Insert/Equal variants
- [similar TextDiffConfig docs](https://docs.rs/similar/latest/similar/struct.TextDiffConfig.html) - Algorithm and timeout configuration

### Tertiary (LOW confidence)
- None -- all findings verified against official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - `similar` v2.7.0 confirmed via crates.io, API verified via docs.rs, feature flags confirmed via Cargo.toml
- Architecture: HIGH - Two-pass approach follows established codebase pattern, `iter_inline_changes` API matches WordSpan type exactly
- Pitfalls: HIGH - UTF-8/UTF-16 offset mismatch is well-documented, feature flag requirement confirmed from source

**Research date:** 2026-03-28
**Valid until:** 2026-04-28 (stable crate, stable API)
