mod common;

use common::context::TestContext;

/// Helper to create a repo with 3 linear commits and return the OIDs.
fn make_three_commit_ctx() -> (TestContext, Vec<git2::Oid>) {
    let ctx = TestContext::builder()
        .with_file("file.txt", "initial")
        .with_commit("Initial commit")
        .with_file("file.txt", "second")
        .with_commit("Second commit")
        .with_file("file.txt", "third")
        .with_commit("Third commit")
        .build();

    let repo = ctx.repo();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk
        .set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE)
        .unwrap();
    revwalk.push_head().unwrap();
    let oids: Vec<git2::Oid> = revwalk.map(|r| r.unwrap()).collect();

    (ctx, oids)
}

#[test]
fn get_rebase_todo_returns_commits_oldest_first() {
    let (ctx, oids) = make_three_commit_ctx();
    let base_oid = oids[0].to_string();

    let items = ctx.get_rebase_todo(&base_oid, false).unwrap();

    assert_eq!(items.len(), 2, "Should return 2 commits (excluding base)");
    assert_eq!(
        items[0].summary, "Second commit",
        "First item should be oldest (Second commit)"
    );
    assert_eq!(
        items[1].summary, "Third commit",
        "Second item should be newest (Third commit)"
    );
}

#[test]
fn get_rebase_todo_inclusive_includes_base_commit() {
    let (ctx, oids) = make_three_commit_ctx();
    let base_oid = oids[1].to_string(); // Second commit

    let items = ctx.get_rebase_todo(&base_oid, true).unwrap();

    assert_eq!(items.len(), 2, "Should return 2 commits (including base)");
    assert_eq!(
        items[0].summary, "Second commit",
        "Base commit should be included"
    );
    assert_eq!(items[1].summary, "Third commit");
}

#[test]
fn get_rebase_todo_returns_empty_when_base_equals_head() {
    let (ctx, oids) = make_three_commit_ctx();
    let base_oid = oids[2].to_string(); // HEAD commit as base

    let items = ctx.get_rebase_todo(&base_oid, false).unwrap();

    assert_eq!(
        items.len(),
        0,
        "Should return empty vec when base equals HEAD"
    );
}

#[test]
fn get_rebase_todo_item_has_correct_fields() {
    let (ctx, oids) = make_three_commit_ctx();
    let base_oid = oids[0].to_string();

    let items = ctx.get_rebase_todo(&base_oid, false).unwrap();

    let item = &items[0];
    assert_eq!(
        item.oid,
        oids[1].to_string(),
        "OID should match second commit"
    );
    assert_eq!(
        item.short_oid,
        &oids[1].to_string()[..7],
        "short_oid should be first 7 chars"
    );
    assert_eq!(item.summary, "Second commit");
    assert_eq!(item.author_name, "Test User");
    assert!(
        item.author_timestamp > 0,
        "author_timestamp should be positive"
    );
}

#[test]
fn get_fork_point_returns_merge_base() {
    let (ctx, oids) = make_three_commit_ctx();

    // Create a branch at the initial commit
    {
        let repo = ctx.repo();
        let initial_commit = repo.find_commit(oids[0]).unwrap();
        repo.branch("feature", &initial_commit, false).unwrap();
    }

    let result = ctx.get_fork_point("feature").unwrap();

    assert_eq!(
        result,
        oids[0].to_string(),
        "Fork point should be the initial commit (merge-base of feature and HEAD)"
    );
}
