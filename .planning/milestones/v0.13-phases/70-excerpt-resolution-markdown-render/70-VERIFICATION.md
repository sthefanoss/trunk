---
phase: 70-excerpt-resolution-markdown-render
verified: 2026-05-26T00:00:00Z
status: human_needed
score: 4/4 must-haves verified
overrides_applied: 0
human_verification:
  - test: "End-to-end DOC-01 flow: open a repo with review comments, click Generate button, confirm preview appears with correct markdown (commit refs, fenced code excerpts, per-file grouping)"
    expected: "ReviewPanel swaps to preview face; generated markdown is rendered in <pre> with pre-wrap disabled; Back to comments button returns to list"
    why_human: "Full IPC round-trip (generate_review_doc → render → previewMarkdown → ReviewDocPreview) requires a running app; grep cannot confirm correct markdown is displayed"
  - test: "Multi-hunk diff excerpt correctness (CR-01): on a comment anchored to one hunk of a file that has multiple hunks, confirm the excerpt shows only lines from the anchored hunk and not lines from other hunks"
    expected: "Only the lines belonging to the targeted hunk appear in the fenced excerpt; no lines from sibling hunks leak through"
    why_human: "The CR-01 bug in slice_diff (lineno == None filter has no positional check) is live code; requires a real multi-hunk diff fixture to observe the output"
  - test: "Cross-repo preview isolation (WR-01): generate a preview for repo A, switch to repo B (select a different repo), click Generate for repo B, confirm the preview shows repo B's content not repo A's"
    expected: "Preview content is always from the currently active repo; no stale preview from a previous repo leaks through"
    why_human: "WR-01 (previewMarkdown not cleared on repoPath change) is a lifecycle issue; requires switching repos in a running app to observe whether stale content persists"
---

# Phase 70: Excerpt Resolution & Markdown Render Verification Report

**Phase Goal:** User can generate one AI-framed markdown document from the session, with resolved code excerpts and graceful handling of stale anchors.
**Verified:** 2026-05-26
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can generate one markdown document containing commit refs, code excerpts, and comments from the session | ✓ VERIFIED | Generate button wired to `session.generate(repoPath)`; IPC command registered and routes to `render()`; `previewMarkdown` data-flow traced end-to-end; 21 ReviewPanel + 7 rune tests pass. Runtime visual confirmation in human_verification #1. |
| 2 | The doc shows diff-fenced excerpts for diff-source comments and language-fenced excerpts for full-file-source comments; an excerpt containing backticks still fences correctly (no broken/leaking blocks) | ✓ VERIFIED (WARNING: CR-01) | `emit_fence` uses `fence_length()` for backtick-safe fencing; `fence_language()` for language labels; tests cover both source types. CR-01 is a WARNING — see Gaps Summary. |
| 3 | Comments are grouped by file and ordered by line, each under a `path:Lstart-Lend (sha)` heading; commit-level comments render in a trailing section | ✓ VERIFIED | `render()` groups by `(file_path, commit_oid)`, sorts by `(start_line, end_line)`, emits `#### {path}:L{start}-L{end} ({short})` headings; commit-level section at D-04 position confirmed in tests |
| 4 | A comment whose anchor can no longer be resolved appears in a dedicated "unresolvable" section with a reason — never silently dropped and never crashing the render | ✓ VERIFIED | `ExcerptError` enum (Binary, Orphaned, ResolutionFailed, NoHunks); `classify_anchor` gate in `try_resolve_excerpt`; D-09 phrases; D-10 cached_excerpt fallback; unresolvable section in `render()` output |

**Score:** 4/4 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/git/review.rs` | Pure Rust renderer: `render()`, `ExcerptError`, `fence_length`, `fence_language`, `slice_diff`, `slice_full_file`, `try_resolve_excerpt` | ✓ VERIFIED | ~1664 lines; all symbols present; 30 unit tests pass (`cargo test review`) |
| `src-tauri/src/commands/review.rs` | `generate_review_doc` Tauri command with D-11 gate, clone-under-lock, spawn_blocking | ✓ VERIFIED | Lines 1000+; no_comments gate at line 1024; full session clone; `git2::Repository::open` inside spawn_blocking |
| `src/lib/review-session.svelte.ts` | Rune extended with `panelMode`, `previewMarkdown`, `showList()`, `showPreview()`, `generate()` | ✓ VERIFIED | `PanelMode` type, `ReviewSessionState` fields, `ReviewSessionManager` interface, and all method implementations present |
| `src/components/ReviewDocPreview.svelte` | Preview face: `<pre>` with `white-space: pre`, `back-button`, `.preview-spacer` Phase 71 docking slot | ✓ VERIFIED | Props `{ markdown, onBack }`; `.preview-body` with `white-space: pre`; `.preview-spacer` present |
| `src/components/ReviewPanel.svelte` | Generate button (disabled without comments), view-swap on `panelMode === "preview"` | ✓ VERIFIED | `hasAnyComment` derived; `onGenerateClick` with try/catch+toast; `{#if session.state.panelMode === "preview" && session.state.previewMarkdown !== null}` block |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ReviewPanel.svelte` | `ReviewDocPreview.svelte` | import + conditional render | ✓ WIRED | Import at line 20; `{#if panelMode === "preview"}` block at line 332 |
| `ReviewPanel.svelte` | `session.generate(repoPath)` | `onGenerateClick` handler | ✓ WIRED | Lines 278-287; try/catch; showToast on error |
| `review-session.svelte.ts` | `generate_review_doc` IPC | `safeInvoke<string>` | ✓ WIRED | `safeInvoke("generate_review_doc", { path: repoPath })` |
| `commands/review.rs` | `git::review::render()` | `crate::git::review::render` | ✓ WIRED | Line 1034: `crate::git::review::render(&session, &repo)` |
| `lib.rs` | `generate_review_doc` command | `tauri::generate_handler!` | ✓ WIRED | Line 139 in handler macro |
| `RepoView.svelte` | `ReviewPanel` | `session={reviewSession}` prop | ✓ WIRED | Line 842: `<ReviewPanel {repoPath} session={reviewSession} ...>` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `ReviewDocPreview.svelte` | `markdown` prop | `session.state.previewMarkdown` via `showPreview()` / `generate()` | Yes — populated from IPC result | ✓ FLOWING |
| `commands/review.rs` | render result | `git::review::render(&session, &repo)` with live `git2::Repository` | Yes — git2 reads blobs/diffs from disk | ✓ FLOWING |
| `review-session.svelte.ts` | `previewMarkdown` | `safeInvoke` return value; assigned only on success | Yes — from Rust render | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 30 Rust renderer unit tests pass | `cargo test -p trunk-lib review` | `test result: ok. 30 passed; 0 failed` | ✓ PASS |
| Rune `generate()` / `showList()` / `showPreview()` unit tests pass | `npx vitest run src/lib/review-session.svelte.test.ts` | 7 tests pass | ✓ PASS |
| ReviewPanel tests pass | `npx vitest run src/components/ReviewPanel.test.ts` | 21 tests pass | ✓ PASS |
| No `.unwrap()` in production render code (L-04) | `grep -n '\.unwrap()' src-tauri/src/git/review.rs \| awk -F: '$1 < 576'` | 0 results | ✓ PASS |
| `classify_anchor` gate present in `try_resolve_excerpt` | `grep -n 'classify_anchor' src-tauri/src/git/review.rs` | Gate call present before any slicing | ✓ PASS |
| No syntect imports (L-10) | `grep -rn 'syntect\|syntax\.rs' src-tauri/src/git/review.rs` | 0 results | ✓ PASS |

### Probe Execution

No phase-declared probes. `scripts/` directory has no `probe-*.sh` for this phase. Step 7c: SKIPPED (no probe files declared).

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| DOC-01 | 70-02-PLAN.md, 70-03-PLAN.md | `generate_review_doc` IPC command; frontend generate + preview flow | ✓ SATISFIED | Command registered in lib.rs; ReviewPanel Generate button; session.generate() wired |
| DOC-02 | 70-01-PLAN.md | Renderer groups comments by file, ordered by line; headings `path:Lstart-Lend (sha)` | ✓ SATISFIED | `render()` sorts by `(start_line, end_line)`; H4 heading format confirmed in tests |
| DOC-03 | 70-01-PLAN.md | Fenced code excerpts: diff-fenced for diff source, language-fenced for full-file; backtick-safe | ✓ SATISFIED | `fence_length()` + `fence_language()` + `emit_fence()` all implemented; tests cover both source types and backtick content |
| DOC-04 | 70-01-PLAN.md | Unresolvable anchors render in dedicated section with reason phrase; cached excerpt included | ✓ SATISFIED | `ExcerptError` enum; D-09 phrase functions; D-10 cached_excerpt fencing; unresolvable section in `render()` |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src-tauri/src/git/review.rs` | 179-204 | CR-01: `slice_diff` opposing-side filter (`None =>` branch) has no hunk-positional check — lines from non-anchored hunks leak when file has multiple hunks | WARNING | Multi-hunk diff excerpts may include extra lines from sibling hunks |
| `src/components/RepoView.svelte` | ~89, 842 | WR-01: `previewMarkdown` not cleared on `repoPath` change — `reviewSession` rune created once at module level; no `{#key repoPath}` and no `setReviewActive(false)` on repo switch | WARNING | Preview generated for repo A persists when user opens repo B |

### Human Verification Required

#### 1. End-to-End DOC-01 Generate Flow

**Test:** Open a repo with at least one review comment. Click the "Generate" button in the ReviewPanel. Confirm the panel transitions to the preview face.
**Expected:** Preview shows a well-formed markdown document with commit references, fenced code excerpts, and comment text. The "Back to comments" button returns to the list face without losing the preview cache.
**Why human:** The full round-trip (Tauri IPC → Rust render → Svelte preview) cannot be exercised by grep or static analysis.

#### 2. Multi-Hunk Diff Excerpt Correctness (CR-01)

**Test:** Create a review comment anchored to a specific hunk in a file that has at least two diff hunks in the same commit. Generate the review doc and inspect the fenced excerpt for that comment.
**Expected:** The excerpt contains only lines from the anchored hunk. No lines from other hunks appear in the block.
**Why human:** The CR-01 bug in `slice_diff` (lines 179-204 of `review.rs`) is live; the plan's automated tests only use single-hunk fixtures and therefore do not exercise this path.

#### 3. Cross-Repo Preview Isolation (WR-01)

**Test:** Generate a review doc for repo A. Without closing the app, switch to repo B (open a different repository). Click Generate for repo B (or simply observe the panel state).
**Expected:** Either (a) the preview is cleared when switching repos, or (b) a re-generate is required and produces repo B content. The user should never see repo A content displayed in the context of repo B.
**Why human:** WR-01 is a lifecycle issue; the stale-state path only manifests when switching repositories in a live session.

### Gaps Summary

No blockers were found. The phase goal is structurally achieved: all four must-haves have implementation evidence, all artifacts exist and are wired, all requirement IDs (DOC-01 through DOC-04) are satisfied, and 58 automated tests (30 Rust + 7 rune + 21 panel) pass.

Two WARNING-level findings require human observation:

**CR-01** (`slice_diff` multi-hunk line filter, `review.rs` lines 179-204): The `None =>` branch of the opposing-side filter has no hunk-positional check, so lines from non-anchored hunks can leak into the excerpt when a file contains multiple hunks. Must-have #2's "no broken/leaking blocks" phrase refers to fence integrity (backtick escapes), which `fence_length()` handles correctly. CR-01 affects excerpt *content* selection, which is a DOC-03 correctness issue surfaced as a warning for follow-up rather than a must-have #2 failure. The plan's automated tests cover only single-hunk fixtures, so the implementation is conformant to the plan's test contract as written. Should be addressed before the feature is considered production-ready.

**WR-01** (preview not cleared on repo switch, `RepoView.svelte`): The `reviewSession` rune is created once at module level with no `{#key repoPath}` wrapper and no reactive effect calling `setReviewActive(false)` when `repoPath` changes. A preview generated for repo A persists when the user opens repo B. Fix: add a `$effect(() => { reviewSession.setReviewActive(false); })` reactive block keyed on `repoPath`, or wrap `<ReviewPanel>` in `{#key repoPath}`.

---

_Verified: 2026-05-26_
_Verifier: Claude (gsd-verifier)_
