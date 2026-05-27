# Phase 70: Excerpt Resolution + Markdown Render - Context

**Gathered:** 2026-05-26
**Status:** Ready for planning

<domain>
## Phase Boundary

The user generates **one** AI-framed markdown document from the active review
session. The renderer pulls the session's commits + comments, **re-resolves**
each line-anchored comment's excerpt against the live repo at the anchor's
commit (fresh git2 read — `full_file` = `commit.tree().get_path(file).peel_to_blob()`
slice; `diff` = re-run `diff_tree_to_tree(parent, commit)` and slice overlapping
hunk lines), and emits a single markdown string that an AI coding agent can act
on (DOC-01..04). Comments whose anchor no longer resolves render in a dedicated
"unresolvable" section — never silently dropped, never crashing the render.

**In scope:** the Rust pure render in `src-tauri/src/git/review.rs` (returning
one markdown string), a Tauri command wrapping it, a "Generate" affordance in
the Phase 69 ReviewPanel header that swaps the panel to a markdown preview
view, the in-panel preview itself, fence-length calculation
(`max(3, longest_backtick_run_in_excerpt + 1)`), CRLF→LF normalization, a
single line-counting convention reconciled between capture (Phase 67/68) and
render, binary-file `[binary file, no excerpt]` placeholder, fully `Result`-typed
resolution that routes failures into the unresolvable section using the
independently-stored comment text + cached_excerpt.

**Out of scope (later phases / deferred):**
- Clipboard / Save-to-file actions (Phase 71 / OUT-01, OUT-02). The Phase 70
  preview view is the surface those Phase 71 buttons will attach to.
- Re-anchoring on history rewrite (REQUIREMENTS Out of Scope — the doc is a
  static snapshot).
- New comment metadata (severity, author, threading) — REQUIREMENTS Out of Scope.
- Inline gutter badges on anchored diff/full-file lines (still deferred from
  Phase 67/69).
- Per-line word-span / syntax-token enrichment in excerpts (ROADMAP §70 Notes:
  "Skip syntax/word-span enrichment for excerpts — plain text in a fence is
  what the AI needs"). The renderer does NOT call into `syntax.rs`.

</domain>

<decisions>
## Implementation Decisions

### Trigger surface (discussed)
- **D-01:** **Generate button in the ReviewPanel header.** Lives next to the
  panel's existing controls (`src/components/ReviewPanel.svelte`). Disabled
  until the session has ≥1 comment with a tooltip ("Add at least one comment
  to generate"). Resolvable AND unresolvable comments both count toward
  enablement (unresolvable still renders in its own section). View-menu /
  hotkey-only / floating-footer alternatives were rejected — the button sits
  with the artifact it acts on, and Phase 71 will add Copy/Save to the same
  area.
- **D-02:** **Click → swap the panel to a markdown preview view.** A
  panel-internal swap (comment-list view ↔ generated-doc view), mirroring the
  Phase 69 D-07 right-pane swap pattern. The preview renders the markdown in a
  monospace, scrollable container; a "Back to comments" affordance returns to
  the list. Modal / new-tab / state-only-no-preview alternatives were rejected —
  a modal contradicts the app's non-modal pattern, a tab implies
  refresh/persistence semantics the static-snapshot doc does not have, and
  state-only would leave the user with no way to verify the doc this phase.
  **Phase 71 attaches Copy/Save buttons to this same preview view** — its shape
  must leave room for those controls (Discretion).

### Doc anatomy (discussed)
- **D-03:** **Top section: H1 title + short prose framing + commit refs list.**
  H1 reads "Code review: <repo-name>" (planner picks exact wording, Discretion).
  Framing is one or two sentences explicitly telling the AI what the doc is — a
  human-authored code review with anchored excerpts; address each comment in
  context. Then the commit refs list. Detailed system-prompt-style preambles
  were rejected (bakes prompt-engineering into a snapshot artifact); bare-
  structure-only was rejected (a general-purpose AI may misread the intent
  without explicit framing).
- **D-04:** **Section order:** resolved per-file content → commit-level section
  → unresolvable section. Strongest signal first (actionable, anchored
  feedback), then anchor-less commit-level notes, then failure cases at the
  end. Per-file-inline-unresolvable was rejected — "commit gone" has no clear
  file home and the renderer would be messier.
- **D-05:** **Per-(file, commit) grouping.** The same file in different commits
  is meaningfully different code → each (file, commit) pair gets its own group
  containing all anchors in that file at that commit, ordered by line (DOC-03).
  One-section-per-file-with-anchors-sorted-by-(commit,line) was rejected — it
  mixes commits inside a file group and loses the per-commit framing the
  `(sha)` heading already implies. Flat list (no per-file headings) was rejected
  — loses file-level scaffolding the AI uses to track scope.
- **D-06:** **Within one anchor block: excerpt first, comment text after.**
  AI reads "here's the code; here's the reviewer's feedback on it" — quote-then-
  react ordering matches human review conventions and discourages the AI from
  planning a fix before seeing the lines. Comment-first and comment-only
  (no excerpt) were rejected.

### Commit refs detail (discussed)
- **D-07:** **Bullet list of `- <short-sha> — <subject>`.** 7-char short SHA +
  the commit's subject line. Minimal at-a-glance map for the AI; author/date
  add nothing for a static doc consumed by an LLM. SHA-only was rejected
  (loses human-readable context); full-SHA / SHA+author+date were rejected
  (noise for the recipient).
- **D-08:** **7-char short SHA EVERYWHERE.** Same short SHA in the top refs
  list AND in the locked `path:Lstart-Lend (sha)` per-anchor headings (DOC-03)
  AND in the commit-level section's commit reference. Internal consistency
  beats the tiny ambiguity risk at this app's scale. Mixed (refs full, headings
  short) and all-full were rejected.

### Unresolvable + empty (discussed)
- **D-09:** **Human-readable reason phrasing.** Map `OrphanReason` variants to
  plain-English strings the AI can act on:
  - `CommitGone` → "commit no longer exists in the repository"
  - `FileGone` → "file no longer exists at this commit/side"
  - `LineOutOfRange` → "anchor line range is outside the current file bounds"

  Raw PascalCase tokens and phrase+token-both were rejected. The doc is for
  an AI reading prose, not for mechanical parsing.
- **D-10:** **Unresolvable excerpt fences by `Source` per DOC-02**, labelled as
  cached at attach time (e.g., a one-line note above the fence: "Anchor no
  longer resolves; excerpt is the cached snapshot from attach time."). Diff-
  fenced for `Source::Diff`, language-fenced for `Source::FullFile`. The
  comment body is `cached_excerpt` (ROADMAP §70: "use independently-stored
  comment text and cached excerpt"). Always-plain-fence and omit-excerpt were
  rejected — the former loses the diff-vs-file signal the AI uses; the latter
  contradicts the ROADMAP note.
- **D-11:** **No zero-comment render path.** The D-01 trigger gating is the
  contract; the Rust renderer is invoked only when the session has ≥1 comment.
  No defensive zero-comment branch in the renderer or the command. Keeps the
  renderer's invariants tight. (A future non-UI invocation that bypasses the
  gate would surface as an explicit error from the command, not a soft empty
  doc.)

### Locked carry-forwards (from ROADMAP §70 Notes + Phase 65/67/68/69 — do NOT re-litigate)
- **L-01:** **Render in Rust, pure logic** in `src-tauri/src/git/review.rs`,
  returning ONE markdown string. Not a per-line enriched payload.
- **L-02:** **Re-resolution mechanics, fresh from git2:**
  - `Source::FullFile` excerpt: `commit.tree().get_path(file).to_object().peel_to_blob()`,
    then slice the requested 1-based line range from the blob bytes. Never
    re-derive a `FullFile` excerpt by re-running the diff.
  - `Source::Diff` excerpt: re-run `diff_tree_to_tree(parent, commit)` for the
    anchored file and slice the lines that overlap the anchor's `(side,
    start_line, end_line)` range — keeping `-` lines in the excerpt per
    Phase 67 L-03.
  - **Phase 68 D-02/D-03/D-04 still hold** for FullFile excerpts: deletes
    excluded from blob slicing (blob has no deletes anyway), gap-marker
    `… N lines unchanged …` where applicable, plain content (no `+/-`).
- **L-03:** **Fence length = `max(3, longest_backtick_run_in_excerpt + 1)`.**
  Never indent the fence. Preserve exact indentation INSIDE the fence
  (no re-indenting, no tab-expansion).
- **L-04:** **Every resolution step returns `Result`. Never `unwrap`.** On
  failure (any kind: commit gone, file gone, blob not utf8 in a way that
  matters, hunk slice failure, anything), the comment routes to the
  unresolvable section using independently-stored `Comment.text` +
  `Comment.cached_excerpt`. Render NEVER crashes.
- **L-05:** **Binary files** → `[binary file, no excerpt]` placeholder in place
  of the excerpt fence. Apply when the blob's content fails utf8 conversion
  in a way that prevents line-slicing, or when blob.is_binary() (planner picks
  the exact detection mechanism — Discretion).
- **L-06:** **Normalize CRLF → LF.** Reconcile capture's line-counting
  convention with the renderer's so a line-number computed at capture
  (Phase 67/68) and the same line-number computed at render against the same
  blob agree. Fix the divergence in ONE place; document it. (Capture today
  uses `str::lines()` semantics — see `resolve_session_comments`
  `classify_anchor` at `src-tauri/src/commands/review.rs:355-358`.)
- **L-07:** **DOC-02 fencing:** diff-fenced for `Source::Diff`, language-fenced
  for `Source::FullFile`. The opening fence INCLUDES a language tag for
  full-file excerpts (planner's language detector, Discretion); diff excerpts
  fence with `` ``` `` + `diff` info string.
- **L-08:** **DOC-03 layout:** comments grouped by file, ordered by line within
  a file/commit group, each under `path:Lstart-Lend (sha)`. Commit-level
  comments render in the trailing section.
- **L-09:** **DOC-04:** unresolvable section is mandatory, never silently
  dropped, never crashes the render.
- **L-10:** **No syntax/word-span enrichment.** The renderer does NOT call into
  `src-tauri/src/git/syntax.rs`. Plain text in a fence is what the AI needs.

### Claude's Discretion (delegated to planner)
- **Module placement:** introduce `src-tauri/src/git/review.rs` for the pure
  render (ROADMAP names this file explicitly). Decide whether helpers for
  excerpt re-resolution (blob-slice, diff-replay-slice) live next to the
  render or in `review_store.rs` — keep them pure, no `tauri::*` imports.
- **Tauri command surface:** likely `generate_review_doc(path)` in
  `commands/review.rs`, mirroring the `_inner(data_dir, …)` + thin
  `#[tauri::command]` wrapper pattern. **No `session-changed` emit** — render
  is read-only, no mutation. No `mutate_session_rmw` involvement.
- **`id` generation:** N/A — Phase 69 already shipped stable `Comment.id`.
- **`OrphanReason` reuse vs render-side classifier:** the render path may
  reuse `OrphanReason` (Phase 69) for "anchor commit/file/line gone" failures
  and ADD a render-only kind for failures specific to fresh excerpt resolution
  (e.g. binary already gets a placeholder, not unresolvable; blob-read failure
  routes to unresolvable). Planner picks: extend the enum, mirror with a
  render-only enum, or stringify reasons inline. The user-facing strings in
  D-09 are the contract; the internal type is the planner's call.
- **File-group heading text + level:** e.g. `## src/foo.rs @ abc1234` vs
  `## src/foo.rs (abc1234)` vs `## abc1234 — src/foo.rs`. The per-anchor
  `path:Lstart-Lend (sha)` is locked by ROADMAP but its heading level (`###`
  vs `####`) is planner's call. Keep nesting consistent: H1 doc title → H2
  per-(file, commit) → H3 per-anchor seems natural but is not locked.
- **Exact prose framing text in D-03's preamble.** Plain, declarative, ≤2
  sentences. Avoid prompt-engineering tone; do not instruct the AI on response
  format (that would belong with Phase 71's "Copy" UX, not the snapshot doc).
- **Language detection for full-file language fences:** extension-based
  mapping with a `text` fallback. Reuse anything already in
  `src-tauri/src/git/syntax.rs`'s language table if it can be lifted without
  pulling in syntax highlighting itself (L-10 forbids the highlighter, not
  the language-name lookup). Otherwise hand-roll a small `.rs/.ts/.svelte/.md/
  …` table.
- **Line-counting reconciliation (L-06):** identify any divergence between
  capture (Phase 67/68) and the render's blob/diff slicing — `str::lines()`
  vs `split('\n')` vs handling of a missing final newline. Lock ONE convention
  and apply it both at capture (retrofit `buildDiffAnchor`/`buildFullFileAnchor`
  if needed) and at render. Document the chosen convention in `review.rs`.
- **Panel preview view:** new component file (e.g. `ReviewDocPreview.svelte`)
  vs. inline branch in `ReviewPanel.svelte`. Either is fine; "Back to comments"
  affordance shape (X icon, back arrow, "Back to comments" link) is the
  planner's call. **Leave space for Phase 71's Copy / Save buttons** — likely a
  toolbar slot above or beside the preview text, so Phase 71 lands cleanly.
- **`review-session.svelte.ts` (Phase 69 rune):** add a generate action +
  preview-state-vs-list-state discriminator. Exact field names + reset
  semantics (e.g. `endSession()` clears the generated doc) are planner's call.
- **Empty / one-only states:** D-11 lets the renderer assume ≥1 comment, but
  cases like "all comments unresolvable" or "only commit-level, no anchored"
  are valid — render with whatever sections apply; skip empty section headings.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone & phase spec
- `.planning/ROADMAP.md` §"Phase 70: Excerpt Resolution + Markdown Render" —
  goal, the four success criteria, Requirements (DOC-01..04), and the locked
  Notes: render in Rust pure logic returning one string, the `full_file` vs
  `diff` re-resolution mechanics, the fence-length formula, never-`unwrap`
  + warning-block-on-failure, binary-file placeholder, CRLF→LF + single line-
  counting convention shared with capture. Also read §"Phase 71" — what the
  Generate preview view (D-02) must enable next phase (Copy / Save buttons,
  capabilities, fire-and-forget vs awaited copy).
- `.planning/REQUIREMENTS.md` — DOC-01..04 (this phase) plus the "Out of Scope"
  table (no re-anchoring on history rewrite; no severity / author / threading;
  static snapshot).

### Phase 65 keystone (the schema being read)
- `.planning/phases/65-data-model-persistence-session-lifecycle/65-CONTEXT.md`
  — D-01 (anchor schema: source coordinates only — `(commit_oid, file_path,
  source, side, start_line, end_line)`); D-04 (text stored independent of
  anchor resolvability — underpins the unresolvable section's contract);
  D-13 (end hard-deletes — render runs against live in-memory session);
  D-15/D-16 (corrupt / newer-version — render is invoked only on a loaded
  session; not relevant to the render path itself).
- `src-tauri/src/git/types.rs:288-345` — the review schema being read:
  `Source{Diff,FullFile}`, `Side{Old,New}`, `Anchor`, `Comment { id, text,
  anchor: Option<Anchor>, cached_excerpt: Option<String>, commit_oid:
  Option<String> }`, `ReviewSession { schema_version, commits, comments,
  draft_comment }`.

### Phase 67 (diff-source capture — what the renderer re-resolves)
- `.planning/phases/67-diff-source-anchor-capture/67-CONTEXT.md` — L-01
  (source coords only, never diff-array positions); L-03 (mixed selection
  defaults to `new` side, drops pure-Delete lines from the range but KEEPS
  them as `-` lines in the cached excerpt — render-side `Source::Diff`
  excerpts mirror this format); L-06 (`cached_excerpt` is canonical body
  in diff format — used by unresolvable section per D-10).

### Phase 68 (full-file-source capture — what the renderer re-resolves)
- `.planning/phases/68-full-file-source-anchor-capture/68-CONTEXT.md` — D-02
  (deletes excluded from new-side blob coordinates); D-03 (gap-crossing
  selections keep correct `start..end` blob coords + cached excerpt holds
  visible lines with `… N lines unchanged …` marker — render-side fresh-blob
  slice does NOT need the marker because it reads contiguous blob lines, but
  the unresolvable fallback `cached_excerpt` may contain it); D-04
  (`cached_excerpt` is plain new-side content, no `+/-` prefixes, language-
  fenced at render per L-07).

### Phase 69 (panel + resolver this phase reuses)
- `.planning/phases/69-comment-management-ui/69-CONTEXT.md` — D-06
  (eager `resolve_session_comments` + git2-backed classifier — same shape
  the render reuses for the "anchor still resolves" check before attempting
  to re-resolve the excerpt); the panel itself (host of the Generate button).
- `src-tauri/src/commands/review.rs:296-398` — `OrphanReason` (CommitGone /
  FileGone / LineOutOfRange) + `classify_anchor` + `resolve_all`. The render
  path can call `classify_anchor` (or its equivalent) to short-circuit
  unresolvable cases before attempting a blob/diff slice.
- `src-tauri/src/commands/review.rs` (general) — `_inner(data_dir, …)` + thin
  `#[tauri::command]` wrapper pattern the new generate command mirrors.
  `mutate_session_rmw` is NOT used (render is read-only, no `session-changed`
  emit).
- `src-tauri/src/git/review_store.rs` — `load_session` is how the command
  reads the canonical session before rendering.
- `src/components/ReviewPanel.svelte` — Phase 69's real panel (the host for
  the Generate button + preview swap). Header area + group-by-commit list
  already in place.
- `src/lib/review-session.svelte.ts` — Phase 69's rune module (the action +
  preview state lives here).

### Codebase patterns / conventions
- `.planning/codebase/CONVENTIONS.md`, `STACK.md`, `ARCHITECTURE.md`,
  `INTEGRATIONS.md` — Rust/Svelte conventions, git2-only for local reads,
  theme CSS custom properties (no inline colors), `safeInvoke<T>` for all IPC.
- `src-tauri/src/git/syntax.rs` — for language-name lookup ONLY if the planner
  decides to lift a `.ext → language-name` table. L-10 forbids invoking the
  highlighter itself.
- `src/lib/invoke.ts` `safeInvoke<T>` — the IPC wrapper to invoke the new
  generate command from the rune action.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`OrphanReason` + `classify_anchor` + `resolve_all`** (Phase 69,
  `src-tauri/src/commands/review.rs:296-398`): the render can call into the
  same classifier to decide "this anchor resolves, attempt the excerpt" vs
  "route to unresolvable". Or use it as the basis for a render-side enum that
  adds a `BlobReadFailed`-like variant (planner's call).
- **`_inner(data_dir, …)` + thin `#[tauri::command]` wrapper pattern**
  (`commands/review.rs`): the shape the new generate command follows. Read-
  only, no `session-changed` emit, no `mutate_session_rmw`.
- **`load_session`** (`src-tauri/src/git/review_store.rs`): how to read the
  canonical session before rendering.
- **`ReviewPanel.svelte` + `review-session.svelte.ts`** (Phase 69): the host
  UI for the Generate button + preview swap. The panel already owns
  `session-changed` listener, per-repo canonical-path filter, lifecycle
  states, and the comment list grouped by commit.
- **`safeInvoke<T>`** (`src/lib/invoke.ts`): the IPC wrapper for the rune's
  generate action.

### Established Patterns
- **serde conventions:** review-schema enums (Source/Side) serialize PascalCase
  with NO `rename_all`; Serialize-default structs use snake_case; frontend-
  facing request types use `#[serde(rename_all = "camelCase")]`. Any new
  command request DTO must follow this.
- **git2 ownership:** `git2::Repository` is not `Sync` — keep its construction
  inside the command call (open per-call via `app.path()` + the session's
  canonical path), never stash it in state. Mirrors `src-tauri/src/state.rs`'s
  PathBuf-only rule.
- **Theme CSS custom properties only** — preview view styling uses `--color-*`
  vars (no inline colors). Mono font via the existing token.
- **No `tauri-plugin-store` for sessions** (Phase 65 D-09) — render is
  read-only against the per-repo JSON file already managed by `review_store`.

### Integration Points
- **`src-tauri/src/git/review.rs` (new module):** pure render. No `tauri::*`
  imports; takes (session, repo) and returns `String` (or
  `Result<String, RenderError>` if any non-recoverable wrapping failure can
  occur — though L-04 routes resolution failures into the doc, not the
  error). Add to `src-tauri/src/git/mod.rs` `pub mod review;` (mirrors
  `review_store` registration).
- **`src-tauri/src/commands/review.rs`:** add the generate command (e.g.
  `generate_review_doc`) following `_inner` + wrapper pattern. Register in
  `src-tauri/src/lib.rs` `invoke_handler`.
- **`src/lib/review-session.svelte.ts`:** add generate action + preview-vs-
  list view-state on the rune. Reset on `endSession` + on `session-changed`
  events that change session identity.
- **`src/components/ReviewPanel.svelte`:** Generate button in header
  (disabled-until-≥1-comment per D-01); render-time swap between
  comment-list and preview view (per D-02). New preview component or inline
  branch (Discretion). Leave room for Phase 71's Copy/Save toolbar.
- **Line-counting reconciliation (L-06):** capture lives in
  `src/lib/diff-anchor.ts` (Phase 67) and `src/lib/full-file-anchor.ts`
  (Phase 68); render lives in `src-tauri/src/git/review.rs` (new). One
  convention, applied both sides — likely "1-based inclusive, `str::lines()`
  semantics (no trailing-newline empty line)" to match `classify_anchor`
  (`src-tauri/src/commands/review.rs:355-358`).

</code_context>

<specifics>
## Specific Ideas

- The doc's recipient is an **AI coding agent** — keep prose framing minimal
  and structural. The structure (H1 + framing + commit refs + per-(file,
  commit) sections + commit-level + unresolvable) IS the prompt; do not bake
  response-format instructions into a snapshot artifact (those belong with
  whoever pastes the doc into an AI session, not the doc itself).
- **Static snapshot** — generating the doc creates a fresh string each time;
  there is nothing to persist, no "last generated" cache to invalidate.
  Regenerate is the only update path. Phase 71 will copy/save the string;
  Phase 70 just produces it.
- **Phase 71 is the same panel area** — Generate's preview view is where Copy
  and Save will dock. Planning the toolbar/layout so those land without
  rework is cheap; doing it post-hoc means a layout pass in Phase 71.

</specifics>

<deferred>
## Deferred Ideas

- **Clipboard / Save-to-file** — Phase 71 (OUT-01, OUT-02). The Generate
  preview view (D-02) is the surface for those next-phase buttons.
- **Re-anchoring on history rewrite** — REQUIREMENTS Out of Scope. Doc is a
  static snapshot; if the user rebases between Generate and Copy, the doc
  reflects the pre-rebase state, and that's intentional.
- **Inline gutter badges / in-diff comment browser** — deferred again from
  Phase 67/69; still backlog.
- **Syntax / word-span enrichment in excerpts** — explicitly NOT in scope
  here (L-10 + ROADMAP §70 Notes); not a candidate for a later phase either
  unless the AI-recipient assumption changes.
- **No scope-creep ideas surfaced** — discussion stayed within the phase
  boundary.

</deferred>

---

*Phase: 70-excerpt-resolution-markdown-render*
*Context gathered: 2026-05-26*
