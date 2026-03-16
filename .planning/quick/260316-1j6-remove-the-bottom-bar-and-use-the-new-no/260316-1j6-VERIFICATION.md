---
id: 260316-1j6
status: passed
verified: 2026-03-16
---

# Verification: Remove the bottom bar and use the new notification system for state updates

## Must-have checks

### 1. StatusBar.svelte is deleted
- **PASS** — `src/components/StatusBar.svelte` does not exist on disk (glob returns no results).

### 2. No dangling references to StatusBar in src/
- **PASS** — Only one reference remains: a comment in `Toolbar.svelte:18` (`// relocated from StatusBar`). No imports, no component usage, no functional references.

### 3. App.svelte no longer imports or uses StatusBar
- **PASS** — `App.svelte` has no `import StatusBar` line and no `<StatusBar .../>` usage. The layout is `<div class="flex flex-col h-screen">` with Toolbar, main content, and `<Toast />` — no bottom bar.

### 4. Toolbar.svelte has the remote-progress listener
- **PASS** — `Toolbar.svelte:18-30` contains a `$effect` that calls `listen<{ path: string; line: string }>('remote-progress', ...)` and updates `remoteState.progressLine`.

### 5. PullDropdown.svelte has toast notifications for success/error
- **PASS** — `PullDropdown.svelte` imports `showToast` (line 5) and calls it for success (line 61) and error (line 66) in its `runRemote` function.

### 6. remoteState still works (isRunning used for button disabling)
- **PASS** — `remote-state.svelte.ts` exports `remoteState` with `isRunning`, `progressLine`, and `error` fields. `isRunning` is set/unset in both `Toolbar.svelte` (lines 106, 111, 115) and `PullDropdown.svelte` (lines 54, 59, 63). Toolbar buttons use `disabled={remoteState.isRunning}` (lines 228, 231, 233).

### 7. npm run check passes
- **PASS** — `npm run check` completes successfully. All reported errors/warnings are pre-existing in the `virtual-list/` utility JS files (untyped parameters, missing JSDoc types) and unrelated a11y warnings. No errors reference StatusBar or any file touched by this task.

## Summary

All 7 must-haves verified against the codebase. The task is complete.
