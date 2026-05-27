# Phase 73: Review Lifecycle - Pattern Map

**Mapped:** 2026-05-27
**Files analyzed:** 2 (1 modified, 1 modified)
**Analogs found:** 2 / 2 (in-file analogs — every pattern carries forward from the same file/test pair)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/components/ReviewPanel.svelte` (MODIFIED) | component | request-response + event-driven | `src/components/ReviewPanel.svelte` (Phase 72 Copy section) | exact (in-file carry-forward) |
| `src/components/ReviewPanel.test.ts` (MODIFIED) | test | request-response + event-driven | `src/components/ReviewPanel.test.ts` (Phase 72 `describe("Copy", …)` block) | exact (in-file carry-forward) |

**Scope check:** RESEARCH explicitly bounds this phase to "pure frontend wiring … no backend schema change." No new files; no Rust changes. `src-tauri/src/commands/review.rs` is read-only context (existing `_inner` + thin-command primitives are reused).

---

## Pattern Assignments

### `src/components/ReviewPanel.svelte` — modifications

**Analog:** the file itself. Every new affordance has a verbatim precedent in the same component.

#### Imports pattern (lines 8-20)

```typescript
import { Clipboard, MessageSquarePlus } from "@lucide/svelte";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { safeInvoke, type TrunkError } from "../lib/invoke.js";
import type { ReviewSessionManager } from "../lib/review-session.svelte.js";
import { showToast } from "../lib/toast.svelte.js";
import type {
    Comment,
    CommentResolution,
    OrphanReason,
    SessionCommit,
    SessionStatus,
} from "../lib/types.js";
```

**To add for Phase 73:** extend the lucide import with `Trash2` (no other import changes — `safeInvoke`, `showToast`, `SessionStatus`, `listen` are all already in scope):

```typescript
import { Clipboard, MessageSquarePlus, Trash2 } from "@lucide/svelte";
```

`SessionState` (the kebab-case string union) lives in `src/lib/types.ts:336`; if a `sessionState` local rune is added, import the type alongside `SessionStatus`.

#### Rune-state pattern — two-step End button (analog: `copied` + `copyTimer` at lines 130-134)

```typescript
// Phase 72 — Copy state. Pattern carry-forward from the Phase 71 preview
// component (being deleted in Plan 04; this is now the canonical home).
let copied = $state(false);
// Plain handle, not $state — only used to clear; reactivity is on `copied`.
let copyTimer: ReturnType<typeof setTimeout> | null = null;
```

**Pattern to copy verbatim for `endConfirming` + `endTimer`:**
- Boolean `$state` flag drives the label/color swap.
- Timer handle is plain (`let endTimer: ReturnType<typeof setTimeout> | null = null`), NOT `$state` — reactivity is on the flag only.
- Same `clearTimeout` before `setTimeout` discipline (see `onCopyClick` excerpt below).

#### Async destructive IPC handler (analog: `onCopyClick` at lines 307-325)

```typescript
async function onCopyClick() {
    try {
        const md = await session.generate(repoPath);
        await writeText(md);
        // Pitfall 2 carry-forward: clear any in-flight revert timer before
        // scheduling a new one. Rapid re-clicks must extend the affordance,
        // not race against it.
        if (copyTimer !== null) clearTimeout(copyTimer);
        copied = true;
        copyTimer = setTimeout(() => {
            copied = false;
            copyTimer = null;
        }, 1500);
    } catch (e) {
        // `unknown` in TS strict; narrow rather than cast. Pitfall 1 + CLAUDE.md.
        const msg = e instanceof Error ? e.message : String(e);
        showToast(`Failed to copy: ${msg}`, "error");
    }
}
```

**For `onEndClick`:** SAME shape (try / await `safeInvoke` / catch with `errorMessage(e, "Failed to end review")`), but split across two paths:
1. First click: schedule the auto-revert timer, flip `endConfirming = true`, return early — no IPC yet.
2. Second click within the window: cancel timer, invoke `end_review_session`, let the `session-changed` listener handle the canonical reload (do NOT manually clear arrays on success — that is the listener's job; see Pitfall 2 in RESEARCH).

Use `errorMessage(e, "Failed to end review")` (not the inline `e instanceof Error ? e.message : String(e)` from Copy) — that helper at lines 152-156 handles both native `Error` and `TrunkError` shapes, which is what `safeInvoke` rejections produce.

#### Error narrowing (analog: `isTrunkError` + `errorMessage` at lines 139-156)

```typescript
function isTrunkError(e: unknown): e is TrunkError {
    return (
        typeof e === "object" &&
        e !== null &&
        "code" in e &&
        "message" in e &&
        typeof (e as { message: unknown }).message === "string"
    );
}

function errorMessage(e: unknown, fallback: string): string {
    if (e instanceof Error) return e.message;
    if (isTrunkError(e)) return e.message;
    return fallback;
}
```

**Already in the file** — do NOT redefine. The resume catch and end catch both call `errorMessage(e, "Failed to resume review")` / `errorMessage(e, "Failed to end review")`. No `as TrunkError` casts (CLAUDE.md / coding_style_typescript §1).

#### `reload()` modification — cold-boot resume branch (analog: existing body at lines 210-253)

```typescript
async function reload() {
    try {
        const status = await safeInvoke<SessionStatus>(
            "get_review_session_status",
            { path: repoPath },
        );
        canonicalPath = status.canonical_path;
    } catch {
        // Tolerate — the panel can still try the list reads below…
    }

    try {
        const [nextCommits, nextComments, nextResolutions] = await Promise.all([
            safeInvoke<SessionCommit[]>("list_session_commits", { path: repoPath }),
            safeInvoke<Comment[]>("list_session_comments", { path: repoPath }),
            safeInvoke<CommentResolution[]>("resolve_session_comments", {
                path: repoPath,
            }),
        ]);
        commits = nextCommits;
        comments = nextComments;
        resolutions = nextResolutions;
    } catch (e) {
        if (isTrunkError(e) && e.code === "no_session") {
            commits = []; comments = []; resolutions = [];
            return;
        }
        showToast("Failed to load review comments. Reload the panel to retry.", "error");
    }
}
```

**Phase 73 modification (D-01, D-07):** insert the cold-boot resume between the status read and the parallel reads. Two invariants to preserve:
- Keep `canonicalPath = status.canonical_path` **before** the resume call so the `session-changed` listener at lines 353-367 can filter (WR-02 carry-forward).
- The existing `no_session` swallow at lines 242-247 must keep working. A resume rejection on `RefusedNewer` should surface a toast but NOT prevent the subsequent reads (which then return `no_session` → cold empty state).
- `sessionState` (new local `$state<SessionState>`) is assigned from `status?.state ?? "none"` inside the same try block, so empty-state copy gating has a single source of truth.

#### Header layout (analog: lines 376-403)

```svelte
<div
    class="flex items-center"
    style="
        gap: 8px;
        padding: 6px 12px;
        background: var(--color-surface);
        border-bottom: 1px solid var(--color-border);
        flex-shrink: 0;
        font-size: 12px;
    "
>
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
</div>
```

**Phase 73 modification (D-03):** add the End button as a sibling button NEXT to the Copy button inside the same `flex items-center` row — DO NOT introduce a new flex container, DO NOT absolutely position (CLAUDE.md: "Never fight layout with positioning hacks"). Gate on `sessionState !== 'none'` via `{#if}` block (UI-SPEC: hide entirely, don't render disabled — Pitfall 5 in RESEARCH). Icon: `<Trash2 size={14} />` matches the `<Clipboard size={14} />` precedent at line 399.

#### Empty-state copy (analog: lines 417-431)

```svelte
{#if groups.length === 0}
    <div class="flex flex-col" style="gap: 4px; padding: 12px;">
        <span>No commits in this review yet.</span>
        <span style="color: var(--color-text-muted); font-size: 11px;">
            Add commits from the graph to start reviewing.
        </span>
    </div>
{:else if !hasAnyComment}
    <div class="flex flex-col" style="gap: 4px; padding: 12px;">
        <span>No comments yet.</span>
        <span style="color: var(--color-text-muted); font-size: 11px;">
            Select lines in a diff to comment, or add a note to a commit above.
        </span>
    </div>
{/if}
```

**Phase 73 modification (D-06, UI-SPEC § Copywriting Contract):** add a `sessionState === 'none'` branch BEFORE the existing `groups.length === 0` branch, with copy "No active review" / "Toggle review mode in the toolbar to start." The existing two-span muted-caption shape (`gap: 4px; padding: 12px` outer; `color: var(--color-text-muted); font-size: 11px` body) is the exact pattern — reuse verbatim. The warm-with-commits-but-no-comments string is updated from "No comments yet." to "Review started." per UI-SPEC.

#### CSS — danger button (analog: `.copy-button` at lines 775-796 + `.card-action-danger` at lines 728-730)

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
.copy-button:hover:not([disabled]),
.copy-button:focus-visible:not([disabled]) {
    color: var(--color-text);
    background: var(--color-hover);
}
.copy-button[disabled] {
    cursor: not-allowed;
    opacity: 0.5;
}
```

```css
.card-action-danger { color: var(--color-danger); }
.card-action-danger:hover,
.card-action-danger:focus-visible { color: var(--color-danger); }
```

**Phase 73 addition:** new `.end-button` class that copies `.copy-button`'s triplet (`display`, `padding`, `border-radius`, `font-size`, `font-family`, `gap`, `border: 1px solid var(--color-border)`), but with `color: var(--color-text-muted)` in idle and a `.end-button.confirming` modifier that flips to `--color-danger-bg` background + `--color-danger-border` border + `--color-on-accent` text (per UI-SPEC § Interaction States). All four color values are existing `:root` custom properties — verify presence in `src/app.css` before use; NO hex/rgb literals (CLAUDE.md "Rules" + Pitfall 4).

#### Session-changed listener (analog: lines 353-367) — UNTOUCHED

```typescript
$effect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listen<string>("session-changed", (event) => {
        if (canonicalPath && event.payload !== canonicalPath) return;
        reload();
    }).then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
    });
    return () => {
        cancelled = true;
        unlisten?.();
    };
});
```

**Phase 73: do not modify.** Multi-tab End (D-09) reuses this listener as-is — tab A's `end_review_session` emits `session-changed`; tab B's listener calls `reload()`; `reload()` sees `status.state === 'none'`; reads return `no_session`; cold empty state renders. RESEARCH Pattern 2 verifies the recursion is self-stabilizing (resume is gated on `'resume-available'`, not re-triggered after promotion to `'active'`).

#### Timer cleanup on destroy (Pitfall 3 in RESEARCH — analog: the listener-cleanup pattern above)

The Copy timer at line 134 has **no** explicit destroy-cleanup today (the test suite does not exercise unmount-during-revert). For the End button's timer, RESEARCH Pitfall 3 calls out the leak risk explicitly. Use a `$effect` whose return function clears `endTimer`:

```typescript
$effect(() => {
    return () => {
        if (endTimer !== null) clearTimeout(endTimer);
    };
});
```

Mirrors the cancellation-leak protection idiom in the `session-changed` listener.

---

### `src/components/ReviewPanel.test.ts` — modifications

**Analog:** the file itself — specifically the `describe("Copy", …)` block at lines 641-813 and the top-of-file `installReads` dispatcher at lines 100-121.

#### Test mocks (already in place at lines 22-40)

```typescript
vi.mock("../lib/invoke.js", () => ({
    safeInvoke: vi.fn(),
}));

vi.mock("../lib/toast.svelte.js", () => ({
    showToast: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
    listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
    writeText: vi.fn().mockResolvedValue(undefined),
}));
```

**Phase 73 modification:** the multi-tab test (REQ-73-MULTITAB) needs `listen` to **call back synchronously** instead of just resolving. Change the listen mock to capture the callback for later invocation:

```typescript
let sessionChangedHandler: ((event: { payload: string }) => void) | null = null;
vi.mock("@tauri-apps/api/event", () => ({
    listen: vi.fn((_event: string, cb: (event: { payload: string }) => void) => {
        sessionChangedHandler = cb;
        return Promise.resolve(() => { sessionChangedHandler = null; });
    }),
}));
```

Tests then call `sessionChangedHandler?.({ payload: "/repo" })` and `await flushFake()` to simulate cross-tab emission.

#### `installReads` extension (analog: lines 100-121)

```typescript
function installReads(opts: {
    commits?: SessionCommit[];
    comments?: Comment[];
    resolutions?: CommentResolution[];
    generateDoc?: string;
}) {
    vi.mocked(safeInvoke).mockReset();
    vi.mocked(safeInvoke).mockImplementation((cmd: string) => {
        switch (cmd) {
            case "list_session_commits":
                return Promise.resolve(opts.commits ?? []);
            case "list_session_comments":
                return Promise.resolve(opts.comments ?? []);
            case "resolve_session_comments":
                return Promise.resolve(opts.resolutions ?? []);
            case "generate_review_doc":
                return Promise.resolve(opts.generateDoc ?? "# stub\n");
            default:
                return Promise.resolve(undefined);
        }
    });
}
```

**Phase 73 modification (RESEARCH §Wave 0 Gaps):** extend `opts` with `status?: SessionStatus`, `resumeRejection?: unknown`, `endRejection?: unknown`, and add the three new cases (`get_review_session_status`, `resume_review_session`, `end_review_session`). Default `status` to `{ state: "active", file_exists: true, canonical_path: "/repo" }` so existing tests that don't set it continue to render the warm path.

```typescript
function installReads(opts: {
    commits?: SessionCommit[];
    comments?: Comment[];
    resolutions?: CommentResolution[];
    generateDoc?: string;
    status?: SessionStatus;
    resumeRejection?: unknown;
    endRejection?: unknown;
}) {
    const status: SessionStatus = opts.status ?? {
        state: "active", file_exists: true, canonical_path: "/repo",
    };
    vi.mocked(safeInvoke).mockReset();
    vi.mocked(safeInvoke).mockImplementation((cmd: string) => {
        switch (cmd) {
            case "list_session_commits":     return Promise.resolve(opts.commits ?? []);
            case "list_session_comments":    return Promise.resolve(opts.comments ?? []);
            case "resolve_session_comments": return Promise.resolve(opts.resolutions ?? []);
            case "generate_review_doc":      return Promise.resolve(opts.generateDoc ?? "# stub\n");
            case "get_review_session_status": return Promise.resolve(status);
            case "resume_review_session":
                return opts.resumeRejection !== undefined
                    ? Promise.reject(opts.resumeRejection)
                    : Promise.resolve(undefined);
            case "end_review_session":
                return opts.endRejection !== undefined
                    ? Promise.reject(opts.endRejection)
                    : Promise.resolve(undefined);
            default: return Promise.resolve(undefined);
        }
    });
}
```

The default `status: "active"` keeps the existing ~50 tests passing without modification.

#### Describe block scaffold (analog: lines 641-657)

```typescript
describe("Copy", () => {
    // Scope fake timers to THIS describe only. The file-global `flush` helper
    // at the top uses `setTimeout(r, 0)` which deadlocks under fake timers —
    // the tests inside this block use a local `flushFake` instead.
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    // Microtask flush — safe under fake timers (no setTimeout(0)).
    async function flushFake() {
        await Promise.resolve();
        await tick();
    }

    // … tests using vi.advanceTimersByTime(1500), getByRole etc.
});
```

**Phase 73 — three new describe blocks:**
- `describe("cold-boot resume", …)` — REAL timers (no `setTimeout` involved); 5 tests asserting `resume_review_session` is or isn't called depending on `status.state`.
- `describe("End review", …)` — FAKE timers per the Copy template above; 6 tests covering first/second click + 3000ms revert + unmount cleanup.
- `describe("empty states", …)` and `describe("summary line", …)` — REAL timers; DOM assertions.

Critical: do NOT promote `vi.useFakeTimers()` to the file-global `beforeEach`. The file-global `flush()` at lines 123-126 uses `setTimeout(r, 0)` and deadlocks under fake timers — the scoping is mandatory.

#### Test-pattern excerpts (analog: copy tests at lines 683-815)

`installReads`-driven test shape (carry-forward verbatim — the new tests should read the same):

```typescript
it("Copy button is disabled when no comments", async () => {
    installReads({ commits, comments: [], resolutions: [] });
    render(ReviewPanel, {
        props: {
            repoPath: "/repo",
            session: createReviewSession(),
            onJump: vi.fn(),
            onJumpToCommit: vi.fn(),
        },
    });
    await flushFake();

    const copyBtn = getCopyButton();
    expect(copyBtn).toBeDisabled();
});

it("reverts to Copy after 1500ms", async () => {
    renderWithComment();
    await flushFake();
    await fireEvent.click(getCopyButton());
    await flushFake();
    expect(screen.getByRole("button", { name: /Copied/ })).toHaveTextContent(/Copied/);
    vi.advanceTimersByTime(1500);
    await tick();
    expect(screen.getByRole("button", { name: /Cop(y|ied)/ })).toHaveTextContent(/^Copy$/);
});
```

The End-review tests follow the exact same template — replace `Copy/Copied` with `End review/Click again to confirm`, replace `1500` with `3000`, and assert on `safeInvoke` call counts (e.g. `vi.mocked(safeInvoke).mock.calls.filter(c => c[0] === "end_review_session").length`) instead of `writeText` count.

---

## Shared Patterns

### Pattern A: `clearTimeout` before `setTimeout` discipline (Phase 71 carry-forward)

**Source:** `ReviewPanel.svelte:314-319` (Copy handler).
**Apply to:** the End button's `startEndConfirm` helper.
**Why:** Rapid re-clicks must extend the confirm window, not race against the previous revert timer.

```typescript
if (copyTimer !== null) clearTimeout(copyTimer);
copied = true;
copyTimer = setTimeout(() => {
    copied = false;
    copyTimer = null;
}, 1500);
```

### Pattern B: Error narrowing via `errorMessage` helper (Phase 72 carry-forward)

**Source:** `ReviewPanel.svelte:139-156`.
**Apply to:** every new `catch` block in this phase — `resume_review_session` reject, `end_review_session` reject.
**Forbidden:** `(e as TrunkError).message` casts. `DiffPanel.svelte:145` uses this; it is a pre-existing wart, NOT a pattern to extend (RESEARCH §Anti-Patterns to Avoid).

```typescript
showToast(errorMessage(e, "Failed to end review"), "error");
```

### Pattern C: CSS custom properties only — no inline color literals (CLAUDE.md "Rules")

**Source:** `ReviewPanel.svelte` (`var(--color-surface)`, `var(--color-danger)`, `var(--color-text-muted)`, `var(--color-hover)`, `var(--color-border)`).
**Apply to:** every new style declaration in this phase. UI-SPEC § Color enumerates the seven tokens used by Phase 73; all must already exist in `src/app.css` `:root`. No new custom properties added.

```css
/* OK */
color: var(--color-danger);
background: var(--color-danger-bg);

/* NOT OK */
color: #f87171;
background: rgba(248,113,113,0.15);
```

### Pattern D: `session-changed` listener — reuse, don't add new events

**Source:** `ReviewPanel.svelte:353-367`.
**Apply to:** multi-tab End coordination (D-09, REQ-73-MULTITAB).
**Why:** The backend already emits on every mutating thin command including `end_review_session` (`commands/review.rs:1142` per RESEARCH); a new event type is YAGNI.

### Pattern E: `safeInvoke<T>` for every Tauri IPC

**Source:** every IPC call in `ReviewPanel.svelte` (e.g. line 220, 232-236).
**Apply to:** the new `get_review_session_status` (already there for status), `resume_review_session`, `end_review_session` calls.
**Why:** The wrapper parses `TrunkError` from the stringified payload — direct `invoke` returns raw strings on errors and breaks `errorMessage`.

### Pattern F: Fake-timers scoped per describe block

**Source:** `ReviewPanel.test.ts:641-657` (Copy describe).
**Apply to:** the new `describe("End review", …)` block; do NOT extend to the global `beforeEach`.
**Why:** The top-level `flush()` at lines 123-126 uses `setTimeout(r, 0)` and deadlocks under fake timers. Local `flushFake` (`await Promise.resolve(); await tick();`) is the safe replacement inside the fake-timer describe.

### Pattern G: `installReads` dispatcher — extend, don't replace

**Source:** `ReviewPanel.test.ts:100-121`.
**Apply to:** Wave 0 — adding the three new IPC cases (status/resume/end) plus default `status` so existing tests untouched.
**Why:** A command-aware dispatcher is the only safe shape when the panel issues 4+ IPCs via `Promise.all`; sequential mocks become fragile.

---

## No Analog Found

> All Phase 73 work is in-file carry-forward. No file in the codebase lacks a precedent for the pattern it needs. Section empty.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | — |

---

## Notes for the Planner

- **Single chokepoint:** `ReviewPanel.svelte`'s `reload()` is where cold-boot resume + post-End cleanup + multi-tab session-changed converge (RESEARCH §Integration Points). Concentrate plan-task slicing around it; do not spread the resume logic into multiple call sites.
- **No backend changes:** RESEARCH confirms all `_inner` + thin-command primitives (`start/resume/end_review_session_inner`, `get_review_session_status`, `merge_status`) already exist. Treat any plan-task proposing a Rust file edit as scope creep.
- **TDD-eligible boundaries** (per RESEARCH §TDD Eligibility): every behavior with defined IPC I/O — cold-boot resume branching, two-step End rune toggle, IPC invocation gating, empty-state copy gating, summary-line gating, multi-tab reload. CSS class names, Lucide icon choices, and exact pixel paddings are UI/glue (not TDD).
- **Anti-patterns to flag in plan review:** reintroducing `panelMode`, adding states to `_inner`, modal dialog for End, `as TrunkError` casts, hex/rgb color literals, rebinding `Cmd+Shift+R` (RETRACTED REQ-72-1b).

## Metadata

**Analog search scope:** `src/components/ReviewPanel.svelte`, `src/components/ReviewPanel.test.ts`, `src/lib/types.ts`, `src/lib/invoke.ts`, `src/lib/toast.svelte.ts`.
**Files scanned:** 5 source files, 3 phase documents.
**Pattern extraction date:** 2026-05-27.
**Confidence:** HIGH — every cited line number was read in this session; every pattern has a verbatim precedent in the file being modified.
