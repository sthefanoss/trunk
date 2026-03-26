mod common;

use common::context::TestContext;

#[test]
fn builder_creates_repo_with_initial_commit() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    ctx.assert_commit_count(1);
    ctx.assert_head_message("Initial commit");
    ctx.assert_file_content("README.md", "hello");
}

#[test]
fn builder_creates_branch_and_merge() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("feature.txt", "work")
        .with_commit("Feature work")
        .checkout("main")
        .merge("feature")
        .build();

    ctx.assert_head_at("main");
    ctx.assert_branch_exists("feature");
    // merge commit + feature commit + initial = 3 commits on main
    ctx.assert_commit_count(3);
}

#[test]
fn builder_creates_conflict_state() {
    let ctx = TestContext::builder()
        .with_file("file.txt", "original")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("file.txt", "feature version")
        .with_commit("Feature change")
        .checkout("main")
        .with_file("file.txt", "main version")
        .with_commit("Main change")
        .with_conflict("feature")
        .build();

    ctx.assert_conflict_state();
}

#[test]
fn builder_creates_tag() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .with_tag("v1.0.0")
        .build();

    ctx.assert_tag_exists("v1.0.0");
}

#[test]
fn builder_creates_binary_file() {
    let ctx = TestContext::builder()
        .with_binary_file("image.png", &[0x89, 0x50, 0x4E, 0x47])
        .with_commit("Add binary file")
        .build();

    let content = std::fs::read(ctx.repo_path().join("image.png")).unwrap();
    assert_eq!(content, vec![0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn empty_context_has_no_commits() {
    let ctx = TestContext::new_empty();
    let repo = ctx.repo();
    assert!(repo.head().is_err(), "expected empty repo to have no HEAD");
}

#[test]
fn status_clean_after_commit() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    ctx.assert_status_clean();
}
