# Phase 70: Excerpt Resolution + Markdown Render — Pattern Map

**Mapped:** 2026-05-26
**Files analyzed:** 8 (5 new/modified Rust + 3 frontend) + diff-anchor/full-file-anchor flagged
**Analogs found:** 8 / 8 (every Phase 70 file has a strong codebase analog)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src-tauri/src/git/review.rs` (NEW) | service (pure render) | transform (session+repo → String) | `src-tauri/src/commands/review.rs::classify_anchor` + `resolve_all` (lines 296–405) + `src-tauri/src/commands/diff.rs::walk_diff` (lines 177–249) | exact (composed) — pure git2-backed transform over the same schema |
| `src-tauri/src/git/mod.rs` (MODIFIED) | module registration | static | itself (lines 1–5) | exact — add one `pub mod review;` line, mirrors `pub mod review_store;` |
| `src-tauri/src/commands/review.rs` (MODIFIED — add `generate_review_doc`) | controller (Tauri IPC adapter) | request-response (read-only) | `commands/review.rs::list_session_comments` (lines 907–932) and `resolve_session_comments` (lines 942–977) | exact — read-only, no `_inner`, clone-under-lock then `spawn_blocking` |
| `src-tauri/src/lib.rs` (MODIFIED — register handler) | wiring | static | itself (lines 124–138, review command registrations) | exact — append one line inside `invoke_handler` |
| `src/components/ReviewPanel.svelte` (MODIFIED — Generate button + preview swap) | component | event-driven (click → IPC → state swap) | itself (header + group rendering, lines 339–390; safeInvoke usage, lines 197–220) | exact (extends current panel) |
| `src/components/ReviewDocPreview.svelte` (NEW — RESEARCH Q4 recommendation) | component (presentation) | one-way prop render | `src/components/ReviewPanel.svelte` style block (lines 519–657) | role-match — same theme-token + scrollable-container shape |
| `src/lib/review-session.svelte.ts` (MODIFIED — add `panelMode` + `previewMarkdown` + `generate`) | rune (state factory) | request-response action | itself (lines 18–75; existing `RightPaneMode` + `showPanel/showDiff` + `jumpTo`) | exact — extend the existing manager interface |
| `src/components/ReviewPanel.test.ts` (MODIFIED — new tests) | test | request-response (mocked IPC) | itself (lines 1–100, command-aware `safeInvoke` mock) | exact — same dispatcher style |
| **Backend tests** in `src-tauri/src/git/review.rs` `#[cfg(test)] mod tests` (NEW) | test | transform | `commands/review.rs:2065-2089` (`FileRepo` + `commit_with_file` helper) | exact — lift the helper verbatim |
| `src/lib/diff-anchor.ts` / `src/lib/full-file-anchor.ts` (flagged but **NOT** modified) | — | — | — | **NO RETROFIT REQUIRED.** RESEARCH Item 2 verified line numbers are emitted by libgit2 in Rust; TS does not count lines. Planner: confirm and skip. |

## Pattern Assignments

### `src-tauri/src/git/review.rs` (NEW — pure render)

**Analogs:**
- `src-tauri/src/commands/review.rs:324-371` — `classify_anchor` (the gate the renderer reuses)
- `src-tauri/src/commands/diff.rs:177-249` — `walk_diff` (the `diff.foreach` callback pattern the diff-replay slice mirrors)
- `src-tauri/src/commands/diff.rs:397-417` — `diff_commit_inner` (the `diff_tree_to_tree(parent, commit)` + root-commit guard pattern)
- `src-tauri/src/commands/review.rs:2065-2089` — `FileRepo` + `commit_with_file` (the test fixture pattern to lift)

**Module placement pattern** — purity rule from analog:

`commands/review.rs:296-405` keeps `OrphanReason`, `classify_anchor`, `resolve_all` as pure functions taking `(&Anchor, &git2::Repository)` / `(&[Comment], &git2::Repository)` and returning data — no `tauri::*` imports, no async, no I/O. `git/review.rs` follows the SAME purity rule. Public surface:

```rust
// In git/review.rs — pure, no tauri::* imports.
pub fn render(session: &ReviewSession, repo: &git2::Repository) -> String;
```

**Gate-then-resolve pattern (the load-bearing structure)** — lifted from `classify_anchor`:

`commands/review.rs:339-346` shows the side-aware tree pick and root-commit guard. Render calls `classify_anchor` FIRST per comment; on `Ok(())` attempts fresh excerpt re-resolution. On `Err(reason)` routes to the unresolvable section with the D-09 phrasing.

```rust
// In git/review.rs — every per-comment dispatch goes through this guard.
match (&comment.anchor, classify_anchor(anchor, repo)) {
    (Some(anchor), Ok(())) => match try_resolve_excerpt(repo, anchor) {
        Ok(excerpt) => emit_anchored(out, comment, anchor, &excerpt),
        Err(ExcerptError::Binary) => emit_binary_placeholder(out, comment, anchor),
        Err(other)                => stash_unresolvable(comment, other),
    },
    (Some(_anchor), Err(reason))  => stash_unresolvable(comment, ExcerptError::Orphaned(reason)),
    (None,          _)            => /* commit-level */
}
```

The `classify_anchor` call is **mandatory** — never call `slice_blob` / `slice_diff` without it (Pitfall 1: root-commit `Side::Old` causes `commit.parent(0)` to error).

**Side-aware tree pick** (lift verbatim from `commands/review.rs:339-346`):

```rust
let tree = match anchor.side {
    Side::New => commit.tree().map_err(|_| OrphanReason::FileGone)?,
    Side::Old => commit
        .parent(0)
        .map_err(|_| OrphanReason::FileGone)?
        .tree()
        .map_err(|_| OrphanReason::FileGone)?,
};
```

**Line-counting convention (L-06)** — lift the `String::from_utf8_lossy(blob.content()).lines()` semantic from `commands/review.rs:358`:

```rust
// commands/review.rs:355-358 — the canonical line-counting convention.
// `str::lines()` does NOT count a trailing newline as a final empty line, so
// end_line == line_count is in-range.
let line_count = String::from_utf8_lossy(blob.content()).lines().count() as u32;
```

Render MUST mirror this for the FullFile blob slice. **CRLF→LF body normalization** (L-06 second clause) — RESEARCH Item 2 Option (a): replace `\r\n` with `\n` on the lossy string before slicing.

**Diff replay-slice (L-02 + Phase 67 L-03)** — lift the `diff.foreach` callback shape from `commands/diff.rs:182-243`:

```rust
// commands/diff.rs:222-242 — the line callback that the render's diff-replay-slice mirrors.
Some(&mut |_delta, _hunk, line| {
    let origin = match line.origin() {
        '+' => DiffOrigin::Add,
        '-' => DiffOrigin::Delete,
        _ => DiffOrigin::Context,
    };
    let content = String::from_utf8_lossy(line.content()).into_owned();
    // ... push line with origin + content + old_lineno + new_lineno ...
    true
}),
```

And the `DiffOptions::pathspec` filter from `commands/diff.rs:354-356` to scope `diff_tree_to_tree` to one file:

```rust
let mut opts = git2::DiffOptions::new();
opts.pathspec(file_path);
```

Plus the root-commit guard from `commands/diff.rs:410-414`:

```rust
let diff = if commit.parent_count() == 0 {
    repo.diff_tree_to_tree(None, Some(&commit_tree), Some(&mut opts))?
} else {
    let parent_tree = commit.parent(0)?.tree()?;
    repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), Some(&mut opts))?
};
```

The render combines these three: pathspec-filter to the anchor's file, replay the diff, walk lines, keep ones overlapping the anchor's `(side, start_line, end_line)` AND keep opposing-side `-`/`+` rows per Phase 67 L-03. **Pitfall 2:** if the walk produces an empty `String`, return `Err(ExcerptError::NoHunks)` — do NOT emit an empty fence body.

**Binary detection (L-05)** — lift the same `is_binary()` call already used at `commands/diff.rs:190`:

```rust
// commands/diff.rs:190 — codebase pattern for binary detection.
let is_binary = delta.old_file().is_binary() || delta.new_file().is_binary();
```

Render uses the simpler direct-blob form:

```rust
if blob.is_binary() {
    return Err(ExcerptError::Binary);  // → [binary file, no excerpt]
}
```

**ExcerptError type (RESEARCH Item 7)** — render-only enum that does NOT cross the IPC wire:

```rust
// In git/review.rs — pure, never serialized to TS.
enum ExcerptError {
    Binary,                    // → [binary file, no excerpt] placeholder
    Orphaned(OrphanReason),    // wraps Phase 69's enum (D-09 phrasing)
    ResolutionFailed,          // generic "could not be re-resolved"
    NoHunks,                   // diff replay emitted zero lines
}
```

Do NOT extend `OrphanReason` — it crosses the IPC wire and would force TS-side churn.

**Language fence table (L-07)** — hand-rolled, distinct from `syntax.rs::fallback_extension`:

`src-tauri/src/git/syntax.rs:41-46` maps to syntect syntax IDs (`ts → js`), NOT to markdown fence tags. Render needs the AI-facing name (`typescript`, not `js`). Hand-roll the table per RESEARCH Item 3 inside `review.rs`. The `extension_from_path` helper at `syntax.rs:60-65` IS lift-able (L-10-safe — no syntect call).

**Fence length formula (L-03)** — linear scan, no allocation, per RESEARCH Item 5:

```rust
fn fence_length(body: &str) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for b in body.as_bytes() {
        if *b == b'`' {
            current += 1;
            if current > longest { longest = current; }
        } else {
            current = 0;
        }
    }
    std::cmp::max(3, longest + 1)
}
```

**Test fixture pattern (lift verbatim)** — `commands/review.rs:2065-2089`:

```rust
// commands/review.rs:2065-2089 — golden-test fixture
struct FileRepo {
    _dir: TempDir,
    repo: Repository,
    root: Oid,      // A: empty tree, no files
    with_file: Oid, // B: foo.rs present (3 lines)
}

fn commit_with_file(
    repo: &Repository,
    message: &str,
    parents: &[Oid],
    path: &str,
    content: &str,
) -> Oid {
    let blob_oid = repo.blob(content.as_bytes()).unwrap();
    let mut builder = repo.treebuilder(None).unwrap();
    builder
        .insert(path, blob_oid, git2::FileMode::Blob.into())
        .unwrap();
    let tree = repo.find_tree(builder.write().unwrap()).unwrap();
    // ... parents, sig, repo.commit ...
}
```

The Phase 70 `#[cfg(test)] mod tests` block uses the same helper for `generate_doc_has_all_sections`, `diff_source_uses_diff_fence`, `unresolvable_uses_cached_excerpt`, etc. (full list in RESEARCH lines 1099–1110).

---

### `src-tauri/src/git/mod.rs` (MODIFIED — register module)

**Analog:** itself (lines 1–5).

```rust
// src-tauri/src/git/mod.rs (current contents, lines 1-5)
pub mod graph;
pub mod repository;
pub mod review_store;
pub mod syntax;
pub mod types;
```

**Action:** insert `pub mod review;` in alphabetical position (between `repository` and `review_store`). One-line addition, no other changes.

---

### `src-tauri/src/commands/review.rs` (MODIFIED — add `generate_review_doc`)

**Analog:** `commands/review.rs:907-977` — `list_session_comments` (read-only, no `_inner`) + `resolve_session_comments` (read-only, `spawn_blocking` for git2). The new command is structurally `resolve_session_comments` with a different inner function.

**RESEARCH Q1 recommendation:** SKIP `_inner`. The existing `_inner` pattern exists to make disk behavior testable via `TempDir`; this command has no disk I/O. Test `git::review::render` directly. Document the deviation in the command doc-comment.

**Imports pattern** (the file already has these; no new imports needed at the file level beyond `crate::git::review`):

```rust
// commands/review.rs:14-22 — existing imports the new command reuses.
use crate::error::TrunkError;
use crate::git::types::{Comment, DraftComment, ReviewSession};
use crate::state::{RepoState, ReviewSessionsState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};
```

**`canonical_repo_path` pattern** (lift verbatim from `commands/review.rs:61-69`):

```rust
fn canonical_repo_path(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<PathBuf, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    std::fs::canonicalize(path_buf).map_err(|e| TrunkError::new("io", e.to_string()))
}
```

The new command calls this with the same `not_open` semantics.

**Clone-comments-under-lock + `spawn_blocking` core pattern** (lift from `resolve_session_comments`, `commands/review.rs:942-977`):

```rust
// commands/review.rs:942-977 — the structure the new command mirrors verbatim.
#[tauri::command]
pub async fn resolve_session_comments(
    path: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
) -> Result<Vec<CommentResolution>, String> {
    let state_map = state.0.lock().unwrap().clone();
    let canonical =
        canonical_repo_path(&path, &state_map).map_err(|e| serde_json::to_string(&e).unwrap())?;

    // Clone .comments out under the lock; never hold lock across spawn_blocking.
    let comments = {
        let map = sessions.0.lock().unwrap();
        map.get(&canonical)
            .ok_or_else(|| {
                serde_json::to_string(&TrunkError::new(
                    "no_session",
                    "No active review session for this repository",
                ))
                .unwrap()
            })?
            .comments
            .clone()
    };

    let result = tauri::async_runtime::spawn_blocking(
        move || -> Result<Vec<CommentResolution>, TrunkError> {
            let repo = git2::Repository::open(&path).map_err(TrunkError::from)?;
            Ok(resolve_all(&comments, &repo))
        },
    )
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    Ok(result)
}
```

**Differences for `generate_review_doc`:**
1. Clone the **whole `ReviewSession`** (not just `.comments`) — render needs `commits` for the refs list.
2. D-11 zero-comment gate before `spawn_blocking`: `if session.comments.is_empty() { return Err(... "no_comments" ...) }`.
3. Inside `spawn_blocking`: open repo + call `crate::git::review::render(&session, &repo)`, return `String`.
4. **No `session-changed` emit** (read-only, render-only).
5. Path resolution: the canonical-key clone uses canonical PathBuf to look up the session; the actual `git2::Repository::open` uses `path` (the original arg) to mirror `resolve_session_comments:968` exactly — git2 doesn't care about canonical vs symlinked.

**Error contract (RESEARCH Item 7):**

| Code | When |
|------|------|
| `not_open` | `canonical_repo_path` failure |
| `no_session` | session lookup miss |
| `no_comments` | D-11 gate (≥1 contract) |
| `spawn_error` | `spawn_blocking` join failure |
| (git2 error code via `TrunkError::from`) | `Repository::open` failure |

Per-comment failures (binary / blob-read / NoHunks / orphan) **NEVER** error — they route INTO the returned markdown.

---

### `src-tauri/src/lib.rs` (MODIFIED — register handler)

**Analog:** itself, lines 124–138 (review command registrations).

```rust
// src-tauri/src/lib.rs:124-138 (existing review handler block)
commands::review::start_review_session,
commands::review::resume_review_session,
commands::review::end_review_session,
commands::review::get_review_session_status,
commands::review::seed_review_range,
commands::review::add_review_commit,
commands::review::remove_review_commit,
commands::review::list_session_commits,
commands::review::add_comment,
commands::review::save_draft_comment,
commands::review::add_commit_comment,
commands::review::edit_comment,
commands::review::delete_comment,
commands::review::list_session_comments,
commands::review::resolve_session_comments,
```

**Action:** append `commands::review::generate_review_doc,` to this list. One-line addition.

---

### `src/components/ReviewPanel.svelte` (MODIFIED — Generate button + preview swap)

**Analog:** itself.

**Header button pattern** — the existing per-commit "Add note" button is the closest local analog for a header-area affordance (lines 369–389):

```svelte
<!-- src/components/ReviewPanel.svelte:369-390 (per-group "Add note" button) -->
<button
  type="button"
  class="flex items-center"
  onclick={() => openAddNote(group.oid)}
  style="
    gap: 4px;
    background: transparent;
    color: var(--color-text-muted);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    padding: 2px 4px;
    flex-shrink: 0;
    font-size: 12px;
  "
  onmouseenter={(e) => (e.currentTarget.style.background = "var(--color-hover)")}
  onmouseleave={(e) => (e.currentTarget.style.background = "transparent")}
>
  <MessageSquarePlus size={14} />
  <span>Add note</span>
</button>
```

**Disabled-state derivation** — lifted from existing `draftValid`-driven `disabled={!draftValid}` Save buttons (line 414):

```svelte
disabled={!draftValid}
```

The new Generate button uses `disabled={!hasAnyComment}` where `hasAnyComment` already exists at line 123:

```ts
// src/components/ReviewPanel.svelte:123 (existing $derived)
const hasAnyComment = $derived(comments.length > 0);
```

D-01 tooltip: `title="Add at least one comment to generate"` when disabled (HTML standard, no custom widget needed).

**`safeInvoke` + try/catch + `showToast` pattern** (lift from `saveAddNote` lines 241–254):

```ts
// src/components/ReviewPanel.svelte:241-254 (the call-site shape for the new Generate handler)
async function saveAddNote(oid: string) {
    if (!draftValid) return;
    const text = draftText;
    cancelComposer();
    try {
        await safeInvoke("add_commit_comment", {
            path: repoPath,
            commitOid: oid,
            text,
        });
    } catch (e) {
        showToast((e as TrunkError).message ?? "Failed to add note", "error");
    }
}
```

The new `onGenerateClick` handler follows the same shape with `safeInvoke<string>("generate_review_doc", { path: repoPath })`, then calls the rune's `showPreview(md)` action.

**View-swap pattern (D-02)** — the existing `{#if groups.length === 0}` / `{:else if !hasAnyComment}` / `{#if groups.length > 0}` conditional block (lines 323–337, 339+) is the closest existing inline-conditional analog. The new preview swap is at a SIBLING level:

```svelte
<!-- Sketch — wraps the existing comment-list rendering block. -->
{#if previewMode}
  <ReviewDocPreview markdown={previewMarkdown} onBack={() => session.showList()} />
{:else}
  <!-- existing groups rendering, unchanged -->
{/if}
```

`previewMode` and `previewMarkdown` come from the rune (see `review-session.svelte.ts` extension below).

**Theme-token compliance** — every existing style uses `var(--color-*)`. The new Generate button + preview view follow the same rule (CLAUDE.md "Never inline colors").

---

### `src/components/ReviewDocPreview.svelte` (NEW — RESEARCH Q4 recommendation)

**Analog:** `src/components/ReviewPanel.svelte:519-657` (the existing style block — same theme tokens, same `--color-*` palette, same monospace + scrollable shape).

**Recommended shape** (lifted from RESEARCH Item 6):

```svelte
<script lang="ts">
  interface Props {
    markdown: string;
    onBack: () => void;
  }
  let { markdown, onBack }: Props = $props();
</script>

<div class="preview-wrap">
  <header class="preview-header">
    <button type="button" onclick={onBack}>← Back to comments</button>
    <!-- Phase 71 docks Copy / Save buttons here. Leave the spacer. -->
    <span class="preview-spacer"></span>
  </header>
  <pre class="preview-body">{markdown}</pre>
</div>

<style>
  .preview-wrap {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    background: var(--color-bg);
    color: var(--color-text);
  }
  .preview-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    font-size: 12px;
  }
  .preview-spacer { flex: 1; }
  .preview-body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    margin: 0;
    padding: 12px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 12px;
    line-height: 1.5;
    white-space: pre;
    background: var(--color-bg);
    color: var(--color-text);
  }
</style>
```

**Why `<pre>` not rendered HTML:** the recipient is an AI coding agent; the user previews the raw text the AI will see. No new markdown-parsing dependency.

**Phase 71 forward-compat:** the `.preview-spacer` flex-1 cell explicitly reserves room for Copy / Save buttons on the right of the header — Phase 71 drops them in beside the spacer without restructuring.

---

### `src/lib/review-session.svelte.ts` (MODIFIED — add panelMode + previewMarkdown + generate)

**Analog:** itself (the existing rune structure, lines 18–75).

**Existing manager interface pattern** (lines 37–43) — extend with three new members:

```ts
// src/lib/review-session.svelte.ts:18-43 (current shape — extend, don't replace)
export type RightPaneMode = "panel" | "diff";

export interface ReviewSessionState {
    reviewActive: boolean;
    rightPaneMode: RightPaneMode;
    // ADD:
    panelMode: "list" | "preview";       // D-02 panel-internal view-state
    previewMarkdown: string | null;       // null when no preview generated
}

export interface ReviewSessionManager {
    state: ReviewSessionState;
    setReviewActive(active: boolean): void;
    showPanel(): void;
    showDiff(): void;
    jumpTo(comment: Comment, deps: JumpDeps): Promise<void>;
    // ADD:
    showList(): void;                            // panelMode = "list"
    showPreview(md: string): void;               // panelMode = "preview", cache md
    generate(repoPath: string): Promise<void>;   // safeInvoke + showPreview
}
```

**Existing factory pattern** (lines 45–75) — extend in the same shape:

```ts
// src/lib/review-session.svelte.ts:45-75 (current factory — extend the state literal + returned methods)
export function createReviewSession(): ReviewSessionManager {
    const state: ReviewSessionState = $state({
        reviewActive: false,
        rightPaneMode: "panel" as RightPaneMode,
        panelMode: "list" as const,         // ADD
        previewMarkdown: null,              // ADD
    });

    return {
        state,
        setReviewActive(active: boolean) { state.reviewActive = active; },
        showPanel() { state.rightPaneMode = "panel"; },
        showDiff()  { state.rightPaneMode = "diff"; },
        // ADD:
        showList()  { state.panelMode = "list"; /* keep previewMarkdown — regenerate is the only update path */ },
        showPreview(md) { state.previewMarkdown = md; state.panelMode = "preview"; },
        async generate(repoPath) {
            const md = await safeInvoke<string>("generate_review_doc", { path: repoPath });
            state.previewMarkdown = md;
            state.panelMode = "preview";
        },
        async jumpTo(comment, deps) { /* unchanged */ },
    };
}
```

**Reset semantics** — `endSession()` (or whatever the existing reset hook is) clears `previewMarkdown` and resets `panelMode` to `"list"`. The doc is a static snapshot of the just-ended session — no reason to keep it. Existing `setReviewActive(false)` is the natural reset hook; planner extends it (Discretion).

**Import for safeInvoke** — add `import { safeInvoke } from "./invoke.js";` (the rune does not currently import it; the `generate` action introduces the dependency).

---

### `src/components/ReviewPanel.test.ts` (MODIFIED — add Generate + preview-swap tests)

**Analog:** itself (lines 1–100).

**Command-aware mock dispatcher pattern** (lift the existing shape from lines 19–21 and 89–100):

```ts
// src/components/ReviewPanel.test.ts:19-21 (the mock shape — extend, don't replace)
vi.mock("../lib/invoke.js", () => ({
    safeInvoke: vi.fn(),
}));
```

Extend `installReads` to also accept `generate_review_doc: string` and route it in the dispatcher's `switch`:

```ts
// Sketch — extend src/components/ReviewPanel.test.ts:89-100
function installReads(opts: {
    commits?: SessionCommit[];
    comments?: Comment[];
    resolutions?: CommentResolution[];
    generateDoc?: string;            // ADD
}) {
    vi.mocked(safeInvoke).mockReset();
    vi.mocked(safeInvoke).mockImplementation((cmd: string) => {
        switch (cmd) {
            case "list_session_commits":  return Promise.resolve(opts.commits ?? []);
            case "list_session_comments": return Promise.resolve(opts.comments ?? []);
            case "resolve_session_comments": return Promise.resolve(opts.resolutions ?? []);
            case "generate_review_doc": return Promise.resolve(opts.generateDoc ?? "# stub\n");  // ADD
            // ... existing cases ...
            default: return Promise.resolve();
        }
    });
}
```

New tests (per RESEARCH lines 1119–1147):
- `generate button is disabled when no comments` — `installReads({ commits: [], comments: [], resolutions: [] })`, find button by role, assert `toBeDisabled()`.
- `generate click invokes generate_review_doc and swaps to preview` — `installReads({ ..., generateDoc: "# Code review: trunk\n\nfoo" })`, click, assert preview-body shows the string.
- `back to comments returns to list view` — after preview is showing, click back, assert comment file ref renders.

---

## Shared Patterns

### Authentication
**N/A.** Local desktop app, no auth (RESEARCH "Security Domain").

### Error Handling (Rust → TS wire)
**Source:** `src/lib/invoke.ts:1-28` (`safeInvoke<T>` parses Rust `Result<T, String>` where `String` is JSON-encoded `TrunkError`).
**Apply to:** Both the new Rust command's error returns AND the new TS handler's try/catch.

Rust side — error serialization (lift from any existing command, e.g. `commands/review.rs:915`):
```rust
.map_err(|e| serde_json::to_string(&e).unwrap())?
```

TS side — error parsing (lift from `ReviewPanel.svelte:209-220`):
```ts
try {
    const md = await safeInvoke<string>("generate_review_doc", { path: repoPath });
    // ...
} catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? "Failed to generate review doc", "error");
}
```

### Git2 ownership (lock-discipline)
**Source:** `src-tauri/src/state.rs` ("Store PathBuf ONLY — git2::Repository is not Sync.") + `commands/review.rs:942-977`.
**Apply to:** The new `generate_review_doc` command.

Pattern:
1. Clone the `ReviewSession` out of `sessions: State<'_, ReviewSessionsState>` under the lock.
2. Drop the lock.
3. Enter `spawn_blocking`.
4. Open `git2::Repository::open(&path)` INSIDE the closure.
5. Never hold the lock across `spawn_blocking`.

### serde naming
**Source:** `src-tauri/src/git/types.rs:288-345` (review schema) + CONTEXT "Established Patterns".
**Apply to:** Any new wire DTOs.

Rules:
- Review-schema enums (Source, Side, OrphanReason): PascalCase, **no** `rename_all` (see `OrphanReason` at `commands/review.rs:299-308`).
- Frontend-facing request bodies (if any new ones): `#[serde(rename_all = "camelCase")]`.
- Serialize-default structs: snake_case fields.

Phase 70 adds NO new wire DTOs (the command takes `path: String` and returns `String`), so this is a future-compat reminder.

### Theme tokens
**Source:** `src/components/ReviewPanel.svelte:519-657` (the existing style block) + CLAUDE.md ("Never inline colors").
**Apply to:** `ReviewDocPreview.svelte` and the new Generate button styles in `ReviewPanel.svelte`.

Every color goes through `var(--color-bg)`, `var(--color-surface)`, `var(--color-text)`, `var(--color-text-muted)`, `var(--color-border)`, `var(--color-hover)`. No hex literals, no rgba.

### Markdown-string emission (NEW PATTERN, not lifted)
The pure render builds the markdown via `String` + `writeln!(out, "…")`. No `pulldown-cmark`, no `comrak` (RESEARCH "Don't Hand-Roll" + "Alternatives Considered"). One direction, controlled output, supply-chain-free.

```rust
// Render shape (sketch — see RESEARCH "Code Examples" for full version).
fn emit_fence(out: &mut String, body: &str, source: Source, lang: &str) {
    let n = fence_length(body);
    let fence: String = std::iter::repeat('`').take(n).collect();
    let info = match source { Source::Diff => "diff", Source::FullFile => lang };
    writeln!(out, "{fence}{info}").unwrap();
    out.push_str(body);
    if !body.ends_with('\n') { out.push('\n'); }
    writeln!(out, "{fence}").unwrap();
    writeln!(out).unwrap();
}
```

This pattern is local to Phase 70 — no codebase analog because Phase 70 is the first markdown-emitting Rust module.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src-tauri/src/git/review.rs` — markdown emission helpers (`emit_fence`, `fence_length`, language table) | service | transform | Phase 70 is the first markdown-emitting Rust module. Use RESEARCH "Code Examples" section as the reference (lines 1233–1313). |
| `src/components/ReviewDocPreview.svelte` (if planner picks the new-file route) | component | one-way prop render | No existing single-content-display component in the codebase. The shape in RESEARCH Item 6 is greenfield — but theme tokens + `--color-*` rules from `ReviewPanel.svelte:519-657` still apply. |

---

## Flagged for Planner Verification

### `src/lib/diff-anchor.ts` and `src/lib/full-file-anchor.ts`

The orchestrator prompt asked: "ONLY if L-06 line-counting reconciliation requires retrofit — RESEARCH says it does NOT; flag as 'verify with planner'."

**Confirmed: NO retrofit required.** RESEARCH Item 2 (lines 448–504) verified by reading both files: TypeScript does NOT do its own line counting. `DiffLine.new_lineno` / `old_lineno` are emitted by libgit2 in Rust (`commands/diff.rs:235-236`); the TS adapters at `diff-anchor.ts:69-73` and `full-file-anchor.ts:50-52` just pick min/max over numbers libgit2 already produced. The line-counting reconciliation is **render-internal** (the Phase 70 renderer mirrors `classify_anchor:358`'s `String::from_utf8_lossy(...).lines().count()` semantic), not capture↔render.

**Recommendation to planner:** state explicitly in PLAN-NN-L06-line-counting that no TS file changes — call out as an anti-task so it doesn't get added.

---

## Metadata

**Analog search scope:**
- `src-tauri/src/commands/review.rs` (the primary backend analog — `_inner` pattern, `classify_anchor`, `resolve_all`, `list_session_comments`, `resolve_session_comments`, the `commit_with_file` test helper)
- `src-tauri/src/commands/diff.rs` (the diff-replay analog — `walk_diff`, `DiffOptions::pathspec`, root-commit guard, `is_binary()`)
- `src-tauri/src/git/mod.rs`, `src-tauri/src/git/syntax.rs`, `src-tauri/src/git/review_store.rs` (module placement + the lift-able `extension_from_path`)
- `src-tauri/src/lib.rs` (the `invoke_handler` block)
- `src/components/ReviewPanel.svelte` (the host panel + existing button/safeInvoke patterns)
- `src/components/ReviewPanel.test.ts` (the command-aware mock dispatcher)
- `src/lib/review-session.svelte.ts` (the rune to extend)
- `src/lib/invoke.ts` (the IPC wrapper)
- `src/lib/diff-anchor.ts`, `src/lib/full-file-anchor.ts` (verified NO retrofit needed)

**Files scanned:** 10
**Pattern extraction date:** 2026-05-26

## PATTERN MAPPING COMPLETE

**Phase:** 70 - excerpt-resolution-markdown-render
**Files classified:** 9 (excluding the two flagged-but-unchanged TS anchor files)
**Analogs found:** 8 / 9 (the one greenfield surface is the markdown-emission helpers in `review.rs`, where RESEARCH "Code Examples" is the reference)

### Coverage
- Files with exact analog: 8 (review.rs Tauri command structure; review.rs module reg; lib.rs handler reg; ReviewPanel.svelte buttons + safeInvoke; ReviewPanel.test.ts mock; review-session.svelte.ts rune extension; backend test fixture; ExcerptError + classify_anchor reuse)
- Files with role-match analog: 1 (ReviewDocPreview.svelte — theme-token shape lifted from ReviewPanel.svelte's style block, but the component itself is greenfield)
- Files with no analog: 1 surface (markdown-emission helpers — use RESEARCH "Code Examples" as the reference)

### Key Patterns Identified
- All review commands use the same `canonical_repo_path` + clone-under-lock + `spawn_blocking` (+ open repo INSIDE closure) shape. Read-only commands skip `_inner`; mutation commands keep it for disk-testability.
- Pure git2 helpers (`classify_anchor`, `resolve_all`) live in `commands/review.rs` today but follow the purity rule the new `git/review.rs` adopts: take refs, return data, no `tauri::*`. Phase 70 may not relocate `classify_anchor` (avoid scope creep) — `git/review.rs` calls into it via `crate::commands::review::classify_anchor`.
- Render-only failures use a render-only enum (`ExcerptError`) — DO NOT extend `OrphanReason` (it crosses the IPC wire and would force TS-side churn).
- The diff-replay-slice mirrors `commands/diff.rs::walk_diff`'s `diff.foreach` line callback, scoped to one file via `DiffOptions::pathspec` and one anchor's `(side, start_line, end_line)` overlap.

### File Created
`/Users/joaofnds/code/trunk/.planning/phases/70-excerpt-resolution-markdown-render/70-PATTERNS.md`

### Ready for Planning
Pattern mapping complete. Planner can now reference analog patterns in PLAN.md files with concrete file:line anchors.
