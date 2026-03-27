//! Integration tests: Multi-step git workflow sequences
//! Per D-04, these compose existing Phase 53 drivers into realistic user flows.
//! Individual commands are already unit-tested; these verify correct composition.

mod common;
use common::context::TestContext;
use trunk_lib::git::types::OperationType;

// -- Workflow tests --

#[test]
fn workflow_edit_stage_commit_cycle() {
    let ctx = TestContext::builder()
        .with_file("README.md", "initial content")
        .with_commit("Initial commit")
        .build();

    // Edit the file
    std::fs::write(ctx.repo_path().join("README.md"), "updated content").unwrap();

    // Verify file appears in unstaged
    let status = ctx.get_status().unwrap();
    assert!(
        status.unstaged.iter().any(|f| f.path == "README.md"),
        "README.md should appear in unstaged after edit"
    );

    // Stage the file
    ctx.stage_file("README.md").unwrap();
    ctx.assert_file_staged("README.md");

    // Commit
    ctx.create_commit("Update readme", None).unwrap();
    ctx.assert_status_clean();
    ctx.assert_head_message("Update readme");
    ctx.assert_commit_count(2);
}

#[test]
fn workflow_branch_commit_merge() {
    let mut ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    // Create and checkout feature branch
    ctx.create_branch("feature", None).unwrap();
    ctx.checkout_branch("feature").unwrap();
    ctx.assert_head_at("feature");

    // Write new file, stage, commit on feature
    std::fs::write(ctx.repo_path().join("feature.txt"), "feature work").unwrap();
    ctx.stage_file("feature.txt").unwrap();
    ctx.create_commit("Add feature file", None).unwrap();

    // Switch back to main and add a commit so branches diverge (forces real merge)
    ctx.checkout_branch("main").unwrap();
    std::fs::write(ctx.repo_path().join("main.txt"), "main work").unwrap();
    ctx.stage_file("main.txt").unwrap();
    ctx.create_commit("Add main file", None).unwrap();

    let result = ctx.merge_branch("feature");
    assert!(result.is_ok(), "merge_branch should succeed: {:?}", result);

    // Verify: feature file exists on main
    ctx.assert_file_content("feature.txt", "feature work");
    ctx.assert_head_at("main");
    // initial + main commit + feature commit + merge commit = 4
    ctx.assert_commit_count(4);
}

#[test]
fn workflow_stash_save_checkout_pop() {
    let mut ctx = TestContext::builder()
        .with_file("README.md", "original content")
        .with_commit("Initial commit")
        .with_branch("feature")
        .build();

    // Modify tracked file
    std::fs::write(ctx.repo_path().join("README.md"), "modified content").unwrap();

    // Stash the changes
    ctx.stash_save("work in progress").unwrap();
    ctx.assert_status_clean();

    // Switch to feature and back
    ctx.checkout_branch("feature").unwrap();
    ctx.assert_head_at("feature");
    ctx.checkout_branch("main").unwrap();
    ctx.assert_head_at("main");

    // Pop the stash
    ctx.stash_pop(0).unwrap();

    // Verify: file has modified content and stash list is empty
    ctx.assert_file_content("README.md", "modified content");
    let stashes = ctx.list_stashes().unwrap();
    assert!(stashes.is_empty(), "stash list should be empty after pop");
}

#[test]
fn workflow_cherry_pick_from_branch() {
    let mut ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .build();

    // Checkout feature, create new file, commit
    ctx.checkout_branch("feature").unwrap();
    std::fs::write(ctx.repo_path().join("cherry.txt"), "cherry content").unwrap();
    ctx.stage_file("cherry.txt").unwrap();
    ctx.create_commit("Add cherry file", None).unwrap();

    // Get the feature commit OID
    let oid = ctx.resolve_ref("feature").unwrap();

    // Switch back to main
    ctx.checkout_branch("main").unwrap();
    ctx.assert_head_at("main");

    // Cherry-pick the feature commit
    let result = ctx.cherry_pick(&oid);
    assert!(
        result.is_ok(),
        "cherry_pick should succeed: {:?}",
        result.err()
    );

    // Verify: cherry-picked file exists on main
    ctx.assert_file_content("cherry.txt", "cherry content");
    ctx.assert_head_at("main");
    // initial + cherry-picked = 2 (no merge commit)
    ctx.assert_commit_count(2);
}

#[test]
fn workflow_tag_and_branch_management() {
    let mut ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .with_branch("temp")
        .build();

    // Get HEAD oid for tagging
    let oid = ctx.resolve_ref("main").unwrap();

    // Create tag
    ctx.create_tag(&oid, "v1.0", "Release v1.0").unwrap();
    ctx.assert_tag_exists("v1.0");

    // Delete branch
    ctx.delete_branch("temp").unwrap();
    ctx.assert_branch_not_exists("temp");

    // Verify refs
    let refs = ctx.list_refs().unwrap();
    assert!(
        refs.tags.iter().any(|t| t.short_name == "v1.0"),
        "tags should contain v1.0"
    );
    assert!(
        !refs.local.iter().any(|b| b.name == "temp"),
        "branches should not contain temp"
    );

    // Delete tag and verify
    ctx.delete_tag("v1.0").unwrap();
    let refs = ctx.list_refs().unwrap();
    assert!(
        !refs.tags.iter().any(|t| t.short_name == "v1.0"),
        "tags should not contain v1.0 after deletion"
    );
}

#[test]
fn workflow_undo_redo_commit() {
    let ctx = TestContext::builder()
        .with_file("README.md", "first")
        .with_commit("First")
        .with_file("README.md", "second")
        .with_commit("Second")
        .build();

    // Verify undo is available
    assert!(
        ctx.check_undo_available().unwrap(),
        "undo should be available with 2 commits"
    );

    // Undo the last commit
    let undo_result = ctx.undo_commit().unwrap();
    ctx.assert_head_message("First");
    assert_eq!(undo_result.subject, "Second");

    // Redo the undone commit
    ctx.redo_commit(&undo_result.subject, undo_result.body.as_deref())
        .unwrap();
    ctx.assert_head_message("Second");
}

#[test]
fn workflow_diff_staging_cycle() {
    let ctx = TestContext::builder()
        .with_file("code.txt", "line1\nline2\n")
        .with_commit("Initial commit")
        .build();

    // Modify the file
    std::fs::write(ctx.repo_path().join("code.txt"), "line1\nline2\nline3\n").unwrap();

    // Check unstaged diff has the addition
    let unstaged_diff = ctx.diff_unstaged("code.txt").unwrap();
    assert_eq!(unstaged_diff.len(), 1, "should have 1 file diff");
    assert!(
        !unstaged_diff[0].hunks.is_empty(),
        "should have at least 1 hunk"
    );

    // Stage the file
    ctx.stage_file("code.txt").unwrap();

    // Staged diff should show the same change
    let staged_diff = ctx.diff_staged("code.txt").unwrap();
    assert_eq!(staged_diff.len(), 1, "should have 1 staged file diff");
    assert!(
        !staged_diff[0].hunks.is_empty(),
        "staged diff should have at least 1 hunk"
    );

    // Unstaged diff should be empty now
    let unstaged_after = ctx.diff_unstaged("code.txt").unwrap();
    assert!(
        unstaged_after.is_empty() || unstaged_after[0].hunks.is_empty(),
        "unstaged diff should be empty after staging"
    );

    // Unstage the file
    ctx.unstage_file("code.txt").unwrap();

    // Unstaged diff should be back
    let unstaged_restored = ctx.diff_unstaged("code.txt").unwrap();
    assert_eq!(
        unstaged_restored.len(),
        1,
        "should have 1 file diff after unstaging"
    );
    assert!(
        !unstaged_restored[0].hunks.is_empty(),
        "unstaged diff should have hunks after unstaging"
    );
}

#[test]
fn workflow_search_commit_history() {
    let mut ctx = TestContext::builder()
        .with_file("README.md", "init")
        .with_commit("Initial setup")
        .with_file("feature.txt", "feature")
        .with_commit("Add feature X")
        .with_file("bugfix.txt", "fix")
        .with_commit("Fix bug in Y")
        .build();

    // Populate the cache before searching
    ctx.populate_cache();

    // Search for "feature"
    let results = ctx.search_commits("feature").unwrap();
    assert_eq!(results.len(), 1, "should find 1 commit matching 'feature'");

    // Search for "bug"
    let results = ctx.search_commits("bug").unwrap();
    assert_eq!(results.len(), 1, "should find 1 commit matching 'bug'");

    // Search for author "Test User" (all commits should match)
    let results = ctx.search_commits("Test User").unwrap();
    assert_eq!(
        results.len(),
        3,
        "should find 3 commits matching author 'Test User'"
    );
}
