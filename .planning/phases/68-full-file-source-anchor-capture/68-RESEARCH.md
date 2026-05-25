# Phase 68: Full-File-Source Anchor Capture — Research

**Researched:** 2026-05-25
**Phase goal:** User can select a line range in the full-file-at-commit view and attach a comment, anchored to absolute (1-based) blob line numbers on the `new` side; persists via the shared `add_comment` command and survives an app restart.
**Requirement:** ANCH-02

> **Provenance note:** The original `gsd-phase-researcher` run completed its investigation (read CONTEXT.md, REQUIREMENTS, ROADMAP §67/§68/§70, and the analog source files; advisor-confirmed) but its connection dropped at the write boundary twice before persisting. This document was authored by the orchestrator from that agent's confirmed findings plus first-hand reads of the cited source. All file:line references below were verified against the working tree on 2026-05-25.

---

## 1. Headline Finding — This Is a Frontend-Only Phase

The backend is **already shipped and tested**. There is no Rust command work in this phase.

- `add_comment` / `save_draft_comment` are shared dumb writers that persist whatever `source`/`side` the TS adapter supplies (`src-tauri/src/commands/review.rs`). Phase 67 (L-08) made them source-agnostic.
- The review schema is **frozen** (Phase 65): `Source { Diff, FullFile }`, `Side { Old, New }`, `Anchor`, `Comment { text, anchor, cached_excerpt }`, `DraftComment { text, anchor }` at `src-tauri/src/git/types.rs:296-336`. `types.rs:292` carries the locked invariant: *"The Anchor NEVER carries hunk_index/line_index/context_lines/ignore_whitespace"* (L-01).
- A `Source::FullFile` round-trip test already passes: `add_comment_persists_full_file_source_unchanged` (`src-tauri/src/commands/review.rs:1257`).

**Implication:** Plan the phase as TS/Svelte only. The single open question is whether a small **full-file excerpt helper** is warranted on the frontend — it is (see §3), and it is pure TS, not a Rust command.

---

## 2. The Direct Analog — Phase 67's Diff-Source Path

Mirror this pattern; diverge only where CONTEXT.md decisions say to.

### 2.1 Pure capture-time adapter — `src/lib/diff-anchor.ts`

`buildDiffAnchor(commitOid, file, hunkIdx, selectedLineIndices) → { anchor, cachedExcerpt }`:
- Operates on `file.hunks[hunkIdx].lines` (a single hunk).
- `side` resolved via `resolveSide(status, selected)`.
- `start_line/end_line = min/max` over the chosen side's line numbers, **after `.filter(n => n !== null)`** (diff-anchor.ts:69-73). This filter already drops `null` line numbers — for the full-file (`side=New`) case it means Delete lines (`new_lineno=null`) are naturally excluded from the range, which is exactly **D-02**.
- `cachedExcerpt` is built over the contiguous **index** span `slice(spanStart, spanEnd+1)` and `prefixLine`-formatted (`+`/`-`/space) — this is the diff-format excerpt (L-06) that Phase 68 **diverges from** (D-04: plain content).

### 2.2 Inline composer — `src/components/diff/CommentComposer.svelte`

- Props: `{ file, hunkIdx, selectedLineIndices, commitOid, repoPath, onclose }`.
- `captured = $derived(buildDiffAnchor(...))` — single source of truth for both the persisted range and the preview.
- 300ms debounced `save_draft_comment` on input (`DRAFT_DEBOUNCE_MS = 300`, `scheduleDraftSave`/`persistDraft`); submit → `add_comment` with `{ path, text, anchor, cachedExcerpt }`; clears `text` + calls `onclose()` on success.
- `submitDisabled = text.trim() === "" || submitting` (empty-text submit disabled — matches CONTEXT.md).
- Exports instance method `confirmDiscardIfDirty()` the host calls before switching selection.
- **Coupling:** the composer hard-imports `buildDiffAnchor` and takes `file`/`hunkIdx`/`selectedLineIndices`. To reuse for full-file, the cleanest seam is to **inject the captured `{ anchor, cachedExcerpt }` result** rather than the raw selection inputs (decouples the composer from which adapter produced it). Whether to refactor the composer to accept an injected `captured` prop vs. add a `source` mode is the planner's call (CONTEXT.md "Claude's Discretion"), but injection is the lower-coupling option and keeps the diff path unchanged.

### 2.3 On-selection affordance — `src/components/diff/HunkView.svelte:302-326`

The "Comment ({selectedCount})" button appears when `diffKind === 'commit'` and the hunk has a selection. It is `disabled={isMerge}` with a merge-disable title (Phase 67 L-07). **The full-file affordance must NOT mirror the `isMerge` disable** — L-05: merge commits are valid for full-file capture. Line selectability in the diff path is `isSelectable = line.origin !== 'Context'` (HunkView.svelte:331); for full-file it becomes **`isSelectable = line.new_lineno !== null`** (D-02 — only new-side lines are selectable endpoints).

### 2.4 Threading — `src/components/diff/DiffViewer.svelte`

- DiffViewer threads `selectedHunkKey`, `selectedLineIndices`, `selectedCount`, `isMerge`, and callbacks (`onlineclick`, `oncommentlines`, …) down to `HunkView`/`SplitView`.
- **`FullFileView` is currently invoked with only `{fileDiffs} {showInvisibles} {wordWrap}`** (DiffViewer.svelte:118). This invocation must be extended to thread the selection/comment props.
- **Gap to close:** `commitOid` and `repoPath` (the composer needs both) are **not** current DiffViewer props. The host (DiffPanel, which owns the diff-path selection state and mounts the composer) already has them. The planner must thread `commitOid`/`repoPath` from the host down to the full-file affordance/composer, mirroring however DiffPanel hosts the diff composer.

### 2.5 The capture surface — `src/components/diff/FullFileView.svelte`

- Renders a **flat continuous list**: `allLines = fd.hunks.flatMap(h => h.lines)` (FullFileView.svelte:49) — no hunk boundaries, no `selectedHunkKey` concept. This is why the diff view's single-hunk `selectedHunkKey` + per-hunk `Set<number>` model **does not map** (D-01).
- Has per-line `old_lineno`/`new_lineno` gutters and `lineBackground(origin)` (no `isSelected` param yet — needs one, like HunkView's `lineBackground(origin, isSelected)`).
- **Has zero selection state today.** This is the net-new work.

---

## 3. The Full-File Adapter (the core net-new pure unit)

Recommend a new pure module `src/lib/full-file-anchor.ts` (sibling of `diff-anchor.ts`; keeps the diff adapter untouched), exporting e.g.:

```
buildFullFileAnchor(commitOid, file: FileDiff, selectedNewLinenos /* or flat indices */) → { anchor, cachedExcerpt }
```

Behavior (grounded in the decisions):
- **Input is the flat line list** `file.hunks.flatMap(h => h.lines)`, not a single hunk (contrast diff-anchor's `file.hunks[hunkIdx]`).
- `anchor = { commit_oid, file_path, source: "FullFile", side: "New", start_line, end_line }` where `start_line/end_line = min/max` of the selected lines' `new_lineno` (L-01: absolute 1-based blob line numbers on the new side).
- **D-02:** Delete lines (`new_lineno === null`) are not valid endpoints and are excluded from both range and excerpt.
- **D-04:** `cachedExcerpt` is **plain new-side content** (the lines' `content` verbatim, no `+`/`-`/space prefix — no `prefixLine`). Language-fenced rendering is Phase 70's job; the cache just stores plain content.
- **D-03 (gap marker):** the full-file view is a `context_lines(100_000)` diff (`src-tauri/src/commands/diff.rs:33-34`: `show_full_file → 100_000`), **not** a blob read. For files >100k lines, or with very large unchanged regions, hunks can still be dropped and `new_lineno` skips at those boundaries. A contiguous selection straddling such a gap keeps a **correct** `start..end` (blob coords are monotonic), and the cached excerpt inserts a `… N lines unchanged …` marker at the gap (N = the `new_lineno` delta across the boundary minus 1). Render (Phase 70) re-resolves from a fresh git2 tree→blob read, so the cache is an **offline fallback only** — gap-crossing is allowed, never rejected.
- Keep it **pure** (no IPC, no mutation), tested exactly like `diff-anchor.test.ts`.

**Why a separate file, not extending `diff-anchor.ts`:** the two adapters share almost no logic once D-02/D-04 diverge (no `resolveSide`, no `prefixLine`, flat list vs. single hunk). A sibling module avoids conditionalizing the shipped, tested diff adapter. (Final call delegated to planner per CONTEXT.md, but this is the low-risk default.)

---

## 4. Selection State Model (D-01)

- A plain click sets a single-line selection (anchor line). Shift-click extends to a **contiguous span** (mirrors GitHub `#Lstart-Lend`). No discontiguous `Set` semantics needed for the *result* — the persisted anchor is a single `start..end`.
- Implementation can still track endpoints as `{ anchorIndex, focusIndex }` (or `anchorNewLineno`/`focusNewLineno`) and derive the contiguous span; selectable endpoints are new-side lines only (D-02). A span may *pass over* Delete lines and gaps visually, but they are excluded from the persisted range/excerpt.
- **Ownership:** Two viable shapes (CONTEXT.md "Claude's Discretion"): (a) local `$state` in `FullFileView` with an `oncommentlines`-equivalent callback bubbling up to the host that mounts the composer; or (b) parent-owned `$state` threaded down, mirroring the diff path's DiffPanel-owned selection. Option (a) is simpler given the flat model and the fact that no other component shares full-file selection; option (b) is more consistent with the diff path. Either satisfies D-01.
- Clear selection + composer on submit; rely on the existing `session-changed` event → panel reload (mostly-silent-success convention). Optional toast at planner's discretion.

---

## 5. Pitfalls / Landmines

1. **Don't copy the `isMerge` disable** from HunkView into the full-file affordance (L-05). This is the single most likely accidental regression because the affordance is otherwise a near-copy.
2. **`new_lineno` is the only valid coordinate** for full-file. Reading `old_lineno` anywhere in the full-file path is a bug (D-02 + L-01). The diff adapter's `resolveSide` machinery does not belong here.
3. **Don't reuse `prefixLine`** — full-file excerpts are plain content (D-04). Carrying `+`/`-`/space prefixes would corrupt the Phase 70 language-fenced render contract (DOC-02).
4. **Thread `commitOid` + `repoPath`** — they are not current `FullFileView`/`DiffViewer` props. Missing either silently breaks `save_draft_comment`/`add_comment`.
5. **Zero-hunk (unchanged) file** → `allLines` is empty → nothing selectable → the Comment affordance simply never appears. No special-case UI (CONTEXT.md). Don't throw.
6. **Selection highlight color must be a theme `--color-*` var** (CLAUDE.md: never inline colors). Add an `isSelected` arg to `lineBackground` like HunkView.
7. **Gap N computation:** the `… N lines unchanged …` count is derived from the `new_lineno` jump across a dropped-hunk boundary, not from index arithmetic (indices are contiguous in the flat list; line numbers are what skip).

---

## 6. Validation Architecture

Nyquist is enabled — this section is the source for VALIDATION.md. Each observable maps to the lowest sufficient validation level. The pure adapter carries most of the risk and is cheapest to test, so it gets the densest coverage (à la `diff-anchor.test.ts`).

| # | Observable behavior | Decision/Req | Validation level | What proves it |
|---|---------------------|--------------|------------------|----------------|
| V1 | Single-line + contiguous span selection produces `anchor.start_line/end_line = min/max new_lineno` | D-01, L-01, ANCH-02 | **Pure unit** (`full-file-anchor.test.ts`) | Adapter returns `{ source:"FullFile", side:"New", start_line, end_line }` for a known flat-line fixture |
| V2 | Delete lines are excluded from range AND excerpt | D-02 | **Pure unit** | A span passing over a `new_lineno:null` line yields a range/excerpt with that line absent |
| V3 | Cached excerpt is plain new-side content (no `+`/`-`/space prefixes) | D-04 | **Pure unit** | Excerpt string equals the selected lines' `content` joined by `\n`, no prefix chars |
| V4 | Gap-crossing selection keeps correct blob range and inserts `… N lines unchanged …` marker | D-03 | **Pure unit** | Fixture with a `new_lineno` jump → range spans the gap; excerpt contains the marker with correct N |
| V5 | Empty/zero-hunk file exposes no affordance and never throws | CONTEXT discretion | **Component test** | `FullFileView` with empty `allLines` renders no Comment button |
| V6 | Click + shift-click drives a contiguous selection; only new-side lines are selectable endpoints | D-01, D-02 | **Component test** (`FullFileView.test.ts`) | Simulated click then shift-click highlights the contiguous span; clicking a Delete line is a no-op endpoint |
| V7 | Affordance + composer submit calls `add_comment` with the FullFile anchor + plain excerpt | ANCH-02, L-02 | **Component test** (mock `safeInvoke`) | On submit, `add_comment` invoked once with `{ anchor.source:"FullFile", cachedExcerpt }` |
| V8 | Draft persists on change via `save_draft_comment` (300ms debounce) | L-03 | **Component test** (fake timers) | After input + 300ms, `save_draft_comment` called with current `text` + anchor |
| V9 | Comment survives an app restart (round-trip persistence) | ANCH-02, L-01, L-03 | **Backend integration** | Reuse/extend the existing `Source::FullFile` round-trip (`review.rs:1257`); assert a full-file anchor written then re-read from the session store is byte-identical |
| V10 | Merge commits ARE valid for full-file capture (no disable) | L-05 | **Component test (negative)** | With `isMerge=true`, the full-file Comment affordance is enabled (asserts Phase 67's disable was NOT copied) |

**Test scaffolding to reuse:** `diff-anchor.test.ts` provides `addLine`/`deleteLine`/`contextLine`/`file` fixture builders — clone them for `full-file-anchor.test.ts`. Component tests follow the existing `CommentComposer.test.ts` mock-`safeInvoke` pattern.

**TDD note:** TDD mode is ON. V1–V4 (the pure adapter) are textbook RED→GREEN→REFACTOR candidates — defined input/output, no IO. V5–V8, V10 are component-level (test-after or test-alongside is acceptable; the adapter is the unit that must be TDD'd). V9 is an extension of an existing passing Rust test.

---

## 7. Recommended Plan Shape (input to gsd-planner — non-binding)

1. **Pure full-file adapter** (`src/lib/full-file-anchor.ts` + `full-file-anchor.test.ts`) — TDD. Covers V1–V4. No UI. (D-01..D-04, L-01)
2. **Full-file selection + affordance + composer wiring** in `FullFileView.svelte` / `DiffViewer.svelte` (+ host threading of `commitOid`/`repoPath`), reusing/adapting `CommentComposer` via injected captured result. Covers V5–V8, V10. (D-01, D-02, L-02, L-03, L-05)
3. (If the planner separates it) **persistence/restart validation** extending the existing round-trip test. Covers V9. (L-01, L-03, L-04)

Two plans (adapter; UI+wiring) is the natural decomposition, with V9 folded into plan 2's verification. The planner owns the final split.

## RESEARCH COMPLETE

Phase 68 is frontend-only: the backend (`add_comment`, `save_draft_comment`, frozen `Source::FullFile` schema, round-trip test) shipped in Phases 65/67. The work is a pure full-file capture adapter (sibling of `diff-anchor.ts`, diverging on D-02 delete-exclusion and D-04 plain-content excerpt, with D-03 gap markers driven by the `context_lines(100_000)` full-file diff) plus net-new contiguous click/shift-click selection state in `FullFileView` and reuse of the existing inline `CommentComposer`. Key landmines: do NOT copy HunkView's `isMerge` disable (L-05), use `new_lineno` only, no `prefixLine`, and thread `commitOid`/`repoPath` down to the composer. A 10-item Validation Architecture (§6) feeds VALIDATION.md, with the pure adapter (V1–V4) as the TDD core.
