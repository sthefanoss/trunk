# Phase 72: Review-Pane UX Integration — Research

**Researched:** 2026-05-27
**Domain:** Svelte 5 / Tauri 2 frontend UX refactor — toolbar entry-point + state-machine simplification
**Confidence:** HIGH (every claim grounded in the on-disk codebase; no external doc lookups required for the design as locked)

## Summary

CONTEXT.md is design-locked and unusually concrete: every file/line edit, the state-machine before/after, the threat model, and the carry-forward patterns are already nailed down. The research surface that actually moves the planner forward is therefore narrow: confirm each *codebase precondition* the design relies on, surface the call sites the simplification touches, and pre-stage the test-pattern carry-forward so the planner can hand the executor a line-by-line template instead of a recipe.

Five concrete findings change how the plan should be shaped:

1. **No frontend `emit()` precedent** — every existing `@tauri-apps/api/event` import in `src/` uses `listen`, never `emit`. The Toolbar will be the first emitter. The API is exported (`@tauri-apps/api/event` exports `listen, once, emit, emitTo, TauriEvent` — verified at `node_modules/@tauri-apps/api/event.d.ts:145`), but the executor is walking new ground inside this codebase. Reusing the existing menu→`app.emit("review-toggle", ())` Rust handler is correct — but the frontend mirror is `import { emit } from "@tauri-apps/api/event"; await emit("review-toggle")`.
2. **`reviewActive` is App-owned, not RepoView-owned** — the source of truth is `App.svelte:57 (reviewPanelOpen = $state(false))` and it's piped into each `RepoView` as the `reviewActive` prop (`App.svelte:603`). The Toolbar lives at the App level (`App.svelte:584`, single instance outside the per-tab loop). For the Toolbar's active-state styling, the cleanest wiring is to pass `reviewPanelOpen` to `Toolbar` as a new prop — *not* to read it from the per-tab `reviewSession` rune.
3. **Theme is forced dark, no light mode** — `src/app.css:4` reads `/* Color tokens — dark theme, forced (no OS media query) */`. No `prefers-color-scheme`, no `data-theme`. The CONTEXT.md "Validate against theme tokens and contrast in *both light/dark mode*" wording and the 71-UAT "toggle OS theme" step assume light/dark switching that does not exist in this codebase. **Light-mode verification is not actionable** — the planner should drop it from the validation matrix and call out the design-doc wording as obsolete to user constraint context.
4. **`previewMarkdown` lives in *two* surface areas, not one** — `src/lib/review-session.svelte.ts` (the rune) AND `src/lib/review-session.svelte.test.ts` (the rune's tests). The CONTEXT.md "Tests (if present)" hedge is too soft: the test file *is* present and has six tests touching `panelMode`/`previewMarkdown` directly. Plan must include rewriting/deleting tests at `review-session.svelte.test.ts:18-82`, not just optionally updating them.
5. **Existing `ReviewPanel.test.ts` also asserts the preview swap** — lines 632-714 verify `generate` → preview swap and the "Back to comments" affordance. These tests will fail the moment the implementation lands. Plan must replace them with the new Copy-flow assertions in a single coordinated edit (not delete-then-add as two separate tasks, or the suite goes red between them).

**Primary recommendation:** Land Phase 72 as **five waves**:
- Wave 0 — pre-flight (verify dev env, no setup tasks needed).
- Wave 1 (TDD candidates, parallelizable) — write red tests for: simplified rune `generate(repoPath): Promise<string>`, ReviewPanel Copy click handler, Toolbar Review button emit + active state.
- Wave 2 (implement-to-green, sequenced after Wave 1) — simplify rune, refactor ReviewPanel, add Toolbar button + prop wiring.
- Wave 3 (structural cleanup, parallelizable) — delete `ReviewDocPreview.svelte` + `.test.ts`; delete blue-button strip from `RepoView.svelte:813-828`; add menu accelerator.
- Wave 4 — `just check` + manual UAT (Cmd+Shift+R, click toolbar button, copy flow).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Review-mode toggle state | Frontend (App.svelte) | — | Already owned at App level via `reviewPanelOpen`. No change to ownership; Toolbar consumes via prop. |
| Toolbar Review button (emit + render) | Frontend (Toolbar.svelte) | — | New `.toolbar-btn`, sibling to existing Branch/Stash/Pop group. |
| `review-toggle` event bus | Tauri event system | App.svelte listener | Rust `app.emit("review-toggle", ())` already fires from menu; frontend `emit()` adds the Toolbar producer. App.svelte's `listen` (line 557) is the single consumer that flips `reviewPanelOpen`. Bus pattern is preserved verbatim. |
| Keyboard accelerator (Cmd+Shift+R) | Tauri menu (src-tauri/src/lib.rs) | — | Mirror of existing `find` shortcut at lib.rs:22; routes through the same menu-event handler. |
| Copy button + clipboard write | Frontend (ReviewPanel.svelte) | `@tauri-apps/plugin-clipboard-manager` | Carry-forward of Phase 71 pattern; no new IPC, no new capability. |
| Markdown generation | Tauri command `generate_review_doc` (unchanged) | — | Already registered (`lib.rs:139`); only the *caller* changes (rune mutation → return value). |

## Standard Stack

### Core (already installed — no new deps)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `@lucide/svelte` | 0.577.0 | Icon set (Toolbar button glyph, Copy glyph) | [VERIFIED: node_modules/@lucide/svelte/package.json] Already in use across Toolbar, ReviewPanel, ReviewDocPreview |
| `@tauri-apps/api/event` | (from Tauri 2) | `listen` / `emit` for the `review-toggle` bus | [VERIFIED: node_modules/@tauri-apps/api/event.d.ts:145] |
| `@tauri-apps/plugin-clipboard-manager` | (granted in Phase 65) | `writeText` for the Copy action | [VERIFIED: imports exist in App.svelte, CommitGraph, StagingPanel, CommitDetail, ReviewDocPreview] |
| `vitest` + `@testing-library/svelte` | (project pinned) | Test framework | [VERIFIED: existing ReviewPanel.test.ts and ReviewDocPreview.test.ts patterns] |

### Lucide icon shortlist (verified present)
| Export | File offset | Candidate role |
|--------|-------------|----------------|
| `MessagesSquare` | icons/index.js:1042 | Toolbar Review button (proposed default) |
| `MessageSquareText` | icons/index.js:1038 | Alternative |
| `BookOpenCheck` | icons/index.js:241 | Alternative (review-completion semantics) |
| `ClipboardList` | icons/index.js:445 | Alternative |
| `Clipboard` | icons/index.js:453 | Already used by ReviewDocPreview's Copy — proposed for the new ReviewPanel Copy button (consistency carry-forward) |

**Installation:** N/A — zero new packages.

## Package Legitimacy Audit

> Skipped — phase installs no new packages (CONTEXT.md Net effect table confirms "Files added: 0"; threat model T-72-SC confirms "zero new packages"). All packages referenced are pre-existing in `package.json` and were vetted in prior phases.

## Architecture Patterns

### System Data Flow Diagram (new shape)

```
                            ┌──────────────────────────────────┐
                            │  User input surfaces (any of):   │
                            │  • Toolbar Review button click   │  NEW
                            │  • Cmd+Shift+R                   │  NEW (menu accelerator)
                            │  • View → Start/End Code Review  │  EXISTING
                            └────────────┬─────────────────────┘
                                         │ emit("review-toggle")
                                         ▼
                         App.svelte $effect listen("review-toggle")
                                         │
                                         ▼
                                reviewPanelOpen = !reviewPanelOpen
                                         │ prop
                                         ▼
                       RepoView.svelte $effect (line 89)
                                         │
                                         ▼
                       reviewSession.setReviewActive(reviewActive)
                                         │
                                         ▼
                          {#if reviewSession.state.reviewActive}
                                         │
                                         ▼
                       ┌─────────────────┴────────────────┐
                       │  rightPaneMode === 'diff'?       │
                       └───┬──────────────────────────┬───┘
                       Yes │                          │ No (default)
                           ▼                          ▼
                       DiffPanel                ReviewPanel
                           │                          │
              onclose=showPanel()    [Copy click in panel header]
                                                      │
                                                      ▼
                                  await session.generate(repoPath)  ← returns string
                                                      │
                                                      ▼
                                  await writeText(markdown)
                                                      │
                                                      ▼
                                  copied=true; 1500ms timer; ✓ Copied
```

### Project structure (unchanged)
No new folders. Edits are surgical:
```
src/
├── App.svelte                       # +1 prop pass-through on <Toolbar>
├── components/
│   ├── Toolbar.svelte               # +Review button, +emit, +prop
│   ├── Toolbar.test.ts              # +Review-button tests
│   ├── ReviewPanel.svelte           # -preview swap, +Copy button
│   ├── ReviewPanel.test.ts          # -preview tests, +Copy tests
│   ├── RepoView.svelte              # -blue-button header strip (lines 813-828)
│   ├── ReviewDocPreview.svelte      # DELETED
│   └── ReviewDocPreview.test.ts     # DELETED
├── lib/
│   ├── review-session.svelte.ts     # -panelMode/previewMarkdown/showList/showPreview
│   └── review-session.svelte.test.ts# rewrite preview-state suite
└── src-tauri/
    └── src/lib.rs                   # +.accelerator("CmdOrCtrl+Shift+R")
```

### Pattern 1 — Frontend emit() mirror of menu emit
**What:** The menu handler at `lib.rs:67-69` fires `app.emit("review-toggle", ())`. The Toolbar emits the same string from the frontend.
**Why:** Single subscriber pattern — `App.svelte:557` already listens. Routing every entry point through the same event means zero new state plumbing.
**Example:**
```svelte
<!-- Toolbar.svelte -->
<script lang="ts">
  import { emit, listen } from "@tauri-apps/api/event";
  import { MessagesSquare } from "@lucide/svelte";
  // existing props ...
  interface Props {
    repoPath: string;
    remoteState: RemoteState;
    undoRedo: UndoRedoManager;
    reviewActive: boolean;   // NEW
  }
  let { repoPath, remoteState, undoRedo, reviewActive }: Props = $props();

  function handleReviewToggle() {
    void emit("review-toggle");
  }
</script>

<button
  class="toolbar-btn"
  class:toolbar-btn-active={reviewActive}
  onclick={handleReviewToggle}
  aria-pressed={reviewActive}
>
  <MessagesSquare size={14} /> Review
</button>
```
- `aria-pressed` is recommended for toggle buttons (a11y) — the CONTEXT.md doesn't mandate it but it's free and consistent with other toggle affordances.
- `void emit(...)` matches the fire-and-forget style at `CommitGraph.svelte:757` (`writeText(...).catch(() => {})`); errors emitting an event are not user-facing so they don't need toast surfacing.

### Pattern 2 — `generate(repoPath)` returns the markdown string
**What:** The rune's `generate` becomes a pure async function that does *one* thing (the IPC) and returns the result.
**Why:** Eliminates the panelMode/previewMarkdown coupling; the caller decides what to do with the bytes.
**Example:**
```ts
// review-session.svelte.ts (after)
async generate(repoPath: string): Promise<string> {
  return await safeInvoke<string>("generate_review_doc", { path: repoPath });
}
```
Test contract: `expect(await session.generate("/repo")).toBe("# stub\n")`.

### Pattern 3 — Copy click handler in ReviewPanel
**What:** Single try/catch wraps `generate → writeText`. On success: ✓ Copied for 1500ms with `clearTimeout` re-arm. On failure: `showToast("Failed to copy: ${msg}", "error")` and stay on Copy.
**Why:** Direct carry-forward of `ReviewDocPreview.svelte:28-44` lines, just hosted in the comments view header.
**Example:**
```ts
let copied = $state(false);
let copyTimer: ReturnType<typeof setTimeout> | null = null;

async function onCopyClick() {
  try {
    const md = await session.generate(repoPath);
    await writeText(md);
    if (copyTimer !== null) clearTimeout(copyTimer);
    copied = true;
    copyTimer = setTimeout(() => { copied = false; copyTimer = null; }, 1500);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    showToast(`Failed to copy: ${msg}`, "error");
  }
}
```
Replaces `onGenerateClick` at `ReviewPanel.svelte:278-287` and the button at lines 359-368.

### Anti-Patterns to Avoid
- **Don't store the markdown in the rune** — the whole point of the simplification. If the executor sees `state.previewMarkdown = md`, they've reintroduced the coupling.
- **Don't make Toolbar read the per-tab `reviewSession`** — `reviewSession` is created inside `RepoView` (line 86), one per repo tab. The Toolbar is App-level and cannot reach it. Use the App-owned `reviewPanelOpen` flag piped as a prop.
- **Don't catch errors from `emit()`** — the channel is local and reliable; wrapping it in try/catch adds noise. `void emit(...)` is the pattern.
- **Don't add a separate "Generate" button** — Copy *is* the only action. Generation happens transparently inside the click handler.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cross-component "is review mode on?" coordination | A new store / shared rune / context API | The existing `review-toggle` Tauri event + App-level `reviewPanelOpen` prop pass | The event bus is already wired; reuse it. |
| Keyboard shortcut handling for Cmd+Shift+R | A `window.addEventListener("keydown", ...)` in Svelte | `MenuItemBuilder::with_id("review-toggle", ...).accelerator("CmdOrCtrl+Shift+R")` | Tauri menu accelerators are the canonical path; macOS already has the menu entry, and OS-managed accelerators get proper key-equivalent display and chord conflict handling for free. |
| Toggle-button active styling | Inline `style="background: ...; color: ..."` ternaries (per `RepoView.svelte:819-826`) | A `:global(.toolbar-btn-active)` class with `class:toolbar-btn-active={reviewActive}` | The blue button at RepoView:815 is the exact anti-pattern we're deleting. Don't recreate it on the new button. |
| Storing the generated markdown across clicks | A `state.previewMarkdown` cache or in-component `let lastMd = $state(...)` | Re-call `generate(repoPath)` on each Copy click | YAGNI (per CONTEXT.md design rationale); the IPC is bounded and cheap. |

## Common Pitfalls

### Pitfall 1: Re-introducing the preview pattern via memoization
**What goes wrong:** Executor adds `let lastMd = $state<string | null>(null)` or caches in the rune "as a perf optimization."
**Why it happens:** Caching looks free; the previous design did it.
**How to avoid:** The design-locked decision is **no cache**. The executor must not add one.
**Warning signs:** Any `$state<string | null>` for markdown, any conditional skip of the `await session.generate(...)` call.

### Pitfall 2: Light-mode visual validation that can't be performed
**What goes wrong:** Manual UAT includes "toggle OS theme to light mode" and a tester reports "I don't see anything change."
**Why it happens:** The 71-UAT.md test #5 inherits the assumption from earlier phases; CONTEXT.md repeats it; the app is forced-dark (verified at `src/app.css:4`).
**How to avoid:** Drop light-mode checks from the validation map. Document this constraint in CONTEXT and in the validation architecture.
**Warning signs:** UAT step says "test in light mode" — should be removed pre-execution.

### Pitfall 3: Tests fail in the gap between deletion and replacement
**What goes wrong:** Delete `ReviewDocPreview.test.ts` first, then write new tests. The suite is red between commits. CI/precommit blocks.
**Why it happens:** Treating "delete X" and "add Y" as separate atomic units when they assert on the same surface.
**How to avoid:** Plan must group: (a) delete `ReviewDocPreview` (component + test) ONLY AFTER (b) the existing `ReviewPanel.test.ts:632-714` preview tests are rewritten as Copy tests, and (c) the rune simplification + component refactor have landed. Order: refactor first, then delete.
**Warning signs:** A Wave structure that puts "delete ReviewDocPreview" parallel to "refactor ReviewPanel".

### Pitfall 4: Toolbar not knowing about `reviewActive`
**What goes wrong:** Executor adds the button but it never shows active styling because Toolbar has no way to observe `reviewPanelOpen`.
**Why it happens:** The CONTEXT.md doesn't explicitly say "add a `reviewActive` prop to Toolbar"; it just says "active-state styling (when `reviewActive === true`)".
**How to avoid:** Plan must include the prop addition at `App.svelte:584` (`<Toolbar reviewActive={reviewPanelOpen} ... />`) and the matching `Props` interface change.
**Warning signs:** No edit to `App.svelte` in the plan.

### Pitfall 5: `aria-pressed` missing → a11y regression
**What goes wrong:** Toggle button has no ARIA role indicator; screen readers say "Review button" instead of "Review toggle, pressed/not pressed".
**Why it happens:** Toolbar has no precedent for toggle buttons — only momentary actions.
**How to avoid:** Add `aria-pressed={reviewActive}` to the button. Cheap and standard.
**Warning signs:** Tests don't probe `aria-pressed` at all.

## Code Examples

### Verified pattern: clipboard mock for tests
```ts
// Carry-forward from ReviewDocPreview.test.ts:21-23 — verified pattern.
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn().mockResolvedValue(undefined),
}));
```

### Verified pattern: fake-timers + flush helpers
```ts
// From ReviewDocPreview.test.ts:29-47 — line-for-line carry-forward.
beforeEach(() => {
  vi.clearAllMocks();
  vi.useFakeTimers();
});
afterEach(() => {
  vi.useRealTimers();
});

async function flush() {
  await Promise.resolve();
  await tick();
}
// NEVER setTimeout(r, 0) under fake timers — deadlocks.
```

### Verified pattern: `Cop(y|ied)` regex selector
```ts
// From ReviewDocPreview.test.ts:52-54 — covers both states with one query.
function getCopyButton() {
  return screen.getByRole("button", { name: /Cop(y|ied)/ });
}
```

### Verified pattern: Tauri menu accelerator (mirror of existing)
```rust
// src-tauri/src/lib.rs — after edit
let review_item = MenuItemBuilder::with_id("review-toggle", "Start/End Code Review")
    .accelerator("CmdOrCtrl+Shift+R")
    .build(app)?;
```
- `CmdOrCtrl` maps Cmd on macOS and Ctrl on Windows/Linux — matches the existing `find` shortcut at lib.rs:21-23.
- No app code changes besides the chained `.accelerator(...)` call.

### Verified pattern: emit() in Svelte (will be the first frontend emitter)
```svelte
<script lang="ts">
  import { emit } from "@tauri-apps/api/event";
  function handleReviewToggle() {
    void emit("review-toggle");
  }
</script>
```
- `emit` is exported alongside `listen` from `@tauri-apps/api/event` (verified `node_modules/@tauri-apps/api/event.d.ts:145`).
- The payload is `void` to match the Rust emitter at `lib.rs:68`.

## Runtime State Inventory

> Skipped — this phase is a pure source-code refactor. No persisted user data uses the strings `panelMode`/`previewMarkdown`/etc.; no external service stores them; no installed binaries embed them. Verified by grep: all matches are in `src/` (component or test code) — see Step "grep panelMode/previewMarkdown" results in `### Open Implementation Choices` below.

## Environment Availability

> Skipped — this phase only edits checked-in source files. The build/test toolchain (`just check`: biome, svelte-check, vitest, cargo) is already established by the running project; if it wasn't, the user couldn't be at Phase 72.

## Validation Architecture

Test infrastructure is mature: vitest for `src/`, cargo test for `src-tauri/`. Both wired into `just check`. The Phase 70/71 `ReviewPanel.test.ts` + `ReviewDocPreview.test.ts` + `review-session.svelte.test.ts` triad already exercises the surface that's being refactored; the migration is mostly about *replacing* assertions, not authoring net-new infrastructure.

### Test Framework
| Property | Value |
|----------|-------|
| Framework (frontend) | vitest with `@testing-library/svelte`; config at `vite.config.ts` / `vitest.config.ts` (project-managed) |
| Framework (backend) | `cargo test` (no new Rust tests for this phase per CONTEXT.md) |
| Quick run (component) | `bun run vitest run src/components/ReviewPanel.test.ts src/components/Toolbar.test.ts src/lib/review-session.svelte.test.ts` |
| Full suite | `just check` |
| Phase gate | `just check` exits 0 |

### Phase Requirements → Test Map

CONTEXT.md success criteria are numbered 1–6 in the source. Mapped here as REQ-72-1 through REQ-72-6. Note also the two gaps the phase closes per init: **G-71-A** (Copy off the preview pane) and **G-71-B** (review-pane navigation + dead button).

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-72-1a | Toolbar Review button click emits `review-toggle` | unit | `vitest run src/components/Toolbar.test.ts -t "review-toggle"` | ❌ Wave 1 |
| REQ-72-1b | Cmd+Shift+R triggers the menu item (Rust-side wiring) | manual UAT | (none — accelerator is config-only) | manual |
| REQ-72-1c | View menu "Start/End Code Review" still works | manual UAT (regression) | (none) | manual |
| REQ-72-2 | Toolbar button shows active styling when `reviewActive === true` | unit | `vitest run src/components/Toolbar.test.ts -t "active state"` | ❌ Wave 1 |
| REQ-72-3a | Copy click calls `generate` then `writeText` with the returned markdown | unit | `vitest run src/components/ReviewPanel.test.ts -t "copy click invokes generate and writeText"` | ❌ Wave 1 (overwrites existing 632-714) |
| REQ-72-3b | ✓ Copied affordance for 1500ms with re-arm | unit | `vitest run src/components/ReviewPanel.test.ts -t "remains clickable during window"` | ❌ Wave 1 |
| REQ-72-3c | Error toast on writeText/generate failure with `instanceof Error` narrowing | unit | `vitest run src/components/ReviewPanel.test.ts -t "shows error toast on failure"` | ❌ Wave 1 |
| REQ-72-3d | Non-`Error` rejection coerced via `String(e)` | unit | `vitest run src/components/ReviewPanel.test.ts -t "coerces non-Error rejection"` | ❌ Wave 1 |
| REQ-72-3e | Copy button stays in "Copy" on failure (does NOT flip to Copied) | unit | `vitest run src/components/ReviewPanel.test.ts -t "does not flip copied on failure"` | ❌ Wave 1 |
| REQ-72-4a | `ReviewDocPreview.svelte` deleted | unit (negative — file absence) | `test ! -f src/components/ReviewDocPreview.svelte` (or vitest does an import-resolves-no test) | manual file check |
| REQ-72-4b | `panelMode` / `previewMarkdown` / `showList` / `showPreview` removed from rune | unit | `vitest run src/lib/review-session.svelte.test.ts` (the rewritten suite must NOT reference these names) | ❌ Wave 1 |
| REQ-72-4c | `generate(repoPath)` returns the markdown string | unit | `vitest run src/lib/review-session.svelte.test.ts -t "generate returns markdown string"` | ❌ Wave 1 |
| REQ-72-5a | Blue-button header strip removed from RepoView | unit (DOM absence) | `vitest run src/components/RepoView.test.ts -t "no review header strip"` | manual UAT may suffice |
| REQ-72-5b | DiffPanel close still returns to ReviewPanel (regression of existing wiring) | manual UAT (or extension of RepoView.test.ts) | (existing test if any) | manual |
| REQ-72-6 | `just check` green | smoke | `just check` | wired |
| G-71-A | Copy lives on comments view, not preview pane | covered by REQ-72-3a/4a | (same as above) | — |
| G-71-B | Smooth entry/exit + no dead button | covered by REQ-72-1a/2/5a | (same as above) | — |

### Sampling Rate
- **Per task commit (Nyquist):** `bun run vitest run <touched file>.test.ts`
- **Per wave merge:** `bun run vitest run` (full Vitest suite) + `cargo test`
- **Phase gate:** `just check` exits 0; manual UAT for keyboard accelerator and View-menu regression.

### Wave 0 Gaps
- [ ] Confirm whether `RepoView.test.ts` already asserts on the deleted header strip (lines 813-828). If yes, update it; if no, accept manual UAT for REQ-72-5a.
- [ ] No framework install required.
- [ ] No new fixtures required; the existing `installReads` dispatcher in `ReviewPanel.test.ts:92-113` already handles `generate_review_doc`.

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | no | No new untrusted inputs; markdown comes from a trusted local IPC handler |
| V6 Cryptography | no | — |

### Known Threat Patterns for Tauri 2 / Svelte 5 desktop

| Pattern | STRIDE | Mitigation |
|---------|--------|-----------|
| Event-bus spoofing (an attacker process emitting `review-toggle` to drive UI state) | Spoofing | Tauri 2's event bus is in-process within the app's webview ↔ Rust core only; no IPC permission needed for `app.emit` / frontend `emit`. Not externally reachable. |
| Clipboard write to leak secrets | Info disclosure | The capability `clipboard-manager:allow-write-text` was already granted in Phase 65; this phase writes the same data shape (review markdown) the user explicitly clicked Copy for. T-72-I in CONTEXT accepts LOW. |
| Markdown render XSS | — | N/A — preview is being deleted; nothing renders the markdown to HTML. Copy writes raw bytes to OS clipboard. |

Threat model is unchanged from Phase 71 → Phase 72 (no new principal, no new IPC, no new capability). CONTEXT.md threat table is authoritative; this section confirms no contradictions surfaced during research.

## Sources

### Primary (HIGH confidence)
- `src/App.svelte` lines 57, 555-565, 584, 603 — App-level `reviewPanelOpen` ownership + Toolbar mounting + RepoView prop pass
- `src/components/Toolbar.svelte` lines 1-285 — current Toolbar shape, `.toolbar-btn` CSS, group structure
- `src/components/ReviewPanel.svelte` lines 1-762 — current panel; preview swap branch (332-339), Generate button (359-368), `onGenerateClick` (278-287)
- `src/components/ReviewDocPreview.svelte` lines 1-137 — Copy handler, 1500ms timer, error narrowing
- `src/components/ReviewDocPreview.test.ts` lines 1-168 — verified clipboard test patterns
- `src/components/ReviewPanel.test.ts` lines 1-716 — existing preview tests at 632-714 that need replacement
- `src/components/RepoView.svelte` lines 810-845 — blue-button strip + DiffPanel close wiring
- `src/components/Toolbar.test.ts` lines 1-128 — existing Toolbar test patterns to extend
- `src/lib/review-session.svelte.ts` lines 1-123 — current rune surface area
- `src/lib/review-session.svelte.test.ts` lines 1-82 — preview-state tests that need rewriting
- `src-tauri/src/lib.rs` lines 14-72 — menu registration + emit pattern
- `node_modules/@tauri-apps/api/event.d.ts:145` — confirms `emit` export
- `node_modules/@lucide/svelte/dist/icons/index.js` lines 241, 445, 453, 1038, 1042 — confirms icon exports
- `node_modules/@lucide/svelte/package.json` — version 0.577.0
- `src/app.css:4` — confirms dark-only theme (no OS media query)

### Secondary (MEDIUM confidence)
- None required — every claim is grounded in primary sources.

### Tertiary (LOW confidence)
- None.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The Toolbar receiving a new `reviewActive` prop (vs. the Toolbar subscribing to `review-toggle` itself) is the right design | Pitfall 4 / Pattern 1 | LOW — both paths work; prop is more idiomatic in Svelte and matches existing `remoteState`/`undoRedo` pattern. If executor picks the listener path, the visual result is the same. |
| A2 | `aria-pressed={reviewActive}` should be added even though CONTEXT.md doesn't mandate it | Pattern 1 / Pitfall 5 | LOW — purely additive a11y improvement, no behavior change. |
| A3 | Light-mode validation should be dropped from the UAT | Pitfall 2 | LOW — verified `app.css:4` says forced dark; no observable behavior to test. Worst case: user wanted light-mode support and this phase doesn't add it (out of scope per CONTEXT.md anyway). |
| A4 | The 8 ReviewDocPreview.test.ts test cases map 1:1 to new ReviewPanel.test.ts Copy-flow tests | Validation Architecture | LOW — direct carry-forward; the rename is from "ReviewDocPreview" describe block to a new "Copy" describe inside the existing ReviewPanel suite. |
| A5 | `void emit("review-toggle")` (fire-and-forget) is appropriate; no toast on emit failure | Pattern 1 | LOW — matches existing `writeText(...).catch(() => {})` precedent at `CommitGraph.svelte:757`; emit on a local event bus is not user-failure-surface. |

## Open Questions (for planner to flag, not block)

1. **Single Copy button position in ReviewPanel header** — CONTEXT.md says "rename header 'Generate' button to 'Copy'" but the panel header currently uses `<span class="preview-spacer" style="flex: 1;">` + button at the right. Confirm during planning that the button keeps the right-docked position. (Recommendation: yes; mirror ReviewDocPreview's docking.)
   - What we know: current header is `flex` with `gap: 8px`; spacer at flex:1 pushes button right.
   - What's unclear: nothing meaningful — the swap is mechanical.
   - Recommendation: keep current position; only the label/icon/handler change.

2. **Toolbar.test.ts mocking of `@tauri-apps/api/event`** — the existing test file mocks `listen` only (line 9). The new test needs to spy on `emit` *or* mock the module to expose a captured `emit`. The cleanest carry-forward is:
   ```ts
   vi.mock("@tauri-apps/api/event", () => ({
     listen: vi.fn().mockResolvedValue(() => {}),
     emit: vi.fn().mockResolvedValue(undefined),
   }));
   ```
   - Recommendation: include this pattern verbatim in the plan.

3. **Does `RepoView.test.ts` test the blue-button strip?** — Plan should grep and only add a regression test if no existing coverage exists.
   - Recommendation: planner should include a one-line "grep then decide" task before authoring a redundant test.

4. **`Toolbar.svelte`'s App.svelte prop pass** — at `App.svelte:584` the Toolbar only renders if `activeTab?.repoPath` is truthy. When the user is on a no-repo tab, the Toolbar is hidden — and so is the Review button. Confirm this is acceptable (Cmd+Shift+R still works because the menu accelerator is global; the View menu item works; only the toolbar entry is hidden). CONTEXT.md is silent.
   - Recommendation: accept as-is — review mode is meaningless without a repo; if a user is on a no-repo tab and hits Cmd+Shift+R, the menu accelerator fires `review-toggle`, `reviewPanelOpen` flips, but `tab.id === activeTabId` is true only for an active no-repo tab, so `reviewActive` on the prop becomes `true && true = true`… BUT `RepoView` is only mounted when `tab.repoPath` exists (App.svelte:591), so review mode has no UI surface. The flag toggles invisibly. This is fine, but worth a one-line comment in the executor's note: "Hitting Cmd+Shift+R from a no-repo tab is a no-op visible only when a repo tab is later opened." Optional polish, not a blocker.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every package verified on disk in `node_modules/`
- Architecture: HIGH — every code path traced via grep + line read
- Pitfalls: HIGH — derived from concrete codebase invariants (forced-dark theme, prop ownership in App vs. RepoView, existing test coverage)
- Validation map: HIGH — directly mirrors existing `ReviewDocPreview.test.ts` patterns
- Tests-to-write count: HIGH — 8 from ReviewDocPreview + 2 new for Toolbar + 4 rewrites in review-session.svelte.test = bounded and known

**Research date:** 2026-05-27
**Valid until:** 2026-06-26 (no fast-moving APIs in scope; codebase grep findings are stable as long as no other phase rewrites these files first).

## RESEARCH COMPLETE

**What is now known:** Every codebase precondition for CONTEXT.md's locked design is verified — the `review-toggle` event bus is wired (Rust emits at `lib.rs:68`, App listens at `App.svelte:557`, Toolbar will be the first frontend emitter via the also-exported `emit` from `@tauri-apps/api/event`); the rune simplification is non-invasive (only one component reads `panelMode`/`previewMarkdown`/`showList`/`showPreview` — `ReviewPanel.svelte`, plus one test file `review-session.svelte.test.ts` whose preview-state suite needs a full rewrite); all four candidate Lucide icons + `Clipboard` are present in `@lucide/svelte@0.577.0`; the Tauri menu accelerator addition is a one-line mirror of the existing `find` shortcut; the carry-forward test patterns (clipboard mock, fake-timers, `Cop(y|ied)` regex, 1500ms re-arm) are extractable line-for-line from `ReviewDocPreview.test.ts` into the new Copy block of `ReviewPanel.test.ts`.

**Material findings beyond the design doc:** (1) **Toolbar lives at App level, not per-tab** — it must receive `reviewActive` as a new prop wired from `App.svelte:584`; the design doc's "active-state styling when `reviewActive === true`" was silent on how the Toolbar observes that flag. (2) **The app is forced-dark** (`src/app.css:4`) — the design doc's and 71-UAT's "both light and dark mode" wording should be dropped from the validation matrix. (3) **`ReviewPanel.test.ts:632-714` actively asserts the preview swap** — the plan must rewrite those tests in the same commit/wave as the component refactor, not as a separate later task, or the suite goes red between commits. (4) **There is no frontend `emit()` precedent in the codebase** — this phase introduces it; the executor needs the pattern handed to them explicitly.

**Residual unknowns the planner should flag (not blockers):** (a) whether `RepoView.test.ts` currently asserts on the blue-button header strip — quick grep; (b) preferred Lucide icon between the four candidates — pure visual judgment, recommend `MessagesSquare` for distinctness from `MessageSquareText` (used elsewhere as add-note glyph) and `Clipboard` (used for the Copy action); (c) whether to add `aria-pressed` (recommend yes — free a11y win, no design change needed). All other choices in CONTEXT.md are design-locked; no other discovery work is needed before planning.
