# Phase 75: Message Editor Infrastructure - Pattern Map

**Mapped:** 2026-05-28
**Files analyzed:** 5 (3 new, 1 modified, 1 in-file test block)
**Analogs found:** 5 / 5

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/components/MessageEditor.svelte` | component (modal) | request-response (host-owned promise) | `src/components/InputDialog.svelte` | exact (modal shell, Esc, autofocus, CSS custom props) |
| `src/components/MessageEditor.test.ts` | test (svelte component) | event-driven (DOM events) | `src/components/InputDialog.test.ts` | exact (vitest + @testing-library/svelte + fireEvent + tauri-mock import) |
| `src-tauri/src/git/editor.rs` | utility (process editor helper) | file-I/O + subprocess | `src-tauri/src/commands/interactive_rebase.rs:131-179` | role-match (queue variant — Phase 75 builds single-shot extract) |
| `src-tauri/src/git/mod.rs` | config (module declaration) | n/a | existing `pub mod` lines in same file | exact (one-line addition) |
| `src-tauri/src/git/editor.rs` `#[cfg(test)]` block | test (Rust unit) | file-I/O assertions | `src-tauri/src/git/review_store.rs:173-336` | role-match (tempfile::tempdir + .exists() assertions, no subprocess) |

---

## Pattern Assignments

### `src/components/MessageEditor.svelte` (component, modal)

**Analog:** `src/components/InputDialog.svelte`

**Critical project rule (CLAUDE.md):** "Never inline colors — always use CSS custom properties from the theme." InputDialog already obeys this — mirror its `style="…var(--color-X)…"` exactly. Mirror its grid/flexbox layout — no positioning hacks.

**Imports + autofocus action** (`InputDialog.svelte:1-29, 73-76`):
```svelte
<script lang="ts">
interface Props {
    title: string;
    /* MessageEditor adds: open(default: string) → Promise<string | null> */
}

let { title }: Props = $props();

function autofocus(node: HTMLElement) {
    node.focus();
}
</script>
```
Copy: `autofocus` action verbatim, use on the single textarea. Change: replace `Field[] + onsubmit/oncancel` props with `title: string` only — the open/resolve API is host-owned (see "Host API" below).

**Esc + Enter-vs-newline guard** (`InputDialog.svelte:57-65`):
```ts
function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
        e.preventDefault();
        oncancel();
    } else if (e.key === "Enter" && !(e.target instanceof HTMLTextAreaElement)) {
        e.preventDefault();
        handleSubmit();
    }
}
```
Copy: the `!(e.target instanceof HTMLTextAreaElement)` real-`instanceof` guard (this is the codebase pattern that lets plain Enter insert a newline — satisfies D-05/D-06).
Change for MessageEditor (D-05):
- Always allow plain Enter to insert newline (the target is always a textarea — drop the guard's branch, or keep it as-is for safety).
- Add a `Cmd/Ctrl+Enter` branch that calls Save: `if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) { e.preventDefault(); handleSubmit(); }`.
- Esc handler resolves the host promise with `null` (D-02).

**Backdrop click → cancel** (`InputDialog.svelte:67-71`):
```ts
function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
        oncancel();
    }
}
```
Copy verbatim. Maps to "Cancel = resolve(null)" per D-02/D-04.

**Modal shell + backdrop + CSS custom properties** (`InputDialog.svelte:78-89`):
```svelte
<div
  class="fixed inset-0 flex items-center justify-center"
  style="z-index: 9999; background: var(--color-backdrop);"
  onkeydown={handleKeydown}
  onclick={handleBackdropClick}
>
  <div
    class="rounded-lg shadow-xl"
    style="background: var(--color-surface); border: 1px solid var(--color-border); min-width: 340px; max-width: 480px; padding: 16px;"
  >
    <h3 class="text-sm font-semibold mb-3" style="color: var(--color-text);">{title}</h3>
```
Copy verbatim. The full CSS-custom-property palette to use: `var(--color-backdrop)`, `var(--color-surface)`, `var(--color-border)`, `var(--color-text)`, `var(--color-text-muted)`, `var(--color-bg)`, `var(--color-accent)`, `var(--color-on-accent)`. Do NOT introduce new hex colors. Size knobs (`min-width: 340px`, `max-width: 480px`) are reasonable defaults; bump max-width to ~640px for the textarea variant.

**Textarea styling** (`InputDialog.svelte:97-105`):
```svelte
<textarea
  id="message-editor-textarea"
  class="w-full rounded text-sm"
  style="background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-text); padding: 6px 8px; resize: vertical; min-height: 60px;"
  bind:value={text}
  use:autofocus
></textarea>
```
Copy verbatim. Change: bump `min-height` (e.g. `min-height: 200px`) since this is the only input and holds multi-line `.git/MERGE_MSG` content.

**Cancel + Save buttons** (`InputDialog.svelte:140-156`):
```svelte
<div class="flex justify-end gap-2 mt-4">
  <button
    class="rounded px-3 py-1.5 text-xs font-medium"
    style="background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-text);"
    onclick={oncancel}
  >
    Cancel
  </button>
  <button
    class="rounded px-3 py-1.5 text-xs font-medium"
    style="background: var(--color-accent); color: var(--color-on-accent); opacity: 1;"
    onclick={handleSubmit}
  >
    Save
  </button>
</div>
```
Copy verbatim. Change per D-04: Save button stays clickable even when input is empty/whitespace. The trimmed-empty check lives inside `handleSubmit` → resolves the host promise with `null`. No `disabled` attribute, no `opacity` toggle. (Cancel + empty-Save both produce `null` — single uniform abort signal.)

**Host-owned `open(default) → Promise<string | null>` API (NEW — no exact analog):**

The closest precedent in the codebase is `RebaseEditor` exposed via `onopenrebaseeditor` callback in `RepoView.svelte:567,888` (referenced by CONTEXT.md `<code_context>` Integration Points). Pattern to implement on `MessageEditor.svelte`:

```svelte
<script lang="ts">
interface Props { title: string; }
let { title }: Props = $props();

let isOpen = $state(false);
let text = $state("");
let resolveFn: ((value: string | null) => void) | null = null;

export function open(defaultValue: string): Promise<string | null> {
    text = defaultValue;
    isOpen = true;
    return new Promise((resolve) => { resolveFn = resolve; });
}

function close(result: string | null) {
    isOpen = false;
    resolveFn?.(result);
    resolveFn = null;
}

function handleSubmit() {
    // D-04: empty/whitespace → null (same as Cancel)
    const trimmed = text.trim();
    close(trimmed.length === 0 ? null : text);
}

function oncancel() { close(null); }
</script>

{#if isOpen}
  <!-- modal markup here -->
{/if}
```

Notes:
- `export function open(...)` is how Svelte 5 components expose imperative methods (consumed via `bind:this={ref}` then `ref.open(...)`).
- `resolveFn` lives in module scope of the `<script>` block; it is cleared after each resolve to prevent double-resolution.
- Modal renders only while `isOpen` is true (D-02: "renders only while a call is pending").
- D-04: pass the **untrimmed** original on success — only the empty check is on the trimmed value. Git's `$EDITOR` semantics: trailing whitespace in the buffer is preserved; only the all-whitespace case aborts.

---

### `src/components/MessageEditor.test.ts` (test, svelte component)

**Analog:** `src/components/InputDialog.test.ts`

**Imports + tauri-mock + describe shell** (`InputDialog.test.ts:1-19`):
```ts
import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import MessageEditor from "./MessageEditor.svelte";
import "../__tests__/helpers/tauri-mock";

describe("MessageEditor", () => {
    const defaultProps = {
        title: "Merge commit message",
    };
```
Copy verbatim. The `import "../__tests__/helpers/tauri-mock"` is mandatory boilerplate (sets up `@tauri-apps/api/core` invoke mock — required for any component that touches Tauri APIs directly or transitively).

**Test naming convention** (from `coding_style.md` testing rules + `InputDialog.test.ts`):
- Imperative, lowercase, third-person present tense ("renders title", "calls oncancel on Escape").
- No "should", no method-name echoes.
- Top-level describe = symbol under test (`"MessageEditor"`).

**Esc cancels pattern** (`InputDialog.test.ts:70-78`):
```ts
it("calls oncancel on Escape", async () => {
    const oncancel = vi.fn();
    render(InputDialog, { props: { ...defaultProps, oncancel } });
    const input = screen.getByPlaceholderText("feature/...");
    await fireEvent.keyDown(input, { key: "Escape" });
    expect(oncancel).toHaveBeenCalled();
});
```
Adapt for `MessageEditor`'s host-API shape — InputDialog uses callback props; MessageEditor uses `ref.open(default).then(result => ...)`. Test shape becomes:
```ts
it("resolves null on Escape", async () => {
    let ref!: { open: (s: string) => Promise<string | null> };
    render(MessageEditor, { props: { title: "t", ref: (r) => (ref = r) } });
    // ... or use bind:this / component instance handle per Svelte 5 testing-library idiom
    const promise = ref.open("default text");
    const textarea = screen.getByRole("textbox");
    await fireEvent.keyDown(textarea, { key: "Escape" });
    expect(await promise).toBeNull();
});
```
(Implementation detail of how to capture the ref is up to the planner — `bind:this` via a wrapper test component is the canonical Svelte 5 testing pattern. The InputDialog test shape only covers callback props, so this is the one place to extend rather than copy.)

**Input-then-submit pattern** (`InputDialog.test.ts:50-59, 80-89`):
```ts
it("calls onsubmit with field values", async () => {
    const onsubmit = vi.fn();
    render(InputDialog, { props: { ...defaultProps, onsubmit } });
    const input = screen.getByPlaceholderText("feature/...");
    await fireEvent.input(input, { target: { value: "my-branch" } });
    await fireEvent.click(screen.getByText("OK"));
    expect(onsubmit).toHaveBeenCalledWith({ name: "my-branch" });
});
```
Copy the `fireEvent.input(el, { target: { value: ... } })` pattern for typing into the textarea — this is the project's idiom (no `@testing-library/user-event` in this codebase; grep confirms zero matches). Adapt to assert on the resolved promise value, not a callback.

**Cmd+Enter saves (NEW — no exact analog):**
```ts
it("resolves edited text on Cmd+Enter", async () => {
    /* render + open() */
    const textarea = screen.getByRole("textbox");
    await fireEvent.input(textarea, { target: { value: "new message" } });
    await fireEvent.keyDown(textarea, { key: "Enter", metaKey: true });
    expect(await promise).toBe("new message");
});
```
`metaKey: true` covers macOS Cmd; add a parallel test or single parameterized test for `ctrlKey: true` (cross-platform per D-05).

**D-10 coverage checklist** (per CONTEXT.md):
1. Default pre-fill — `open("hello")` → textarea value is `"hello"`.
2. Edit + Save round-trip — type "world", click Save → promise resolves `"world"`.
3. Esc → null.
4. Cancel button click → null.
5. Empty/whitespace-only input + Save → null (D-04 — the trimmed-empty check).
6. `Cmd+Enter` saves (resolves with current text).
7. `Esc` cancels (covered by #3).

Each test = one behavior. Happy-path tests first; edge cases (null-returning paths) in nested `describe("when input is empty", ...)` or as flat sibling `it`s — match InputDialog.test.ts's flat style.

---

### `src-tauri/src/git/editor.rs` (utility, file-I/O + subprocess)

**Analog:** `src-tauri/src/commands/interactive_rebase.rs:131-179` (the queue variant — extract the single-shot core)

**Critical project rule (CLAUDE.md):** "All git operations go through git2 crate, no shelling out (except GIT_EDITOR for rebase/merge message editing)." This helper IS the sanctioned exception. Per D-08, the rebase queue script stays inline; we extract only the single-shot pattern.

**TrunkError shape** (`src-tauri/src/error.rs:1-26`):
```rust
#[derive(Debug, Serialize)]
pub struct TrunkError {
    pub code: String,
    pub message: String,
}

impl TrunkError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        TrunkError { code: code.into(), message: message.into() }
    }
}
```
Helper returns `Result<EditorHandle, TrunkError>`. Use `TrunkError::new("io_error", e.to_string())` for fs errors — mirrors the existing pattern in `interactive_rebase.rs:128, 134, 152, 173`.

**Single-shot script + chmod pattern to extract** (`interactive_rebase.rs:131-141`, the seq-editor variant — simplest single-shot in the file):
```rust
// 2. Write GIT_SEQUENCE_EDITOR script (script file for reliable $1 handling)
let seq_editor_path = session_dir.join("trunk-seq-editor.sh");
let seq_editor_script = format!("#!/bin/sh\ncp \"{}\" \"$1\"\n", todo_path.display());
std::fs::write(&seq_editor_path, &seq_editor_script)
    .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&seq_editor_path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
}
```
Copy: shell shebang, `cp "$src" "$1"`, `#[cfg(unix)]` + `PermissionsExt` + `0o755` (D-09).
Change for the new helper:
- Write the user's edited message to a temp message file (NOT a queue dir, NOT `ls | sort | head -1`).
- Script becomes `#!/bin/sh\ncp "{msg_file}" "$1"\n` — single-shot, no queue, no indirection (D-07).
- Use `tempfile` crate (already a dep — `Cargo.toml:39`) for both the script path and message file. `tempfile::NamedTempFile` gives RAII cleanup on its own — but per D-07 the `EditorHandle` struct owns explicit `Drop` to clean up BOTH files together on success and error paths.

**`EditorHandle` struct shape (NEW — synthesized from D-07/D-09):**
```rust
pub struct EditorHandle {
    script_path: PathBuf,
    msg_path: PathBuf,
}

impl EditorHandle {
    pub fn script_path(&self) -> &Path { &self.script_path }
}

impl Drop for EditorHandle {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.script_path);
        let _ = std::fs::remove_file(&self.msg_path);
    }
}

pub fn prepare(message: &str) -> Result<EditorHandle, TrunkError> {
    // 1. tempfile::Builder → unique paths under std::env::temp_dir()
    //    OR pass-in session_dir if Phase 76 callers have one (CONTEXT integration: cmd.env("GIT_EDITOR", handle.script_path()))
    // 2. write message bytes to msg_path
    // 3. write shell script `#!/bin/sh\ncp "$msg_path" "$1"\n` to script_path
    // 4. #[cfg(unix)] chmod 0o755 script_path
    // 5. return EditorHandle { script_path, msg_path }
}
```

Notes:
- `let _ = remove_file(...)` swallows errors in `Drop` — Drop cannot return Result. This is the standard Rust pattern for best-effort cleanup; matches the spirit of the comment at `interactive_rebase.rs:144` (`let _ = std::fs::create_dir_all(...)`).
- D-07: cleanup happens on BOTH success and error paths. The `Drop` impl guarantees this even on early `?` returns from `prepare` — but only AFTER `EditorHandle` has been successfully constructed. So `prepare`'s body must also clean up partial state if it fails mid-way (e.g. script write succeeds but chmod fails → delete the script before returning Err). Simplest: write msg first, then script, then chmod; if any step after the first write fails, manually `remove_file` the prior step before propagating.
- `script_path()` accessor returns `&Path` so callers can do `cmd.env("GIT_EDITOR", handle.script_path())` per the CONTEXT integration sketch.

---

### `src-tauri/src/git/mod.rs` (config, module declaration)

**Analog:** existing one-line `pub mod` declarations in the same file (`src-tauri/src/git/mod.rs:1-7`):
```rust
pub mod graph;
pub mod repository;
pub mod review;
pub mod review_store;
pub mod syntax;
pub mod types;
```
Add: `pub mod editor;` — alphabetical position between `pub mod repository;` and `pub mod review;` (matches existing alphabetical convention). Single-line edit.

---

### `src-tauri/src/git/editor.rs` `#[cfg(test)]` block (test, Rust unit)

**Analog:** `src-tauri/src/git/review_store.rs:173-336` (closest because it: uses `tempfile::tempdir()`, asserts on filesystem state via `.exists()` / `.is_file()`, runs zero subprocesses, lives in the same `src-tauri/src/git/` module as the new file)

**Test-block scaffolding** (`review_store.rs:173-176, 208-221`):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // (+ tempfile, fs, Path imports as needed)

    #[test]
    fn v1_session_loads_and_backfills_ids_without_corrupting() {
        let dir = tempfile::tempdir().unwrap();
        // arrange: seed files into dir.path()
        // act
        // assert on dir.path() filesystem state
    }
}
```

**Filesystem assertions** (`review_store.rs:304-316, 331-334`):
```rust
let after = fs::read(session_path(dir.path(), canonical)).unwrap();
assert_eq!(before, after, "the newer file must be left byte-unchanged (D-16)");
assert!(
    !corrupt_sidecar_path(dir.path(), canonical).exists(),
    "a refused file must NOT be quarantined",
);
assert!(
    corrupt_sidecar_path(dir.path(), canonical).exists(),
    "a .corrupt sidecar must be created (D-15)",
);
```
Copy: `.exists()` on PathBuf for presence/absence assertions, `fs::read(path).unwrap()` to assert on file contents byte-equality, assertion messages that name the decision id (D-XX) being verified.

**D-11 coverage checklist (4 tests):**
1. `script_file_created_with_executable_perms`
   - Call `prepare("hello")`.
   - Assert `handle.script_path().exists()`.
   - `#[cfg(unix)]` block: read `fs::metadata(handle.script_path()).unwrap().permissions().mode() & 0o777` and assert it equals `0o755`.
2. `temp_message_file_written_with_payload`
   - Call `prepare("hello world")`.
   - Read the msg file via internal accessor (or expose `pub(crate) fn msg_path()` for tests — pattern: tests live in the same module so private fields are visible via `super::*`).
   - Assert content bytes == `b"hello world"`.
3. `drop_removes_both_files_on_happy_path`
   - Call `prepare("x")`, capture both paths via accessors, drop the handle (`drop(handle);` or let it leave scope).
   - Assert `!script_path.exists() && !msg_path.exists()`.
4. `drop_on_explicit_drop_removes_both_files_error_path`
   - Call `prepare("x")`, simulate an early-return / explicit `drop()` mid-flight (no subprocess invocation per D-12).
   - Assert same as test 3.

D-11 is explicit: **no subprocess invocation**. Tests assert purely on filesystem state. Tests 3 + 4 cover the same `Drop` impl from two angles (happy + early-return) — both are kept because they document the contract independently, satisfying GOOS "one behavior per test."

Test naming convention (from `coding_style.md` + `review_store.rs`): snake_case, imperative, third-person — `script_file_created_with_executable_perms`, not `test_script_file` or `should_create_script_file`.

---

## Shared Patterns

### CSS custom properties only (frontend)
**Source:** `src/components/InputDialog.svelte` (every `style=` attribute)
**Apply to:** `MessageEditor.svelte` — every color/background/border value goes through `var(--color-X)`.
**Project rule (CLAUDE.md):** "Never inline colors — always use CSS custom properties from the theme."
**Palette in use:** `--color-backdrop`, `--color-surface`, `--color-bg`, `--color-border`, `--color-text`, `--color-text-muted`, `--color-accent`, `--color-on-accent`.

### `tauri-mock` import in component tests
**Source:** `src/components/InputDialog.test.ts:4` → `import "../__tests__/helpers/tauri-mock";`
**Apply to:** `MessageEditor.test.ts` — required even if the component itself doesn't call `invoke`, because the import order matters for transitive imports.

### `fireEvent` (not `userEvent`) for typing
**Source:** Every `*.test.ts` in `src/components/` (zero matches for `userEvent` across the suite).
**Apply to:** `MessageEditor.test.ts` — use `fireEvent.input(el, { target: { value: "..." } })` and `fireEvent.keyDown(el, { key: "...", metaKey: true })`. Do NOT introduce `@testing-library/user-event`.

### `TrunkError::new("io_error", e.to_string())` for fs failures
**Source:** `src-tauri/src/commands/interactive_rebase.rs:128, 134, 140, 152, 173, 178`
**Apply to:** `editor.rs` — every `?` on a `std::fs::*` call uses `.map_err(|e| TrunkError::new("io_error", e.to_string()))`.

### `#[cfg(unix)] + PermissionsExt + 0o755` for shell scripts
**Source:** `src-tauri/src/commands/interactive_rebase.rs:136-141, 174-179`
**Apply to:** `editor.rs::prepare()` — gate the chmod block with `#[cfg(unix)]`; the Tauri rebase path is already Unix-only and the helper follows the same gating (per CONTEXT `<code_context>` Established Patterns).

### `tempfile::tempdir()` + `.exists()` assertions in Rust tests
**Source:** `src-tauri/src/git/review_store.rs:210, 314, 332`
**Apply to:** `editor.rs::tests` — assert on filesystem state via `path.exists()` and `fs::read(path)`, no subprocess (D-11 + D-12).

---

## No Analog Found

| File / Concern | Reason |
|---|---|
| Host-owned `open(default) → Promise<string \| null>` API on `MessageEditor.svelte` | No existing component in the codebase exposes an `export function open(...)`-style imperative Promise API. Closest precedent is `RebaseEditor` opened via callback props (`onopenrebaseeditor` in `RepoView.svelte:567,888`), but that pattern uses host-supplied state, not a Promise return. Planner should follow the synthesized pattern in the `MessageEditor.svelte` section above. |
| `Cmd+Enter` keyboard handler | InputDialog handles plain Enter only (text-input variant); no existing component combines `metaKey/ctrlKey + Enter` save. Synthesized pattern documented above. |

---

## Metadata

**Analog search scope:** `src/components/`, `src/__tests__/helpers/`, `src-tauri/src/commands/`, `src-tauri/src/git/`, `src-tauri/src/error.rs`, `src-tauri/Cargo.toml`.
**Files scanned:** ~30 (component sources + tests + Rust git/commands modules).
**Pattern extraction date:** 2026-05-28.
