use crate::error::TrunkError;
use crate::git::{graph, repository};
use crate::state::{kill_process, CommitCache, RepoState, RunningOp};
use crate::watcher::{self, WatcherState};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn open_repo(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    watcher_state: State<'_, WatcherState>,
    app: AppHandle,
) -> Result<(), String> {
    let path_clone = path.clone();

    let result = tauri::async_runtime::spawn_blocking(
        move || -> Result<crate::git::types::GraphResult, TrunkError> {
            let path_buf = std::path::PathBuf::from(&path_clone);
            repository::validate_and_open(&path_buf)?;
            let mut repo = git2::Repository::open(&path_buf)?;
            graph::walk_commits(&mut repo, 0, usize::MAX)
        },
    )
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    let path_buf = std::path::PathBuf::from(&path);
    state
        .0
        .lock()
        .unwrap()
        .insert(path.clone(), path_buf.clone());
    cache.0.lock().unwrap().insert(path.clone(), result);
    watcher::start_watcher(path_buf, app, &watcher_state);

    Ok(())
}

#[tauri::command]
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

#[tauri::command]
pub async fn force_close_repo(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    watcher_state: State<'_, WatcherState>,
    running: State<'_, RunningOp>,
) -> Result<(), String> {
    // Cancel running remote op first (D-03)
    {
        let mut guard = running.0.lock().unwrap();
        if let Some(pid) = guard.remove(&path) {
            kill_process(pid);
        }
    }
    // Then clean up all other state (same as close_repo)
    state.0.lock().unwrap().remove(&path);
    cache.0.lock().unwrap().remove(&path);
    watcher::stop_watcher(&path, &watcher_state);
    Ok(())
}
