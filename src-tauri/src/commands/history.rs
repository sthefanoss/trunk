use std::collections::HashMap;
use serde::Serialize;
use tauri::State;
use crate::state::{CommitCache, RepoState};
use crate::error::TrunkError;
use crate::git::{graph, types::{GraphCommit, GraphResult, MatchType, SearchResult}};

#[derive(Debug, Serialize, Clone)]
pub struct GraphResponse {
    pub commits: Vec<GraphCommit>,
    pub max_columns: usize,
}

#[tauri::command]
pub async fn get_commit_graph(
    path: String,
    offset: usize,
    cache: State<'_, CommitCache>,
) -> Result<GraphResponse, String> {
    let lock = cache.0.lock().unwrap();
    let graph_result = lock.get(&path).ok_or_else(|| {
        serde_json::to_string(&TrunkError::new("repo_not_open", "Repository not open")).unwrap()
    })?;

    let len = graph_result.commits.len();
    let start = offset.min(len);
    let end = (offset + 200).min(len);
    Ok(GraphResponse {
        commits: graph_result.commits[start..end].to_vec(),
        max_columns: graph_result.max_columns,
    })
}

#[tauri::command]
pub async fn refresh_commit_graph(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
) -> Result<GraphResponse, String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();

    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        let path_buf = state_map
            .get(&path_clone)
            .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path_clone)))?;
        let mut repo = git2::Repository::open(path_buf).map_err(TrunkError::from)?;
        graph::walk_commits(&mut repo, 0, usize::MAX)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    let len = graph_result.commits.len();
    let end = 200.min(len);
    let response = GraphResponse {
        commits: graph_result.commits[..end].to_vec(),
        max_columns: graph_result.max_columns,
    };

    cache.0.lock().unwrap().insert(path, graph_result);

    Ok(response)
}

pub fn search_commits_inner(
    path: &str,
    query: &str,
    cache_map: &HashMap<String, GraphResult>,
) -> Result<Vec<SearchResult>, TrunkError> {
    todo!("implement search")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::graph;
    use crate::git::repository::tests::make_test_repo;
    use crate::git::types::MatchType;

    /// Helper: build cache map from a test repo for search_commits_inner
    fn build_cache(dir: &std::path::Path) -> (String, HashMap<String, GraphResult>) {
        let path = dir.to_string_lossy().to_string();
        let mut repo = git2::Repository::open(dir).unwrap();
        let result = graph::walk_commits(&mut repo, 0, usize::MAX).unwrap();
        let mut map = HashMap::new();
        map.insert(path.clone(), result);
        (path, map)
    }

    #[test]
    fn empty_query_returns_empty() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let results = search_commits_inner(&path, "", &map).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn whitespace_query_returns_empty() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let results = search_commits_inner(&path, "   ", &map).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn sha_prefix_match() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let commits = &map.get(&path).unwrap().commits;
        let first_oid = &commits[0].oid;
        let prefix = &first_oid[..6];

        let results = search_commits_inner(&path, prefix, &map).unwrap();
        assert!(!results.is_empty(), "expected at least one SHA match");
        assert!(results[0].match_types.contains(&MatchType::Sha));
    }

    #[test]
    fn sha_match_case_insensitive() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let commits = &map.get(&path).unwrap().commits;
        let first_oid = &commits[0].oid;
        let prefix_upper = first_oid[..6].to_uppercase();

        let results = search_commits_inner(&path, &prefix_upper, &map).unwrap();
        assert!(!results.is_empty(), "expected case-insensitive SHA match");
        assert!(results[0].match_types.contains(&MatchType::Sha));
    }

    #[test]
    fn message_summary_match() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let results = search_commits_inner(&path, "Initial", &map).unwrap();
        assert!(!results.is_empty(), "expected message match for 'Initial'");
        assert!(results.iter().any(|r| r.match_types.contains(&MatchType::Message)));
    }

    #[test]
    fn message_body_match() {
        // make_test_repo commits have no body, so we test that body=None doesn't crash
        // and that summary match still works as fallback
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        // "Feature commit" has no body — should still match on summary
        let results = search_commits_inner(&path, "feature commit", &map).unwrap();
        assert!(!results.is_empty(), "expected match on 'feature commit'");
    }

    #[test]
    fn message_match_case_insensitive() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let results = search_commits_inner(&path, "FEATURE", &map).unwrap();
        assert!(!results.is_empty(), "expected case-insensitive message match for 'FEATURE'");
        assert!(results.iter().any(|r| r.match_types.contains(&MatchType::Message)));
    }

    #[test]
    fn ref_match() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let results = search_commits_inner(&path, "feature", &map).unwrap();
        assert!(
            results.iter().any(|r| r.match_types.contains(&MatchType::Ref)),
            "expected ref match for 'feature'"
        );
    }

    #[test]
    fn ref_match_case_insensitive() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let results = search_commits_inner(&path, "MAIN", &map).unwrap();
        assert!(
            results.iter().any(|r| r.match_types.contains(&MatchType::Ref)),
            "expected case-insensitive ref match for 'MAIN'"
        );
    }

    #[test]
    fn author_match() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let results = search_commits_inner(&path, "Test", &map).unwrap();
        assert!(
            results.iter().any(|r| r.match_types.contains(&MatchType::Author)),
            "expected author match for 'Test'"
        );
    }

    #[test]
    fn author_match_case_insensitive() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let results = search_commits_inner(&path, "test", &map).unwrap();
        assert!(
            results.iter().any(|r| r.match_types.contains(&MatchType::Author)),
            "expected case-insensitive author match for 'test'"
        );
    }

    #[test]
    fn multi_field_match() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        // "main" matches ref "main" AND message "Merge feature into main"
        let results = search_commits_inner(&path, "main", &map).unwrap();
        let multi = results.iter().find(|r| {
            r.match_types.contains(&MatchType::Ref) && r.match_types.contains(&MatchType::Message)
        });
        assert!(
            multi.is_some(),
            "expected a result with both Ref and Message match for 'main'"
        );
    }

    #[test]
    fn no_match_returns_empty() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let results = search_commits_inner(&path, "zzzznonexistent", &map).unwrap();
        assert!(results.is_empty(), "expected no matches for 'zzzznonexistent'");
    }

    #[test]
    fn results_in_graph_order() {
        let dir = make_test_repo();
        let (path, map) = build_cache(dir.path());
        let commits = &map.get(&path).unwrap().commits;
        // "test" matches author_name "Test User" on all commits
        let results = search_commits_inner(&path, "test", &map).unwrap();
        assert!(results.len() >= 2, "expected at least 2 results");

        // Results should be in same order as graph commits
        let result_oids: Vec<&str> = results.iter().map(|r| r.oid.as_str()).collect();
        let graph_oids: Vec<&str> = commits.iter().map(|c| c.oid.as_str()).collect();

        // Each result oid should appear in graph order
        let mut last_idx = 0;
        for oid in &result_oids {
            let idx = graph_oids.iter().position(|g| g == oid).expect("result oid not in graph");
            assert!(idx >= last_idx, "results not in graph order");
            last_idx = idx;
        }
    }
}
