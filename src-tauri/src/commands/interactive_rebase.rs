use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;
use crate::error::TrunkError;
use crate::git::types::RebaseTodoItem;
use crate::state::RepoState;

fn open_repo(path: &str, state_map: &HashMap<String, PathBuf>) -> Result<git2::Repository, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    git2::Repository::open(path_buf).map_err(TrunkError::from)
}

pub fn get_rebase_todo_inner(
    path: &str,
    base_oid: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<Vec<RebaseTodoItem>, TrunkError> {
    let repo = open_repo(path, state_map)?;

    let base = git2::Oid::from_str(base_oid)
        .map_err(|e| TrunkError::new("invalid_oid", e.to_string()))?;

    let mut revwalk = repo.revwalk().map_err(TrunkError::from)?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME).map_err(TrunkError::from)?;
    revwalk.push_head().map_err(TrunkError::from)?;
    revwalk.hide(base).map_err(TrunkError::from)?;

    let mut items: Vec<RebaseTodoItem> = Vec::new();
    for oid_result in revwalk {
        let oid = oid_result.map_err(TrunkError::from)?;
        let commit = repo.find_commit(oid).map_err(TrunkError::from)?;
        let oid_str = oid.to_string();
        let short_oid = oid_str.chars().take(7).collect();
        let summary = commit.summary().unwrap_or("").to_owned();
        let author_name = commit.author().name().unwrap_or("").to_owned();
        let author_timestamp = commit.time().seconds();

        items.push(RebaseTodoItem {
            oid: oid_str,
            short_oid,
            summary,
            author_name,
            author_timestamp,
        });
    }

    // Revwalk returns newest-first; rebase todo needs oldest-first
    items.reverse();

    Ok(items)
}

pub fn get_fork_point_inner(
    path: &str,
    branch: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<String, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;

    let output = std::process::Command::new("git")
        .args(["merge-base", branch, "HEAD"])
        .current_dir(path_buf)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| TrunkError::new("fork_point_error", e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("fork_point_error", stderr.to_string()));
    }

    let oid = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok(oid)
}

#[tauri::command]
pub async fn get_rebase_todo(
    path: String,
    base_oid: String,
    state: State<'_, RepoState>,
) -> Result<Vec<RebaseTodoItem>, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        get_rebase_todo_inner(&path, &base_oid, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e: TrunkError| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn get_fork_point(
    path: String,
    branch: String,
    state: State<'_, RepoState>,
) -> Result<String, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        get_fork_point_inner(&path, &branch, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e: TrunkError| serde_json::to_string(&e).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_test_repo() -> (TempDir, HashMap<String, PathBuf>, Vec<git2::Oid>) {
        let dir = TempDir::new().unwrap();
        let path_str = dir.path().to_str().unwrap().to_owned();
        let mut oids = Vec::new();

        {
            let repo = git2::Repository::init(dir.path()).unwrap();
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test").unwrap();
            config.set_str("user.email", "test@test.com").unwrap();
            drop(config);

            let sig = repo.signature().unwrap();

            // Commit 1: Initial commit
            fs::write(dir.path().join("file.txt"), "initial").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let c1 = repo.commit(Some("refs/heads/main"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
            oids.push(c1);

            // Commit 2: Second commit
            fs::write(dir.path().join("file.txt"), "second").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let parent = repo.find_commit(c1).unwrap();
            let c2 = repo.commit(Some("refs/heads/main"), &sig, &sig, "Second commit", &tree, &[&parent]).unwrap();
            oids.push(c2);

            // Commit 3: Third commit
            fs::write(dir.path().join("file.txt"), "third").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let parent = repo.find_commit(c2).unwrap();
            let c3 = repo.commit(Some("refs/heads/main"), &sig, &sig, "Third commit", &tree, &[&parent]).unwrap();
            oids.push(c3);

            repo.set_head("refs/heads/main").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force())).unwrap();
        }

        let mut state_map = HashMap::new();
        state_map.insert(path_str.clone(), dir.path().to_owned());
        (dir, state_map, oids)
    }

    #[test]
    fn get_rebase_todo_returns_commits_oldest_first() {
        let (dir, state_map, oids) = make_test_repo();
        let path = dir.path().to_str().unwrap();
        let base_oid = oids[0].to_string(); // Initial commit as base

        let items = get_rebase_todo_inner(path, &base_oid, &state_map).unwrap();

        assert_eq!(items.len(), 2, "Should return 2 commits (excluding base)");
        assert_eq!(items[0].summary, "Second commit", "First item should be oldest (Second commit)");
        assert_eq!(items[1].summary, "Third commit", "Second item should be newest (Third commit)");
    }

    #[test]
    fn get_rebase_todo_returns_empty_when_base_equals_head() {
        let (dir, state_map, oids) = make_test_repo();
        let path = dir.path().to_str().unwrap();
        let base_oid = oids[2].to_string(); // HEAD commit as base

        let items = get_rebase_todo_inner(path, &base_oid, &state_map).unwrap();

        assert_eq!(items.len(), 0, "Should return empty vec when base equals HEAD");
    }

    #[test]
    fn get_rebase_todo_item_has_correct_fields() {
        let (dir, state_map, oids) = make_test_repo();
        let path = dir.path().to_str().unwrap();
        let base_oid = oids[0].to_string();

        let items = get_rebase_todo_inner(path, &base_oid, &state_map).unwrap();

        let item = &items[0];
        assert_eq!(item.oid, oids[1].to_string(), "OID should match second commit");
        assert_eq!(item.short_oid, &oids[1].to_string()[..7], "short_oid should be first 7 chars");
        assert_eq!(item.summary, "Second commit");
        assert_eq!(item.author_name, "Test");
        assert!(item.author_timestamp > 0, "author_timestamp should be positive");
    }

    #[test]
    fn get_fork_point_returns_merge_base() {
        let (dir, state_map, oids) = make_test_repo();
        let path = dir.path().to_str().unwrap();

        // Create a branch at the initial commit
        {
            let repo = git2::Repository::open(dir.path()).unwrap();
            let initial_commit = repo.find_commit(oids[0]).unwrap();
            repo.branch("feature", &initial_commit, false).unwrap();
        }

        let result = get_fork_point_inner(path, "feature", &state_map).unwrap();

        assert_eq!(result, oids[0].to_string(), "Fork point should be the initial commit (merge-base of feature and HEAD)");
    }
}
