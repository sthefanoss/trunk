mod common;

use common::context::TestContext;
use trunk_lib::git::repository::build_ref_map;
use trunk_lib::git::types::RefType;

#[test]
fn ref_map_head() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("feature.txt", "feature work")
        .with_commit("Feature commit")
        .checkout("main")
        .merge("feature")
        .build();

    let mut repo = ctx.repo();
    let map = build_ref_map(&mut repo);
    assert!(
        map.values().flatten().any(|r| r.is_head),
        "expected at least one ref with is_head == true"
    );
}

#[test]
fn ref_map_stash() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("feature.txt", "feature work")
        .with_commit("Feature commit")
        .checkout("main")
        .merge("feature")
        .with_stash(Some("test stash"))
        .build();

    let mut repo = ctx.repo();
    let map = build_ref_map(&mut repo);
    assert!(
        map.values()
            .flatten()
            .any(|r| matches!(r.ref_type, RefType::Stash)),
        "expected at least one RefLabel with ref_type == Stash"
    );
}
