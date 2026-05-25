# Phase 67: Diff-Source Anchor Capture - Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

The user selects a line range in a **commit's diff (hunk/split) view** and attaches
a comment. The comment is persisted as a stable **source-line anchor** —
`(commit_oid, file_path, source=Diff, side, start_line, end_line)` — that survives
a diff re-fetch (context-line / whitespace-ignore toggle) and an app restart. This
is the first of two capture paths; Phase 68 adds full-file-source capture and shares
the same `add_comment` command.

**In scope:** an attach affordance + inline comment composer on a diff line
selection, the capture-time adapter translating selected diff indices →
`(side, start_line, end_line)` via each line's `old_lineno`/`new_lineno`/`origin`,
caching the excerpt at attach-time, writing into the Phase 65 schema/store via a
(shared) `add_comment` command, persisting the in-progress draft, and the
merge-commit / file-status capture constraints (ANCH-01).

**Out of scope (later phases):** full-file-source capture (68), the real
comment-management panel — list / edit / delete / jump-to-anchor (69, replaces
today's stub), commit-level comments with no anchor (69 / ANCH-03), markdown
render incl. render-time excerpt re-resolution and the "unresolvable" section
(70), clipboard / save output (71). Showing accumulated comments as gutter
markers/badges in the diff is **Phase 69**, not here — Phase 67 confirms a
successful attach, it does not build the in-diff comment browser.

</domain>

<decisions>
## Implementation Decisions

### Attach Flow & Composer (discussed)
- **D-01:** **Inline composer in the diff.** A floating "Comment" button appears on
  the active diff selection (the same on-selection action affordance that surfaces
  Stage/Unstage/Discard today); clicking it opens an expanding textarea directly
  under the selected lines (GitHub-PR style). Chosen over routing to the review
  panel because it keeps the comment visually tied to the code and does **not**
  depend on the Phase 69 panel shape (today's `ReviewPanel.svelte` is a throwaway
  stub). NOTE: commit diffs currently render no Stage/Unstage/Discard buttons, so
  the Comment affordance is net-new in that context (see code_context).

### Draft Collision (discussed)
- **D-02:** **Confirm before discard.** The schema has exactly one `draft_comment`
  slot and drafts persist on every keystroke (L-05), so a half-typed draft is
  durable state. Selecting a *new* range while the current draft has non-empty text
  prompts "Discard your unsaved comment?" before switching. An **empty** draft
  switches silently (no prompt). Mirrors the existing confirm-on-discard pattern
  (`DiffPanel.handleDiscardLines` uses `@tauri-apps/plugin-dialog` `ask`).

### Selection → Range Collapse (discussed)
- **D-03:** **Collapse to min–max on the chosen side, with a preview.** The anchor
  stores a single `start_line..end_line` range, but a v0.7 selection can be a
  non-contiguous `Set` within one hunk. Capture collapses to
  `min..max` of the source line numbers on the chosen `side`, and the composer
  shows a "Comments on lines N–M" preview so the collapse (and any included gaps)
  is transparent. No rejection of non-contiguous selections.

### Blocked-Anchor Feedback (discussed)
- **D-04:** **Disabled affordance + tooltip on merge commits.** Diff-source
  anchoring is disabled on merge commits (carried from Phase 66 D-08 / L-07). On a
  merge commit's diff the Comment affordance renders **disabled** with a tooltip
  explaining "Diff comments aren't available on merge commits" — teaching the rule
  at the point of action rather than hiding it (silently confusing) or allowing the
  attempt then toasting (after-the-fact). **File-status side constraints are NOT a
  "blocked" case** — Added/Deleted/Renamed simply *force* the side (L-04); the
  affordance stays enabled.

### Locked Carry-Forwards (from ROADMAP §"Phase 67" Notes + Phase 65/66 — do NOT re-litigate)
- **L-01:** Persist the anchor as source coordinates only. **Never** persist
  `hunk_index` / `line_index` (positions in the in-memory diff array) or the diff
  options (`context_lines` / `ignore_whitespace`). Schema is frozen (Phase 65 D-01):
  `Anchor { commit_oid, file_path, source, side, start_line, end_line }`.
- **L-02:** A capture-time adapter translates selected diff indices →
  `(side, start_line, end_line)` via each line's `old_lineno` / `new_lineno` /
  `origin`. Add lines carry only `new_lineno`, Delete lines only `old_lineno`.
- **L-03:** For a selection mixing Added and Deleted lines, default to the `new`
  side and **drop pure-Delete lines from the persisted line range**, but keep them
  as `-` lines in the cached excerpt. Requires a `side` discriminator on every
  diff-source anchor.
- **L-04:** File-status constraints (`DiffStatus`): Added → `new`-side only,
  Deleted → `old`-side only, Renamed → store the **new** path with `new` side.
  Store the path as it exists at the anchored commit on the anchor's side.
- **L-05:** Persist the anchor **immediately on attach** (so it survives the
  watcher's `repo-changed` diff re-fetch) and persist the draft **on change**, not
  only on submit (Phase 65 DP-02 → `draft_comment` field already on the session).
- **L-06:** Cache the excerpt at attach-time as the canonical comment body
  (`Comment.cached_excerpt`), in diff format (with `-` context per L-03). Render
  (Phase 70) re-resolves from source coords with the cached excerpt as fallback —
  re-resolution and the "unresolvable" section are **Phase 70**, not here.
- **L-07:** Merge commits are disabled for diff-source anchoring (surfaced/deferred
  in Phase 66 D-08). Full-file-source review of a merge remains valid (Phase 68).
- **L-08:** The `add_comment` command is **shared with Phase 68** (Phase 68 is
  independent/parallelizable and reuses it) — design it so `source`/`side` are
  parameters, not hard-coded to `Diff`.

### Claude's Discretion (delegated to planner)
- **Attach-success feedback:** on submit, clear the composer + selection; rely on
  the existing `session-changed` event → panel reload (Phase 65/66 pattern) for
  confirmation. A success toast is optional and consistent with the project's
  mostly-silent-success convention. Planner's call.
- **Empty-text submit** is disabled (no zero-text comments); exact validation point
  is the planner's.
- **Command surface & naming:** whether `add_comment` (and any draft-persist
  command) lives in the existing `src-tauri/src/commands/review.rs` (recommended) or
  a new file; exact struct/command names following established serde conventions
  (snake_case Serialize-default structs; camelCase via `rename_all` for
  frontend-facing request types).
- **Composer rendering across layout modes:** the diff view has v0.12
  hunk/split layouts — both are diff-source (split's right/`new` panel already
  supports line selection, Phase 64). The full-file **content mode** is Phase 68
  territory; Phase 67's affordance lives in the hunk/split diff rendering.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone & phase spec
- `.planning/ROADMAP.md` §"Phase 67: Diff-Source Anchor Capture" — goal, the three
  success criteria, and the locked Notes (L-01..L-07 above): index→source
  translation, `side` discriminator, mixed-selection policy, file-status
  constraints, excerpt caching, persist-on-attach. Also read §"Phase 68" Notes
  (shared `add_comment`, the diff-vs-blob distinction that bounds what "diff-source"
  means) and §"Phase 70" (render-time re-resolution the cached excerpt feeds).
- `.planning/REQUIREMENTS.md` — **ANCH-01** (this phase), the downstream
  ANCH-02/03 → CMT → DOC → OUT chain the anchor feeds, and the "Out of Scope" table
  (no threading, no severity tags, no re-anchoring on history rewrite, static
  snapshot).

### Phase 65 keystone (the schema + store this phase writes into — FROZEN)
- `.planning/phases/65-data-model-persistence-session-lifecycle/65-CONTEXT.md` —
  D-01 (anchor = source coords, never index/options), D-04 (text stored
  independent of resolvability; cache excerpt at attach), D-05 (full schema locked
  now), DP-02 (draft lives on the persisted session), D-09/D-10 (own JSON store,
  atomic tmp+rename — NOT tauri-plugin-store), D-11 (canonical-path keying).
- `src-tauri/src/git/types.rs:288-336` — the frozen review schema:
  `Source{Diff,FullFile}`, `Side{Old,New}`, `Anchor`, `Comment{text, anchor,
  cached_excerpt}`, `DraftComment{text, anchor}` (note: **no** `cached_excerpt` on
  the draft — excerpt is computed at submit), `ReviewSession{schema_version,
  commits, comments, draft_comment}`.
- `src-tauri/src/git/types.rs:187-223` — `DiffLine{origin, content, old_lineno,
  new_lineno, spans}`, `DiffHunk`, `DiffStatus{Added,Deleted,Modified,Renamed,
  Copied,Untracked,Unknown}`, `FileDiff{path, status, is_binary, hunks}` — the
  inputs the capture-time adapter (L-02) reads.
- `src-tauri/src/git/review_store.rs` — `save_session`/`load_session`/
  `delete_session` (atomic tmp+rename); the persistence path every mutation uses.
- `src-tauri/src/commands/review.rs` — Phase 65/66 commands; the
  `_inner(data_dir, …)` testable core + thin `#[tauri::command]` wrapper +
  `app.emit("session-changed", canonical)` pattern to mirror for `add_comment` and
  draft persistence.

### Phase 66 (merge-commit deferral)
- `.planning/phases/66-commit-selection/66-CONTEXT.md` — **D-08**: merge commits are
  selectable, but the diff-source-anchor restriction is deferred to capture time —
  i.e. **enforced here** (L-07 / D-04).

### Codebase patterns to mirror
- `src/components/DiffPanel.svelte` — `handleLineClick` (line ~298), the
  `selectedLineIndices: Set<number>` + `selectedHunkKey` selection model (single-hunk
  scope, non-contiguous allowed), and the on-selection action handlers
  (`handleStageLines`/`handleUnstageLines`/`handleDiscardLines`, ~341-410) the
  Comment affordance parallels. **Critical reuse constraint:** `handleLineClick`
  returns early when `diffKind === "commit"` (line ~307), so v0.7 line selection is
  currently *disabled* in commit diffs — capture must lift that guard for the
  comment use case while keeping staging buttons absent.
- `src/components/ReviewPanel.svelte` — the throwaway stub already listening for
  `session-changed`; the composer is inline (D-01), so the panel is NOT the
  capture surface this phase.
- `.planning/codebase/CONVENTIONS.md`, `STACK.md`, `ARCHITECTURE.md` — Rust/Svelte
  conventions, git2-only (no shelling out for local reads), theme CSS custom
  properties (no inline colors), `safeInvoke<T>` for all IPC, command structure.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **v0.7 line-selection model** (`DiffPanel.svelte` `handleLineClick`,
  `selectedLineIndices`, `selectedHunkKey`): single-hunk-scoped selection (selecting
  in another hunk resets), non-contiguous toggling, shift-click contiguous range
  that skips Context lines. Capture reuses this to LET THE USER PICK lines, then
  collapses to a range (D-03).
- **On-selection action affordance** (`handleStageLines` etc., ~341-410): the visual
  pattern the floating "Comment" button (D-01) mirrors.
- **Phase 65/66 store + event** (`review_store.rs`, `commands/review.rs`): atomic
  save + `session-changed` emit are done; `add_comment` / draft-persist reuse them.
- **Confirm dialog** (`DiffPanel.handleDiscardLines` → plugin-dialog `ask`): the
  pattern for D-02's discard-draft confirmation.

### Established Patterns
- **`_inner(data_dir, …)` + thin command wrapper + emit** in `review.rs` — the
  testable command shape `add_comment` should follow.
- **serde conventions** (Phase 59-66): snake_case Serialize-default structs;
  camelCase via `rename_all` for frontend-facing request types; review-schema enums
  serialize PascalCase with NO `rename_all`.
- **Theme CSS custom properties only** — any new affordance/highlight uses a
  `--color-*` var, never an inline color.

### Integration Points
- `DiffPanel.svelte` — the commit-diff line-selection guard (line ~307) is lifted
  for comment capture; the Comment button + inline composer attach to the existing
  selection state. Split view (Phase 64) already selects on the `new`/right panel.
- `commands/review.rs` + `lib.rs` `invoke_handler` — register `add_comment` and any
  draft-persist command.
- The commit-diff data path (`diffKind === "commit"`) provides the `FileDiff` /
  `DiffStatus` the adapter (L-02/L-04) reads to derive side + path.

</code_context>

<specifics>
## Specific Ideas

- The recipient of the eventual rendered doc is an **AI coding agent** — the cached
  excerpt + comment text IS the instruction, so keep capture lean (no severity,
  author, threading).
- "Diff-source" specifically means the hunk/split diff of a reviewed commit; the
  full-file-at-commit view is a separate source (Phase 68). The `source` enum value
  is set at capture, which is exactly why `add_comment` must parameterize it (L-08).
- The composer staying inline (not in the panel) is deliberate: the panel is a
  throwaway stub until Phase 69, and tying the comment to the code reads better.

</specifics>

<deferred>
## Deferred Ideas

- **In-diff comment browser** (gutter markers / badges / click-to-edit on anchored
  lines) — Phase 69 (Comment Management UI / CMT-04 jump-to-anchor). Phase 67 only
  confirms a successful attach.
- **Commit-level comments with no anchor** — Phase 69 (ANCH-03).
- **Render-time excerpt re-resolution + "unresolvable" section** — Phase 70
  (DOC-04). The cached excerpt (L-06) exists to make this possible.
- No scope-creep ideas surfaced — discussion stayed within the phase boundary.

### Reviewed Todos (not folded)
- **`2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md`**
  — matched on keyword "commit" (score 0.4) but is about merge/revert *commit-message
  editing*, unrelated to review-session anchor capture. Already reviewed and declined
  in Phase 66; not folded.

</deferred>

---

*Phase: 67-diff-source-anchor-capture*
*Context gathered: 2026-05-25*
