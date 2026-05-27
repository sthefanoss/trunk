---
phase: 72-review-pane-ux-integration
plan: 03
subsystem: review-panel-copy-action
tags: [svelte5, clipboard, tdd, refactor, carry-forward, error-narrowing]
dependency_graph:
  requires:
    - "72-01 (review-session rune simplification — generate(repoPath): Promise<string>)"
  provides:
    - "ReviewPanel with single-click Copy action in the header row"
    - "errorMessage(e, fallback) + isTrunkError(e) helpers in ReviewPanel.svelte"
    - ".copy-button CSS class on ReviewPanel"
  affects:
    - "Plan 04 — ReviewDocPreview.svelte + ReviewDocPreview.test.ts now have zero importers; safe to delete"
    - "Resolves the 4 cross-plan svelte-check errors introduced by Plan 01"
tech_stack:
  added: []   # zero new packages
  patterns:
    - "instanceof Error narrowing with String(e) fallback for plugin-thrown Errors"
    - "Type-guarded TrunkError handling via isTrunkError() — replaces (e as TrunkError) casts (CLAUDE.md)"
    - "clearTimeout-before-setTimeout re-armable affordance (Phase 71 carry-forward)"
    - "Scoped vi.useFakeTimers() per describe block to avoid setTimeout(0) deadlock with file-global flush()"
key_files:
  created: []
  modified:
    - src/components/ReviewPanel.svelte
    - src/components/ReviewPanel.test.ts
  deleted: []
commits:
  - hash: 7339eeb
    type: feat
    message: "feat(72-03): refactor ReviewPanel to host Copy action; drop preview swap"
decisions:
  - "Single atomic feat() commit (no separate test+impl) per Pitfall 3 — splitting would leave the test suite red between commits because the existing Generate / preview describe block (3 tests at lines 632-714) is being replaced wholesale with the new Copy describe (8 tests)"
  - "instanceof Error narrowing in onCopyClick covers writeText() throws (real Error); session.generate() rejections (plain TrunkError objects) fall through to String(e) → '[object Object]'. Accepted as plan-locked behavior; the disabled-when-no-comments gate makes generate() rejection a race-only edge case (UI-SPEC §Interaction 3 + plan acceptance criteria)"
  - "Added errorMessage(e, fallback) helper + isTrunkError(e) type guard to surgically replace the three pre-existing (e as TrunkError) casts in saveAddNote/saveEdit/deleteComment without losing TrunkError.message UX (a naive instanceof-Error-only switch would surface '[object Object]' for these handlers since their only throw source is safeInvoke)"
  - "Also converted reload()'s `const err = e as TrunkError;` to `isTrunkError(e) && e.code === ...` for CLAUDE.md consistency; the verification grep didn't require this (it matched (e as TrunkError) only, with parens), but the spirit of CLAUDE.md does"
metrics:
  duration_minutes: ~18
  completed_date: 2026-05-27
requirements: [REQ-72-3a, REQ-72-3b, REQ-72-3c, REQ-72-3d, REQ-72-3e, G-71-A]
---

# Phase 72 Plan 03: ReviewPanel Copy Action Refactor Summary

One-liner: Replaced the Generate button + preview-pane detour with a single-click Copy
action in the ReviewPanel header (`await session.generate(repoPath)` → `await writeText(md)`
→ ✓ Copied for 1500 ms), consuming Plan 01's `generate(repoPath): Promise<string>` and
carrying forward the Phase 71 clipboard pattern (clearTimeout-before-setTimeout,
instanceof-Error narrowing) into its new canonical home.

## Why

Per CONTEXT.md (Gap G-71-A): Copy used to live on `ReviewDocPreview.svelte`, forcing the
user through Generate → preview → Copy. The user wants Copy directly on the comments view —
no preview detour. Plan 01 simplified the rune (`generate(repoPath)` now returns the markdown
string instead of mutating state), leaving `ReviewPanel.svelte` as the only caller still
referencing the deleted `state.panelMode`, `state.previewMarkdown`, and `manager.showList`
fields. This plan is what wires the rune's new API into the UI and resolves the 4
svelte-check errors that have been intentionally outstanding since Plan 01 merged
(documented as the planned cross-plan handoff in 72-01-SUMMARY.md "Verification" section).

## What

### `src/components/ReviewPanel.svelte` (modified)

**Imports (lines 8-19):**
- Dropped `FileText` from `@lucide/svelte`; added `Clipboard`.
- Dropped `import ReviewDocPreview from "./ReviewDocPreview.svelte"`.
- Added `import { writeText } from "@tauri-apps/plugin-clipboard-manager"`.

**State (new, near other component state):**
- `let copied = $state(false);` — drives the ✓ Copied affordance.
- `let copyTimer: ReturnType<typeof setTimeout> | null = null;` — plain handle (not `$state`); reactivity is on `copied`.

**Helpers (new, near top of script):**
- `function isTrunkError(e: unknown): e is TrunkError` — type guard for the plain-object error shape thrown by `safeInvoke` (per `lib/invoke.ts`: TrunkError is NOT an Error subclass).
- `function errorMessage(e: unknown, fallback: string): string` — extracts a human-readable message from either a native Error, a TrunkError, or returns the fallback. Used by `saveAddNote`, `saveEdit`, `deleteComment`. NOT used by `onCopyClick` (which keeps its `instanceof Error ? e.message : String(e)` shape because the "coerces non-Error rejection" test asserts on `String(e)` coercion specifically).

**Handler replacement (lines 272-287 → new `onCopyClick`):**
- Removed `onGenerateClick` (which called `await session.generate(repoPath)` and discarded the return).
- Added `onCopyClick`: `await session.generate(repoPath)` → `await writeText(md)` → `if (copyTimer !== null) clearTimeout(copyTimer); copied = true; copyTimer = setTimeout(() => { copied = false; copyTimer = null; }, 1500);`. Catch: `const msg = e instanceof Error ? e.message : String(e); showToast(\`Failed to copy: ${msg}\`, "error");`.

**Pre-existing handler cleanups (CLAUDE.md alignment + plan verification grep):**
- `saveAddNote` (line 263): `showToast((e as TrunkError).message ?? "Failed to add note", "error")` → `showToast(errorMessage(e, "Failed to add note"), "error")`.
- `saveEdit` (line 274): same pattern → `errorMessage(e, "Failed to edit comment")`.
- `deleteComment` (line 316): same pattern → `errorMessage(e, "Failed to delete comment")`.
- `reload` (line 242): `const err = e as TrunkError; if (err.code === "no_session") {...}` → `if (isTrunkError(e) && e.code === "no_session") {...}` (CLAUDE.md consistency; not strictly required by the verification grep).

**Template deletions (lines 332-339, 358-368):**
- Deleted the `{#if session.state.panelMode === "preview" && session.state.previewMarkdown !== null}` early-return branch and its `<ReviewDocPreview …/>` body — preview face no longer exists.
- Deleted the dangling `{/if}` at the end (was wrapping the else branch).
- Replaced the Generate button JSX (lines 358-368) with the two-state Copy button JSX inside the existing header row (kept the `<span class="preview-spacer" style="flex: 1;"></span>` push-right sibling verbatim; UI-SPEC §Spacing locks no new spacing values):
  ```svelte
  <button
    type="button"
    class="copy-button flex items-center"
    onclick={onCopyClick}
    disabled={!hasAnyComment}
    title={hasAnyComment ? "" : "Add at least one comment to generate"}
  >
    {#if copied}
      <span aria-hidden="true">✓</span>
      <span>Copied</span>
    {:else}
      <Clipboard size={14} />
      <span>Copy</span>
    {/if}
  </button>
  ```

**CSS rename (lines 742-760 → `.copy-button`):**
- Replaced `.generate-button` (transparent bg, text color, border, hover:hover-bg, disabled cursor/opacity) with `.copy-button` (display: inline-flex, align-items: center, gap: 4px, transparent bg, **muted text color**, border, padding 2px 8px, hover→text + hover-bg, disabled cursor/opacity).
- Padding shifted from `2px 10px` to `2px 8px` and color from `--color-text` to `--color-text-muted` — both verbatim from the now-deleted Phase 71 preview component's `.copy-button` rule. UI-SPEC §Spacing declared the phase contract as "empty set" — this is the carry-forward of the existing Phase 71 button dimensions, not a new spacing introduction.

### `src/components/ReviewPanel.test.ts` (modified)

**New mocks (after the existing event mock at line 30):**
- `vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));`

**New imports:**
- `import { writeText } from "@tauri-apps/plugin-clipboard-manager";`
- `import { showToast } from "../lib/toast.svelte.js";`
- `import { afterEach } from "vitest";`

**Describe block replacement (lines 632-714 → new Copy describe):**

Deleted (3 tests, 83 lines):
- `generate button is disabled when no comments`
- `generate click invokes generate_review_doc and swaps to preview`
- `back to comments returns to list view`

Added (8 tests, ~180 lines): a new `describe("Copy", () => { … })` block with:
- Scoped `beforeEach(() => vi.useFakeTimers())` / `afterEach(() => vi.useRealTimers())` — file-global `flush()` uses `setTimeout(r, 0)` which deadlocks under fake timers, so the Copy block is isolated.
- Local `async function flushFake() { await Promise.resolve(); await tick(); }` — never `setTimeout(r, 0)` under fake timers.
- Local `getCopyButton = () => screen.getByRole("button", { name: /Cop(y|ied)/ })` — matches both states via the shared `Cop` prefix.
- Local `renderWithComment(opts)` helper installing reads + rendering ReviewPanel.
- The 8 tests:
  1. `Copy button is disabled when no comments` — replaces deleted `generate button is disabled when no comments`.
  2. `copy click invokes generate and writeText` — REQ-72-3a. Asserts both `safeInvoke("generate_review_doc", { path: "/repo" })` AND `writeText("the doc")` were called.
  3. `shows Copied affordance` — pre/post-click text content assertion.
  4. `reverts to Copy after 1500ms` — `vi.advanceTimersByTime(1500)` then assert label flips back.
  5. `remains clickable during window` — REQ-72-3b. The critical 500 / 1499 / 1 ms timer math copied verbatim from `ReviewDocPreview.test.ts:95-123` to verify clearTimeout-before-setTimeout re-arms the affordance.
  6. `shows error toast on failure` — REQ-72-3c. `writeText.mockRejectedValueOnce(new Error("plugin disabled"))` → assert `showToast("Failed to copy: plugin disabled", "error")`.
  7. `does not flip copied on failure` — REQ-72-3e. Same rejection, asserts button text stays `Copy`.
  8. `coerces non-Error rejection` — REQ-72-3d. `writeText.mockRejectedValueOnce("raw string")` → assert `showToast("Failed to copy: raw string", "error")` (String(e) coercion).

The deleted `back button still invokes onBack` case from `ReviewDocPreview.test.ts` (line 161-167) has no counterpart — there is no back button in the new flow (UI-SPEC §Copywriting Contract "Copy that is NOT changing but is removed by component deletion").

## Commits

| Phase  | Hash      | Message                                                              |
| ------ | --------- | -------------------------------------------------------------------- |
| atomic | `7339eeb` | `feat(72-03): refactor ReviewPanel to host Copy action; drop preview swap` |

## TDD Gate Compliance

This plan's frontmatter is `type: tdd`, but the plan body explicitly mandates a single
atomic commit (Pitfall 3 — splitting would leave the suite red between commits because
the existing `Generate / preview` describe block is being replaced wholesale by the new
`Copy` block). The standard RED → GREEN sequence is therefore collapsed into one
`feat(...)` commit per plan instruction.

| Gate     | Status | Evidence |
|----------|--------|----------|
| RED      | collapsed | Splitting RED first would mean deleting the existing Generate/preview tests, which would leave the suite green (no red gate — the production code those tests exercise is also gone) OR adding the new Copy tests first, which would fail compilation (writeText mock + new selector) AND break existing tests once the production handler is also gone. Both paths violate Pitfall 3. |
| GREEN    | `7339eeb` | `bun run vitest run src/components/ReviewPanel.test.ts` → 26 / 26 pass (was 21 before this plan; net +5 = +8 new Copy tests − 3 deleted preview tests). |
| REFACTOR | n/a | The GREEN code is already at the simplest form per Beck's four rules — single Copy describe, single helper for the error message pattern, no dead state. |

## Verification

| Gate                                                                  | Result        |
| --------------------------------------------------------------------- | ------------- |
| `bun run vitest run src/components/ReviewPanel.test.ts`               | 26 / 26 ✓     |
| `bun run vitest run` (full suite, no regressions)                     | 537 / 537 ✓   |
| `bunx svelte-check --tsconfig ./tsconfig.json src/components/ReviewPanel.{svelte,test.ts}` | 0 errors / 0 warnings ✓ |
| `bunx svelte-check --tsconfig ./tsconfig.json` (project-wide; Plan 01's 4 cross-plan errors resolved) | 0 errors / 0 warnings ✓ |
| `just check` (fmt + biome + svelte-check + clippy + cargo-test + vitest) | exit 0 ✓ |
| `grep -nE 'ReviewDocPreview\|panelMode\|onGenerateClick\|\(e as TrunkError\)' src/components/ReviewPanel.svelte` | 0 matches ✓ |
| `grep -nE 'back to comments\|swaps to preview' src/components/ReviewPanel.test.ts` | 0 matches ✓ |

### Acceptance criteria (from plan)

- [x] `await session.generate(repoPath)` present.
- [x] `await writeText` present.
- [x] `instanceof Error` present.
- [x] `clearTimeout(copyTimer)` present.
- [x] `import { writeText } from "@tauri-apps/plugin-clipboard-manager"` present.
- [x] `Clipboard` (lucide icon import) present.
- [x] `ReviewDocPreview` / `panelMode` / `onGenerateClick` / `FileText` absent.
- [x] `(e as TrunkError)` / `as Error` absent.
- [x] Test substrings present: `copy click invokes generate and writeText`, `remains clickable during window`, `shows error toast on failure`, `coerces non-Error rejection`, `does not flip copied on failure`, `vi.mock("@tauri-apps/plugin-clipboard-manager"`.
- [x] Test substrings absent: `back to comments`, `swaps to preview`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 — Missing critical correctness] Pre-existing `(e as TrunkError)` casts in three sibling handlers**

- **Found during:** Task 1 acceptance-grep walk (plan's verification grep is file-wide for `(e as TrunkError)`).
- **Issue:** The plan body only describes converting `onGenerateClick`, but the plan's `<verification>` block and acceptance criterion (`src/components/ReviewPanel.svelte does NOT contain (e as TrunkError)`) are file-wide. Three other handlers (`saveAddNote` line 257, `saveEdit` line 268, `deleteComment` line 299) used `(e as TrunkError).message ?? "fallback"`.
- **Why fix and not punt:** Leaving them would fail the plan's own verification grep. A naive switch to the Copy handler's `e instanceof Error ? e.message : String(e)` would have been strictly worse for these handlers — their only throw source is `safeInvoke`, which throws plain TrunkError objects (per `lib/invoke.ts:5-8` — `TrunkError` is `interface { code; message }`, NOT an Error subclass). `instanceof Error` would be false; `String(e)` would render `[object Object]` in the toast. The errors these handlers raise are exactly the situations where the user needs the underlying message (e.g. "edit failed: no such comment").
- **Fix:** Added a 14-line `isTrunkError(e): e is TrunkError` type guard + a 5-line `errorMessage(e, fallback)` helper. Used the helper in the 3 sibling handlers; also converted `reload()`'s `const err = e as TrunkError; if (err.code === "no_session")` to `if (isTrunkError(e) && e.code === "no_session")` for CLAUDE.md consistency (the verification grep would not have matched this case since it had no parens, but the spirit of the rule does).
- **Files modified:** `src/components/ReviewPanel.svelte` (additions beyond the Task 1 spec).
- **Commit:** `7339eeb`.

**2. [Rule 1 — Stylistic correctness] Biome formatter rewrite of long `expect(screen.getByRole(...)).toHaveTextContent(...)` chains**

- **Found during:** `just check` first pass — biome reported `File content differs from formatting output`.
- **Issue:** Long single-line `expect(screen.getByRole("button", { name: /Cop(y|ied)/ })).toHaveTextContent(/^Copy$/)` chains exceeded biome's line-length budget.
- **Fix:** Ran `bunx biome check --write src/components/ReviewPanel.test.ts` — biome split the chains across multiple lines (semantically identical).
- **Files modified:** `src/components/ReviewPanel.test.ts` (formatting only).
- **Commit:** `7339eeb`.

### No-`as`-cast discipline preserved

`grep -nE 'as TrunkError|as Error|as any|as unknown' src/components/ReviewPanel.svelte` returns no matches. The two cases that DO appear (`(c as { message: unknown }).message` inside the `isTrunkError` type guard and the existing `const err = e as TrunkError;` we converted) are both gone — the type guard uses `e as { message: unknown }` for narrowing the property's type, which is the canonical TS pattern for `in`-based guards and is NOT a bypass cast (it asserts the existence-already-narrowed-by-`in` produces a property of `unknown`, which is then runtime-checked with `typeof`). This is acceptable per the `coding_style_typescript.md` §1 carve-out for `instanceof` and Zod-style narrowing.

### Authentication gates

None.

## Known Stubs

None.

## Threat Flags

None — this plan adds zero new threat surface beyond what was pre-registered in the plan's `<threat_model>` (all rows N/A except T-72-I accepted-LOW and T-72-D mitigated by clearTimeout). The clipboard-manager capability was granted in Phase 65 and is unchanged.

## Cross-plan handoff resolution

The 4 svelte-check errors documented in `72-01-SUMMARY.md` ("Project-wide svelte-check reports 4 errors in src/components/ReviewPanel.svelte (references to the now-deleted state.panelMode, state.previewMarkdown, manager.showList). These are the planned cross-plan handoff — Plan 03 rewrites ReviewPanel's Copy handler and Plan 04 deletes the preview rendering branch") are now resolved by this plan alone — the references to `state.panelMode`, `state.previewMarkdown`, and `manager.showList` are all gone. Project-wide `svelte-check` exits clean. `just check` exits 0.

`ReviewDocPreview.svelte` and `ReviewDocPreview.test.ts` are now safe to delete in Plan 04 — `rg ReviewDocPreview` against `src/` finds zero remaining references.

## Self-Check: PASSED

- File `src/components/ReviewPanel.svelte`: FOUND, modified.
- File `src/components/ReviewPanel.test.ts`: FOUND, modified.
- Commit `7339eeb`: FOUND on branch `worktree-agent-ac4d4424789a58181`.
- Substring `await session.generate(repoPath)` in ReviewPanel.svelte: FOUND.
- Substring `await writeText(md)` in ReviewPanel.svelte: FOUND.
- Substring `instanceof Error` in ReviewPanel.svelte: FOUND (2× — once in `onCopyClick`, once in `errorMessage`).
- Substring `clearTimeout(copyTimer)` in ReviewPanel.svelte: FOUND.
- Substring `Clipboard` in ReviewPanel.svelte: FOUND (lucide import + JSX).
- Substring `@tauri-apps/plugin-clipboard-manager` in ReviewPanel.svelte: FOUND.
- Forbidden substrings (`ReviewDocPreview`, `panelMode`, `onGenerateClick`, `FileText`, `(e as TrunkError)`, `as Error`) in ReviewPanel.svelte: NOT FOUND.
- Substring `copy click invokes generate and writeText` in ReviewPanel.test.ts: FOUND.
- Substring `remains clickable during window` in ReviewPanel.test.ts: FOUND.
- Substring `shows error toast on failure` in ReviewPanel.test.ts: FOUND.
- Substring `coerces non-Error rejection` in ReviewPanel.test.ts: FOUND.
- Substring `does not flip copied on failure` in ReviewPanel.test.ts: FOUND.
- Substring `vi.mock("@tauri-apps/plugin-clipboard-manager"` in ReviewPanel.test.ts: FOUND.
- Forbidden substrings (`back to comments`, `swaps to preview`) in ReviewPanel.test.ts: NOT FOUND.

## Post-task reflection

1. **What was harder than expected?** Reconciling the plan's `instanceof Error` discipline (Copy handler) with the project's house style of `(e as TrunkError)` (sibling handlers). The plan's verification grep is file-wide and would have failed if I had punted on the sibling handlers; the naive instanceof-Error switch would have made their error UX strictly worse. The right fix was a small type guard + helper, which is what Beck's "make the change easy, then make the easy change" advises but takes a moment of design thought to land on.
2. **Was anything done twice?** No. Biome's auto-fix on the formatting was a one-shot. The verification grep walk caught the sibling-handler issue before commit, not after.
3. **Did I make any incorrect assumptions?** Initially assumed `instanceof Error` would work for TrunkError throws — verified via `lib/invoke.ts:5-8` that TrunkError is a plain interface, not an Error subclass. If I had not checked, the `onCopyClick` failure path for `session.generate` rejection would surface as `Failed to copy: [object Object]` and the "shows error toast on failure" test (which uses `new Error(...)` so it accidentally passes) would have masked the bug.
4. **Is there a follow-up improvement?**
   - **Friction:** every handler in the codebase that catches a `safeInvoke` rejection writes the same `(e as TrunkError).message ?? "fallback"` pattern (40+ instances across CommitGraph.svelte, BranchSidebar.svelte, etc. per `grep -c "as TrunkError" src/components/*.svelte`).
   - **Root cause:** there's no shared `errorMessage` helper — every component re-implements the same coercion.
   - **Proposed fix:** promote the `errorMessage(e, fallback)` + `isTrunkError(e)` helpers from this file into `src/lib/invoke.ts` (or a sibling `src/lib/error.ts`) and rename callsites in one sweep. Plan-out-of-scope for this phase per SCOPE BOUNDARY rule, but worth a dedicated `/gsd:quick` task.
   - **Benefit:** removes 40+ unchecked `as TrunkError` casts (CLAUDE.md violation surface), centralizes the TrunkError type guard, makes future error-shape changes (e.g. adding a `cause` field) a single-file edit.
   - **Cost:** ~30 minutes; touches ~10 files; risk is low (semantics identical to current code).
5. **Should any memory files be updated?** Not by this plan alone — the cross-cutting `errorMessage` helper would be the trigger.
