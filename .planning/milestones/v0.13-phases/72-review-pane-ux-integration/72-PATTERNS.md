# Phase 72: Review-Pane UX Integration — Pattern Map

**Mapped:** 2026-05-27
**Files analyzed:** 5 modified, 2 deleted, 0 new
**Analogs found:** 7 / 7 (every change has a concrete in-repo template)

## File Classification

| Modified/Deleted File | Role | Data Flow | Closest Analog | Match Quality |
|-----------------------|------|-----------|----------------|---------------|
| `src/components/Toolbar.svelte` | Svelte component (chrome) | event-driven (emit `review-toggle`) | sibling buttons in same file (Branch/Stash/Pop) + `App.svelte:557` listener pair | exact (intra-file clone) |
| `src/components/Toolbar.test.ts` | vitest component test | render + assertion | existing Toolbar.test.ts blocks (lines 40-127) | exact (sibling tests in same file) |
| `src/components/ReviewPanel.svelte` | Svelte component (panel chrome) | request-response (Copy click → IPC → clipboard) | `ReviewDocPreview.svelte` Copy handler (lines 28-44) + Copy button JSX (lines 55-63) | exact (line-for-line carry-forward, source being deleted) |
| `src/components/ReviewPanel.test.ts` | vitest component test | event simulation + IPC mock + fake timers | `ReviewDocPreview.test.ts` (all 8 `it` blocks, lines 38-168) | exact (carry-forward, source being deleted) |
| `src/components/RepoView.svelte` | Svelte component (layout) | rendering only | already-wired DiffPanel close at `RepoView.svelte:839` | exact (the surviving back-affordance, no change) |
| `src/lib/review-session.svelte.ts` | Svelte rune (state-machine) | request-response (single async return) | the existing `generate` method (lines 93-103) minus its state mutations | role-match (collapsing same method) |
| `src/lib/review-session.svelte.test.ts` | vitest unit test | rune state assertions | existing `it("generate awaits safeInvoke…")` at lines 41-50 | role-match (one test survives shape-shifted, six get deleted) |
| `src-tauri/src/lib.rs` | Rust Tauri menu config | menu accelerator binding | `find` MenuItemBuilder + `.accelerator("CmdOrCtrl+F")` chain at lines 21-23 | exact (one-line mirror) |

---

## Pattern Assignments

### `src/components/Toolbar.svelte` (Svelte component, event-driven)

**Analog:** itself — the new `.toolbar-group` clones an existing one. The `emit` half has no in-repo precedent (Toolbar will be the first frontend emitter, per RESEARCH Finding 1); the canonical-shape sample comes from the Rust side at `src-tauri/src/lib.rs:67-69` which already emits the same string from the menu handler.

**Imports pattern** (current file, lines 1-18) — pattern to extend with `emit` + new icon:
```svelte
<script lang="ts">
import {
	Archive,
	ArrowDown,
	ArrowUp,
	GitBranch,
	PackageOpen,
	Redo2,
	Undo2,
} from "@lucide/svelte";
import { listen } from "@tauri-apps/api/event";
// ... etc
```
After the edit, the lucide block adds `MessagesSquare` (verified at `node_modules/@lucide/svelte/dist/icons/index.js:1042`, see UI-SPEC §Iconography), and the event import becomes `import { emit, listen } from "@tauri-apps/api/event";` (verified export at `node_modules/@tauri-apps/api/event.d.ts:145`).

**Props interface pattern** (lines 20-26) — extend with `reviewActive`:
```ts
interface Props {
	repoPath: string;
	remoteState: RemoteState;
	undoRedo: UndoRedoManager;
}

let { repoPath, remoteState, undoRedo }: Props = $props();
```
After the edit (per RESEARCH Pattern 1, Pitfall 4): add `reviewActive: boolean;` to Props and to the destructure. App.svelte:584 must pass the new prop simultaneously (see App.svelte pattern below).

**Toolbar-button clone template** (the Branch/Stash/Pop group at lines 264-274, the rightmost existing group — the new Review group is appended *after* this):
```svelte
<div class="toolbar-group">
  <button class="toolbar-btn" onclick={handleBranch}>
    <GitBranch size={14} /> Branch
  </button>
  <button class="toolbar-btn" onclick={handleStash}>
    <Archive size={14} /> Stash
  </button>
  <button class="toolbar-btn" onclick={handlePop}>
    <PackageOpen size={14} /> Pop
  </button>
</div>
```
The new group is a single-button group of the same shape. UI-SPEC §Interaction Contract Layout placement: "new `.toolbar-group` appended as the rightmost group… after the existing Branch / Stash / Pop group at `Toolbar.svelte:264-274`."

**Existing `.toolbar-btn` CSS** (lines 209-230, hands-off — clone, do NOT modify):
```css
.toolbar-btn {
  background: none;
  border: none;
  border-radius: 4px;
  color: var(--color-text);
  font-size: 12px;
  padding: 4px 10px;
  cursor: pointer;
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 4px;
  height: 26px;
}
.toolbar-btn:hover:not(:disabled) {
  background: var(--color-border);
}
.toolbar-btn:disabled {
  opacity: 0.5;
  cursor: default;
  pointer-events: none;
}
```
The new button uses `class="toolbar-btn"` verbatim. UI-SPEC §Spacing: phase introduces zero new spacing values.

**Active-state class addition** (NEW — no existing precedent in `Toolbar.svelte`; UI-SPEC §Color is the contract):
```css
.toolbar-btn.toolbar-btn-active {
  background: var(--color-accent);
  color: var(--color-on-accent);
}
.toolbar-btn.toolbar-btn-active:hover {
  background: var(--color-accent); /* unchanged on hover when already filled */
}
```
Apply via `class:toolbar-btn-active={reviewActive}`. Do NOT inline `style="background: …"` ternaries — that is the deleted blue-button anti-pattern (`RepoView.svelte:815-827`), explicitly forbidden by RESEARCH §Don't Hand-Roll and UI-SPEC §Color "Wiring discipline."

**New Review button JSX** (template — synthesizing the clone + active class + emit; verified Lucide export, verified emit export):
```svelte
<script>
function handleReviewToggle() {
  void emit("review-toggle");
}
</script>

<div class="toolbar-group">
  <button
    class="toolbar-btn"
    class:toolbar-btn-active={reviewActive}
    aria-pressed={reviewActive}
    onclick={handleReviewToggle}
  >
    <MessagesSquare size={14} /> Review
  </button>
</div>
```
- `void emit(...)` — matches the fire-and-forget house style at `CommitGraph.svelte:757` per RESEARCH Pattern 1 + Assumption A5; emit on the in-process event bus never user-fails so no try/catch.
- `aria-pressed` per UI-SPEC §Accessibility Contract and RESEARCH Pitfall 5.

---

### `src/components/Toolbar.test.ts` (vitest, render + assertion)

**Analog:** existing `Toolbar.test.ts` blocks (lines 40-127) — same render shape; new tests add `reviewActive` to props and a new `emit` mock.

**Module mocks pattern** (lines 5-21) — extend with `emit`:
```ts
// Shared Tauri mock
import "../__tests__/helpers/tauri-mock";

// Explicitly mock @tauri-apps/api/event to prevent real listen calls
vi.mock("@tauri-apps/api/event", () => ({
	listen: vi.fn().mockResolvedValue(() => {}),
}));

// Mock invoke module — safeInvoke for check_undo_available etc.
vi.mock("../lib/invoke.js", () => ({
	safeInvoke: vi.fn().mockResolvedValue(false),
}));

// Mock toast module
vi.mock("../lib/toast.svelte.js", () => ({
	showToast: vi.fn(),
}));
```
After the edit (per RESEARCH §Open Questions #2 — exact recommended shape):
```ts
vi.mock("@tauri-apps/api/event", () => ({
	listen: vi.fn().mockResolvedValue(() => {}),
	emit: vi.fn().mockResolvedValue(undefined),
}));
```

**Render-with-props pattern** (lines 41-50) — extend every existing call site + new test:
```ts
it("renders Pull button", () => {
	render(Toolbar, {
		props: {
			repoPath: "/test/repo",
			remoteState: makeRemoteState(),
			undoRedo: makeUndoRedo(),
		},
	});
	expect(screen.getByText("Pull")).toBeInTheDocument();
});
```
Every existing `render` call now requires a `reviewActive: false` field on the props object (compile-time enforcement once the Props interface adds the field — TypeScript will reject the missing prop).

**New tests to add** (REQ-72-1a + REQ-72-2 per RESEARCH §Validation Architecture). Pattern:
```ts
it("emits review-toggle on click", async () => {
	const { emit } = await import("@tauri-apps/api/event");
	render(Toolbar, {
		props: {
			repoPath: "/test/repo",
			remoteState: makeRemoteState(),
			undoRedo: makeUndoRedo(),
			reviewActive: false,
		},
	});
	await fireEvent.click(screen.getByText("Review").closest("button")!);
	expect(vi.mocked(emit)).toHaveBeenCalledWith("review-toggle");
});

it("shows active state when reviewActive is true", () => {
	render(Toolbar, {
		props: {
			repoPath: "/test/repo",
			remoteState: makeRemoteState(),
			undoRedo: makeUndoRedo(),
			reviewActive: true,
		},
	});
	const btn = screen.getByText("Review").closest("button")!;
	expect(btn).toHaveClass("toolbar-btn-active");
	expect(btn).toHaveAttribute("aria-pressed", "true");
});
```
- `fireEvent` import already present in repo via `@testing-library/svelte`; check imports.

---

### `src/components/ReviewPanel.svelte` (Svelte component, request-response)

**Analog (handler + state):** `ReviewDocPreview.svelte:28-44` — line-for-line carry-forward; `ReviewDocPreview.svelte` is being DELETED in the same phase so this is the only place this pattern will live afterward.

**Imports edit** (current ReviewPanel.svelte lines 8-20) — remove `FileText` (Generate button icon, no longer used), `ReviewDocPreview` (component deleted), `MessageSquarePlus` keeps; ADD `Clipboard` + `writeText`:
```ts
// CURRENT (excerpt)
import { FileText, MessageSquarePlus } from "@lucide/svelte";
// …
import ReviewDocPreview from "./ReviewDocPreview.svelte";
```
After edit (mirroring `ReviewDocPreview.svelte:13-14`):
```ts
import { Clipboard, MessageSquarePlus } from "@lucide/svelte";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
// (drop the ReviewDocPreview import — file is being deleted)
```
- `Clipboard` verified at `node_modules/@lucide/svelte/dist/icons/index.js:453`.
- `writeText` capability `clipboard-manager:allow-write-text` granted in Phase 65 (CONTEXT.md depends_on); no new permission.

**Copy handler — line-for-line carry-forward** from `ReviewDocPreview.svelte:24-44` (the source file is being deleted, so this is the canonical home from now on). Adjusted only to call `session.generate` first instead of using a `markdown` prop:
```ts
// Carry-forward of the Copy state from ReviewDocPreview.svelte:24-26
let copied = $state(false);
let copyTimer: ReturnType<typeof setTimeout> | null = null;

// REPLACES onGenerateClick at ReviewPanel.svelte:278-287.
// Composes ReviewDocPreview.svelte:28-44's clipboard pattern with the rune's
// (simplified) generate that now RETURNS the markdown.
async function onCopyClick() {
	try {
		const md = await session.generate(repoPath);
		await writeText(md);
		// Pitfall 2 carry-forward: clear in-flight revert before rescheduling.
		if (copyTimer !== null) clearTimeout(copyTimer);
		copied = true;
		copyTimer = setTimeout(() => {
			copied = false;
			copyTimer = null;
		}, 1500);
	} catch (e) {
		// `unknown` in TS strict; narrow rather than cast. CLAUDE.md + carry-forward.
		const msg = e instanceof Error ? e.message : String(e);
		showToast(`Failed to copy: ${msg}`, "error");
	}
}
```
Note the *intentional* divergence from the current `onGenerateClick` (lines 278-287), which uses `(e as TrunkError).message` — that cast is being upgraded to `instanceof Error` narrowing because the post-72 handler can throw from either `generate` (TrunkError) or `writeText` (plugin Error). UI-SPEC §Interaction 3 + CLAUDE.md "Never `as any`, never `as Error`" both lock this in.

**Code being deleted from ReviewPanel.svelte** — surgical removal list:
- Lines 272-287 (`onGenerateClick` + its docstring) → replaced by `onCopyClick` above.
- Line 20 import of `ReviewDocPreview`.
- Lines 332-339 (the `{#if session.state.panelMode === "preview" …}` early-return branch and its accompanying `<ReviewDocPreview … />`) → deleted entirely; the panel renders the list view unconditionally now.
- Lines 359-368 (Generate button JSX) → replaced by Copy button JSX below.

**Generate button being replaced** (current lines 358-368):
```svelte
<span class="preview-spacer" style="flex: 1;"></span>
<button
  type="button"
  class="generate-button flex items-center"
  onclick={onGenerateClick}
  disabled={!hasAnyComment}
  title={hasAnyComment ? "" : "Add at least one comment to generate"}
>
  <FileText size={14} />
  <span>Generate</span>
</button>
```

**Copy button JSX** (carry-forward template, fusing `ReviewDocPreview.svelte:55-63` two-state body with the existing disabled-when-no-comments gating from the Generate button above):
```svelte
<span class="preview-spacer" style="flex: 1;"></span>
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
- Keep the `<span class="preview-spacer" style="flex: 1;"></span>` push-right sibling unchanged (lives at line 358; UI-SPEC §Layout placement: "stays in the same docked position the Generate button occupies today").
- `disabled` + `title` semantics preserved verbatim from the current Generate button (UI-SPEC §Copywriting Contract: "inherited verbatim from the current Generate button (`ReviewPanel.svelte:364`). The user's intent — 'generate then copy' — is collapsed into one button but the gating reason is unchanged.").
- The `.copy-button` CSS class needs to be added to ReviewPanel.svelte's `<style>` block — copy verbatim from `ReviewDocPreview.svelte:118-135`:
```css
.copy-button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: transparent;
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  cursor: pointer;
  padding: 2px 8px;
  font-size: 12px;
  font-family: inherit;
}
.copy-button:hover,
.copy-button:focus-visible {
  color: var(--color-text);
  background: var(--color-hover);
}
```
The current ReviewPanel.svelte has a `.generate-button` selector somewhere in its `<style>` block that follows the same pattern — replace it with `.copy-button` (or rename the class). Either way, UI-SPEC §Spacing inherited values forbid introducing *new* spacing numbers; the carry-forward from ReviewDocPreview is line-identical.

---

### `src/components/ReviewPanel.test.ts` (vitest, event simulation + IPC mock + fake timers)

**Analog:** `ReviewDocPreview.test.ts` (lines 1-168) — entire file's pattern moves into a new `describe("Copy")` block inside `ReviewPanel.test.ts`. RESEARCH Assumption A4: 1:1 mapping of the 8 cases.

**Coordination constraint:** RESEARCH Pitfall 3 — the existing `Generate / preview` `describe` block at `ReviewPanel.test.ts:632-715` (three `it` blocks asserting Generate-button label, `panelMode` swap, `back to comments` navigation) MUST be deleted in the SAME edit that adds the new Copy block. Do not split into two commits or the suite goes red between them.

**Clipboard mock — verbatim carry-forward** from `ReviewDocPreview.test.ts:21-23`:
```ts
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
	writeText: vi.fn().mockResolvedValue(undefined),
}));
```
This must be added to ReviewPanel.test.ts's top-of-file mocks block (after the existing `vi.mock("../lib/toast.svelte.js", …)` at line 24).

**Fake-timers lifecycle** — verbatim carry-forward from `ReviewDocPreview.test.ts:29-36`:
```ts
beforeEach(() => {
	vi.clearAllMocks();
	vi.useFakeTimers();
});
afterEach(() => {
	vi.useRealTimers();
});
```
INTERACTION HAZARD: the existing `ReviewPanel.test.ts` `flush()` helper at lines 115-118 uses `await new Promise((r) => setTimeout(r, 0))` — which **deadlocks under `vi.useFakeTimers()`** (RESEARCH §Code Examples warning). The new Copy `describe` block must use a *separate* flush helper:
```ts
// Microtask flush helper — safe under fake timers (no setTimeout(0)).
async function flushFake() {
	await Promise.resolve();
	await tick();
}
```
matching `ReviewDocPreview.test.ts:43-47`. Scope `vi.useFakeTimers()` to inside the Copy `describe` block, not file-global, so the rest of the suite keeps using the real-timer `flush` at line 115.

**Two-state button selector** — verbatim carry-forward from `ReviewDocPreview.test.ts:52-54`:
```ts
function getCopyButton() {
	return screen.getByRole("button", { name: /Cop(y|ied)/ });
}
```
Crucial: `Copy` and `Copied` share only the `Cop` prefix (no `y` in "Copied"), so a substring `/Copy/` would NOT match the success state. Use `/Cop(y|ied)/`.

**Test-case mapping (8 → 8)** — copy each `it` block from `ReviewDocPreview.test.ts` and adjust per the table; setup uses the existing `installReads({ … generateDoc: "the doc" })` dispatcher already present at `ReviewPanel.test.ts:92-113` (RESEARCH §Wave 0 Gaps: "the existing `installReads` dispatcher in `ReviewPanel.test.ts:92-113` already handles `generate_review_doc`"):

| Source line (ReviewDocPreview.test.ts) | Test name (REQ ID) | Adjustment |
|----------------------------------------|--------------------|------------|
| 60-66 `writes markdown prop` | "copy click invokes generate and writeText" (REQ-72-3a) | Drop the `markdown` prop; `installReads({ …, generateDoc: "the doc" })`; assert `writeText("the doc")` AND `safeInvoke("generate_review_doc", …)` |
| 68-79 `shows Copied affordance on success` | "shows Copied affordance" | Use ReviewPanel render shape from existing tests (lines 633-642) |
| 81-93 `reverts after timeout` | "reverts to Copy after 1500ms" | Same shape |
| 95-123 `remains clickable during window` | "remains clickable during window" (REQ-72-3b) | Same; the 500/1499/1 timer math is the critical part — copy verbatim |
| 125-134 `shows error toast on failure` | "shows error toast on failure" (REQ-72-3c) | `vi.mocked(safeInvoke).mockImplementation` to reject `generate_review_doc` OR mock `writeText` to reject — pick writeText to match source verbatim; either branch lands in the same catch |
| 136-148 `does not flip copied on failure` | "does not flip copied on failure" (REQ-72-3e) | Same |
| 150-159 `coerces non-Error rejection` | "coerces non-Error rejection" (REQ-72-3d) | Same |
| 161-167 `back button still invokes onBack` | DELETE (no back button — preview pane gone) | — |

That's 7 new tests inside a new `describe("Copy")` block, replacing the 3 tests in the deleted `Generate / preview` describe.

**Render-with-props pattern** for the new Copy tests — copy from the existing `ReviewPanel.test.ts:633-642`:
```ts
installReads({
	commits,
	comments: [lineAnchoredComment("c1", COMMIT_A, "look here")],
	resolutions: [resolvable("c1")],
	generateDoc: "the doc",
});
render(ReviewPanel, {
	props: {
		repoPath: "/repo",
		session: createReviewSession(),
		onJump: vi.fn(),
		onJumpToCommit: vi.fn(),
	},
});
await flushFake();
```

**Tests being deleted** (lines 632-715, the `Generate / preview` describe block): all three `it` blocks — `generate button is disabled when no comments`, `generate click invokes generate_review_doc and swaps to preview`, `back to comments returns to list view`. The "disabled when no comments" assertion is replaced by an equivalent "disabled when no comments" assertion inside the new Copy describe (button selector becomes `getCopyButton()` matching `/Cop(y|ied)/`; the rest of the assertion is identical).

---

### `src/components/RepoView.svelte` (Svelte component, layout)

**Analog:** the file's own surviving code at line 839 — `onclose={() => { handleDiffClose(); reviewSession.showPanel(); }}` — confirms the DiffPanel close path that survives the deletion is already wired correctly. RESEARCH §Architectural Responsibility Map and CONTEXT.md success criterion 5 confirm: "Returning from a jumped-to diff back to the panel is handled by `DiffPanel`'s existing close affordance (already wired to `reviewSession.showPanel()` at `RepoView.svelte:839`)."

**Code being deleted** — header strip at lines 813-828 (the blue button + its hosting div), EXACT block per CONTEXT.md "delete lines 813-828":
```svelte
<div class="flex flex-col" style="height: 100%; min-height: 0;">
  <div class="flex items-center" style="gap: 8px; padding: 4px 8px; border-bottom: 1px solid var(--color-border); flex-shrink: 0; background: var(--color-surface);">
    <button
      type="button"
      onclick={() => reviewSession.showPanel()}
      style="
        background: {reviewSession.state.rightPaneMode === 'panel' ? 'var(--color-accent)' : 'transparent'};
        color: {reviewSession.state.rightPaneMode === 'panel' ? 'var(--color-bg)' : 'var(--color-text-muted)'};
        border: 1px solid {reviewSession.state.rightPaneMode === 'panel' ? 'var(--color-accent)' : 'var(--color-border)'};
        border-radius: 4px;
        cursor: pointer;
        padding: 2px 10px;
        font-size: 12px;
      "
    >Review</button>
  </div>
```
**This is the inline-style ternary anti-pattern called out in UI-SPEC §Color "Wiring discipline" and RESEARCH §Don't Hand-Roll** — when removing it, the executor MUST NOT recreate the same shape on the new Toolbar button.

**Code being kept** — the inner conditional + DiffPanel/ReviewPanel render at lines 829-845 stays, but loses its wrapping `<div class="flex flex-col" style="height: 100%; min-height: 0;">` (the outer wrapper at line 813). The simplified shape:
```svelte
{#if reviewSession.state.reviewActive}
  <div class="flex flex-col" style="flex: 1; min-height: 0; overflow: hidden;">
    {#if reviewSession.state.rightPaneMode === 'diff' && showDiff}
      <DiffPanel
        bind:this={diffPanelRef}
        fileDiffs={currentDiffFiles}
        commitDetail={commitDetail}
        selectedPath={selectedCommitFile ?? selectedFile?.path ?? null}
        diffKind="commit"
        {repoPath}
        loading={stagingDiffLoading}
        onclose={() => { handleDiffClose(); reviewSession.showPanel(); }}
      />
    {:else}
      <ReviewPanel {repoPath} session={reviewSession} onJump={handleReviewJump} onJumpToCommit={handleReviewJumpToCommit} />
    {/if}
  </div>
{:else if showMergeEditor && selectedFile}
  …
```
CONTEXT.md component-changes row: "Inside `{#if reviewSession.state.reviewActive}` the body renders `ReviewPanel` or `DiffPanel` directly with no surrounding strip." Children flow naturally via the parent's existing flex column at line 809 (UI-SPEC §Layout placement).

---

### `src/lib/review-session.svelte.ts` (Svelte rune, state-machine)

**Analog:** the file's own current `generate` method at lines 93-103 — the simplification is a deletion of its state-mutation tail plus a return statement.

**Current `generate`** (the surface being simplified):
```ts
async generate(repoPath: string) {
  // IMPORTANT: the assignment order is "await then write." A rejection
  // from safeInvoke (e.g. TrunkError code "no_comments") propagates
  // before any state mutation, so the panel stays on the list view and
  // the cached previewMarkdown (if any) is untouched.
  const md = await safeInvoke<string>("generate_review_doc", {
    path: repoPath,
  });
  state.previewMarkdown = md;
  state.panelMode = "preview";
},
```

**After simplification** (per CONTEXT.md design + RESEARCH Pattern 2):
```ts
async generate(repoPath: string): Promise<string> {
  return await safeInvoke<string>("generate_review_doc", { path: repoPath });
},
```
Caller composes the result (see ReviewPanel `onCopyClick` above). Rejection still propagates verbatim; no state to taint anymore.

**Other deletions in this file** (per CONTEXT.md component-changes row "remove `panelMode: PanelMode`, `previewMarkdown`, `showList()`, `showPreview()`"):
- Lines 21-25: `PanelMode` type export + comment.
- Lines 31-35: `panelMode` + `previewMarkdown` fields on `ReviewSessionState`.
- Lines 56-58: `showList` + `showPreview` method declarations in `ReviewSessionManager`.
- Line 62 docstring: "Phase 70 DOC-01: calls `generate_review_doc` IPC and, on success, stores…" → rewrite as "calls `generate_review_doc` IPC and returns the markdown. State is untouched; the caller composes the result (e.g. writeText for clipboard)."
- Lines 69-70: the corresponding state-factory initial values.
- Lines 81-84: the dead-cleanup branch inside `setReviewActive(false)` (`state.previewMarkdown = null; state.panelMode = "list";`).
- Lines 86-92: the `showList()` and `showPreview(md)` method bodies.

`reviewActive`, `rightPaneMode`, `setReviewActive`, `showPanel`, `showDiff`, `jumpTo` all stay verbatim. The signature change is `Promise<void>` → `Promise<string>` on `generate`.

**Updated `ReviewSessionState` shape** (after edit, mirroring CONTEXT.md state-machine "After"):
```ts
export interface ReviewSessionState {
  reviewActive: boolean;
  rightPaneMode: RightPaneMode;
}
```

---

### `src/lib/review-session.svelte.test.ts` (vitest, rune state assertions)

**Analog:** the surviving `it("generate awaits safeInvoke for generate_review_doc and stores the result", …)` at lines 41-50 — shape-shifted to assert return value instead of mutated state.

**Current "generate" test** (lines 41-50):
```ts
it("generate awaits safeInvoke for generate_review_doc and stores the result", async () => {
	mockInvoke.mockResolvedValueOnce("# generated markdown");
	const m = createReviewSession();
	await m.generate("/some/path");
	expect(mockInvoke).toHaveBeenCalledWith("generate_review_doc", {
		path: "/some/path",
	});
	expect(m.state.previewMarkdown).toBe("# generated markdown");
	expect(m.state.panelMode).toBe("preview");
});
```

**After edit** (REQ-72-4c: `generate(repoPath)` returns the markdown string):
```ts
it("generate returns the markdown string", async () => {
	mockInvoke.mockResolvedValueOnce("# generated markdown");
	const m = createReviewSession();
	const result = await m.generate("/some/path");
	expect(mockInvoke).toHaveBeenCalledWith("generate_review_doc", {
		path: "/some/path",
	});
	expect(result).toBe("# generated markdown");
});
```

**Tests being deleted** (all reference removed surface — RESEARCH Finding 4: "Plan must include rewriting/deleting tests at `review-session.svelte.test.ts:18-82`"):
- Lines 19-23: `starts with panelMode 'list' and previewMarkdown null`
- Lines 25-30: `showPreview sets previewMarkdown and switches panelMode to 'preview'`
- Lines 32-39: `showList returns panelMode to 'list' and preserves previewMarkdown`
- Lines 52-58: `setReviewActive(false) clears previewMarkdown and resets panelMode to 'list'`
- Lines 60-66: `setReviewActive(true) does NOT touch preview fields`
- Lines 68-81: `generate propagates rejection and leaves state untouched` — REWRITE to drop the `m.state.panelMode === "list"` / `m.state.previewMarkdown === "# previous"` assertions; keep the rejection-propagation assertion:
```ts
it("generate propagates rejection", async () => {
	mockInvoke.mockRejectedValueOnce(
		'{"code":"no_comments","message":"Generate requires at least one comment in the session"}',
	);
	const m = createReviewSession();
	await expect(m.generate("/repo")).rejects.toMatchObject({
		code: "no_comments",
	});
});
```

Describe-block name change: `describe("createReviewSession — preview state", …)` → `describe("createReviewSession — generate", …)` (or fold into a different existing describe if one exists elsewhere — there is none in this file; current file is 82 lines, single describe).

---

### `src-tauri/src/lib.rs` (Rust Tauri menu config)

**Analog:** the `find` accelerator at `src-tauri/src/lib.rs:21-23` — verbatim shape mirror, one chained method call.

**Existing pattern** (lines 21-23):
```rust
let find = MenuItemBuilder::with_id("find", "Find")
    .accelerator("CmdOrCtrl+F")
    .build(app)?;
```

**Current Review item** (line 27-28):
```rust
let review_item =
    MenuItemBuilder::with_id("review-toggle", "Start/End Code Review").build(app)?;
```

**After edit** (CONTEXT.md component-changes row: "add `.accelerator(\"CmdOrCtrl+Shift+R\")` to the `review-toggle` MenuItemBuilder (line 28). Direct mirror of how `find` got `CmdOrCtrl+F` at line 22."):
```rust
let review_item = MenuItemBuilder::with_id("review-toggle", "Start/End Code Review")
    .accelerator("CmdOrCtrl+Shift+R")
    .build(app)?;
```
- `CmdOrCtrl` maps `Cmd` on macOS, `Ctrl` on Windows/Linux — same convention as the existing `find` shortcut.
- The existing `app.on_menu_event` handler at lines 64-70 already routes `review-toggle` → `app.emit("review-toggle", ())`; no Rust code change beyond the one chained `.accelerator(…)` call.
- macOS displays `⌘⇧R` natively in the View menu next to the item — no frontend rendering.

The comment on line 25-26 ("Temporary trigger for the review-session stub (D-12); replaced by the real panel in Phase 69. Mirrors the find → search-toggle precedent.") is stale — Phase 69 has shipped, the comment can be either deleted or updated to describe the post-72 reality. Optional polish, not in CONTEXT.md scope; leave it unless the executor proactively cleans.

---

## Shared Patterns

### Tauri event-bus emit/listen (single-bus, multi-emitter)
**Producers:** `src-tauri/src/lib.rs:67-69` (menu handler, existing); `Toolbar.svelte` (new, frontend emit).
**Consumer:** `App.svelte:555-565` (single `listen<void>("review-toggle", …)` flips `reviewPanelOpen`).
**Apply to:** new `handleReviewToggle` in `Toolbar.svelte`.
```ts
// Producer side (Rust, existing pattern — do NOT change)
app.emit("review-toggle", ())

// Producer side (Svelte, new — first frontend emitter in repo)
import { emit } from "@tauri-apps/api/event";
void emit("review-toggle");

// Consumer side (Svelte, existing pattern — do NOT change)
import { listen } from "@tauri-apps/api/event";
listen<void>("review-toggle", () => { reviewPanelOpen = !reviewPanelOpen; })
```
Rationale: RESEARCH Pattern 1 + Don't Hand-Roll table. Adding a second producer to an existing bus is zero-state-plumbing.

### App-level prop pass-through for the new Toolbar prop
**Source:** `src/App.svelte:582-585`.
**Apply to:** Toolbar mounting site.
```svelte
{#if activeTab?.repoPath}
  {@const activeState = getOrCreateTabState(activeTabId)}
  <Toolbar repoPath={activeTab.repoPath} remoteState={activeState.remoteState} undoRedo={activeState.undoRedo} />
{/if}
```
After edit (RESEARCH Finding 2 + Pitfall 4): add `reviewActive={reviewPanelOpen}`:
```svelte
<Toolbar
  repoPath={activeTab.repoPath}
  remoteState={activeState.remoteState}
  undoRedo={activeState.undoRedo}
  reviewActive={reviewPanelOpen}
/>
```
Rationale: `reviewPanelOpen` is App-owned (`App.svelte:57`), Toolbar is App-mounted (single instance, not per-tab); the per-tab `reviewSession` rune is unreachable from Toolbar. Pass App-owned state as prop, mirroring the existing `remoteState` / `undoRedo` pattern.

### `instanceof Error` narrowing for `unknown` catches (CLAUDE.md, never `as`)
**Source:** `src/components/ReviewDocPreview.svelte:39-42` (carry-forward source).
**Apply to:** the new `onCopyClick` catch in ReviewPanel.svelte, replacing the legacy `(e as TrunkError).message` style at `ReviewPanel.svelte:283`.
```ts
} catch (e) {
  const msg = e instanceof Error ? e.message : String(e);
  showToast(`Failed to copy: ${msg}`, "error");
}
```
Rationale: CLAUDE.md (project) + `coding_style_typescript.md` §1 "Never use `as any` or `as unknown`" + RESEARCH Pitfall 1; the catch must handle errors from EITHER `generate` (TrunkError-shaped) OR `writeText` (plugin Error) — neither can be `as`-cast safely.

### `clearTimeout` before `setTimeout` for re-armable affordances
**Source:** `src/components/ReviewDocPreview.svelte:31-38`.
**Apply to:** the new `onCopyClick` body in ReviewPanel.svelte; corresponding `remains clickable during window` test pattern from `ReviewDocPreview.test.ts:95-123`.
```ts
if (copyTimer !== null) clearTimeout(copyTimer);
copied = true;
copyTimer = setTimeout(() => { copied = false; copyTimer = null; }, 1500);
```
Rationale: re-click during the affordance window extends it (UI-SPEC §Interaction 3), does not race a stale revert. Carry-forward from Phase 71.

### Vitest fake-timer scoping discipline (avoid `setTimeout(0)` deadlock)
**Source:** `src/components/ReviewDocPreview.test.ts:29-47`.
**Apply to:** the new Copy `describe` block in `ReviewPanel.test.ts`.
- `vi.useFakeTimers()` / `vi.useRealTimers()` lifecycle scoped to the Copy describe ONLY (NOT file-global), so the rest of `ReviewPanel.test.ts` retains its `flush` helper at line 115 using `setTimeout(r, 0)`.
- The Copy describe uses a local `flushFake` that does `await Promise.resolve(); await tick();` — never `setTimeout(r, 0)` under fake timers.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| (none) | — | — | Every change in Phase 72 has a concrete in-repo analog — either an existing pattern in the same file, a sibling component, or the file being deleted whose pattern lives on in the replacement. |

The only borderline "novel" surface is the frontend `emit()` call (Toolbar will be the first emit-side in the codebase per RESEARCH Finding 1). Its analog is the Rust-side emit at `src-tauri/src/lib.rs:67-69` — same event string, same payload shape (`void`), same export module on the JS side (`@tauri-apps/api/event` exports both `listen` and `emit` per `node_modules/@tauri-apps/api/event.d.ts:145`). Treat as "exact analog, cross-language."

---

## Metadata

**Analog search scope:** `src/components/`, `src/lib/`, `src-tauri/src/`, `src/App.svelte`, `node_modules/@tauri-apps/api/event.d.ts`, `node_modules/@lucide/svelte/dist/icons/index.js`.
**Files read for pattern extraction:** 11 (Toolbar.svelte, Toolbar.test.ts, ReviewDocPreview.svelte, ReviewDocPreview.test.ts, ReviewPanel.svelte excerpts, ReviewPanel.test.ts excerpts, RepoView.svelte excerpts, review-session.svelte.ts, review-session.svelte.test.ts, src-tauri/src/lib.rs, App.svelte excerpts).
**Pattern extraction date:** 2026-05-27
