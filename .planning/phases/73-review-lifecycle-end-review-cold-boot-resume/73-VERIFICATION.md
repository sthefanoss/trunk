---
phase: 73-review-lifecycle-end-review-cold-boot-resume
verified: 2026-05-27T18:00:00Z
status: human_needed
score: 5/5 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Cold-boot resume across actual app restart"
    expected: "After `pnpm tauri dev` on a repo that already has an on-disk session, opening the ReviewPanel for the first time shows the existing comments without any manual mutation (no toolbar toggle, no Add Note)."
    why_human: "Vitest mocks IPC; only a real Tauri app boot exercises the persisted-state → first-open path end-to-end. Listed in 73-VALIDATION.md §Manual-Only Verifications."
  - test: "Multi-tab End → cold state in other window"
    expected: "Open two real Tauri windows on the same repo. Click End review (twice to confirm) in window A. Window B re-renders to the cold empty state ('No active review') without manual reload."
    why_human: "Requires two real Tauri windows talking through the actual `session-changed` event bus. Listed in 73-VALIDATION.md §Manual-Only Verifications."
  - test: "Visual treatment of two-step End button danger color"
    expected: "Click End review once: label flips to 'Click again to confirm' and the button background tints with the danger color (`--color-danger-bg`/`--color-danger-border`). Wait ~3s: button reverts to idle 'End review'. Hover while confirming: heightened danger contrast."
    why_human: "CSS custom property + Lucide icon contrast — visual eye check, not assertable in jsdom. Listed in 73-VALIDATION.md §Manual-Only Verifications."
---

# Phase 73: Review Lifecycle (End-review + cold-boot resume) Verification Report

**Phase Goal:** A review session has both lifecycle endpoints in the UI — comments appear on first ReviewPanel open after app boot without requiring a mutation, and the user has an explicit End-review affordance so a review is not implicitly permanent.

**Verified:** 2026-05-27T18:00:00Z
**Status:** human_needed (all 5 ROADMAP success criteria verified in code/tests; 3 manual smokes from VALIDATION.md require real Tauri runtime)
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth (ROADMAP SC) | Status | Evidence |
|---|-------------------|--------|----------|
| SC-1 | Cold boot → open ReviewPanel for a repo with an on-disk session → comments appear without any mutation needed | VERIFIED | `ReviewPanel.svelte:255-268` adds cold-boot resume branch inside `reload()`: when `sessionState === "resume-available"`, calls `safeInvoke("resume_review_session", { path: repoPath })` BEFORE the parallel list reads. Resume happens in the same `reload()` chain as the cold-boot `$effect` at `:425-428`, so the first panel open after app boot triggers the resume + reads in one pass. Test `describe("cold-boot resume")` × 5 covers all paths: resume fires exactly once on cold boot; skipped when active or none; newer_version rejection surfaces toast; arbitrary Error surfaces toast. The recursion-safety assertion uses `statusAfterResume + fireSessionChanged` to prove the listener-triggered second reload reads "active" and skips the resume — call count stays at 1. |
| SC-2 | ReviewPanel has a visible End-review affordance; clicking it (with confirmation) terminates the runtime session and removes the on-disk session file. After End-review, restarting the app shows no session for that repo | VERIFIED (automated portion) | `ReviewPanel.svelte:482-494` renders the End-review button gated on `{#if sessionState !== "none"}` as a sibling BEFORE the Copy button in the header flex row. `onEndClick` handler at `:381-408` implements the two-step confirm: first click arms 3000ms revert via `startEndConfirm`; second click clears the timer (but keeps `endConfirming = true` so the label stays frozen during await) and calls `safeInvoke("end_review_session", { path: repoPath })`. On success the session-changed listener round-trip drives `reload()` → `sessionState === "none"` → button hides via `{#if}` gate (D-08, no manual array clear). `describe("End review")` × 6 tests cover: first-click confirm flip, second-click IPC, 3000ms auto-revert, rapid-reclick re-arm, rejection toast, unmount timer cleanup. **Cross-window post-restart behavior is one of the 3 human smokes (real Tauri runtime required).** |
| SC-3 | Empty-state copy distinguishes "no session active" (cold) from "session active, no comments yet" (warm-empty) — they no longer look identical | VERIFIED | `ReviewPanel.svelte:538-559` replaces the prior two-arm empty-state block with three mutually-exclusive `{#if}` branches in specificity-first order: cold (`sessionState === "none"` → "No active review" / "Toggle review mode in the toolbar to start.") → warm-no-commits (`groups.length === 0` → existing "No commits in this review yet." preserved verbatim) → warm-with-commits (`!hasAnyComment` → "Review started." / "Select diff lines or add a commit note to comment." — replaces prior "No comments yet."). `describe("empty states")` × 3 tests assert each branch renders distinctly and mutual exclusivity holds. Also `describe("summary line")` × 2 tests assert the `{N} comments · {M} commits` (U+00B7) caption at `:527-531` is visible during a session and hidden in the cold branch. |
| SC-4 | Existing Copy flow still works; Copy and End are independently usable from the comments view | VERIFIED | Copy button at `ReviewPanel.svelte:495-509` is unchanged from Phase 72 (only the End button was added BEFORE it in the flex row). The existing `describe("Copy")` block (lines 689-873 in the test file) still has all original assertions passing — Copy handler `onCopyClick` at `:347-365` and CSS `.copy-button` at `:903-924` are untouched. End button has its own handler (`onEndClick`), state (`endConfirming`, `endTimer`), and CSS (`.end-button`), with zero overlap with Copy's state. The two buttons are independent siblings in the same header. All 44 tests in `ReviewPanel.test.ts` pass; no regressions in the pre-Phase-73 Copy describe. |
| SC-5 | `just check` exits 0 with all updated/new tests passing | VERIFIED | `just check` re-run at verification time — exit 0. Output: 547/547 vitest tests pass, 7/7 cargo tests pass, cargo fmt + clippy + svelte-check + biome all green. Three pre-existing biome warnings on `CommentComposer.svelte:43` (noNonNullAssertion) surface as warnings, not errors; do not block the gate. Tracked across all three plan SUMMARYs for a future quick-task. |

**Score:** 5/5 ROADMAP success criteria verified in code + tests.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/components/ReviewPanel.svelte` | Cold-boot resume + sessionState rune + End-button + 3-way empty states + summary caption | VERIFIED | All five changes wired at expected line ranges. `grep -n 'sessionState'` returns declaration at :141, assignment at :242, 4 usages in template (resume gate :255, End button visibility :482, summary visibility :527, empty-state gating :538). `grep -n 'resume_review_session'` returns exactly one safeInvoke call at :257. `grep -n 'end_review_session'` returns exactly one safeInvoke call at :396. `grep -n '"No active review"'` returns :540. `grep -n '"Review started\\."'` returns :554. `grep -n '"No commits in this review yet\\."'` returns :547 (preserved verbatim — regression guard). `grep -n 'comments · '` returns :529 with literal U+00B7 character. No new `#hex` / `rgb(` / `rgba(` literals in new CSS. |
| `src/components/ReviewPanel.test.ts` | `installReads` dispatcher extension + 3 new describe blocks | VERIFIED | `installReads` at :115-169 accepts `status` / `statusAfterResume` / `resumeRejection` / `endRejection`, dispatches `get_review_session_status` / `resume_review_session` / `end_review_session`, and uses a closure `resumed` flag to model backend `"resume-available" → "active"` transition. `sessionChangedHandler` capture + `fireSessionChanged` helper at :37-49. New describes at file scope: `cold-boot resume` (:875, 5 tests), `End review` (:1038, 6 tests, scoped fake timers), `empty states` (:1226, 3 tests), `summary line` (:1319, 2 tests), `multi-tab coordination` (:1382, 2 tests). 18 new tests across the phase; 44/44 in file pass. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `ReviewPanel.svelte:reload()` cold branch | `safeInvoke("resume_review_session", { path })` | `if (sessionState === "resume-available")` | WIRED | Line 255-258; `sessionState` is assigned from `status.state` at :242 inside the same try block, BEFORE the branch reads it, so a successful status fetch always gates correctly. |
| `ReviewPanel.svelte:reload()` resume catch arm | `showToast(\`Failed to resume review: \${errorMessage(e, "unknown error")}\`, "error")` | template-literal prefix + errorMessage extraction | WIRED | Lines 259-267. `errorMessage` (line 169) extracts `.message` from `Error` or `TrunkError`; prefix is added at call site so a TrunkError's actual message reaches the user (not the fallback string). Tests 4 & 5 in `cold-boot resume` cover both shapes. |
| `ReviewPanel.svelte:onEndClick` second-click | `safeInvoke("end_review_session", { path: repoPath })` | `if (!endConfirming) start; else IPC` | WIRED | Lines 381-408. First click branches to `startEndConfirm()` (return early). Second click clears the revert timer, KEEPS `endConfirming = true` (frozen-during-await), calls the IPC. Success: no manual array mutation, listener round-trip refreshes via `session-changed`. Failure: explicit `endConfirming = false` revert + template-literal toast (same shape as resume catch arm). |
| `ReviewPanel.svelte:onEndClick` catch arm | `showToast(\`Failed to end review: \${errorMessage(e, "unknown error")}\`, "error")` | template-literal prefix | WIRED | Lines 399-407; symmetric with resume catch arm. Test 5 in `End review` describe asserts the exact string `"Failed to end review: No active review session"` when a `TrunkError` is thrown. |
| `ReviewPanel.svelte` header flex row | End button rendered conditionally on `sessionState !== "none"` | `{#if sessionState !== "none"}` sibling to Copy button | WIRED | Lines 482-494; placement BEFORE Copy in the row (per Plan 02 decision: destructive reads left of affirmative). When `sessionState === "none"` the entire `<button>` markup is omitted (Pitfall 5: hide entirely, not disabled). |
| `ReviewPanel.svelte` summary caption | `<span>` above empty-state block, gated on `sessionState !== "none"` | `{#if sessionState !== "none"}` inside scroll-body | WIRED | Lines 527-531. Text: `{comments.length} comments · {commits.length} commits` with literal U+00B7 middle dot. Sits ABOVE the three-way empty-state block so users see the count first when the body has content. |
| `ReviewPanel.svelte` session-changed listener | `reload()` round-trip | `if (canonicalPath && event.payload !== canonicalPath) return;` filter | WIRED + UNTOUCHED | Lines 447-461. **Byte-for-byte unchanged across Plans 01/02/03 (D-09 invariant).** Verified: `git diff 272e01b..HEAD -- src/components/ReviewPanel.svelte | grep -E 'listen<string>...|if \(canonicalPath'` produces no output. The cross-repo filter is what makes the multi-tab coordination story safe. |
| `installReads` test dispatcher → captured `sessionChangedHandler` | `fireSessionChanged(payload)` helper drives listener-triggered reload | module-scoped capture in `vi.mock("@tauri-apps/api/event")` | WIRED | Test file lines 37-49. Used in cold-boot resume Test 1 (recursion safety: `statusAfterResume` flip after `fireSessionChanged`) and multi-tab Test 1 (post-end status flip) and Test 2 (cross-repo filter assertion). |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `ReviewPanel.svelte` (resume branch) | `sessionState` | `get_review_session_status` IPC at line 237-242 (existing Phase 65 backend, `src-tauri/src/commands/review.rs`) | Yes — backend returns `{state, file_exists, canonical_path}` from on-disk session file existence + in-memory `Mutex<HashMap<...>>` lookup | FLOWING |
| `ReviewPanel.svelte` (End button + summary caption + empty states) | `sessionState`, `commits`, `comments` | Same `reload()` chain — status drives gating; reads populate arrays | Yes — `list_session_commits` / `list_session_comments` / `resolve_session_comments` (Phase 65 backend) return real data; `no_session` TrunkError clears arrays + renders cold state | FLOWING |
| `ReviewPanel.svelte` (multi-tab refresh) | `canonicalPath` (filter) + reload chain | `session-changed` event payload (canonical path string) → existing listener at :447-461 → `reload()` | Yes — listener filter short-circuits cross-repo, fires `reload()` for matches; reload re-reads status + lists | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| File-scoped vitest suite | `pnpm vitest run src/components/ReviewPanel.test.ts` | 44 passed (44) in 7.74s | PASS |
| Full project check (`just check`) | `just check` | 547/547 vitest, 7/7 cargo, fmt/clippy/svelte-check/biome all green; exit 0 | PASS |
| Listener byte-for-byte unchanged (D-09) | `git diff 272e01b..HEAD -- src/components/ReviewPanel.svelte \| grep -E 'listen<string>...\|canonicalPath \&\&'` | no output | PASS |
| No debt markers in modified files | `grep -nE "TBD\|FIXME\|XXX\|HACK\|PLACEHOLDER" src/components/ReviewPanel.svelte src/components/ReviewPanel.test.ts` | no output | PASS |
| No new hex/rgb color literals | `grep -nE '#[0-9a-fA-F]{3,6}\|rgb\(\|rgba\(' src/components/ReviewPanel.svelte` (false positive on `each ... as` matches only) | no actual color literals; all colors via `var(--color-*)` tokens | PASS |
| Resume fires exactly once on cold boot (recursion safety) | `describe("cold-boot resume")` Test 1 | `resumeCallCount()` === 1 after initial reload + listener-triggered reload | PASS |
| End fires exactly once on second click | `describe("End review")` Test 2 | `endCallCount()` === 1; `callArgs("end_review_session")` === `{path: "/repo"}` | PASS |
| Multi-tab End in another tab clears panel | `describe("multi-tab coordination")` Test 1 | After `fireSessionChanged("/repo")` with swapped status: "No active review" visible, End button hidden, prior comment gone | PASS |
| Cross-repo session-changed is filtered | `describe("multi-tab coordination")` Test 2 | After `fireSessionChanged("/different-repo")`: `safeInvoke.mock.calls.length` unchanged; comment still rendered | PASS |

### Probe Execution

Phase 73 is a frontend-only UI phase with no `scripts/*/tests/probe-*.sh` declared in PLAN/SUMMARY/VALIDATION. Validation contract is vitest + `just check`. Probe execution: SKIPPED (no probes declared).

### Requirements Coverage

Phase 73 declares phase-local requirement IDs (REQ-73-*) in PLAN frontmatter. These IDs are **not** in `.planning/REQUIREMENTS.md` (which tracks the v0.13 milestone's SESS / SEL / ANCH / CMT / DOC / OUT requirements). The ROADMAP entry for Phase 73 explicitly states this:

> **Requirements**: Carry-forward bundle — closes Bug 3 from `.planning/phases/72-review-pane-ux-integration/72-VERIFICATION.md` and the End-review design ask captured in `.planning/todos/pending/phase-73-review-lifecycle.md`

So the REQ-73-* IDs are scoped to this phase only and trace back to (a) Bug 3 closure (REQ-73-RESUME) and (b) the End-review carry-forward (REQ-73-END + REQ-73-EMPTY + REQ-73-SUMMARY + REQ-73-MULTITAB), plus cross-cutting REQ-73-NYQUIST (test coverage rate) and REQ-73-CHECK (gate). All seven map to the five ROADMAP success criteria already verified above.

| Requirement ID | Source Plan | Description | Status | Evidence |
|----------------|-------------|-------------|--------|----------|
| REQ-73-RESUME | 73-01 | Cold-boot resume calls `resume_review_session` exactly once when status is `resume-available`; never when `active` or `none`; rejection surfaces a toast with extracted message | SATISFIED | `describe("cold-boot resume")` × 5 tests; closes Bug 3 from 72-VERIFICATION.md |
| REQ-73-END | 73-02 | Two-step End button with all six behaviors (first-click flip / second-click IPC / auto-revert / re-arm / failure toast / unmount cleanup) | SATISFIED | `describe("End review")` × 6 tests |
| REQ-73-EMPTY | 73-03 | Three distinct empty states branched on `sessionState` + `groups.length` + `hasAnyComment` | SATISFIED | `describe("empty states")` × 3 tests; copy matches UI-SPEC Copywriting Contract |
| REQ-73-SUMMARY | 73-03 | `{N} comments · {M} commits` caption (U+00B7) visible during a session, hidden in cold | SATISFIED | `describe("summary line")` × 2 tests; literal U+00B7 character in template at :529 |
| REQ-73-MULTITAB | 73-03 | Tab A's end_review_session emission drives tab B into the cold empty state; cross-repo payload filtered | SATISFIED | `describe("multi-tab coordination")` × 2 tests; listener byte-for-byte unchanged |
| REQ-73-NYQUIST | 73-01/02/03 | Each lifecycle path has automated test coverage at the Nyquist sampling rate from 73-VALIDATION.md | SATISFIED | 18 new tests total across the phase covering five lifecycle paths |
| REQ-73-CHECK | 73-01/02/03 | `just check` exits 0 | SATISFIED | Re-run at verification time: exit 0 |

**No orphaned requirements.** REQUIREMENTS.md does NOT list REQ-73-* by design (Phase 73 is a UX/lifecycle bundle, not a new milestone-level requirement). Cross-checked: `grep -E "REQ-73" .planning/REQUIREMENTS.md` returns no hits — expected per the ROADMAP carry-forward note.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/components/ReviewPanel.svelte` | — | (none in Phase 73 diff) | — | No new debt markers, no inline colors, no fallback branches against own contract, no `as any` casts, no PascalCase wire-format mismatches |
| `src/components/ReviewPanel.test.ts` | — | (none in Phase 73 diff) | — | All new tests follow the file's existing dispatcher pattern; fake-timer scoping correctly isolated per-describe (does NOT promote to file-global); helpers (`fireSessionChanged`, `renderWithSession`, `endCallCount`, `getEndButton`) reused appropriately |
| `src/components/diff/CommentComposer.svelte` | 43 | `lint/style/noNonNullAssertion` × 3 (`file!`, `hunkIdx!`, `selectedLineIndices!`) | INFO | Pre-existing; surfaced in 73-01/02/03 SUMMARYs under Pre-existing Issues. `just check` reports as warnings (not errors); does not block. Tracked for a future `/gsd:quick fix CommentComposer non-null assertions` task. Per ownership.md, surfaced but explicitly deferred — outside the `files_modified` scope of Phase 73's plans. |

### Human Verification Required

Three smokes from `73-VALIDATION.md` §"Manual-Only Verifications" require a real Tauri runtime and cannot be asserted in jsdom. Their automated counterparts (the 18 new tests) cover the IPC contract and DOM-rendering invariants; the manual smokes verify the end-to-end behavior across the actual app boot loop + cross-window event bus + visual styling.

#### 1. Cold-boot resume across actual app restart

**Test:** With a repo that already has an on-disk review session (e.g., one written by a previous app run via Phase 65's `mutate_session_rmw`), run `pnpm tauri dev`, open that repo, and open the ReviewPanel.
**Expected:** Existing comments appear immediately — no need to click the toolbar Start/Resume button, no need to add a new comment, no need to reload the panel. This is the closure proof for Bug 3 from 72-VERIFICATION.md.
**Why human:** Vitest mocks `safeInvoke`; only a real app boot exercises the persisted-state → first-open path end-to-end with the actual Rust `get_review_session_status` returning `"resume-available"`.

#### 2. Multi-tab End → cold state in other window

**Test:** Open two real Tauri windows on the same repo (Cmd+N or via multi-tab UI from v0.9). Both windows show an active review with comments. In window A: click **End review** once (label flips to "Click again to confirm" + danger color); click again within 3 seconds.
**Expected:** Window A's panel collapses to "No active review" (cold state). Window B's panel re-renders to "No active review" WITHOUT any user action in window B (driven by the real `session-changed` Tauri event bus). The End button disappears in both windows.
**Why human:** Requires two real Tauri windows talking through the actual `session-changed` event bus. The vitest version (`describe("multi-tab coordination")` Test 1) uses `fireSessionChanged("/repo")` to simulate the emit — sufficient for the listener contract but not for the cross-window IPC round-trip.

#### 3. Visual treatment of two-step End button danger color

**Test:** On a repo with an active review, hover the End button (idle muted color), then click it once.
**Expected:** Label flips to "Click again to confirm". Button background tints with `--color-danger-bg` (rgba(248,113,113,0.15)); border switches to `--color-danger-border`; text color uses `--color-on-accent` (white) for legibility on the danger tint. Wait ~3 seconds: button reverts to idle "End review" without any action. Hover while confirming: background heightens to `--color-danger` (#f87171) — a stronger destructive signal for the imminent commit.
**Why human:** CSS custom property application + Lucide `Trash2` icon contrast on the tinted background — an eye check, not assertable in jsdom (which has no rendering layer).

### Gaps Summary

**No automated gaps.** All five ROADMAP success criteria, all seven phase-local REQ-73-* IDs, all key links, and all artifacts pass verification. The phase ships per the plans.

**Three manual-smoke items remain** — they were explicitly listed in `73-VALIDATION.md` as manual-only (requiring real Tauri runtime / multi-window / visual eye-check) and were called out in all three plan SUMMARYs as "outstanding, non-blocking for plan-level merge gate." Per the verifier's decision tree, status is **human_needed**: automated checks all pass, but the goal contains end-to-end claims ("comments appear on first ReviewPanel open after app boot", "user has an explicit End-review affordance") whose final acceptance is the human-observable outcome.

Once a human runs the three smokes and confirms they pass, the phase is verified complete. If any smoke fails, the failure becomes a real gap (BLOCKER) and re-verification with `gaps:` is warranted.

### Pre-existing Issues Noted (ownership.md)

- `src/components/diff/CommentComposer.svelte:43` — three biome `noNonNullAssertion` warnings (pre-existing, surfaced in all three plan SUMMARYs). Does not block `just check` (warnings, not errors). Tracked for a future `/gsd:quick fix CommentComposer non-null assertions` task. Per ownership.md, this is surfaced rather than silently walked past, but is appropriately deferred — it's outside every Phase 73 plan's `files_modified` scope.

---

*Verified: 2026-05-27T18:00:00Z*
*Verifier: Claude (gsd-verifier, model claude-opus-4-7)*
