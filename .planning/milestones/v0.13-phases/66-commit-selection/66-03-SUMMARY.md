---
phase: 66-commit-selection
plan: 03
subsystem: ui
tags: [svelte, tauri, code-review, commit-graph, theme-vars]

# Dependency graph
requires:
  - phase: 66-01
    provides: SessionCommit Rust struct + session schema this TS interface mirrors
  - phase: 66-02
    provides: list_session_commits / remove_review_commit Tauri commands the panel calls
  - phase: 65
    provides: ReviewPanel throwaway lifecycle stub + session-changed listener extended here
provides:
  - SessionCommit TS interface mirroring the Rust struct (oid/short_oid/summary)
  - CommitRow inSession + isPendingBase props with distinct theme-variable inset markers (D-04 / D-01)
  - ReviewPanel minimal in-session commit list with per-row × remove, loaded via list_session_commits and kept live by session-changed (D-05 / D-07 / SEL-04)
affects: [66-04 CommitGraph prop wiring, phase 69 real review panel, phase 67/68 anchor capture]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "D-04 membership marker as a theme-variable box-shadow inset on the plain HTML CommitRow, NEVER the SVG lane pipeline"
    - "Distinct edge/var per marker (inset 3px 0 0 left for inSession, inset 0 -3px 0 bottom for isPendingBase) so they compose with the background ternaries and each other"
    - "Props owned in CommitRow with false defaults so a downstream caller (Plan 04) only passes them at the call site — no shared-file conflict"
    - "Silent reload on list_session_commits failure (inactive session is normal); silent-success remove relying on session-changed emit"

key-files:
  created: []
  modified:
    - src/lib/types.ts
    - src/components/CommitRow.svelte
    - src/components/CommitRow.test.ts
    - src/components/ReviewPanel.svelte
    - src/components/ReviewPanel.test.ts
    - src/app.css

key-decisions:
  - "Two theme vars added in app.css next to --color-selected-row: --color-review-row (= --color-accent) and --color-review-pending-base (= --color-warning) — distinct, theme-driven, no inline literals"
  - "reviewMarker pulled into a $derived comma-joined box-shadow string instead of nesting another ternary into the already-dense style expression"
  - "session-changed listener reloads BOTH status and commits under the existing canonical-path guard (no unconditional sibling listener)"
  - "Renamed the active-branch placeholder from 'No comments yet' to 'No commits selected yet' — this panel surfaces commits, not comments (comments arrive in a later phase)"

patterns-established:
  - "Pattern: review membership tint = box-shadow inset on HTML row via --color-* var; SVG overlay untouched (.claude/rules/commit-graph.md boundary respected)"
  - "Pattern: per-row × remove with aria-label, silent success via the event-driven reload"

requirements-completed: [SEL-03, SEL-04]

# Metrics
duration: ~12min
completed: 2026-05-25
---

# Phase 66 Plan 03: Commit-Selection Visibility (CommitRow markers + ReviewPanel list) Summary

**Makes the review selection visible: a SessionCommit TS type, distinct theme-variable in-session/pending-base tints on the HTML CommitRow (owned here so Plan 04 only wires call sites), and a minimal session-commit list in the ReviewPanel with per-row × remove, kept live by the session-changed listener.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-25T14:50:00Z (approx)
- **Completed:** 2026-05-25T14:55:00Z (approx)
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- `SessionCommit` interface added to `types.ts`, mirroring the Rust struct (snake_case `oid`/`short_oid`/`summary`).
- `CommitRow` gained `inSession` and `isPendingBase` props, each driving a DISTINCT theme-variable inset marker (`--color-review-row` left edge, `--color-review-pending-base` bottom edge) that composes with the existing background ternaries and each other — never an inline literal, never the SVG lane pipeline.
- `ReviewPanel` now renders the minimal in-session commit list (short SHA + summary, graph-ordered/dedup'd server-side) inside the active branch, with a per-row × remove calling `remove_review_commit`, loaded via `list_session_commits` and reloaded on `session-changed` (guarded by the canonical-path filter).
- 20 tests green across `CommitRow` and `ReviewPanel`; svelte-check 0 errors; biome clean.

## Task Commits

Each task was committed atomically:

1. **Task 1 (RED): failing CommitRow inSession/isPendingBase cases** - `ec72566` (test)
2. **Task 1 (GREEN): SessionCommit type + CommitRow tints** - `7cfd92a` (feat)
3. **Task 2: ReviewPanel session-commit list + per-row remove** - `6dc3b84` (feat)

_Task 1 is the tdd="true" task: test → feat (no refactor commit needed)._

## Files Created/Modified
- `src/lib/types.ts` - Added `SessionCommit` interface mirroring the Rust struct.
- `src/components/CommitRow.svelte` - Added `inSession`/`isPendingBase` props (false defaults) and a `reviewMarker` $derived box-shadow wired into the row style expression.
- `src/components/CommitRow.test.ts` - Extended with 6 cases asserting marker appearance/disappearance, distinctness, no-literal-color, and both-on coexistence.
- `src/components/ReviewPanel.svelte` - Added `sessionCommits` state, `reloadCommits`/`removeCommit`, list+× markup in the active branch, and reload wiring in the mount effect + session-changed listener.
- `src/components/ReviewPanel.test.ts` - Extended `setStatus` to seed commits; added 3 cases (list renders SHA+summary, × invokes remove with correct oid, session-changed reloads list); updated the empty-state text assertion.
- `src/app.css` - Added `--color-review-row` and `--color-review-pending-base` next to `--color-selected-row`.

## Decisions Made
- Reused existing accents (`--color-accent`, `--color-warning`) behind two new named review vars rather than inventing new raw colors — keeps the theme surface small and the markers semantically distinct.
- Used `box-shadow` insets on distinct edges so the two markers (and the existing background tints) never collide.
- Kept the list error path silent (inactive session is normal) and the remove path silent-success (relies on the backend `session-changed` emit, consistent with the codebase silent-success pattern).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated stale empty-state assertion after relabeling placeholder**
- **Found during:** Task 2 (ReviewPanel list)
- **Issue:** The Phase 65 stub active-branch placeholder read "No comments yet"; this plan replaces that region with the commit list, and the empty-list text should describe commits, not comments. The existing test asserted the old string and would have failed.
- **Fix:** Relabeled the placeholder to "No commits selected yet" and updated the corresponding test assertion.
- **Files modified:** src/components/ReviewPanel.svelte, src/components/ReviewPanel.test.ts
- **Verification:** `npx vitest run ReviewPanel` green (8/8).
- **Committed in:** 6dc3b84 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug — stale assertion / placeholder wording).
**Impact on plan:** Minimal; keeps the active-branch copy accurate to what the panel now shows. No scope creep.

## Issues Encountered
- `just check` could not run end-to-end: biome's `ci` discovers nested `biome.json` files in sibling parallel-agent worktrees (`.claude/worktrees/agent-*/biome.json`) nested under the main repo and aborts with "nested root configuration". This is a pre-existing worktree-isolation environment issue, NOT caused by this plan's changes. Worked around by running the gate's components scoped to this plan's files: `npx svelte-check` (0 errors), `bunx biome check <my files>` (clean after one autofix on an import line), and `npx vitest run CommitRow ReviewPanel` (20/20 green).

## Threat Surface
No new trust boundary introduced. Per the plan threat register (T-66-03, disposition: accept), both additions are pure presentation of data the backend already returns; the D-04 marker stays on the HTML row (never the SVG pipeline) per `.claude/rules/commit-graph.md`. No network, secrets, or auth.

## Known Stubs
None introduced by this plan. The ReviewPanel itself remains the Phase 65 D-12 throwaway stub (replaced by the real panel in Phase 69) — the list added here is the deliberate D-05 minimal version, not a data stub.

## Next Phase Readiness
- Plan 04 (CommitGraph) can wire `inSession={sessionOids.has(commit.oid)}` and `isPendingBase={...}` at the CommitRow call site without editing CommitRow — props are owned and defaulted here, avoiding the Wave 2/3 file collision.
- `SessionCommit` is available for any consumer needing the in-session list shape.

## Self-Check: PASSED

- Files: FOUND src/lib/types.ts, src/components/CommitRow.svelte, src/components/CommitRow.test.ts, src/components/ReviewPanel.svelte, src/components/ReviewPanel.test.ts, src/app.css
- Commits: FOUND ec72566, 7cfd92a, 6dc3b84

---
*Phase: 66-commit-selection*
*Completed: 2026-05-25*
