use crate::error::TrunkError;
use crate::git::{
    graph,
    types::{GraphResult, UndoResult},
};
use crate::shell_env;
use crate::state::{CommitCache, RepoState};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

fn open_repo(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<git2::Repository, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    git2::Repository::open(path_buf).map_err(TrunkError::from)
}

fn is_dirty(repo: &git2::Repository) -> Result<bool, git2::Error> {
    use git2::{Status, StatusOptions};
    let mut opts = StatusOptions::new();
    opts.include_untracked(false).include_ignored(false);

    let dirty_flags = Status::INDEX_NEW
        | Status::INDEX_MODIFIED
        | Status::INDEX_DELETED
        | Status::INDEX_RENAMED
        | Status::INDEX_TYPECHANGE
        | Status::WT_MODIFIED
        | Status::WT_DELETED
        | Status::WT_RENAMED
        | Status::WT_TYPECHANGE;

    let statuses = repo.statuses(Some(&mut opts))?;
    Ok(statuses.iter().any(|s| s.status().intersects(dirty_flags)))
}

pub fn checkout_commit_inner(
    path: &str,
    oid: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let repo = open_repo(path, state_map)?;

    if is_dirty(&repo)? {
        return Err(TrunkError::new(
            "dirty_workdir",
            "Working tree has uncommitted changes",
        ));
    }

    let obj = repo.revparse_single(oid)?;
    repo.checkout_tree(&obj, Some(&mut git2::build::CheckoutBuilder::new().safe()))?;
    repo.set_head_detached(obj.id())?;
    drop(obj);
    drop(repo);

    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo2, 0, usize::MAX)
}

pub fn create_tag_inner(
    path: &str,
    oid: &str,
    tag_name: &str,
    message: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let repo = open_repo(path, state_map)?;
    let obj = repo.revparse_single(oid)?;
    let sig = repo.signature().map_err(TrunkError::from)?;
    let msg = if message.trim().is_empty() {
        tag_name.to_owned()
    } else {
        message.to_owned()
    };
    repo.tag(tag_name, &obj, &sig, &msg, false)?;
    drop(obj);
    drop(repo);

    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo2, 0, usize::MAX)
}

pub fn delete_tag_inner(
    path: &str,
    tag_name: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let repo = open_repo(path, state_map)?;
    let tag_ref_name = format!("refs/tags/{}", tag_name);
    let mut reference = repo.find_reference(&tag_ref_name)?;
    reference.delete()?;
    drop(reference);
    drop(repo);

    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo2, 0, usize::MAX)
}

pub fn cherry_pick_inner(
    path: &str,
    oid: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;

    let output = std::process::Command::new("git")
        .args(["cherry-pick", oid])
        .current_dir(path_buf)
        .env("PATH", shell_env::system_path())
        .output()
        .map_err(|e| TrunkError::new("cherry_pick_error", e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = if stderr.to_lowercase().contains("conflict") {
            "conflict_state"
        } else {
            "cherry_pick_error"
        };
        return Err(TrunkError::new(code, stderr.to_string()));
    }

    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

pub fn revert_commit_inner(
    path: &str,
    oid: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;

    let output = std::process::Command::new("git")
        .args(["revert", oid, "--no-edit"])
        .current_dir(path_buf)
        .env("PATH", shell_env::system_path())
        .output()
        .map_err(|e| TrunkError::new("revert_error", e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = if stderr.to_lowercase().contains("conflict") {
            "conflict_state"
        } else {
            "revert_error"
        };
        return Err(TrunkError::new(code, stderr.to_string()));
    }

    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

pub fn reset_to_commit_inner(
    path: &str,
    oid: &str,
    mode: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;

    let valid_modes = ["soft", "mixed", "hard"];
    if !valid_modes.contains(&mode) {
        return Err(TrunkError::new(
            "invalid_mode",
            format!("Invalid reset mode: {}", mode),
        ));
    }

    let output = std::process::Command::new("git")
        .args(["reset", &format!("--{}", mode), oid])
        .current_dir(path_buf)
        .env("PATH", shell_env::system_path())
        .output()
        .map_err(|e| TrunkError::new("reset_error", e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("reset_error", stderr.to_string()));
    }

    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

#[tauri::command]
pub async fn reset_to_commit(
    path: String,
    oid: String,
    mode: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        reset_to_commit_inner(&path_clone, &oid, &mode, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn checkout_commit(
    path: String,
    oid: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        checkout_commit_inner(&path_clone, &oid, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn create_tag(
    path: String,
    oid: String,
    tag_name: String,
    message: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        create_tag_inner(&path_clone, &oid, &tag_name, &message, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn delete_tag(
    path: String,
    tag_name: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        delete_tag_inner(&path_clone, &tag_name, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn cherry_pick(
    path: String,
    oid: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        cherry_pick_inner(&path_clone, &oid, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn revert_commit(
    path: String,
    oid: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        revert_commit_inner(&path_clone, &oid, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

pub fn undo_commit_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<UndoResult, TrunkError> {
    let repo = open_repo(path, state_map)?;
    let head = repo.head()?.peel_to_commit()?;

    if head.parent_count() == 0 {
        return Err(TrunkError::new(
            "nothing_to_undo",
            "Cannot undo the initial commit",
        ));
    }
    if head.parent_count() > 1 {
        return Err(TrunkError::new(
            "merge_commit",
            "Cannot undo a merge commit",
        ));
    }

    let subject = head.summary().unwrap_or("").to_owned();
    let body = head.body().map(str::to_owned);
    drop(head);
    drop(repo);

    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;

    let output = std::process::Command::new("git")
        .args(["reset", "--soft", "HEAD~1"])
        .current_dir(path_buf)
        .env("PATH", shell_env::system_path())
        .output()
        .map_err(|e| TrunkError::new("undo_error", e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("undo_error", stderr.to_string()));
    }

    Ok(UndoResult { subject, body })
}

pub fn redo_commit_inner(
    path: &str,
    subject: &str,
    body: Option<&str>,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    super::commit::create_commit_inner(path, subject, body, state_map)
}

pub fn check_undo_available_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<bool, TrunkError> {
    let repo = open_repo(path, state_map)?;
    let head = match repo.head() {
        Ok(h) => match h.peel_to_commit() {
            Ok(c) => c,
            Err(_) => return Ok(false),
        },
        Err(_) => return Ok(false),
    };
    // Can undo if exactly one parent (not initial, not merge)
    Ok(head.parent_count() == 1)
}

#[tauri::command]
pub async fn undo_commit(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<UndoResult, String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let (undo_result, graph_result) = tauri::async_runtime::spawn_blocking(move || {
        let undo = undo_commit_inner(&path_clone, &state_map)?;
        let graph = {
            let path_buf = state_map.get(path_clone.as_str()).ok_or_else(|| {
                TrunkError::new("not_open", format!("Repository not open: {}", path_clone))
            })?;
            let mut repo = git2::Repository::open(path_buf).map_err(TrunkError::from)?;
            graph::walk_commits(&mut repo, 0, usize::MAX)?
        };
        Ok::<(UndoResult, GraphResult), TrunkError>((undo, graph))
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(undo_result)
}

#[tauri::command]
pub async fn redo_commit(
    path: String,
    subject: String,
    body: Option<String>,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        redo_commit_inner(&path_clone, &subject, body.as_deref(), &state_map)?;
        let path_buf = state_map.get(path_clone.as_str()).ok_or_else(|| {
            TrunkError::new("not_open", format!("Repository not open: {}", path_clone))
        })?;
        let mut repo = git2::Repository::open(path_buf).map_err(TrunkError::from)?;
        graph::walk_commits(&mut repo, 0, usize::MAX)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn check_undo_available(
    path: String,
    state: State<'_, RepoState>,
) -> Result<bool, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || check_undo_available_inner(&path, &state_map))
        .await
        .map_err(|e| {
            serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap()
        })?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}
