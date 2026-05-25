---
phase: 65-data-model-persistence-session-lifecycle
verified: 2026-05-25T00:00:00Z
status: human_needed
score: 4/4 must-haves verified
overrides_applied: 0
human_verification:
  - test: "View menu > Start/End Code Review toggles the ReviewPanel; clicking Start creates a session; force-quitting (kill -9) and reopening the same repo shows the resume-available state; Resume loads the same session; End leaves no session on next open."
    expected: "The three D-12 states are visible in order: none → active → (after force-quit + reopen) resume-available → active → none."
    why_human: "Full end-to-end lifecycle requires a running Tauri app; atomic persistence across a force-quit cannot be tested with grep or unit tests alone."
---

# Phase 65: Data Model + Persistence + Session Lifecycle Verification Report

**Phase Goal:** A code review session exists as a durable per-repo document the user can start, resume across restarts, and clear.
**Verified:** 2026-05-25
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can start a code review session for the currently open repository. | VERIFIED | `start_review_session_inner` in `review.rs:63` checks `state_map` for the repo (returns `not_open` if absent) and rejects if a session already exists (`session_exists` error). Thin command `start_review_session` is registered in `lib.rs:124`. Tests `start_creates_session`, `start_rejects_closed_repo`, `start_rejects_when_session_exists` all pass. |
| 2 | After force-quitting mid-session and reopening the same repo, the panel shows the same session with commits/comments intact. | VERIFIED | `review_store::save_session` uses atomic tmp+rename write (`atomic_write_json`, `review_store.rs:66-82`). `resume_review_session_inner` calls `load_session` from disk with no reliance on in-memory state. `resume_after_restart` test explicitly creates a fresh empty in-memory state and loads from disk successfully. HUMAN verification required for the actual force-quit scenario. |
| 3 | The same repo opened via a symlink or a different path string resumes the SAME session. | VERIFIED | `canonical_repo_path` in `review.rs:50-58` calls `std::fs::canonicalize`. `symlink_resumes_same_session` test (`test_review.rs:196-223`) creates a symlink, starts via real path, resumes via symlink path, asserts both `alias_canonical == real_canonical` and `Loaded` outcome. |
| 4 | User can end and clear the active session; restarting shows no session. | VERIFIED | `end_review_session_inner` calls `review_store::delete_session` (hard-delete, `review_store.rs:139-145`). NotFound treated as idempotent. `end_clears_session` test asserts `file_exists == false` and `state == SessionState::None` after end. `repo.rs` close-repo hook drops only in-memory entry, never calls `delete_session` (confirmed by grep: 0 matches). |

**Score:** 4/4 truths verified

---

### Keystone Schema Verification (D-01..D-07)

| Check | Status | Evidence |
|-------|--------|----------|
| `Source` enum PascalCase variants (`Diff`, `FullFile`), no `rename_all` | VERIFIED | `types.rs:295-299`; `session_serde_shape` asserts `serde_json::to_value(Source::Diff) == "Diff"`. No `rename_all` attribute on either enum (grep confirms). |
| `Side` enum PascalCase variants (`Old`, `New`), no `rename_all` | VERIFIED | `types.rs:301-305`; `session_serde_shape` asserts `Side::Old == "Old"`, `Side::New == "New"`. |
| `Anchor` has NO `hunk_index`, `line_index`, `context_lines`, `ignore_whitespace` fields | VERIFIED | `types.rs:307-315`; `session_serde_shape` asserts all four `.is_null()` (D-01 migration guardrail). The `context_lines`/`ignore_whitespace` in `types.rs:165-168` belong to `DiffRequestOptions`, a completely separate struct. |
| `Anchor` carries source coordinates only: `commit_oid`, `file_path`, `source`, `side`, `start_line`, `end_line` | VERIFIED | `types.rs:308-315` exactly matches the locked schema. |
| `Comment` has `text`, `anchor: Option<Anchor>`, `cached_excerpt: Option<String>` (D-04) | VERIFIED | `types.rs:317-322`. |
| `DraftComment` has `text`, `anchor: Option<Anchor>` (DP-02) | VERIFIED | `types.rs:324-328`. |
| `ReviewSession` has `schema_version: u32`, `commits: Vec<String>`, `comments: Vec<Comment>`, `draft_comment: Option<DraftComment>` (D-03, D-06) | VERIFIED | `types.rs:330-336`. |
| No `timestamp`, `author`, `severity`, `status` fields on any review type (D-07) | VERIFIED | `session_serde_shape` asserts all four `.is_null()` on the comment. None appear in the struct definitions. |
| TS mirror string-for-string identical to Rust on-wire shape | VERIFIED | `types.ts:288-327`: `Source = "Diff" \| "FullFile"`, `Side = "Old" \| "New"`, `Anchor` with all six snake_case fields, `Comment`, `DraftComment`, `ReviewSession` match Rust exactly. `SessionState = "active" \| "resume-available" \| "none"`, `SessionStatus` with `state`, `file_exists`, `canonical_path`. |

---

### Required Artifacts

| Artifact | Status | Details |
|----------|--------|---------|
| `src-tauri/src/git/types.rs` | VERIFIED | Contains `pub struct ReviewSession`, `pub struct Comment`, `pub struct Anchor`, `pub struct DraftComment`, `pub enum Source`, `pub enum Side`. All derive `Serialize, Deserialize, Clone`. |
| `src/lib/types.ts` | VERIFIED | Contains `export interface ReviewSession`, `export type Source = "Diff" \| "FullFile"`, `export type Side = "Old" \| "New"`, `export type SessionState = "active" \| "resume-available" \| "none"`, `export interface SessionStatus`. |
| `src-tauri/tests/test_integ_serde.rs` | VERIFIED | Contains `session_serde_shape` and `session_round_trips` tests with PascalCase enum assertions, forbidden-field assertions, and round-trip check. Also `session_serializes_draft_comment_when_present`. |
| `src-tauri/src/git/review_store.rs` | VERIFIED | Contains `pub fn save_session`, `pub fn load_session`, `pub fn delete_session`, `pub fn session_exists`, private `fn session_filename` (FNV-1a, no DefaultHasher), `pub enum LoadOutcome` with all four variants, `#[cfg(test)] mod tests` with `same_canonical_path_same_file`. |
| `src-tauri/tests/test_review.rs` | VERIFIED | Contains `session_round_trips`, `first_write_creates_dir`, `atomic_write_clean`, `corrupt_quarantined`, `newer_version_refused` (persistence tests) plus `start_creates_session`, `start_rejects_closed_repo`, `start_rejects_when_session_exists`, `resume_after_restart`, `symlink_resumes_same_session` (#[cfg(unix)]), `end_clears_session`, `status_inner_never_reports_active`. |
| `src-tauri/tests/common/context.rs` | VERIFIED | `TestContext` holds `_data_dir: tempfile::TempDir` as second owned TempDir; exposes `pub fn data_dir(&self) -> &Path`. |
| `src-tauri/src/state.rs` | VERIFIED | `pub struct ReviewSessionsState(pub Mutex<HashMap<PathBuf, crate::git::types::ReviewSession>>)` keyed by `PathBuf` (not String). |
| `src-tauri/src/commands/review.rs` | VERIFIED | Four `#[tauri::command]` functions, four `_inner` functions taking `data_dir: &Path`. `SessionState` and `SessionStatus` defined here. `merge_status` with in-module tests. |
| `src-tauri/src/commands/mod.rs` | VERIFIED | Contains `pub mod review;`. |
| `src-tauri/src/git/mod.rs` | VERIFIED | Contains `pub mod review_store;`. |
| `src-tauri/src/lib.rs` | VERIFIED | `.manage(ReviewSessionsState(Default::default()))`, four commands in `invoke_handler`, `MenuItemBuilder::with_id("review-toggle", ...)`, `on_menu_event` branch emitting `"review-toggle"`. |
| `src-tauri/src/commands/repo.rs` | VERIFIED | `close_repo` and `force_close_repo` both accept `sessions: State<'_, ReviewSessionsState>` and call `drop_in_memory_session`. No `delete_session` call. |
| `src/components/ReviewPanel.svelte` | VERIFIED | Three D-12 states rendered. `safeInvoke` calls for all four lifecycle commands. `listen<string>("session-changed", ...)` inside `$effect` with cleanup. No inline hex colors. |
| `src/components/ReviewPanel.test.ts` | VERIFIED | Tests for all three states, `start_review_session` invocation with `{ path }`, `session-changed` reload trigger. |
| `src/App.svelte` | VERIFIED | `listen<void>("review-toggle", ...)` toggles `reviewPanelOpen`; `<ReviewPanel repoPath={tab.repoPath} />` rendered conditionally. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `commands/review.rs` | `git/review_store.rs` | `review_store::save_session`, `load_session`, `delete_session`, `session_exists` | VERIFIED | All four calls present in `_inner` functions. `use crate::git::review_store::{self, LoadOutcome}` at top of file. |
| `get_review_session_status` (thin command) | `ReviewSessionsState` in-memory map | `sessions.0.lock().unwrap().contains_key(&canonical)` | VERIFIED | `review.rs:271`. `merge_status` called with disk+memory halves. `SessionState::Active` produced only here. |
| `commands/review.rs` | frontend | `app.emit("session-changed", canonical_path_string)` | VERIFIED | Three emit calls in `start_review_session`, `resume_review_session`, `end_review_session` (lines 171, 226, 248). |
| `commands/repo.rs` | `ReviewSessionsState` | `drop_in_memory_session` removes canonical entry on close | VERIFIED | `repo.rs:11-15`. Called from both `close_repo` and `force_close_repo`. No `delete_session` call (grep: 0 matches). |
| `ReviewPanel.svelte` | lifecycle commands | `safeInvoke("start_review_session" / "resume_review_session" / "end_review_session" / "get_review_session_status")` | VERIFIED | All four invocations present in `ReviewPanel.svelte`. |
| `App.svelte` | `review-toggle` event | `listen<void>("review-toggle", ...)` toggles panel | VERIFIED | `App.svelte:556-557`. Panel rendered at line 593. |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `ReviewPanel.svelte` | `status` (`SessionStatus`) | `safeInvoke("get_review_session_status")` → Rust thin command → `_inner` → `review_store::session_exists` + disk read | Yes — reads real per-repo JSON file from `app_data_dir/sessions/<hash>.json` | FLOWING |

---

### Behavioral Spot-Checks

Step 7b: SKIPPED for review_store and command-layer tests because they require a running Tauri app instance for the thin commands; the `_inner` functions and persistence functions are covered by the Rust test suite (`just check` passes fully with 461 vitest + Rust tests).

Key behavioral assertions confirmed by code reading:

| Behavior | Evidence | Status |
|----------|----------|--------|
| `start` rejects non-open repo with `not_open` | `review.rs:56`; test `start_rejects_closed_repo` asserts `err.code == "not_open"` | PASS |
| `start` rejects when session file exists with `session_exists` | `review.rs:69-73`; test `start_rejects_when_session_exists` asserts `err.code == "session_exists"` | PASS |
| `resume` loads from disk with fresh in-memory state | `review.rs:88-96`; test `resume_after_restart` passes empty state_map re-creation | PASS |
| Symlink resolves to same canonical path and same session | `review.rs:57`; `symlink_resumes_same_session` test (`#[cfg(unix)]`) | PASS |
| `end` hard-deletes file; `get_review_session_status_inner` reports `None` | `review.rs:99-107`; test `end_clears_session` | PASS |
| `_inner` never returns `Active` (disk-only view) | `review.rs:119-123`; test `status_inner_never_reports_active` | PASS |
| Corrupt file quarantined to `.corrupt` sidecar | `review_store.rs:85-88`; test `corrupt_quarantined` | PASS |
| Newer-schema file left byte-identical (no overwrite) | `review_store.rs:124-125`; test `newer_version_refused` asserts `before == after` bytes | PASS |
| Atomic write leaves no `.tmp` residue | `review_store.rs:66-82`; test `atomic_write_clean` | PASS |
| `close_repo`/`force_close_repo` drop in-memory entry only, never call `delete_session` | `repo.rs:62,86`; grep confirms 0 `delete_session` calls in `repo.rs` | PASS |

---

### Requirements Coverage

| Requirement | Plans | Description | Status | Evidence |
|-------------|-------|-------------|--------|----------|
| SESS-01 | 65-01, 65-03, 65-04 | User can start a code review session for the current repository | SATISFIED | `start_review_session` command + `not_open` guard + `session_exists` rejection |
| SESS-02 | 65-01, 65-02, 65-03, 65-04 | User can resume an in-progress review session after restarting the app | SATISFIED | Atomic persistence in `review_store.rs`; `resume_review_session` loads from disk; `resume_after_restart` and `symlink_resumes_same_session` tests pass |
| SESS-03 | 65-01, 65-03, 65-04 | User can end and clear the active review session | SATISFIED | `end_review_session` hard-deletes via `delete_session`; `end_clears_session` test passes; `delete_session` treats NotFound as success |

All three requirements are covered. No orphaned requirements for Phase 65 in REQUIREMENTS.md.

---

### Anti-Patterns Found

No `TBD`, `FIXME`, or `XXX` markers found in phase-modified files. No stub implementations that prevent functionality. The `ReviewPanel.svelte` stub is D-12 intentional throwaway scaffolding with a code comment marking it as such — this is by design and explicitly scoped in the plan.

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `ReviewPanel.svelte` line 2 | `// THROWAWAY STUB (Phase 65, D-12): replaced by the real review panel in Phase 69.` | INFO | Intentional; explicitly scoped as D-12 with a named replacement phase. Not a blocker. |

---

### Human Verification Required

### 1. End-to-End Lifecycle via View Menu

**Test:** Open Trunk on a Git repository. Open the View menu and click "Start/End Code Review". The ReviewPanel should appear showing "No code review session" with a "Start Code Review" button. Click Start. The panel should change to show "Code review in progress" with an "End Review" button. Force-quit Trunk (kill -9 or Activity Monitor force-quit). Reopen Trunk on the same repository. The ReviewPanel should show "A saved review session is available" with Resume and Discard buttons.

**Expected:** The three D-12 states cycle correctly: none → active → (force-quit + reopen) resume-available. Clicking Resume shows the active state again. Clicking End from any state returns to none.

**Why human:** Full end-to-end cycle requires a running Tauri app. The force-quit durability guarantee — that an atomic write survives a kill signal — cannot be proven by unit tests alone, only by actually performing the force-quit scenario.

---

### Gaps Summary

No gaps. All four ROADMAP success criteria are fully implemented and verified at the code level. The single human verification item tests the end-to-end user experience in a running app.

---

_Verified: 2026-05-25_
_Verifier: Claude (gsd-verifier)_
