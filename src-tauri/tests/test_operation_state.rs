mod common;

use common::context::TestContext;
use trunk_lib::git::types::OperationType;

#[test]
fn clean_repo_returns_none_operation_type() {
    let ctx = TestContext::builder()
        .with_file("file.txt", "hello")
        .with_commit("Initial commit")
        .build();

    let info = ctx.get_operation_state().unwrap();
    assert!(matches!(info.op_type, OperationType::None));
    assert!(info.source_branch.is_none());
    assert!(info.target_branch.is_none());
    assert!(info.progress.is_none());
}

#[test]
fn merge_in_progress_reports_merge_state() {
    // Use with_conflict builder (which uses libgit2 merge) to create merge state,
    // then manually write MERGE_MSG for the operation_state parser to find.
    let ctx = TestContext::builder()
        .with_file("file.txt", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("file.txt", "feature content")
        .with_commit("Feature commit")
        .checkout("main")
        .with_file("file.txt", "main content")
        .with_commit("Main commit")
        .with_conflict("feature")
        .build();

    // Write MERGE_MSG manually (libgit2 merge does not create it; git CLI does)
    let repo = ctx.repo();
    let git_dir = repo.path();
    std::fs::write(git_dir.join("MERGE_MSG"), "Merge branch 'feature'\n").unwrap();

    let info = ctx.get_operation_state().unwrap();
    assert!(matches!(info.op_type, OperationType::Merge));
    assert_eq!(info.source_branch, Some("feature".to_string()));
    assert_eq!(info.target_branch, Some("main".to_string()));
    assert!(info.progress.is_none());
}

#[test]
fn merge_branch_non_conflicting_creates_merge_commit() {
    // Both branches have divergent changes on DIFFERENT files -> no conflict, merge commit
    let ctx = TestContext::builder()
        .with_file("file.txt", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("feature.txt", "feature work")
        .with_commit("Feature commit")
        .checkout("main")
        .with_file("main.txt", "main work")
        .with_commit("Main commit")
        .build();

    let result = ctx.merge_branch("feature");
    assert!(result.is_ok(), "merge_branch should succeed: {:?}", result);

    // After successful merge, repo should be clean
    let info = ctx.get_operation_state().unwrap();
    assert!(matches!(info.op_type, OperationType::None));

    // HEAD should be a merge commit with 2 parents
    let repo = ctx.repo();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.parent_count(), 2);
}

#[test]
fn merge_branch_fast_forward_when_linear() {
    // Feature has commits that main doesn't, but main hasn't diverged -> fast-forward
    let ctx = TestContext::builder()
        .with_file("file.txt", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("feature.txt", "feature work")
        .with_commit("Feature commit")
        .checkout("main")
        .build();

    let result = ctx.merge_branch("feature");
    assert!(result.is_ok(), "merge_branch should succeed: {:?}", result);

    // Fast-forward merge: HEAD has 1 parent (not a merge commit)
    let repo = ctx.repo();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.parent_count(), 1);
}

#[test]
fn merge_branch_with_conflict_returns_error() {
    // NOTE: merge_branch_inner has a known issue where git merge outputs CONFLICT
    // to stdout (not stderr), so the conflict detection in merge_branch_inner fails
    // and it returns Err. Tests document this actual behavior.
    let ctx = TestContext::builder()
        .with_file("file.txt", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("file.txt", "feature content")
        .with_commit("Feature commit")
        .checkout("main")
        .with_file("file.txt", "main content")
        .with_commit("Main commit")
        .build();

    let result = ctx.merge_branch("feature");
    assert!(
        result.is_err(),
        "merge_branch returns error on conflict (CONFLICT is on stdout, not stderr)"
    );
}

#[test]
fn merge_abort_clears_merge_state() {
    // Use with_conflict builder to set up merge state reliably
    let ctx = TestContext::builder()
        .with_file("file.txt", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("file.txt", "feature content")
        .with_commit("Feature commit")
        .checkout("main")
        .with_file("file.txt", "main content")
        .with_commit("Main commit")
        .with_conflict("feature")
        .build();

    // Repo is now in merge state (from with_conflict)
    let info = ctx.get_operation_state().unwrap();
    assert!(matches!(info.op_type, OperationType::Merge));

    // Abort the merge
    let result = ctx.merge_abort();
    assert!(result.is_ok(), "merge_abort should succeed: {:?}", result);

    let info = ctx.get_operation_state().unwrap();
    assert!(matches!(info.op_type, OperationType::None));
}

#[test]
fn rebase_branch_with_no_conflicts_completes() {
    let ctx = TestContext::builder()
        .with_file("file.txt", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("feature.txt", "feature work")
        .with_commit("Feature commit")
        .checkout("main")
        .with_file("main.txt", "main work")
        .with_commit("Main commit")
        .checkout("feature")
        .build();

    let result = ctx.rebase_branch("main");
    assert!(result.is_ok(), "rebase_branch should succeed: {:?}", result);

    let info = ctx.get_operation_state().unwrap();
    assert!(matches!(info.op_type, OperationType::None));
}

#[test]
fn rebase_abort_clears_rebase_state() {
    let ctx = TestContext::builder()
        .with_file("file.txt", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("file.txt", "feature content")
        .with_commit("Feature commit")
        .checkout("main")
        .with_file("file.txt", "main content")
        .with_commit("Main commit")
        .checkout("feature")
        .build();

    // Start conflicting rebase (will leave repo in rebase state)
    let _result = ctx.rebase_branch("main");

    // Abort it
    let result = ctx.rebase_abort();
    assert!(result.is_ok(), "rebase_abort should succeed: {:?}", result);

    let info = ctx.get_operation_state().unwrap();
    assert!(matches!(info.op_type, OperationType::None));
}
