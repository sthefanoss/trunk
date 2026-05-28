---
phase: 75-message-editor-infrastructure
reviewed: 2026-05-28T22:50:00Z
depth: deep
files_reviewed: 5
files_reviewed_list:
  - src/components/MessageEditor.svelte
  - src/components/MessageEditor.test.ts
  - src-tauri/src/git/editor.rs
  - src-tauri/src/git/mod.rs
  - src-tauri/Cargo.toml
findings:
  critical: 1
  warning: 4
  info: 3
  total: 8
status: resolved
resolved:
  - CR-01 (3f6b2ee)
  - WR-01 (4d4a148)
  - WR-02 (97148e7)
  - WR-03 (97148e7)
  - WR-04 (3f6b2ee)
  - IN-01 (4d4a148)
deferred:
  - IN-02 (UI affordance for empty/whitespace Save — Phase 76 caller-level concern)
  - IN-03 (Windows-side editor shell — out of phase-75 scope)
---

# Phase 75: Code Review Report

**Reviewed:** 2026-05-28T22:50:00Z
**Depth:** deep
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Phase 75 lays infrastructure for Phase 76's commit-message-editing UX: a Svelte 5 modal component with an imperative `open()` API resolving `Promise<string|null>`, and a Rust temp-editor helper that materialises a `GIT_EDITOR` shell script + pre-filled message file pair with cleanup on `Drop`. Backend tests (8 of 8) and frontend tests (13 of 13) all pass.

The deliverable is largely well-shaped, but there is one BLOCKER in the Svelte component that the test suite did not catch: concurrent `open()` calls silently overwrite the pending resolver, so the first promise hangs forever. For Phase 76's intended use — a Rust handler awaits the modal while holding a child `git` process via `GIT_EDITOR` — a hung promise means a stuck child process and a stuck repo. The contract of an exported `open()` function permits this call sequence; the code must defend against it (or assert it loudly).

The Rust helper has two minor robustness gaps: a documentation overclaim about TOCTOU, and a `$TMPDIR`-driven shell-injection surface inherited from `interactive_rebase.rs`. Both are WARNINGs, not BLOCKERs.

The modal is also missing standard dialog semantics (focus trap, `role="dialog"`, escape-to-close-without-textarea-focus). Per `engineering_judgment.md §2` ("question the premise"), the right fix is the native `<dialog>` element, which provides all of these for free.

## Critical Issues

### CR-01: Concurrent `open()` calls silently drop the first resolver — first promise hangs forever

**File:** `src/components/MessageEditor.svelte:10-18`
**Issue:** `resolveFn` is a single mutable slot. If `open()` is called while the modal is already open (e.g. caller mis-fires, user double-triggers a menu item, or two distinct backend code paths both await the same component instance), the second call overwrites `resolveFn` without invoking the previous one. The first `Promise<string|null>` then never resolves and never rejects — it leaks.

This is acutely dangerous for the Phase 76 use case: the Rust side will be `await`ing this promise from inside a `GIT_EDITOR`-driven handler that holds a live `git` child process. A hung promise → stuck child → stuck rebase/merge/revert state, with no UI affordance to recover.

The test suite has zero coverage for the concurrent-open case (see CR-01 companion: every test calls `open()` exactly once per mounted component), so the bug is invisible to CI.

**Fix:** Either reject the first promise when a second `open()` arrives, or refuse the second call. Rejecting the first is the contract that surprises the least:

```svelte
export function open(defaultValue: string): Promise<string | null> {
    // Cancel any in-flight invocation deterministically — a stale resolver
    // outliving its modal cycle leaves the caller awaiting forever.
    resolveFn?.(null);

    text = defaultValue;
    isOpen = true;
    return new Promise((resolve) => {
        resolveFn = resolve;
    });
}
```

Add a test that covers this:

```ts
it("resolves the previous promise with null when open() is called twice", async () => {
    const { ref } = mount();
    const first = ref.open("a");
    const second = ref.open("b");
    expect(await first).toBeNull();
    // second still pending — drive it to completion to avoid unhandled-rejection noise
    await screen.findByRole("textbox");
    await fireEvent.click(screen.getByText("Cancel"));
    expect(await second).toBeNull();
});
```

## Warnings

### WR-01: `MessageEditor` is missing dialog semantics — no `role="dialog"`, no focus trap, no Escape-without-textarea-focus

**File:** `src/components/MessageEditor.svelte:56-98`
**Issue:** The modal is a `<div>` with `data-testid="message-editor-backdrop"`, `onkeydown`, and `onclick` handlers. Concrete defects that fall out of this:

1. No `role="dialog"`, no `aria-modal="true"`, no `aria-labelledby` pointing at the `<h3>` title. Screen readers cannot announce this as a modal.
2. No focus trap. `Tab` from the textarea moves focus to background DOM. Once focus is outside the backdrop, `keydown` does not bubble back to the handler, so **Escape stops closing the modal**.
3. The backdrop `<div>` has no `tabindex`, so it is not itself focusable — `keydown` only fires when a descendant (textarea, buttons) holds focus.
4. `svelte-ignore a11y_no_static_element_interactions` was added to silence the linter rather than to address the design.

This is one root cause (using `<div>` instead of a native dialog primitive) producing four symptoms. Per `engineering_judgment.md §2` ("question the premise"), the structural fix is the native `<dialog>` element, which provides modal semantics, focus trap, ESC-to-close, and backdrop styling out of the box.

**Fix:** Switch to `<dialog>` with `showModal()` / `close()`, e.g.:

```svelte
<script lang="ts">
  let dialogEl: HTMLDialogElement;
  // open() calls dialogEl.showModal(); close() calls dialogEl.close();
</script>

<dialog
  bind:this={dialogEl}
  onclose={() => close(null)}
  onclick={handleBackdropClick}
  aria-labelledby="message-editor-title"
>
  <h3 id="message-editor-title">{title}</h3>
  ...
</dialog>
```

If `<dialog>` is rejected for styling reasons, at minimum add `role="dialog"`, `aria-modal="true"`, `aria-labelledby`, a focus trap (Tab/Shift+Tab cycle inside the modal), and attach the Escape `keydown` handler at `document` level rather than the backdrop.

### WR-02: `editor.rs` doc-comment overclaims TOCTOU defence — `O_EXCL` semantics are lost on the `.keep()` + re-open path

**File:** `src-tauri/src/git/editor.rs:73-79`
**Issue:** The comment says:

> Reserve both temp paths up-front via `tempfile::Builder` — non-predictable paths under `std::env::temp_dir()` with O_EXCL semantics (T-75-T01 TOCTOU defence).

`tempfile::Builder::tempfile()` does open with `O_EXCL`, but `.keep()` immediately closes the handle and hands back a `PathBuf`. `prepare()` then re-opens that path through `std::fs::write(...)`, which uses `O_TRUNC | O_WRONLY` — neither `O_EXCL` nor `O_NOFOLLOW`. The path exists on disk during this window. A symlink swap by another process in the same temp dir would cause `std::fs::write` to clobber the target.

In practice the path's random suffix makes this near-unreachable, and `/tmp` typically has the sticky bit set, but the documented claim is wrong and the threat-model row T-75-T01 should not be treated as fully mitigated.

**Fix:** Update the comment to describe the actual guarantee (random path under `temp_dir()`, file mode 0o600 from tempfile creation is preserved across the truncating re-open) and the residual risk (re-open does not use `O_NOFOLLOW`). If T-75-T01 must be hard-mitigated, write to the path through the file handle returned by `tempfile()` directly instead of `.keep()`-ing and re-opening — `Builder::tempfile()` returns `(File, NamedTempFile)`; capture the `File` and write through it before calling `.keep()`.

### WR-03: `$TMPDIR`-controlled shell-script interpolation — quoted but not escaped

**File:** `src-tauri/src/git/editor.rs:123`
**Issue:**

```rust
let script_body = format!("#!/bin/sh\ncp \"{}\" \"$1\"\n", msg_path.display());
```

`msg_path` is rooted at `std::env::temp_dir()`, which on Unix is derived from `$TMPDIR`. If the user's environment sets `TMPDIR` to a path containing a `"` character, the generated script is malformed at best and shell-injectable at worst (e.g. `TMPDIR=/tmp/x"; rm -rf $HOME; #`). The threat is self-injection on a single-user machine, so the practical risk is low, but the code asserts `T-75-T04: cp arguments are ALWAYS quoted` — that is true for the literal `"` characters but does not prevent the quoted segment from being terminated by an embedded `"` in `msg_path`.

The same pattern lives in `src-tauri/src/commands/interactive_rebase.rs:133` — this is a pre-existing class of issue, not a phase-75 regression. Per `ownership.md`, flagging it here and recommending a single fix that lands in both places.

**Fix:** Either reject paths containing `"` early with a descriptive error, or shell-quote properly:

```rust
fn shell_single_quote(s: &str) -> String {
    // POSIX-safe: single-quote, replacing embedded ' with '\''
    format!("'{}'", s.replace('\'', "'\\''"))
}

let script_body = format!(
    "#!/bin/sh\ncp {} \"$1\"\n",
    shell_single_quote(&msg_path.display().to_string()),
);
```

### WR-04: Test suite has no coverage for the concurrent-`open()` contract

**File:** `src/components/MessageEditor.test.ts`
**Issue:** Companion to CR-01. The describe block exercises every cancellation path (Escape, Cancel, backdrop, empty Save, whitespace Save) but never calls `open()` twice on the same mounted instance. The behavioural contract of `open()` — "what happens when I'm already open?" — is left undefined by the tests, which is how CR-01 shipped.

**Fix:** Add the test sketched in CR-01's fix block. If the chosen resolution for CR-01 is "second `open()` is a no-op and returns a rejected promise" rather than "first resolves null", encode whichever contract is decided, but encode something.

## Info

### IN-01: Modal `z-index: 9999` is a hardcoded magic number — should be a CSS variable

**File:** `src/components/MessageEditor.svelte:60`
**Issue:** `style="z-index: 9999; ..."` inline. `CLAUDE.md` mandates "never inline colors — always use CSS custom properties from the theme"; the spirit of the rule applies to stacking-context magic numbers too. A second overlay (toast, tooltip, dropdown) added later cannot reason about its layering relative to this modal without grepping for the literal.

**Fix:** Lift to a `--z-modal` theme variable and consume it: `style="z-index: var(--z-modal); ..."`.

### IN-02: `Cmd+Enter` / `Save` on empty or whitespace text silently resolves `null` with no UI affordance

**File:** `src/components/MessageEditor.svelte:27-29`
**Issue:** `handleSubmit()` resolves `null` (i.e. cancels) when `text.trim().length === 0`. The tests confirm this is intentional, but from the user's perspective Save / Cmd+Enter on an empty draft is indistinguishable from Cancel — no error, no shake, no disabled Save button. For commit-message editing in Phase 76 a user could believe they saved when they actually cancelled.

**Fix:** Disable the Save button when `text.trim().length === 0`, or surface a transient inline error instead of silently cancelling. Caller contract stays the same; UI no longer hides the cancellation.

### IN-03: `#[cfg(unix)]`-gated chmod has no Windows alternative — script will not execute as `GIT_EDITOR` on Windows

**File:** `src-tauri/src/git/editor.rs:128-133`
**Issue:** The chmod block is `#[cfg(unix)]`. On Windows, the generated `.sh` file has no executable bit (Windows uses extension-based execution) and `cp` is not a built-in. Any Phase 76 caller invoking this on Windows will get a non-executable script and a missing `cp`.

This mirrors the existing pattern at `interactive_rebase.rs:136-141`, so it is consistent with the rest of the codebase, not a phase-75 regression. Recording it here so the Windows story is owned somewhere rather than discovered at Phase 76 ship time.

**Fix:** Out of phase-75 scope, but file a follow-up: decide whether Trunk supports Windows for editor-driven git operations, and if so, generate a `.cmd` / `.bat` wrapper (or invoke a bundled `sh.exe`) on `#[cfg(windows)]`.

---

_Reviewed: 2026-05-28T22:50:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
