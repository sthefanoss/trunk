---
phase: 66-commit-selection
verified: 2026-05-25T14:00:00Z
status: human_needed
score: 20/20 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Two-right-click range gesture seeding"
    expected: "Right-click commit A → 'Set as review base' highlights row A with pending-base marker (bottom inset, --color-review-pending-base). Right-click a descendant commit B → 'Add range to review' seeds the inclusive [A..B] range into the session and clears the pending-base highlight."
    why_human: "Tauri Menu.popup() and transient $state highlight driven by native context-menu events; vitest cannot drive native right-click gestures"
  - test: "Add to review from context menu"
    expected: "Right-click a commit NOT in the session → menu item reads 'Add to review'; clicking it adds the row in-session marker (left inset, --color-review-row) and the commit appears in the ReviewPanel list"
    why_human: "Requires visual inspection of the in-session box-shadow inset and context-menu label text in the running Tauri app"
  - test: "Remove from review from context menu"
    expected: "Right-click a commit ALREADY in the session → menu item reads 'Remove from review'; clicking it removes the in-session marker and the commit disappears from the ReviewPanel list"
    why_human: "Toggle label flip and disappearance of the box-shadow marker require visual inspection"
  - test: "Merge commit is selectable (D-08)"
    expected: "Right-click a MERGE commit → 'Add to review' item is ENABLED (not greyed out); clicking adds the merge to the session and it appears in the panel. The merge commit can also be used as a range base or range tip."
    why_human: "D-08 requires no is_merge gate; enabled/disabled state of a native Tauri menu item is only visible in the running app"
  - test: "Invalid range shows toast and leaves session unchanged"
    expected: "Set a review base on commit A, then right-click an UNRELATED or SIBLING commit B (not a descendant of A) → 'Add range to review' shows a toast error (e.g. 'Base is not an ancestor of tip') and the session set is unchanged; the pending-base highlight clears either way"
    why_human: "Toast appearance and session-unchanged state require running the app end-to-end; cannot be driven by vitest"
  - test: "Clear review base cancels range gesture"
    expected: "With a pending base set, right-click any commit → 'Clear review base' item present; clicking it clears the pending-base highlight without seeding any range"
    why_human: "Cancel affordance interaction and highlight-clearing require visual inspection in the running app"
  - test: "ReviewPanel per-row remove button"
    expected: "In the panel's commit list, each row has a × button; clicking it removes that commit from the session (row disappears from panel, in-session marker disappears from CommitGraph row)"
    why_human: "Cross-component reactivity (panel remove → graph marker disappears) requires visual inspection in the running app"
  - test: "Range result in panel is graph-ordered with no duplicates"
    expected: "After seeding a range [A..B], the ReviewPanel shows commits in graph order (newest first, matching CommitGraph order) with no duplicate entries"
    why_human: "Graph ordering and deduplication require visual comparison between the panel list and the commit graph in the running app"
  - test: "Pending-base highlight clears when session becomes inactive"
    expected: "If a review session is closed while a pending base is set, the pending-base highlight on the CommitGraph row clears (no stale highlight with no active session)"
    why_human: "Session lifecycle × pending-base state interaction requires running the app"
  - test: "session-changed sync across multiple windows (optional)"
    expected: "Open the same repo in two windows; add/remove a commit in one window; the other window reflects the change (in-session markers and panel list update) via the session-changed event"
    why_human: "Cross-window IPC sync requires two running Tauri windows; cannot be driven by vitest"
---

# Phase 66: Commit Selection Verification Report

**Phase Goal:** User can define which commits the review covers, by range and by hand-picking from the graph.
**Verified:** 2026-05-25T14:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

All 20 automated must-have truths are VERIFIED. One category of must-haves — native Tauri context-menu gesture behavior and visual marker appearance — cannot be verified programmatically and requires human testing in the running app (`just dev`).

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | D-02: `compute_range_oids(base, tip)` returns inclusive set with both endpoints | VERIFIED | `review.rs:200-220`; `compute_range_oids_is_inclusive` test at line ~720 passes |
| 2 | CR-01 fix: all parents of a merge base are hidden (no side-branch leak) | VERIFIED | `review.rs:212-215` iterates `for i in 0..base_commit.parent_count()`; regression test `seed_range_merge_base_excludes_side_branch` passes |
| 3 | D-03: `union_dedup` / `apply_add` deduplicate across range seeds | VERIFIED | `review.rs:225-247`; `union_dedup_no_duplicates` and `union_dedup_preserves_order` tests pass |
| 4 | D-08: merge commits are selectable — no `is_merge` gate in selection logic | VERIFIED | `review.rs`: no `is_merge` check in `validate_range`, `compute_range_oids`, or `apply_add`; `merge_commit_selectable` test passes |
| 5 | Invalid ranges return typed errors (`bad_range`, `unrelated_history`) | VERIFIED | `review.rs:182-197`; `validate_range_rejects_invalid` test passes |
| 6 | SEL-04: `intersect_graph_order` returns commits in graph order | VERIFIED | `review.rs:250-298`; `intersect_graph_order_preserves_graph_order` test passes |
| 7 | D-03: `seed_review_range` uses union semantics (additive, deduplicating) | VERIFIED | `review.rs:365-398`: calls `compute_range_oids` then `union_dedup` in RMW closure |
| 8 | Mutex-serialized RMW prevents lost writes on concurrent add/remove | VERIFIED | `review.rs:304-322` `mutate_session_rmw`; `selection_rmw_serialized` test (50 concurrent adds) passes |
| 9 | Dual path-keying: canonical for sessions map, raw path for CommitCache | VERIFIED | `review.rs:370,403,424,448`: `canonical_path` used for `mutate_session_rmw`; `path` used for `get_commit_graph` |
| 10 | `no_session` vs `not_open` error codes are distinct | VERIFIED | `review.rs:317` returns `TrunkError::no_session()`; `list_session_commits` returns `not_open` via cache miss path |
| 11 | All four commands registered in Tauri handler | VERIFIED | `lib.rs:128-131`: `seed_review_range`, `add_review_commit`, `remove_review_commit`, `list_session_commits` in `generate_handler!` |
| 12 | `SessionCommit` TS interface matches Rust `SessionCommit` struct | VERIFIED | `types.ts:331-335`: `{ oid, short_oid, summary }`; `review.rs:53`: same fields with `#[derive(Serialize)]` |
| 13 | D-04: in-session marker is HTML `box-shadow` inset using CSS custom property | VERIFIED | `CommitRow.svelte:69-76,86`: `reviewMarker` `$derived` with `var(--color-review-row)` applied as `box-shadow`; no SVG-layer change |
| 14 | D-04: CSS custom properties defined in app.css | VERIFIED | `app.css:13-14`: `--color-review-row: var(--color-accent)` and `--color-review-pending-base: var(--color-warning)` |
| 15 | D-01: `isPendingBase` prop drives pending-base visual marker | VERIFIED | `CommitRow.svelte:30-31,69-76`: prop with `false` default; `reviewMarker` derived includes pending-base color branch |
| 16 | D-05: ReviewPanel renders a minimal list of session commits | VERIFIED | `ReviewPanel.svelte:108-153`: list rendered under `sessionState === "active"` guard using `sessionCommits` state |
| 17 | D-07: per-row × button removes commit from session | VERIFIED | `ReviewPanel.svelte:53-59,140-145`: `removeCommit()` calls `remove_review_commit`; × button at row level |
| 18 | SEL-04: ReviewPanel reloads commit list on `session-changed` | VERIFIED | `ReviewPanel.svelte:81-93`: `session-changed` listener calls both `reloadStatus()` and `reloadCommits()`, filtered by `canonical_path` |
| 19 | D-01 / D-06 / D-08: `reviewItems` in CommitGraph — no `is_merge` gate | VERIFIED | `CommitGraph.svelte:681-746`: `reviewItems` array has no `enabled: !commit.is_merge`; all review items unconditionally enabled |
| 20 | A1: `sessionActive` derived from `get_review_session_status`, not `reviewPanelOpen` | VERIFIED | `CommitGraph.svelte:307-317`: `sessionActive = $derived(sessionStatus?.state === "active")`; `reviewPanelOpen` is a separate unrelated state |

**Score:** 20/20 automated truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/commands/review.rs` | Selection pure helpers + Tauri command wrappers | VERIFIED | `validate_range`, `compute_range_oids`, `apply_add`, `apply_remove`, `union_dedup`, `intersect_graph_order`, `mutate_session_rmw`, four `#[tauri::command]` fns; 18 tests |
| `src-tauri/src/lib.rs` | Four commands registered in handler | VERIFIED | Lines 128-131 |
| `src/lib/types.ts` | `SessionCommit` interface | VERIFIED | Lines 331-335 |
| `src/components/CommitRow.svelte` | `inSession`/`isPendingBase` props + HTML box-shadow marker | VERIFIED | Lines 29-31, 69-76, 86 |
| `src/app.css` | `--color-review-row` and `--color-review-pending-base` CSS vars | VERIFIED | Lines 13-14 |
| `src/components/ReviewPanel.svelte` | Commit list + × remove + session-changed reload | VERIFIED | Lines 19, 40-93, 108-153 |
| `src/components/CommitGraph.svelte` | `sessionOids`/`sessionActive`/`pendingBase` state + `reviewItems` context-menu block + CommitRow prop wiring | VERIFIED | Lines 307-340, 681-746, 1381, 1937 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `CommitGraph.svelte` | `seed_review_range` | `safeInvoke` in reviewItems handler | VERIFIED | `CommitGraph.svelte:720-734` |
| `CommitGraph.svelte` | `add_review_commit` | `safeInvoke` in reviewItems toggle | VERIFIED | `CommitGraph.svelte:695-704` |
| `CommitGraph.svelte` | `remove_review_commit` | `safeInvoke` in reviewItems toggle | VERIFIED | `CommitGraph.svelte:707-716` |
| `CommitGraph.svelte` | `list_session_commits` | `reloadSession()` on mount + session-changed | VERIFIED | `CommitGraph.svelte:319-340, 1381` |
| `ReviewPanel.svelte` | `remove_review_commit` | `removeCommit()` per-row × click | VERIFIED | `ReviewPanel.svelte:53-59` |
| `ReviewPanel.svelte` | `list_session_commits` | `reloadCommits()` on mount + session-changed | VERIFIED | `ReviewPanel.svelte:40-48, 74-93` |
| `CommitRow.svelte` | `sessionOids` / `pendingBase` in CommitGraph | Props at call site | VERIFIED | `CommitGraph.svelte:1937` |
| Rust `seed_review_range` | `compute_range_oids` + `union_dedup` | Direct call in RMW closure | VERIFIED | `review.rs:383-390` |
| All four commands | `ReviewSessionsState` mutex via `mutate_session_rmw` | Wraps read→mutate→save→write | VERIFIED | `review.rs:304-322` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `CommitGraph.svelte` | `sessionOids` Set | `list_session_commits` → `intersect_graph_order` → Rust query | Yes — `intersect_graph_order` filters real `graph.commits` | FLOWING |
| `CommitGraph.svelte` | `sessionActive` | `get_review_session_status` → Rust `ReviewSessionsState` | Yes — reads from persisted session state | FLOWING |
| `ReviewPanel.svelte` | `sessionCommits` | `list_session_commits` → `intersect_graph_order` → Rust query | Yes — same backend path | FLOWING |
| `CommitRow.svelte` | `inSession` / `isPendingBase` | Props from CommitGraph call site | Yes — props set from live `sessionOids.has()` and `pendingBase` state | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Rust unit tests pass (includes CR-01 regression) | `cd /Users/joaofnds/code/trunk/src-tauri && cargo test --lib review 2>&1 \| tail -3` | 18 passed, 0 failed | PASS |
| TS component tests pass | `cd /Users/joaofnds/code/trunk && npx vitest run CommitRow ReviewPanel 2>&1 \| tail -3` | 20 passed, 0 failed | PASS |
| Full check suite | `just check` (reported by executor) | 469 vitest + all Rust + clippy/fmt/svelte-check/biome: GREEN | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| SEL-01 | 66-01, 66-02, 66-04 | User can seed a range [base..tip] from the graph | SATISFIED | `compute_range_oids` + `seed_review_range` command + D-01 two-right-click gesture in CommitGraph |
| SEL-02 | 66-01, 66-02, 66-04 | User can add/remove individual commits from the graph | SATISFIED | `apply_add`/`apply_remove` + `add_review_commit`/`remove_review_commit` commands + D-06 Add/Remove toggle in CommitGraph |
| SEL-03 | 66-01, 66-02, 66-03, 66-04 | Selection state persists in review session and is visible in the graph | SATISFIED | `mutate_session_rmw` persists via `save_session`; `sessionOids` Set drives `inSession` prop on every CommitRow; `session-changed` event syncs panel and graph |
| SEL-04 | 66-01, 66-02, 66-03 | Session commit list is shown in graph order in the ReviewPanel | SATISFIED | `intersect_graph_order` orders by graph.commits position; ReviewPanel renders `sessionCommits` which comes from this path |

### Anti-Patterns Found

| File | Location | Pattern | Severity | Impact |
|------|----------|---------|----------|--------|
| `CommitGraph.svelte` | line 1381 | WR-01: session-changed guard is null-permissive when `sessionStatus` is null — all cross-repo events trigger reload | WARNING | Spurious reloads in multi-window/multi-repo setups; not data-corrupting. Deferred per 66-REVIEW.md. |
| `CommitGraph.svelte` | lines 334-339 | WR-02: `reloadSession` catch swallows all errors as "no session" — IPC failures and `not_open` silently clear session state | WARNING | Genuine active session can render as "no session" on IPC failure. Deferred per 66-REVIEW.md. |
| `review.rs` | lines 362-393 | WR-03: `seed_review_range` does full git2 revwalk before checking session exists | WARNING | Returns `bad_range`/`unrelated_history` to callers with no session; misleading error on precondition failure. Deferred per 66-REVIEW.md. |
| `review.rs` | lines 391, 411, 431 | WR-04: `app.emit("session-changed")` failures discarded with `let _ =` | WARNING | Disk+memory state changes but listeners not notified; stale UI until next manual reload. Consistent with Phase 65 style. Deferred per 66-REVIEW.md. |

No TBD / FIXME / XXX markers found in files modified by this phase. No stub patterns. No inline color literals. All four warnings are explicitly deferred and documented in 66-REVIEW.md — none are blockers.

**CR-01 (previously BLOCKER) — RESOLVED:** The 66-REVIEW.md identified that `compute_range_oids` only hid `parent_id(0)` for a merge-base commit, leaking the second-parent side branch into the range. The fix (iterating all parents) is present at `review.rs:212-215` and verified by the `seed_range_merge_base_excludes_side_branch` regression test which passes.

### Human Verification Required

All 10 items below require the running Tauri app (`just dev`) because they exercise native `Menu.popup()` context menus, visual CSS marker rendering, and cross-component reactivity that vitest cannot drive.

#### 1. Two-right-click range gesture (D-01)

**Test:** Open a repo with a linear stretch. Start a code review session (View menu → Start Code Review). Right-click a commit A → "Set as review base".
**Expected:** Row A shows the pending-base highlight (bottom inset, `--color-review-pending-base` / amber). Right-click a descendant commit B → "Add range to review" → panel shows inclusive [A..B] range in graph order; pending-base highlight clears.
**Why human:** Native Tauri Menu.popup() events and transient $state highlight; vitest cannot drive.

#### 2. Add to review via context menu (D-06)

**Test:** Right-click a commit NOT in the session.
**Expected:** Menu item reads "Add to review". Clicking adds the row in-session marker (left inset, `--color-review-row` / accent color) and the commit appears in the ReviewPanel list.
**Why human:** Context-menu label text and box-shadow marker require visual inspection in the running app.

#### 3. Remove from review via context menu (D-06)

**Test:** Right-click a commit ALREADY in the session.
**Expected:** Menu item reads "Remove from review". Clicking removes the in-session marker and the commit disappears from the ReviewPanel list.
**Why human:** Toggle label flip and marker disappearance require visual inspection.

#### 4. Merge commit is selectable everywhere (D-08)

**Test:** Right-click a MERGE commit (parent_count >= 2).
**Expected:** "Add to review" is ENABLED (not greyed out). Clicking adds the merge to the session and it appears in the panel. A merge commit can also be used as range base or range tip.
**Why human:** Enabled/disabled state of a native Tauri menu item is only visible in the running app.

#### 5. Invalid range shows toast, session unchanged

**Test:** Set a review base on commit A. Right-click an UNRELATED or SIBLING commit B (not a descendant of A) → "Add range to review".
**Expected:** A toast error appears (e.g. "Base is not an ancestor of tip" or "share no history"). The session set is unchanged. The pending-base highlight clears.
**Why human:** Toast appearance and session-unchanged state require end-to-end app execution.

#### 6. Cancel range gesture with "Clear review base"

**Test:** Set a pending base on any commit. Right-click any other commit.
**Expected:** A "Clear review base" item is present in the menu. Clicking it clears the pending-base highlight without seeding any range.
**Why human:** Cancel affordance and highlight-clearing require visual inspection.

#### 7. ReviewPanel per-row × remove (D-07)

**Test:** With commits in the session list, click a row's × button in ReviewPanel.
**Expected:** The commit disappears from the panel list AND the in-session marker disappears from that row in CommitGraph.
**Why human:** Cross-component reactivity (panel remove → graph marker) requires visual inspection.

#### 8. Range in panel is graph-ordered with no duplicates

**Test:** Seed a range [A..B]. Inspect the ReviewPanel list order.
**Expected:** Commits appear in graph order (newest-first, matching CommitGraph display order) with no duplicate entries.
**Why human:** Graph ordering requires visual comparison between panel list and commit graph in the running app.

#### 9. Pending-base clears when session ends

**Test:** Set a pending base. Then close/stop the review session.
**Expected:** The pending-base highlight on that CommitGraph row clears (no stale marker with no active session).
**Why human:** Session lifecycle × pending-base state interaction requires running the app.

#### 10. session-changed sync across windows (optional)

**Test:** Open the same repo in two windows. Add/remove a commit in one window.
**Expected:** The other window reflects the change (markers and panel list update) via the session-changed event.
**Why human:** Cross-window IPC sync requires two running Tauri windows.

### Gaps Summary

No blocking gaps. All 20 automated must-have truths are VERIFIED. The CR-01 blocker identified in the code review has been fixed and verified by regression test. WR-01..WR-04 are deferred non-blocking warnings documented in 66-REVIEW.md.

Status is `human_needed` because the phase's core interaction model — the two-right-click range gesture, Add/Remove context-menu toggle, transient pending-base highlight, and cross-component session-changed reactivity — is implemented via native Tauri Menu.popup() and CSS box-shadow markers that vitest cannot drive. The 10 human verification items above must be confirmed via `just dev` before the phase can be fully signed off.

---

_Verified: 2026-05-25T14:00:00Z_
_Verifier: Claude (gsd-verifier)_
