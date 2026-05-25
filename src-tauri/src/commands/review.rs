//! Review-session lifecycle commands (Phase 65, Plan 03).
//!
//! Four thin `#[tauri::command]`s over testable `_inner(data_dir: &Path, ...)`
//! functions, mirroring `stash.rs`. The `_inner` wedge takes plain args (no Tauri
//! state) so disk behavior is provable with a `tempfile::TempDir`.
//!
//! Canonical-path keying (D-11): the repo's `PathBuf` is canonicalized so a repo
//! opened via a symlink or alias resumes the SAME session.
//!
//! Disk-first mutation ordering (D-10): `_inner` writes the file → the thin
//! command then updates `ReviewSessionsState` → then emits `session-changed`, so
//! a failed write can never leave memory and disk diverged.

use crate::error::TrunkError;
use crate::git::review_store::{self, LoadOutcome};
use crate::git::types::ReviewSession;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The three review-session states the frontend renders (D-12). Serializes
/// kebab-case to match the stub strings `active` / `resume-available` / `none`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SessionState {
    /// File on disk AND in-memory entry present (only the thin command derives this).
    Active,
    /// File on disk but no in-memory entry — the user can resume.
    ResumeAvailable,
    /// No file and no in-memory entry.
    None,
}

/// Status payload for `get_review_session_status`. `_inner` fills the DISK half
/// (`file_exists` + `state` = ResumeAvailable/None); the thin command promotes to
/// `Active` after locking `ReviewSessionsState`. `canonical_path` is the
/// canonicalized path as a String so the frontend can match `session-changed`
/// payloads without re-canonicalizing (it cannot call `std::fs::canonicalize`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionStatus {
    pub state: SessionState,
    pub file_exists: bool,
    pub canonical_path: String,
}

/// Look the repo up in `RepoState`'s map and canonicalize its `PathBuf`.
/// Returns `not_open` when the path is not a currently-open repo (SESS-01).
fn canonical_repo_path(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<PathBuf, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    std::fs::canonicalize(path_buf).map_err(|e| TrunkError::new("io", e.to_string()))
}

/// Start a fresh review session for a currently-open repo (SESS-01 / D-08).
/// Rejects with `session_exists` if a file is already present — the client must
/// Resume or End-and-clear first (RESEARCH Open Question 2).
pub fn start_review_session_inner(
    data_dir: &Path,
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(PathBuf, ReviewSession), TrunkError> {
    let canonical = canonical_repo_path(path, state_map)?;
    if review_store::session_exists(data_dir, &canonical) {
        return Err(TrunkError::new(
            "session_exists",
            "A review session already exists for this repository — resume or end it first",
        ));
    }
    let session = ReviewSession {
        schema_version: 1,
        commits: vec![],
        comments: vec![],
        draft_comment: None,
    };
    review_store::save_session(data_dir, &canonical, &session)?;
    Ok((canonical, session))
}

/// Load an existing session from disk for a currently-open repo (SESS-02 / D-14).
/// Returns the canonical path + the `LoadOutcome` so the command layer can branch
/// (Loaded → insert + emit; RecoveredCorrupt → fresh + toast; RefusedNewer → warn).
pub fn resume_review_session_inner(
    data_dir: &Path,
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(PathBuf, LoadOutcome), TrunkError> {
    let canonical = canonical_repo_path(path, state_map)?;
    let outcome = review_store::load_session(data_dir, &canonical)?;
    Ok((canonical, outcome))
}

/// Hard-delete the session file for a currently-open repo (SESS-03 / D-13).
pub fn end_review_session_inner(
    data_dir: &Path,
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<PathBuf, TrunkError> {
    let canonical = canonical_repo_path(path, state_map)?;
    review_store::delete_session(data_dir, &canonical)?;
    Ok(canonical)
}

/// Report the DISK half of the session status (D-14). `_inner` has no Tauri state
/// so it NEVER returns `Active` — it sets `ResumeAvailable` if the file exists,
/// else `None`. The thin command promotes to `Active` after locking the in-memory map.
pub fn get_review_session_status_inner(
    data_dir: &Path,
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<SessionStatus, TrunkError> {
    let canonical = canonical_repo_path(path, state_map)?;
    let file_exists = review_store::session_exists(data_dir, &canonical);
    let state = if file_exists {
        SessionState::ResumeAvailable
    } else {
        SessionState::None
    };
    Ok(SessionStatus {
        state,
        file_exists,
        canonical_path: canonical.to_string_lossy().into_owned(),
    })
}
