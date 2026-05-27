---
phase: 73-review-lifecycle-end-review-cold-boot-resume
plan: 03
subsystem: ui
tags: [svelte, review-session, empty-states, multi-tab, tdd]

# Dependency graph
requires:
  - phase: 73-review-lifecycle-end-review-cold-boot-resume
    plan: 01
    provides: sessionState rune + installReads dispatcher + fireSessionChanged helper
  - phase: 73-review-lifecycle-end-review-cold-boot-resume
    plan: 02
    provides: End-review button gated on sessionState !== "none"
provides:
  - Three-way empty-state branching gated on sessionState + groups + comments arity
  - Session-summary caption `{N} comments · {M} commits` (U+00B7) above the list
  - Multi-tab coordination assertion (emergent — no new production code)
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Specificity-first empty-state branching: cold (sessionState === 'none') → warm-no-commits → warm-with-commits-zero-comments. Three mutually-exclusive {#if} arms consume the lifecycle rune (Plan 01) without adding new state."
    - "Multi-tab coordination assertion is emergent: tab-A `end_review_session` → session-changed → tab-B listener reload → cold render. Zero production code change in Plan 03 (Tasks 1 + 2). Plan 01's rune + Plan 02's gate + Plan 03's branching compose into the observable behavior."

key-files:
  created: []
  modified:
    - src/components/ReviewPanel.svelte
    - src/components/ReviewPanel.test.ts

key-decisions:
  - "Phase 73-03: empty-state branches kept inline (3 × 6 lines) instead of extracted into a {#snippet}. Per plan REFACTOR guidance + coding_style.md §1 ('reveals intent' before 'no duplication'): top-to-bottom each branch reads as a single visual unit; chasing a snippet abstraction would add indirection without removing meaningful repetition."
  - "Phase 73-03: existing test 'shows the no-comments empty state when commits exist but no comments' renamed to 'warm-with-commits' and updated to assert 'Review started.' — the prior assertion was the only test touching the replaced copy. No other tests needed audit-updating (grep confirmed)."
  - "Phase 73-03: multi-tab Test 1 uses a per-test closure-variable dispatcher (currentStatus / currentCommits / currentComments / currentResolutions) instead of installReads, because installReads bakes in a one-shot `resumed` flip whereas this test needs a free-form mid-test mutation. Local helper, ~10 lines, only test in the file that needs it."

patterns-established:
  - "Cold-vs-warm empty-state gate: render branches off `sessionState === 'none'` first, then off `groups.length === 0` (commits arity), then off `!hasAnyComment` (comments arity). All other UI affordances that need cold-vs-warm distinction (Plan 02's End button is the precedent — `{#if sessionState !== 'none'}`) follow the same gate."
  - "Cross-tab coordination test pattern: drive observable state via fireSessionChanged + dispatcher swap, assert on DOM reflecting the post-event reload. No production code under test in Plan 03's multi-tab block — the assertion proves the composition of Plan 01's listener + Plan 02's gate + this plan's branching."

requirements-completed:
  - REQ-73-EMPTY
  - REQ-73-SUMMARY
  - REQ-73-MULTITAB
  - REQ-73-NYQUIST
  - REQ-73-CHECK

# Metrics
duration: 6min
completed: 2026-05-27
---

# Phase 73 Plan 03: Empty-State Gating + Summary Caption + Multi-tab Coordination Summary

**Three-way empty-state branching, session summary caption, and multi-tab coordination assertion close Phase 73. Seven new TDD-driven tests (44/44 in file pass, 547/547 across vitest); zero backend changes; the session-changed listener at ReviewPanel.svelte:447-461 is byte-for-byte unchanged across all three plans (D-09).**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-05-27T15:44:01Z
- **Completed:** 2026-05-27T15:49:59Z
- **Tasks:** 3 (1 TDD feature + 1 multi-tab assertion + 1 phase gate)
- **Files modified:** 2 (`ReviewPanel.svelte`, `ReviewPanel.test.ts`)
- **Tests:** 7 new in Plan 03 (5 empty-state + summary + 2 multi-tab); 18 total across the phase (5 cold-boot + 6 End + 7 here); 44/44 in ReviewPanel.test.ts pass

## Accomplishments

- **Three-way empty-state branching** at `ReviewPanel.svelte:537-561`. Specificity-first arm order:
  - `sessionState === "none"` → cold: "No active review" / "Toggle review mode in the toolbar to start."
  - `groups.length === 0` → warm-no-commits: "No commits in this review yet." / "Add commits from the graph to start reviewing." (existing copy preserved verbatim)
  - `!hasAnyComment` → warm-with-commits: "Review started." / "Select diff lines or add a commit note to comment." (replaces prior "No comments yet.")
- **Session-summary caption** at `ReviewPanel.svelte:527-531`. `{comments.length} comments · {commits.length} commits` (U+00B7 middle dot, no pluralization, font-size 11px in `var(--color-text-muted)`). Visible when `sessionState !== "none"`; sits ABOVE the empty-state block inside the scroll-body container.
- **Empty-state tests** (Task 1): 3 new in `describe("empty states")` cover all three branches; one existing test rephrased + retargeted to the new warm-with-commits copy.
- **Summary-line tests** (Task 1): 2 new in `describe("summary line")` — caption visible during active session, hidden in cold branch.
- **Multi-tab coordination tests** (Task 2): 2 new in `describe("multi-tab coordination")` — tab-A End → cold render in tab-B via `fireSessionChanged`; cross-repo payload filtered by the listener's `canonicalPath` guard (no IPC churn).
- All seven new tests use REAL timers + the file-global `flush()` helper (no fake-timer scoping needed; these arms don't depend on setTimeout).

## Task Commits

1. **Task 1 (RED): Add failing tests for empty states + summary line** — `3292fd8` (test)
2. **Task 1 (GREEN): Three-way empty-state branching + session summary caption** — `3ade24e` (feat)
3. **Task 2 (RED+GREEN): Multi-tab coordination tests** — `778c4f7` (test, no production change)
4. **Task 3 (phase gate): biome formatting** — `5687dd9` (chore, no semantic change)

REFACTOR step examined and skipped: the three empty-state branches share an outer flex-column wrapper + two-span structure (3 × 6 lines). Per plan-prose REFACTOR guidance ("Only do this if the executor sees genuine duplication that obscures intent") + coding_style.md §1 ("reveals intent" before "no duplication"), the duplication is small enough that reading top-to-bottom is clearer than chasing a `{#snippet}` extraction. The three arms are also semantically distinct (different copy, different gating predicate); collapsing them would either parameterize all six fields or fork into helpers per arm — neither simpler than what's inline.

## Files Created/Modified

- `src/components/ReviewPanel.svelte` — Added summary-caption span at lines 527-531; replaced existing 2-arm empty-state block (lines 524-538) with 3-arm specificity-first block at lines 540-561. No other edits; session-changed listener at lines 447-461 untouched.
- `src/components/ReviewPanel.test.ts` — Updated one existing assertion at line 245 ("No comments yet." → "Review started."); added 3 top-level describes at file scope (`empty states`, `summary line`, `multi-tab coordination`) totaling 7 new tests. Biome reformatted one long `getByRole` regex line in the multi-tab block (chore commit; no semantic change).

## Decisions Made

- **Empty-state branches inline, not extracted.** Plan REFACTOR guidance + coding_style.md align: 18 lines of "duplication" across three semantically-distinct arms is not obscuring intent; extracting a `<EmptyState heading body>` snippet would either parameterize six fields (more types, more indirection) or fork into per-arm helpers (more files, same line count). Read top-to-bottom each branch tells you exactly what renders for which state.
- **Existing "no-comments" test rephrased, not deleted.** The test continues to exercise the same code path (commits exist + zero comments → empty state); only the assertion string changes. Renaming from `"shows the no-comments empty state"` to `"shows the warm-with-commits empty state"` keeps the regression-protection intent — if Plan 04 ever re-introduces a third empty-state arm with "No comments yet." copy, this test will catch the conflict.
- **Multi-tab Test 1 uses a per-test closure-variable dispatcher, not installReads.** `installReads` is built for one-shot state — its `resumed` flag flips after the FIRST `resume_review_session` call, not arbitrary mid-test mutations. Test 1 needs to flip status + reads MID-TEST after `fireSessionChanged`, which is what a free-form closure captures. ~25 lines local to one test; not a general pattern worth promoting.
- **Multi-tab Test 2 asserts on `safeInvoke` call-count delta, not `reload()` invocation.** `reload()` is a private function inside the Svelte component — observing it would require either an export hatch or spying. The IPC call-count is the canonical observable: if the listener filter short-circuits, no IPC fires. The assertion `mock.calls.length` before/after === unchanged proves the filter held without coupling to implementation details (testing/00-index.md: prefer output/state over interaction).

## Deviations from Plan

None for Task 1 (RED+GREEN executed exactly as written). Task 2 deviated only in that the predicted "tests should pass immediately" prediction held — no GREEN-step bug surfaced, no production code change needed; both tests passed on the first run. The plan anticipated this exact outcome ("Both tests SHOULD pass immediately — the multi-tab behavior is entirely emergent...").

## Issues Encountered

- **`just check` biome formatter complained on one long line** in the multi-tab Test 1 (`screen.queryByRole("button", { name: /End review/ })` exceeded the line-length limit when chained on a single line). Resolved by `pnpm exec biome check --write` (one wrap; no semantic change). Rolled into the final phase-gate commit (`chore(73-03): apply biome formatting to multi-tab test`).
- Same root cause as Plan 02's biome friction — the configured line length is occasionally tighter than what reads natural inline. Worth flagging as a small follow-up (see Reflection §4).

## Pre-existing Issues (Ownership Note)

Per `ownership.md` — surfaced but still not fixed (out of `files_modified` scope through Plan 03):

- `src/components/diff/CommentComposer.svelte:43` — three `lint/style/noNonNullAssertion` warnings (Biome). Pre-existing; first surfaced in 73-01-SUMMARY; still outstanding. `just check` exits 0 (warnings, not errors). Concrete follow-up: replace the three `!` chains (`file!`, `hunkIdx!`, `selectedLineIndices!`) with proper narrowing on the `$derived` `captured` value — likely a Zod-style guard or an early-return path. Tracked here and in 73-01 / 73-02 summaries; appropriate scope is a future quick-task (`/gsd:quick fix CommentComposer non-null assertions`).

## Threat-Model Compliance

Plan 03 threat register dispositions all hold:

- **T-73-09 (Spoofing — session-changed for unrelated repo) — mitigated.** Test 2 in `describe("multi-tab coordination")` proves the filter at `ReviewPanel.svelte:451` short-circuits: a `fireSessionChanged("/different-repo")` call after the listener has set `canonicalPath = "/repo"` produces zero additional `safeInvoke` calls and leaves the rendered comment in place.
- **T-73-10 (Tampering — race during tab-B reload) — accepted.** Backend serialization invariant (Phase 65 `Mutex<HashMap<...>>` + atomic file ops) was already in place; Plan 03 added no new race surface. Test 1 exercises the deterministic post-deletion path (status flips to `none`, reads return empty arrays, cold empty state renders).
- **T-73-11 (Information Disclosure — caption metadata) — accepted.** Caption counts derive from already-rendered comments/commits arrays; no new IPC; no PII.

**Listener UNTOUCHED invariant (D-09):**
```bash
$ git diff 272e01b..HEAD -- src/components/ReviewPanel.svelte | \
    grep -E 'listen<string>\("session-changed"|if \(canonicalPath'
# (no output — byte-for-byte unchanged)
```
The session-changed listener at lines 447-461 has zero edits across Plans 01, 02, and 03. The `if (canonicalPath && event.payload !== canonicalPath) return;` guard at line 451 is the single line that all three plans depend on for cross-tab isolation.

## Phase 73 REQ → Plan Traceability

| Requirement      | Plan(s)              | Evidence                                                                                |
|------------------|----------------------|-----------------------------------------------------------------------------------------|
| REQ-73-RESUME    | 01                   | `feat(73-01): wire cold-boot resume in ReviewPanel.reload()` (c40ec28)                  |
| REQ-73-END       | 02                   | `feat(73-02): add two-step End-review button to ReviewPanel` (e26d60d)                  |
| REQ-73-EMPTY     | 03 Task 1            | `feat(73-03): three-way empty-state branching…` (3ade24e); 3 `describe("empty states")` tests pass |
| REQ-73-SUMMARY   | 03 Task 1            | summary-caption span at ReviewPanel.svelte:527-531; 2 `describe("summary line")` tests pass |
| REQ-73-MULTITAB  | 03 Task 2            | `test(73-03): multi-tab coordination tests` (778c4f7); 2 tests assert tab-A End → tab-B cold render + cross-repo filter |
| REQ-73-NYQUIST   | 01 + 02 + 03         | 18 automated tests cover the five lifecycle paths (cold resume / end confirm flow / empty-state transitions / summary line / multi-tab) |
| REQ-73-CHECK     | 03 Task 3            | `just check` exit 0 (validated 2026-05-27)                                              |

## Manual Smoke Tests

Per 73-VALIDATION.md § "Manual-Only Verifications" — three rows tracked for `/gsd:verify-work 73`. None blocking for plan-level merge gate; recorded here for the verifier to spot-check.

| # | Smoke                                                                          | Status   | Note                                                                                                         |
|---|--------------------------------------------------------------------------------|----------|--------------------------------------------------------------------------------------------------------------|
| 1 | Cold-boot resume across actual app restart (`pnpm tauri dev`, open repo with on-disk session, comments appear without mutation) | OUTSTANDING | Plan 01 deliverable; non-blocking. Automated coverage at `describe("cold-boot resume")` × 5 tests. |
| 2 | Multi-tab End → empty state in other window (open two windows on same repo, end review in A, observe B re-render to "No active review") | OUTSTANDING | Plan 02 + Plan 03 deliverable; non-blocking. Automated coverage at `describe("multi-tab coordination")` Test 1. |
| 3 | Visual treatment of two-step End button (click End once, observe "Click again to confirm" + danger-tinted background + ~3s auto-revert) | OUTSTANDING | Plan 02 deliverable; non-blocking. Automated coverage at `describe("End review")` × 6 tests. |

All three smokes require a real Tauri runtime + a real on-disk repo; the test harness can only assert the IPC + DOM contract. The visual + cross-window assertions are inherently manual.

## Next Phase Readiness

- Phase 73 is COMPLETE. All five lifecycle requirements (RESUME, END, EMPTY, SUMMARY, MULTITAB) plus the cross-cutting two (NYQUIST, CHECK) are closed.
- Ready for `/gsd:verify-work 73`: the three manual smokes above, the 18-test automated coverage, and the listener-untouched invariant are the spot-check surface.
- No follow-up plans needed within Phase 73. The pre-existing CommentComposer.svelte:43 noNonNullAssertion warnings are tracked across all three SUMMARYs for a future `/gsd:quick` task.

## Continuous Improvement Reflection (continuous_improvement.md §1)

1. **What was harder than expected?** Nothing was hard — Plans 01 and 02 had already absorbed the friction (errorMessage template-literal shape, fake-timer scoping, dispatcher patterns), so Plan 03's tests slotted in directly. The closest thing to friction was deciding whether Multi-tab Test 1 should reuse `installReads` or fork a per-test dispatcher (chose the fork; documented why).
2. **Was anything done twice?** No — the biome long-line wrap repeated Plan 02's same friction, but it's the formatter doing the work both times, not the human. Zero repeated manual steps.
3. **Did I make any incorrect assumptions?** Slight: I initially expected the existing test `it("shows the no-comments empty state...")` to be the only one referencing the replaced copy. A grep confirmed it was — no other audit-update needed.
4. **Is there a follow-up improvement?** Yes — small, recurring: the biome line-length rule is tighter than the natural readable inline of some test-helper invocations (Plan 02 and Plan 03 both tripped it). **Friction:** `just check` fails after the GREEN step on one auto-wrappable line, requiring a `biome --write` + extra commit. **Root cause:** the test-file long-line pattern (`screen.queryByRole("button", { name: /regex/ })`) is just over the limit when inlined into `expect(...).toBeNull()`. **Proposed fix:** either (a) configure biome to allow the project's natural test-call width, or (b) extract a `getXxxButton()` query helper for any role+regex combination used twice — Plan 02 did this with `getEndButton()`, Plan 03 didn't because it was a one-off. **Benefit:** eliminates the extra biome chore-commit per plan (saves ~1 min × N plans). **Cost:** 5 min to either tune biome config or add the helper.
5. **Should any memory files be updated?** Yes — one entry worth adding to `~/.claude/projects/-Users-joaofnds-code-trunk/memory/MEMORY.md` under "GSD workflow gotchas": **"Biome line-length tighter than natural test-call inlining — expect a `biome --write` pass after any TDD feature that adds new `expect(screen.queryByRole(...))` calls; commit it as a separate chore."** This would have saved the brief mid-task surprise both Plan 02 and Plan 03 hit.

## Self-Check

- `git log --oneline | grep '73-03'` — 4 commits present: `3292fd8` (test), `3ade24e` (feat), `778c4f7` (test), `5687dd9` (chore).
- `pnpm vitest run src/components/ReviewPanel.test.ts` — 44/44 pass.
- `pnpm vitest run` — 547/547 pass (no regressions phase-wide).
- `just check` — exit 0.
- `grep -n 'No active review' src/components/ReviewPanel.svelte` — line 540 (cold heading present).
- `grep -n 'Review started\.' src/components/ReviewPanel.svelte` — line 554 (warm-with-commits heading present).
- `grep -n 'No commits in this review yet\.' src/components/ReviewPanel.svelte` — line 547 (preserved verbatim).
- `grep -n 'comments · ' src/components/ReviewPanel.svelte` — line 529 (caption with U+00B7 present).
- `grep -n '"No comments yet"' src/components/ReviewPanel.svelte` — no hits (prior copy replaced).
- `git diff 272e01b..HEAD -- src/components/ReviewPanel.svelte | grep -E 'listen<string>\("session-changed"|if \(canonicalPath'` — no output (session-changed listener byte-for-byte unchanged across all three plans, D-09 invariant).

## Self-Check: PASSED

---
*Phase: 73-review-lifecycle-end-review-cold-boot-resume*
*Completed: 2026-05-27*
