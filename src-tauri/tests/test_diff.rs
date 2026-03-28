mod common;

use common::context::TestContext;
use trunk_lib::git::types::DiffRequestOptions;

// -- diff_unstaged tests --

#[test]
fn modified_tracked_file_produces_unstaged_hunks() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    std::fs::write(
        ctx.repo_path().join("README.md"),
        "modified content for diff",
    )
    .unwrap();

    let file_diffs = ctx
        .diff_unstaged("README.md")
        .expect("diff_unstaged failed");
    assert!(!file_diffs.is_empty(), "expected non-empty file_diffs");

    let fd = &file_diffs[0];
    assert!(!fd.is_binary, "expected is_binary == false");
    assert!(!fd.hunks.is_empty(), "expected non-empty hunks");
}

#[test]
fn clean_file_produces_empty_unstaged_diff() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    let file_diffs = ctx
        .diff_unstaged("README.md")
        .expect("diff_unstaged failed");
    assert!(
        file_diffs.is_empty(),
        "expected empty file_diffs for clean file"
    );
}

#[test]
fn untracked_file_shows_content_in_unstaged_diff() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    std::fs::write(
        ctx.repo_path().join("new_file.txt"),
        "line1\nline2\nline3\n",
    )
    .unwrap();

    let file_diffs = ctx
        .diff_unstaged("new_file.txt")
        .expect("diff_unstaged failed");
    assert!(
        !file_diffs.is_empty(),
        "expected non-empty file_diffs for untracked file"
    );

    let fd = &file_diffs[0];
    assert_eq!(fd.path, "new_file.txt");
    assert!(
        !fd.hunks.is_empty(),
        "expected hunks with content for untracked file"
    );
    assert!(
        !fd.hunks[0].lines.is_empty(),
        "expected lines in hunk for untracked file"
    );
}

#[test]
fn untracked_file_in_subdirectory_shows_in_unstaged_diff() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    std::fs::create_dir_all(ctx.repo_path().join("docs")).unwrap();
    std::fs::write(ctx.repo_path().join("docs/notes.md"), "hello\nworld\n").unwrap();

    let file_diffs = ctx
        .diff_unstaged("docs/notes.md")
        .expect("diff_unstaged failed");
    assert!(
        !file_diffs.is_empty(),
        "expected non-empty file_diffs for untracked file in subdir"
    );

    let fd = &file_diffs[0];
    assert_eq!(fd.path, "docs/notes.md");
    assert!(!fd.hunks.is_empty(), "expected hunks with content");
    assert!(!fd.hunks[0].lines.is_empty(), "expected lines in hunk");
}

// -- diff_staged tests --

#[test]
fn staged_modification_produces_staged_hunks() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    std::fs::write(ctx.repo_path().join("README.md"), "staged content for diff").unwrap();
    {
        let repo = ctx.repo();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("README.md")).unwrap();
        index.write().unwrap();
    }

    let file_diffs = ctx.diff_staged("README.md").expect("diff_staged failed");
    assert!(!file_diffs.is_empty(), "expected non-empty file_diffs");

    let fd = &file_diffs[0];
    assert!(!fd.hunks.is_empty(), "expected non-empty hunks");
}

#[test]
fn staged_file_on_unborn_head_produces_diff() {
    let ctx = TestContext::new_empty();

    std::fs::write(ctx.repo_path().join("new_file.txt"), "brand new content").unwrap();
    {
        let repo = ctx.repo();
        let mut index = repo.index().unwrap();
        index
            .add_path(std::path::Path::new("new_file.txt"))
            .unwrap();
        index.write().unwrap();
    }

    let file_diffs = ctx.diff_staged("new_file.txt").expect("diff_staged failed");
    assert!(
        !file_diffs.is_empty(),
        "expected non-empty file_diffs for unborn HEAD staged file"
    );
}

// -- diff_commit tests --

#[test]
fn diff_commit_succeeds_for_head() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .with_file("README.md", "modified")
        .with_commit("Second commit")
        .build();

    let repo = ctx.repo();
    let head_oid = repo.head().unwrap().target().unwrap().to_string();
    drop(repo);

    let result = ctx.diff_commit(&head_oid);
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[test]
fn diff_commit_root_commit_shows_added_files() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    // Walk to find root commit (parent_count == 0)
    let repo = ctx.repo();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let root_oid = revwalk
        .filter_map(|id| id.ok())
        .find(|&id| {
            repo.find_commit(id)
                .map(|c| c.parent_count() == 0)
                .unwrap_or(false)
        })
        .expect("no root commit found");
    let root_oid_str = root_oid.to_string();
    drop(repo);

    let file_diffs = ctx.diff_commit(&root_oid_str).expect("diff_commit failed");
    assert!(
        !file_diffs.is_empty(),
        "expected non-empty file_diffs for root commit"
    );
}

// -- get_commit_detail tests --

#[test]
fn commit_detail_returns_metadata() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    let repo = ctx.repo();
    let head_oid = repo.head().unwrap().target().unwrap().to_string();
    drop(repo);

    let detail = ctx
        .get_commit_detail(&head_oid)
        .expect("get_commit_detail failed");
    assert_eq!(detail.oid.len(), 40, "expected 40-char oid");
    assert_eq!(detail.short_oid.len(), 7, "expected 7-char short_oid");
    assert!(!detail.summary.is_empty(), "expected non-empty summary");
    assert!(
        !detail.author_name.is_empty(),
        "expected non-empty author_name"
    );
}

#[test]
fn commit_detail_includes_committer_fields() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    let repo = ctx.repo();
    let head_oid = repo.head().unwrap().target().unwrap().to_string();
    drop(repo);

    let detail = ctx
        .get_commit_detail(&head_oid)
        .expect("get_commit_detail failed");
    assert!(
        !detail.committer_name.is_empty(),
        "expected non-empty committer_name"
    );
    assert!(
        !detail.committer_email.is_empty(),
        "expected non-empty committer_email"
    );
    assert!(
        detail.committer_timestamp > 0,
        "expected committer_timestamp > 0"
    );
}

// -- DiffRequestOptions tests --

#[test]
fn diff_unstaged_respects_context_lines() {
    let content: String = (1..=20).map(|i| format!("line {}\n", i)).collect();
    let ctx = TestContext::builder()
        .with_file("big.txt", &content)
        .with_commit("Initial commit")
        .build();

    let modified: String = (1..=20)
        .map(|i| {
            if i == 10 {
                "changed line 10\n".to_string()
            } else {
                format!("line {}\n", i)
            }
        })
        .collect();
    std::fs::write(ctx.repo_path().join("big.txt"), &modified).unwrap();

    let opts_1 = DiffRequestOptions {
        context_lines: 1,
        ..Default::default()
    };
    let result_1 = ctx.diff_unstaged_with_options("big.txt", &opts_1).unwrap();
    let lines_1: usize = result_1[0].hunks.iter().map(|h| h.lines.len()).sum();

    let opts_5 = DiffRequestOptions {
        context_lines: 5,
        ..Default::default()
    };
    let result_5 = ctx.diff_unstaged_with_options("big.txt", &opts_5).unwrap();
    let lines_5: usize = result_5[0].hunks.iter().map(|h| h.lines.len()).sum();

    assert!(
        lines_5 > lines_1,
        "context_lines=5 should produce more lines than context_lines=1: got {} vs {}",
        lines_5,
        lines_1
    );
}

#[test]
fn diff_unstaged_ignores_whitespace_when_enabled() {
    let ctx = TestContext::builder()
        .with_file("ws.txt", "hello world\n")
        .with_commit("Initial commit")
        .build();

    // Only change whitespace (add extra spaces)
    std::fs::write(ctx.repo_path().join("ws.txt"), "hello  world  \n").unwrap();

    // Without whitespace ignore -- should show changes
    let opts_normal = DiffRequestOptions::default();
    let result_normal = ctx
        .diff_unstaged_with_options("ws.txt", &opts_normal)
        .unwrap();
    assert!(
        !result_normal.is_empty(),
        "expected diff without whitespace ignore"
    );
    let has_changes = result_normal[0].hunks.iter().any(|h| !h.lines.is_empty());
    assert!(has_changes, "expected changes in normal diff");

    // With whitespace ignore -- should show no meaningful changes
    let opts_ignore = DiffRequestOptions {
        ignore_whitespace: true,
        ..Default::default()
    };
    let result_ignore = ctx
        .diff_unstaged_with_options("ws.txt", &opts_ignore)
        .unwrap();
    // When ignoring whitespace changes, git2 produces empty hunks or no hunks
    let ignore_lines: usize = result_ignore
        .iter()
        .flat_map(|fd| fd.hunks.iter())
        .flat_map(|h| h.lines.iter())
        .filter(|l| {
            matches!(
                l.origin,
                trunk_lib::git::types::DiffOrigin::Add | trunk_lib::git::types::DiffOrigin::Delete
            )
        })
        .count();
    assert_eq!(
        ignore_lines, 0,
        "expected no add/delete lines when ignoring whitespace"
    );
}

#[test]
fn diff_unstaged_show_full_file_returns_all_lines() {
    let content: String = (1..=50).map(|i| format!("line {}\n", i)).collect();
    let ctx = TestContext::builder()
        .with_file("full.txt", &content)
        .with_commit("Initial commit")
        .build();

    let modified: String = (1..=50)
        .map(|i| {
            if i == 25 {
                "changed line 25\n".to_string()
            } else {
                format!("line {}\n", i)
            }
        })
        .collect();
    std::fs::write(ctx.repo_path().join("full.txt"), &modified).unwrap();

    let opts = DiffRequestOptions {
        show_full_file: true,
        ..Default::default()
    };
    let result = ctx
        .diff_unstaged_with_options("full.txt", &opts)
        .unwrap();
    let total_lines: usize = result[0].hunks.iter().map(|h| h.lines.len()).sum();

    // Full file should have at least 50 lines (50 original context + 1 delete + 1 add = ~52)
    assert!(
        total_lines >= 50,
        "show_full_file should return all lines, got {}",
        total_lines
    );
}

#[test]
fn diff_commit_respects_context_lines() {
    let content: String = (1..=20).map(|i| format!("line {}\n", i)).collect();
    let modified: String = (1..=20)
        .map(|i| {
            if i == 10 {
                "changed line 10\n".to_string()
            } else {
                format!("line {}\n", i)
            }
        })
        .collect();

    let ctx = TestContext::builder()
        .with_file("big.txt", &content)
        .with_commit("Initial commit")
        .with_file("big.txt", &modified)
        .with_commit("Modify line 10")
        .build();

    let repo = ctx.repo();
    let head_oid = repo.head().unwrap().target().unwrap().to_string();
    drop(repo);

    let opts_1 = DiffRequestOptions {
        context_lines: 1,
        ..Default::default()
    };
    let result_1 = ctx.diff_commit_with_options(&head_oid, &opts_1).unwrap();
    let lines_1: usize = result_1[0].hunks.iter().map(|h| h.lines.len()).sum();

    let opts_5 = DiffRequestOptions {
        context_lines: 5,
        ..Default::default()
    };
    let result_5 = ctx.diff_commit_with_options(&head_oid, &opts_5).unwrap();
    let lines_5: usize = result_5[0].hunks.iter().map(|h| h.lines.len()).sum();

    assert!(
        lines_5 > lines_1,
        "context_lines=5 should produce more lines than context_lines=1 for commit diff: got {} vs {}",
        lines_5,
        lines_1
    );
}
