---
phase: 65-data-model-persistence-session-lifecycle
reviewed: 2026-05-25T00:00:00Z
depth: deep
files_reviewed: 11
files_reviewed_list:
  - src-tauri/src/git/types.rs
  - src-tauri/src/git/review_store.rs
  - src-tauri/src/git/mod.rs
  - src-tauri/src/state.rs
  - src-tauri/src/commands/review.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/commands/repo.rs
  - src-tauri/src/lib.rs
  - src/lib/types.ts
  - src/components/ReviewPanel.svelte
  - src/App.svelte
findings:
  critical: 0
  warning: 3
  info: 1
  total: 4
status: issues
---

# Phase 65: Code Review Report

**Reviewed:** 2026-05-25
**Depth:** deep
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Phase 65 introduces a well-structured per-repo review-session persistence layer: atomic
tmp+rename writes, FNV-1a filename hashing for path-traversal mitigation, schema-version
gating before full deserialization, and a clean disk-first mutation ordering across all
four lifecycle commands. The security-critical properties (path traversal, atomic write
durability, corrupt quarantine, newer-schema refusal) are all implemented correctly.
No critical bugs were found.

Three warnings are raised:

1. `resume_review_session` emits `session-changed` even when `LoadOutcome::None` is
   returned (no session file exists), triggering an unnecessary frontend reload cycle.
2. The `RecoveredCorrupt` recovery branch silently replaces lost session data —
   `resume_review_session` returns `Ok(())` with no observable distinction from a
   clean load, making the comment "let the frontend toast the warning" unreachable
   in the current code path.
3. `start_review_session` guards only against a file-on-disk collision, not an
   in-memory collision. If the session file is deleted externally while a session
   is active, the thin command silently overwrites the live in-memory session.

---

## Warnings

### WR-01: Spurious `session-changed` emission when `LoadOutcome::None`

**File:** `src-tauri/src/commands/review.rs:226`

**Issue:** `resume_review_session` emits `session-changed` unconditionally after the
`match outcome { ... }` block. When `LoadOutcome::None` is returned (the repo has no
session file), the command does nothing to memory but still fires the event. The
frontend's `ReviewPanel` receives the event, calls `get_review_session_status`, gets
back the same `none` state it already had, and re-renders unnecessarily. More
importantly, callers cannot distinguish "resumed successfully" from "nothing to resume"
without a follow-up status call. The `LoadOutcome::None` arm could legitimately be
treated as a no-op, but the event says otherwise.

**Fix:** Gate the emit on an outcome that actually changed state, or return a
structured result that lets callers branch:

```rust
// Option A: skip emit for None
match outcome {
    LoadOutcome::Loaded(_) | LoadOutcome::RecoveredCorrupt => {
        // ... existing logic ...
        let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
    }
    LoadOutcome::None => {
        // Nothing changed; no event needed.
    }
    LoadOutcome::RefusedNewer => { ... }
}
```

---

### WR-02: `RecoveredCorrupt` silently loses data — promised toast signal is unreachable

**File:** `src-tauri/src/commands/review.rs:203-214`

**Issue:** The comment on line 205 states: _"let the frontend toast the warning."_
However, `resume_review_session` returns `Ok(())` for `RecoveredCorrupt` — identical
to a successful `Loaded` outcome from the frontend's perspective. `ReviewPanel.svelte`
has no branch on the result of `safeInvoke("resume_review_session", ...)`: it only
checks for thrown errors. There is no toast mechanism wired to distinguish this case,
so the user's session data is silently discarded without any notification. The `.corrupt`
sidecar is preserved on disk (correct per D-15), but the user cannot be informed to
inspect or report it.

**Fix:** Either (a) return a structured success payload that carries a `recovered: bool`
flag, or (b) return a specific error code like `"corrupt_recovered"` that the frontend
can catch and toast. The simplest approach that keeps `_inner` testable:

```rust
// In review.rs (thin command):
LoadOutcome::RecoveredCorrupt => {
    let fresh = ...;
    review_store::save_session(...)...?;
    sessions.0.lock().unwrap().insert(canonical.clone(), fresh);
    // Return a distinct error so ReviewPanel can toast
    // (frontend catches and shows warning, then re-polls status)
    return Err(serde_json::to_string(&TrunkError::new(
        "corrupt_recovered",
        "Your previous review session was unreadable and has been reset. \
         The corrupt file was preserved as .corrupt in the session directory.",
    )).unwrap());
}
```

Alternatively expose a `ResumeOutcome` enum in the return type so the frontend can
distinguish outcomes without overloading the error channel.

---

### WR-03: `start_review_session` does not check in-memory session state

**File:** `src-tauri/src/commands/review.rs:150-173`

**Issue:** `start_review_session_inner` guards against double-start by calling
`session_exists` (disk check). The thin command does not additionally check whether
`sessions.0.lock().unwrap().contains_key(&canonical)` before spawning `_inner`. If the
session file is externally deleted (e.g., by the user, a backup tool, or an OS cleanup
agent) while a session is active in memory, `session_exists` returns `false`, `_inner`
creates a fresh session on disk, and the thin command performs `sessions.0.lock().unwrap().insert(canonical.clone(), session)`, silently overwriting the live in-memory
session and dropping its `commits`, `comments`, and `draft_comment` without warning.

The disk-first contract means this scenario is unlikely in normal UI flow (the Start
button only renders when `state == "none"`, which requires file absent). However the
gap is a defense-in-depth failure: the in-memory guard is the authoritative source of
"active" truth and should be checked.

**Fix:** Add an in-memory guard in the thin command before `spawn_blocking`:

```rust
pub async fn start_review_session(...) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();

    // Defense-in-depth: also reject if an in-memory session is already active,
    // even if the file was externally deleted.
    // Canonicalize using the stored PathBuf (same as _inner will do).
    if let Some(raw_path_buf) = state_map.get(&path) {
        if let Ok(canonical) = std::fs::canonicalize(raw_path_buf) {
            if sessions.0.lock().unwrap().contains_key(&canonical) {
                return Err(serde_json::to_string(&TrunkError::new(
                    "session_exists",
                    "A review session is already active in memory — resume or end it first",
                )).unwrap());
            }
        }
    }

    let data_dir = resolve_data_dir(&app)?;
    // ... rest unchanged ...
}
```

---

## Info

### IN-01: Dead `data_dir_for_save` clone that is `data_dir` renamed

**File:** `src-tauri/src/commands/review.rs:184`

**Issue:** `data_dir_for_save` is a `data_dir.clone()` introduced solely so that
`data_dir` can be moved into the `spawn_blocking` closure while a copy remains
available for the `RecoveredCorrupt` branch. The naming (`_for_save`) implies a
distinct purpose that doesn't exist — both references hold the same path. The clone
is mechanically necessary (the move happens), but the name adds confusion.

**Fix:** Either rename to `data_dir_outer` (matching the dual-use pattern), or capture
a clone in the closure instead:

```rust
// Before:
let data_dir_for_save = data_dir.clone();
let (canonical, outcome) = tauri::async_runtime::spawn_blocking(move || {
    resume_review_session_inner(&data_dir, &path, &state_map)
})...;
// RecoveredCorrupt uses data_dir_for_save

// After (equivalent, clearer intent):
let (canonical, outcome) = tauri::async_runtime::spawn_blocking({
    let data_dir = data_dir.clone();
    move || resume_review_session_inner(&data_dir, &path, &state_map)
})...;
// RecoveredCorrupt uses the outer data_dir
```

---

## Focus Area Verdicts

| Area | Verdict |
|------|---------|
| Path-traversal / filename safety | PASS — FNV-1a hash of canonical path; hex string cannot contain `/` or `..`; `sessions/` dir containment holds |
| Atomic write durability | PASS — tmp in same dir, `sync_all` before rename, `rename` is atomic-replace on both POSIX and Windows (Rust uses `MOVEFILE_REPLACE_EXISTING`) |
| Corrupt / newer-schema recovery | PARTIAL — corrupt quarantine and schema-version refusal are mechanically correct (WR-02 covers missing user notification) |
| Concurrency | PASS — no lock held across `.await`; `spawn_blocking` for all I/O; disk-first ordering prevents memory/disk divergence |
| Path canonicalization correctness | PASS — `ReviewSessionsState` keyed by canonical `PathBuf`; `RepoState`/`CommitCache` unchanged (raw-String keyed); `drop_in_memory_session` re-canonicalizes the raw path to match |
| Schema forward-compat | PASS — `Source`/`Side` PascalCase (no `rename_all`); `schema_version` present; no forbidden fields (`hunk_index`, diff options) in persisted types |

---

_Reviewed: 2026-05-25_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
