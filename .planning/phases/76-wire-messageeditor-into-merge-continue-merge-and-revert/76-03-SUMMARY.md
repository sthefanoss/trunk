---
phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert
plan: 03
subsystem: ui
tags: [svelte5, vitest, message-editor, merge, revert, context-menu, runes]

# Dependency graph
requires:
  - phase: 75-message-editor-infrastructure
    provides: "MessageEditor.open(default) => Promise<string|null> frozen modal with title $props"
  - phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert
    plan: 01
    provides: "merge_branch_begin (MergeBeginResult kind switch) + merge_continue"
  - phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert
    plan: 02
    provides: "revert_commit_begin (RevertBeginResult) + revert_continue + revert_abort"
provides:
  - "RepoView hosts a single <MessageEditor bind:this> with a reactive $state title set before open() (D-03/D-04)"
  - "handleOpenMessageEditor(default, title) => Promise<string|null> host handler"
  - "onopenmessageeditor prop contract threaded to CommitGraph, BranchSidebar, StagingPanel (Plan 04 consumes the StagingPanel/OperationBanner leg)"
  - "CommitGraph revert routed: revert_commit_begin -> editor -> revert_continue (null => no commit)"
  - "CommitGraph + BranchSidebar merge routed: merge_branch_begin kind switch; ready -> editor -> merge_continue; ff/conflicts skip the editor"
affects: [76-04 StagingPanel merge-continue + OperationBanner revert continue/abort UAT]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Single RepoView-hosted modal with reactive $state title flipped per-operation before await ref.open() (mirrors onopenrebaseeditor host+thread)"
    - "Two-step begin -> MessageEditor -> continue trigger-site routing with a single null-guard (cancel/empty leaves the recoverable in-progress state)"
    - "vitest menu-action-capture: the @tauri-apps/api/menu MenuItem.new mock records { text -> action } so context-menu-wired handlers are reachable in jsdom by firing the exact captured callback"

key-files:
  created: []
  modified:
    - src/components/RepoView.svelte
    - src/components/CommitGraph.svelte
    - src/components/BranchSidebar.svelte
    - src/components/StagingPanel.svelte
    - src/components/RepoView.test.ts
    - src/components/CommitGraph.test.ts
    - src/components/BranchSidebar.test.ts

key-decisions:
  - "Self-contained local mocks in BranchSidebar.test.ts (matching CommitGraph.test.ts) instead of a shared menu-capture helper — a named import from the shared tauri-mock reordered vi.mock hoisting and detached the invoke mock the component sees"
  - "StagingPanel.onopenmessageeditor declared (optional) but unused this plan so RepoView threads it without a svelte-check unknown-prop error; Plan 04 wires its merge-continue/OperationBanner consumption"

patterns-established:
  - "Pattern: per-operation modal title via host $state — set editorTitle before await ref.open(default), reuse one MessageEditor instance for all callers"
  - "Pattern: menu-action capture in vitest — drive Tauri-context-menu handlers by invoking the recorded action callback, the genuine user gesture, not by reaching into private functions"

# Frontend trigger-site slice of MSG-02/MSG-03/MSG-06. Per 76-02's gate, the
# user-facing requirements stay In Progress in REQUIREMENTS.md until the Plan 04
# UAT checkpoint — do NOT auto-flip to Complete from this list.
requirements-completed: [MSG-02, MSG-03, MSG-06]

# Metrics
duration: 38min
completed: 2026-05-29
---

# Phase 76 Plan 03: Wire MessageEditor into merge + revert trigger sites Summary

**A single RepoView-hosted MessageEditor with a reactive per-operation title, threaded as `onopenmessageeditor` into CommitGraph (revert + merge) and BranchSidebar (merge); each trigger now runs the two-step `*_begin` -> edit -> `*_continue` flow, skips the editor on fast-forward/conflict, and makes no commit on cancel.**

## Performance

- **Duration:** ~38 min
- **Started:** 2026-05-29
- **Completed:** 2026-05-29
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- RepoView hosts one `<MessageEditor bind:this={messageEditorRef} title={editorTitle} />`; `handleOpenMessageEditor(default, title)` sets the reactive `editorTitle` before `await open()` and returns `string | null` (D-03/D-04).
- `onopenmessageeditor` threaded to CommitGraph, BranchSidebar, and StagingPanel — the prop contract Plan 04 consumes for the merge-continue/OperationBanner-revert leg.
- CommitGraph `handleRevert`: `revert_commit_begin` -> editor (`"Revert commit message"`) -> `revert_continue`; null return leaves the staged revert in its recoverable state with no commit (D-02 / MSG-06).
- CommitGraph + BranchSidebar `handleMergeBranch`: `merge_branch_begin` -> `result.kind` switch; `ready` opens the editor (`"Merge commit message"`) then `merge_continue`; `fast_forwarded`/`conflicts` open no editor (MSG-02 ff skip). Both merge trigger sites route — no second silent merge path remains.
- Old direct `merge_branch` / `revert_commit` invokes removed from both components (closed the planned wave seam from 76-01/76-02).
- Behavior-level tests added that drive the real context-menu action callbacks (revert + merge ready/cancel/ff/conflicts).

## Task Commits

1. **Task 1: Host MessageEditor in RepoView + thread onopenmessageeditor** - `9a37c6d` (feat)
2. **Task 2: Route CommitGraph + BranchSidebar merge/revert through the editor** - `7b2d316` (feat)

**Plan metadata:** (this docs commit)

## Files Created/Modified
- `src/components/RepoView.svelte` - Import MessageEditor; `messageEditorRef` + reactive `editorTitle` `$state`; `handleOpenMessageEditor`; single host render; `onopenmessageeditor` threaded to 3 children.
- `src/components/CommitGraph.svelte` - `onopenmessageeditor` prop; `handleRevert` and `handleMergeBranch` rewired to the two-step begin -> editor -> continue flow with `kind` discrimination and null guard.
- `src/components/BranchSidebar.svelte` - `onopenmessageeditor` prop; `handleMergeBranch` rewired identically, preserving the post-step `loadRefs`/`onrefreshed`.
- `src/components/StagingPanel.svelte` - declared optional `onopenmessageeditor` prop (threaded now, consumed in Plan 04).
- `src/components/RepoView.test.ts` - mount-regression test for the added host.
- `src/components/CommitGraph.test.ts` - menu-action-capture mock + 5 revert/merge routing behaviors.
- `src/components/BranchSidebar.test.ts` - self-contained local Tauri mocks (incl. menu capture) + 3 merge routing behaviors.

## Decisions Made
- **Self-contained local mocks in `BranchSidebar.test.ts`.** I first added a shared menu-capture helper to `src/__tests__/helpers/tauri-mock.ts` and imported its named exports. That reordered `vi.mock` hoisting so the test's local `@tauri-apps/api/core` mock no longer won — the component used a different `invoke` instance and `list_refs` returned `undefined`, timing out every render-based test. Reverted the shared helper and made BranchSidebar.test.ts declare all its Tauri mocks locally (the proven CommitGraph.test.ts shape). No shared infra touched.
- **`StagingPanel.onopenmessageeditor` declared but unused this plan.** RepoView threads it onto StagingPanel now (Task 1 scope); the prop must exist on StagingPanel's `Props` or svelte-check rejects the unknown attribute. Biome does not flag the unused destructured prop. Plan 04 wires its consumption.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Declared `onopenmessageeditor` on StagingPanel to keep svelte-check green**
- **Found during:** Task 1 (threading the prop onto StagingPanel)
- **Issue:** RepoView passing `onopenmessageeditor` to `<StagingPanel>` produced a svelte-check error ("Object literal may only specify known properties") because StagingPanel's strict `Props` interface had no rest spread and did not declare it.
- **Fix:** Added the optional `onopenmessageeditor?: (default, title) => Promise<string | null>` to StagingPanel's `Props` and destructured it (unused until Plan 04).
- **Files modified:** src/components/StagingPanel.svelte
- **Verification:** `just check` (svelte-check) green.
- **Committed in:** `9a37c6d` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 3 blocking). The plan's `<action>` already anticipated threading to StagingPanel; declaring the prop there is the minimal compile-clean realization of that instruction. No scope creep, no contract drift.

## Issues Encountered
- **Menu-wired handlers not reachable in jsdom.** `handleMergeBranch`/`handleRevert` are only triggered through Tauri context-menu `action` callbacks, which the default menu mocks discard. Resolved by enhancing the `MenuItem.new` mock to record `{ text -> action }` and firing the captured action by text — the genuine user gesture, not a private-function reach-in.
- **Shared-mock hoisting regression** (see Decisions) — surfaced by every render-based BranchSidebar test timing out at 1007ms; resolved by reverting to self-contained local mocks. No fix-attempt limit reached.

## Known Stubs
None for this plan's scope. `StagingPanel.onopenmessageeditor` is declared-and-threaded but intentionally not yet consumed — Plan 04 wires the StagingPanel merge-continue and OperationBanner revert-continue/abort that use it. This is the planned wave seam, documented in 76-RESEARCH Open Question 2 (RESOLVED, Plan 04).

## Threat Flags
None. No new network endpoints, auth paths, file access, or schema changes. The edited message crosses WebView -> Rust as a serde-serialized IPC arg (T-76-07, already mitigated by the begin/continue backend in Plans 01/02); the frontend adds no shell surface.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `onopenmessageeditor` is live on StagingPanel; Plan 04 threads it down to OperationBanner and wires `get_merge_message` -> editor -> `merge_continue` for the staging merge-continue path and `revert_continue`/`revert_abort` for the Revert banner.
- Single reactive-title host is ready to serve a fourth caller without a new instance.

## Self-Check: PASSED
- FOUND: src/components/RepoView.svelte, CommitGraph.svelte, BranchSidebar.svelte, StagingPanel.svelte
- FOUND: 76-03-SUMMARY.md
- FOUND: commit 9a37c6d (Task 1), 7b2d316 (Task 2)
- `just check` fully green (fmt, biome, svelte-check, clippy, cargo tests, 575 vitest tests)

---
*Phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert*
*Completed: 2026-05-29*
