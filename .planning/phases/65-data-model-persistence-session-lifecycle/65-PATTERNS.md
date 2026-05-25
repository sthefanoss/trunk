# Phase 65: Data Model + Persistence + Session Lifecycle - Pattern Map

**Mapped:** 2026-05-25
**Files analyzed:** 12 (4 new, 8 modified)
**Analogs found:** 11 / 12 (1 net-new primitive with no codebase analog)

All analogs below were verified against live source on 2026-05-25 (line numbers are current).

## File Classification

| New/Modified File | New/Mod | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|---------|------|-----------|----------------|---------------|
| `src-tauri/src/commands/review.rs` | NEW | command (Tauri) | request-response | `src-tauri/src/commands/stash.rs` | exact (role + flow) |
| `src-tauri/src/git/review_store.rs` | NEW | service (persistence) | file-I/O | *(none)* | no analog — net-new primitive |
| `src-tauri/src/git/types.rs` | MODIFY | model (DTO) | transform (serde) | `RefType` / `DiffStatus` in same file | exact |
| `src-tauri/src/state.rs` | MODIFY | store (managed state) | event-driven (in-mem cache) | `CommitCache` line 32 (same file) | exact |
| `src-tauri/src/lib.rs` | MODIFY | config (wiring/menu) | request-response | `find`/`search-toggle` menu pair + `.manage` block (same file) | exact |
| `src-tauri/src/commands/repo.rs` | MODIFY | command (lifecycle hook) | request-response | `close_repo`/`force_close_repo` (same file) | exact |
| `src/lib/types.ts` | MODIFY | model (TS mirror) | transform | `RefType` line 10 (same file) | exact |
| `src/components/ReviewPanel.svelte` | NEW | component (UI stub) | event-driven | `OperationBanner.svelte` + `App.svelte` listener | role-match |
| `src/components/ReviewPanel.test.ts` | NEW | test (component) | — | `OperationBanner.test.ts` + `tauri-mock.ts` | role-match |
| `src-tauri/tests/test_review.rs` | NEW | test (integration) | file-I/O | `test_integ_serde.rs` + `common/context.rs` | role-match |
| `src-tauri/tests/common/context.rs` | MODIFY | test (harness) | — | tempdir wiring lines 19-31 (same file) | exact |
| `src-tauri/tests/test_integ_serde.rs` | MODIFY | test (serde shape) | transform | `graph_result_serializes_with_expected_fields` (same file) | exact |

---

## Pattern Assignments

### `src-tauri/src/commands/review.rs` (NEW — command, request-response)

**Analog:** `src-tauri/src/commands/stash.rs`

**Imports pattern** (stash.rs:1-9):
```rust
use crate::error::TrunkError;
use crate::git::{graph, types::{GraphResult, StashEntry}};
use crate::state::{CommitCache, RepoState};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};
```
For review.rs: import `ReviewSessionsState` from `crate::state`, the new DTOs from `crate::git::types`, the persistence helpers from `crate::git::review_store`. Add `use tauri::Manager;` so `app.path().app_data_dir()` resolves (RESEARCH note at Pattern 2).

**`not_open` precondition guard** (stash.rs:11-19) — SESS-01 requires "the currently open repository," so lifecycle commands must reject a closed repo with the same `not_open` `TrunkError`:
```rust
fn open_repo(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<git2::Repository, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    git2::Repository::open(path_buf).map_err(TrunkError::from)
}
```
Lifecycle commands don't need the `Repository` handle, but they DO need the `RepoState` membership check. Mirror this: look the path up in `RepoState`'s map; if absent, return `not_open`. (The repo's `PathBuf` is also what you canonicalize for the session key — D-11.)

**Thin-command-over-`_inner` core pattern** (stash.rs:50-78 for `_inner`, stash.rs:161-181 for the command). This is the critical testability wedge — `_inner` takes `data_dir: &Path` exactly as stash's `_inner` takes `state_map: &HashMap`:
```rust
// _inner — testable, takes plain args (NO Tauri State). Mirror stash_save_inner.
pub fn start_review_session_inner(
    data_dir: &Path,
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<ReviewSession, TrunkError> {
    // membership check (not_open), canonicalize, save_session (atomic), return session
}

// thin command — clones state, spawn_blocking, JSON-stringify errors. Mirror stash_save.
#[tauri::command]
pub async fn start_review_session(
    path: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = app.path().app_data_dir()
        .map_err(|e| serde_json::to_string(&TrunkError::new("app_data_dir", e.to_string())).unwrap())?;
    let path_clone = path.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        start_review_session_inner(&data_dir, &path_clone, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
    // disk-first ordering (D-10): write done in _inner → THEN update in-mem → THEN emit
    sessions.0.lock().unwrap().insert(/* canonical PathBuf */, result);
    let _ = app.emit("session-changed", /* canonical_path_string */);
    Ok(())
}
```

**Error-wrapping pattern** (stash.rs:155-158, repeated on every command) — copy verbatim. Every `_inner` error becomes `serde_json::to_string(&e)`; spawn failures become a `spawn_error` `TrunkError`.

**Event broadcast** (stash.rs:179) — `let _ = app.emit("repo-changed", path);` → use `app.emit("session-changed", canonical_path_string)` (DP-01).

Four commands to expose: `start_review_session`, `resume_review_session`, `end_review_session`, `get_review_session_status` (RESEARCH Open Question 2).

---

### `src-tauri/src/git/review_store.rs` (NEW — service, file-I/O)

**Analog:** NONE in the codebase. No file currently writes to `app_data_dir` or does atomic disk persistence. This is the one genuinely net-new primitive.

**Planner instruction:** Build from RESEARCH.md "Code Examples" verbatim — they were synthesized for exactly this file:
- Atomic write helper (`atomic_write_json`, RESEARCH lines 291-312): tmp-in-same-dir + `sync_all` + `rename` (D-10, Pitfall 5). Use `serde_json::to_string_pretty`.
- Filename hash (`session_filename`, RESEARCH lines 317-330): inline FNV-1a, NOT `DefaultHasher` (not version-stable). Store canonical path inside the JSON for collision detection.
- Load with recovery (`load_session` + `quarantine_corrupt`, RESEARCH lines 335-373): `serde_json::Value` peek of `schema_version` → `RefusedNewer` (D-16, leave file untouched) before full deserialize → `RecoveredCorrupt` (D-15, rename to `.corrupt`).
- `create_dir_all(sessions_dir)` before first write (Pitfall 2).

**Error type to use:** `crate::error::TrunkError` (error.rs:1-25) — `TrunkError::new("io", e.to_string())` is the established constructor. Note there's no `From<std::io::Error>` impl yet; map manually as the RESEARCH examples do.

**Required functions:** `load_session`, `save_session`, `delete_session` (hard-delete via `fs::remove_file` for SESS-03 / D-13), `session_exists` (for `get_review_session_status` / D-14 resume detection). All take `data_dir: &Path` (or a resolved sessions dir) so tests inject a `tempfile::TempDir`.

---

### `src-tauri/src/git/types.rs` (MODIFY — model/DTO, transform)

**Analog (enum):** `RefType` (types.rs:25-31) — PascalCase variants, NO `#[serde(rename_all)]`:
```rust
#[derive(Debug, Serialize, Clone)]
pub enum RefType {
    LocalBranch,
    RemoteBranch,
    Tag,
    Stash,
}
```
**Critical:** the new `Source { Diff, FullFile }` and `Side { Old, New }` enums follow THIS, serializing as `"Diff"`/`"FullFile"`/`"Old"`/`"New"` (Pitfall 4). The Phase 59 "snake_case" rule applies to struct *fields*, not enum variants.

**Analog (round-trippable DTO — derives both Serialize AND Deserialize):** `DiffStatus` (types.rs:206-217) — session DTOs are READ BACK from disk, so unlike most write-only DTOs here they need `Deserialize` too:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DiffStatus { /* ... */ }
```
Add `Deserialize` to every new struct/enum: `ReviewSession`, `Comment`, `Anchor`, `Source`, `Side`, `DraftComment`. Struct fields stay snake_case (Serialize default). Exact shapes are locked in RESEARCH lines 377-419 (D-01..D-07, DP-02). `Source`/`Side` should derive `PartialEq` (matches `MatchType` at types.rs:77).

---

### `src-tauri/src/state.rs` (MODIFY — store, in-memory cache)

**Analog:** `CommitCache` (state.rs:32) and `RepoState` (state.rs:8), same file:
```rust
pub struct CommitCache(pub Mutex<HashMap<String, crate::git::types::GraphResult>>);
```
New state — **key by `PathBuf` (canonicalized), not `String`** (D-11 divergence):
```rust
pub struct ReviewSessionsState(pub Mutex<HashMap<PathBuf, crate::git::types::ReviewSession>>);
```
`PathBuf` is already imported (state.rs:2). Honor the top-of-file constraint (state.rs:5-7): "Store PathBuf ONLY — git2::Repository is not Sync." `ReviewSession` is owned plain data, so it's fine.

---

### `src-tauri/src/lib.rs` (MODIFY — config/wiring + menu)

Three insertion points, all with same-file analogs:

**1. `.manage()` registration** (lib.rs:64-67):
```rust
.manage(RepoState(Default::default()))
.manage(CommitCache(Default::default()))
.manage(RunningOp(Default::default()))
.manage(WatcherState(Default::default()))
```
Add `.manage(ReviewSessionsState(Default::default()))` and `use state::ReviewSessionsState;` at the top (joining the line-8 import).

**2. `invoke_handler!` registration** (lib.rs:68-141) — add the four `commands::review::*` entries alongside `commands::stash::*` (lib.rs:108-112).

**3. Menu item + on_menu_event** — the `find`/`search-toggle` pair is the EXACT menu→emit→frontend-listener precedent (D-12 temporary trigger):
```rust
// build the item (lib.rs:21-23):
let find = MenuItemBuilder::with_id("find", "Find")
    .accelerator("CmdOrCtrl+F")
    .build(app)?;
// add it to the View submenu (lib.rs:43):
let view_menu = SubmenuBuilder::new(app, "View").fullscreen().build()?;
// handle the event by emitting (lib.rs:56-60):
app.on_menu_event(|app, event| {
    if event.id().as_ref() == "find" {
        let _ = app.emit("search-toggle", ());
    }
});
```
For Phase 65: add a `"review-toggle"` (or similar) `MenuItemBuilder` item, `.item(&review_item)` into the View `SubmenuBuilder`, and an `else if event.id().as_ref() == "review-toggle"` branch emitting an event the frontend stub listens for.

---

### `src-tauri/src/commands/repo.rs` (MODIFY — lifecycle hook, request-response)

**This is an explicit integration point — easy to miss because the repo keying itself stays UNCHANGED (D-11).** Mark it MODIFY.

**Drop in-memory session on close** — `close_repo` (repo.rs:42-52) and `force_close_repo` (repo.rs:54-74) already remove from every managed map:
```rust
pub async fn close_repo(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    watcher_state: State<'_, WatcherState>,
) -> Result<(), String> {
    state.0.lock().unwrap().remove(&path);
    cache.0.lock().unwrap().remove(&path);
    watcher::stop_watcher(&path, &watcher_state);
    Ok(())
}
```
Add a `ReviewSessionsState` param and remove the canonicalized-path entry here (and in `force_close_repo`). **Do NOT delete the file** — close only drops the in-memory entry so resume works on reopen (Anti-Pattern, RESEARCH line 227). Only `end_review_session` hard-deletes (D-13).

**`open_repo` (repo.rs:8-39): do NOT auto-load a session** (D-14 — resume is prompted, not automatic). Opening keys by raw `path` string (repo.rs:34) — unchanged. Resume detection happens via the separate `get_review_session_status` command the stub invokes, not as a side effect of `open_repo`.

---

### `src/lib/types.ts` (MODIFY — model/TS mirror, transform)

**Analog:** `RefType` (types.ts:10) — PascalCase string union mirroring the Rust enum:
```typescript
export type RefType = "LocalBranch" | "RemoteBranch" | "Tag" | "Stash";
```
Add string-for-string mirrors per RESEARCH lines 420-434:
```typescript
export type Source = "Diff" | "FullFile";
export type Side = "Old" | "New";
export interface Anchor { commit_oid: string; file_path: string; source: Source; side: Side; start_line: number; end_line: number; }
export interface Comment { text: string; anchor: Anchor | null; cached_excerpt: string | null; }
export interface DraftComment { text: string; anchor: Anchor | null; }
export interface ReviewSession { schema_version: number; commits: string[]; comments: Comment[]; draft_comment: DraftComment | null; }
```

---

### `src/components/ReviewPanel.svelte` (NEW — component stub, event-driven)

**Analog (component shape):** `src/components/OperationBanner.svelte` — small Svelte 5 runes component that invokes commands and toasts errors.

**Imports + props + state pattern** (OperationBanner.svelte:1-17):
```svelte
<script lang="ts">
import { safeInvoke, type TrunkError } from "../lib/invoke.js";
import { showToast } from "../lib/toast.svelte.js";
import type { OperationInfo } from "../lib/types.js";

interface Props {
	info: OperationInfo;
	repoPath: string;
	onaction?: () => void;
}
let { info, repoPath, onaction }: Props = $props();
let loading = $state(false);
let isMerge = $derived(info.op_type === "Merge");
```
For ReviewPanel: take `repoPath` (and likely a canonical-path) prop, hold session state in `$state`, derive the three D-12 stub states (`session-active` / `no-session` / `resume-available`) with `$derived`.

**Invoke + toast-on-error pattern** (OperationBanner.svelte:30-43) — use for the lifecycle buttons (Start / Resume / End). The corrupt-file recovery toast (D-15) reuses `showToast(msg, "error")` here:
```svelte
async function handleContinue() {
	loading = true;
	try {
		await safeInvoke(cmd, { path: repoPath });
		showToast("...", "success");
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "...failed", "error");
	} finally {
		loading = false;
		onaction?.();
	}
}
```

**`session-changed` listener pattern (DP-01):** mirror the `repo-changed` listener in `App.svelte:519-543` exactly — `$effect` + `listen<string>` + canonical-path payload match + cleanup:
```typescript
import { listen } from "@tauri-apps/api/event";
$effect(() => {
	let unlisten: (() => void) | undefined;
	listen<string>("session-changed", async (event) => {
		if (event.payload !== myCanonicalPath) return;
		// reload session via get_review_session_status
	}).then((fn) => { unlisten = fn; });
	return () => { unlisten?.(); };
});
```
The menu trigger arrives as a separate event (mirroring `CommitGraph.svelte:1295` `listen<void>("search-toggle", ...)`) — wire whichever component owns the View-menu response to toggle the panel.

**Note (D-12):** throwaway stub, replaced by the real panel in Phase 69. Smallest thing that makes SESS-01/02/03 hand-verifiable. Do not over-invest.

---

### `src/components/ReviewPanel.test.ts` (NEW — component test)

**Analog:** `src/components/OperationBanner.test.ts` for render/interaction structure; `src/__tests__/helpers/tauri-mock.ts` + `factories.ts` to mock `invoke` and `@tauri-apps/api/event` `listen`. Cover the three stub states and the `session-changed` reload (RESEARCH Test Map, D-12 row).

---

### `src-tauri/tests/test_review.rs` (NEW — integration test, file-I/O)

**Analog:** `src-tauri/tests/test_integ_serde.rs` (structure) + `src-tauri/tests/common/context.rs` (`TestContext`).

**Test module + context pattern** (test_integ_serde.rs:1-24):
```rust
mod common;
use common::context::TestContext;

#[test]
fn graph_result_serializes_with_expected_fields() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();
    let result = ctx.stash_save("test stash").unwrap();
    let json = serde_json::to_value(&result).expect("...");
    assert!(json["commits"].is_array(), "...");
}
```
For test_review.rs: call the `review.rs` `_inner` functions directly, threading a `tempfile::TempDir` as `data_dir`. Cover SESS-01/02/03, D-10 (no `.tmp` left, valid file after), D-15 (`.corrupt` sidecar), D-16 (`schema_version: 2` refused + file untouched), canonicalization (symlink → same file, crit #3), and Pitfall 2 (first write on non-existent dir succeeds). Full list: RESEARCH "Phase Requirements → Test Map".

---

### `src-tauri/tests/common/context.rs` (MODIFY — test harness)

**Analog:** the file's own tempdir wiring (context.rs:19-31). Add a session `data_dir` helper — a second `tempfile::TempDir` to stand in for `app_data_dir`, exposed via an accessor like the existing `repo_path()` (context.rs:46-49):
```rust
pub fn new_empty() -> Self {
    let dir = tempfile::tempdir().expect("failed to create tempdir");
    // ...existing repo setup...
}
pub fn repo_path(&self) -> &Path { self._dir.path() }
```

---

### `src-tauri/tests/test_integ_serde.rs` (MODIFY — serde shape test)

**Analog:** `graph_result_serializes_with_expected_fields` (test_integ_serde.rs:7-60), shown above. Add a `session_serde_shape` test asserting: `Source`/`Side` serialize PascalCase (`"Diff"`, `"Old"`), struct fields are snake_case, and the forbidden fields are absent (`assert!(json["..."]["hunk_index"].is_null())`). Build a `ReviewSession` literal, `serde_json::to_value`, assert per-field — same idiom as the existing test.

---

## Shared Patterns

### Error type (`TrunkError`)
**Source:** `src-tauri/src/error.rs:1-25`
**Apply to:** `review.rs`, `review_store.rs`, all Rust `_inner` functions
```rust
#[derive(Debug, Serialize)]
pub struct TrunkError { pub code: String, pub message: String }
impl TrunkError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self { /* ... */ }
}
```
Use `TrunkError::new("io", e.to_string())`, `TrunkError::new("not_open", ...)`, `TrunkError::new("app_data_dir", ...)`. There is NO `From<std::io::Error>` — map IO errors manually. Commands stringify via `serde_json::to_string(&e).unwrap()`. Frontend mirror is `TrunkError` in `src/lib/invoke.ts:5`.

### Thin-command → spawn_blocking → `_inner(plain args)` (testability wedge)
**Source:** `src-tauri/src/commands/stash.rs` (every command)
**Apply to:** all four `review.rs` commands
The single most important structural rule (RESEARCH Wave 0): `_inner` takes `data_dir: &Path` so tests bypass Tauri state and inject a `TempDir`. The command resolves `data_dir` from `app.path().app_data_dir()` and runs `_inner` in `tauri::async_runtime::spawn_blocking`. Disk-first mutation ordering: atomic write succeeds → update `ReviewSessionsState` → `app.emit` (RESEARCH Pattern 3).

### Tauri event broadcast for live coordination (DP-01)
**Source:** emit `stash.rs:179`; listen `App.svelte:523` and `CommitGraph.svelte:1295`
**Apply to:** every successful session mutation (Rust) + `ReviewPanel.svelte` (frontend)
```rust
let _ = app.emit("session-changed", canonical_path_string); // mirrors repo-changed
```
```typescript
listen<string>("session-changed", (e) => { if (e.payload === myCanonicalPath) reload(); });
```

### Menu item → emit → frontend listener (D-12 trigger)
**Source:** `lib.rs:21-23` (item), `lib.rs:43` (View submenu), `lib.rs:56-60` (on_menu_event emit), `CommitGraph.svelte:1295` (listener)
**Apply to:** the temporary "Start/End Code Review" View-menu trigger
The `find` → `search-toggle` flow is the exact end-to-end precedent: a `MenuItemBuilder::with_id` item added to a `SubmenuBuilder`, an `on_menu_event` branch that `app.emit`s, and a frontend `listen` that reacts.

### serde DTO conventions
**Source:** `src-tauri/src/git/types.rs` (enums) + `src/lib/types.ts` (mirrors)
**Apply to:** all new session DTOs (Rust + TS)
Enums: PascalCase variants, NO `rename_all`, serialize as PascalCase strings (`RefType`). Structs: snake_case fields (Serialize default). Session DTOs additionally derive `Deserialize` (read back from disk — like `DiffStatus`). TS mirror: snake_case interface fields, PascalCase string unions.

### Test harness (`TestContext` + `TempDir`)
**Source:** `src-tauri/tests/common/context.rs` + `test_integ_serde.rs`
**Apply to:** `test_review.rs`, the serde-shape test
`TestContext::builder()...build()` for repo fixtures; thread a `tempfile::TempDir` as the session `data_dir`; assert serde shape with `serde_json::to_value` + per-field `assert!`.

### Frontend invoke + toast-on-error
**Source:** `src/lib/invoke.ts` (`safeInvoke`), `src/lib/toast.svelte.ts` (`showToast`), used in `OperationBanner.svelte:30-43`
**Apply to:** `ReviewPanel.svelte` lifecycle buttons and the D-15 corrupt-recovery warning
`try { await safeInvoke(cmd, { path }) } catch (e) { showToast((e as TrunkError).message, "error") }`.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src-tauri/src/git/review_store.rs` | service | file-I/O | No existing code writes to `app_data_dir`, does atomic tmp+rename writes, FNV-1a filename hashing, or schema-version-peek recovery. These are net-new primitives — planner uses RESEARCH.md "Code Examples" (lines 291-373) verbatim, not a codebase analog. The `TrunkError` error type and the thin-command caller (`review.rs`) DO have analogs; only the persistence internals are novel. |

---

## Metadata

**Analog search scope:** `src-tauri/src/commands/`, `src-tauri/src/git/`, `src-tauri/src/state.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/error.rs`, `src-tauri/tests/`, `src/components/`, `src/lib/`
**Files scanned:** ~12 source files read against live source (all line numbers verified 2026-05-25)
**Pattern extraction date:** 2026-05-25
