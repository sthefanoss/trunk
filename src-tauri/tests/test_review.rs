mod common;

use common::context::TestContext;
use std::fs;
use std::path::{Path, PathBuf};
use trunk_lib::git::review_store::{load_session, save_session, LoadOutcome};
use trunk_lib::git::types::ReviewSession;

fn empty_session() -> ReviewSession {
    ReviewSession {
        schema_version: 1,
        commits: vec!["abc123".to_string()],
        comments: vec![],
        draft_comment: None,
    }
}

/// The single `.json` file in the sessions dir (panics if not exactly one).
fn the_session_file(data_dir: &Path) -> PathBuf {
    let entries: Vec<PathBuf> = fs::read_dir(data_dir.join("sessions"))
        .expect("sessions dir should exist after a save")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .collect();
    assert_eq!(entries.len(), 1, "expected exactly one .json session file");
    entries.into_iter().next().unwrap()
}

#[test]
fn session_round_trips() {
    let ctx = TestContext::new_empty();
    let canonical = ctx.repo_path().canonicalize().unwrap();
    let session = empty_session();

    save_session(ctx.data_dir(), &canonical, &session).unwrap();
    let outcome = load_session(ctx.data_dir(), &canonical).unwrap();

    let LoadOutcome::Loaded(loaded) = outcome else {
        panic!("expected Loaded, got a different outcome");
    };
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&session).unwrap(),
    );
}

#[test]
fn first_write_creates_dir() {
    let ctx = TestContext::new_empty();
    let canonical = ctx.repo_path().canonicalize().unwrap();

    assert!(
        !ctx.data_dir().join("sessions").exists(),
        "sessions dir must not exist before the first save",
    );

    save_session(ctx.data_dir(), &canonical, &empty_session()).unwrap();

    assert!(
        ctx.data_dir().join("sessions").is_dir(),
        "first save should create the sessions dir",
    );
}

#[test]
fn atomic_write_clean() {
    let ctx = TestContext::new_empty();
    let canonical = ctx.repo_path().canonicalize().unwrap();

    save_session(ctx.data_dir(), &canonical, &empty_session()).unwrap();

    let session_file = the_session_file(ctx.data_dir());
    let raw = fs::read_to_string(&session_file).unwrap();
    serde_json::from_str::<serde_json::Value>(&raw).expect("session file should be valid JSON");

    let leftover_tmp = fs::read_dir(ctx.data_dir().join("sessions"))
        .unwrap()
        .any(|e| e.unwrap().path().to_string_lossy().ends_with(".json.tmp"));
    assert!(
        !leftover_tmp,
        "no .tmp file should remain after a clean save"
    );
}

#[test]
fn corrupt_quarantined() {
    let ctx = TestContext::new_empty();
    let canonical = ctx.repo_path().canonicalize().unwrap();

    save_session(ctx.data_dir(), &canonical, &empty_session()).unwrap();
    let session_file = the_session_file(ctx.data_dir());
    fs::write(&session_file, b"}}}not valid json{{{").unwrap();

    let outcome = load_session(ctx.data_dir(), &canonical).unwrap();

    assert!(matches!(outcome, LoadOutcome::RecoveredCorrupt));
    let corrupt_sidecar = session_file.with_extension("json.corrupt");
    assert!(
        corrupt_sidecar.exists(),
        ".corrupt sidecar should exist after quarantine",
    );
    assert!(
        !session_file.exists(),
        "original .json should be gone after quarantine",
    );
}

#[test]
fn newer_version_refused() {
    let ctx = TestContext::new_empty();
    let canonical = ctx.repo_path().canonicalize().unwrap();

    save_session(ctx.data_dir(), &canonical, &empty_session()).unwrap();
    let session_file = the_session_file(ctx.data_dir());
    fs::write(
        &session_file,
        br#"{"schema_version":2,"commits":[],"comments":[],"draft_comment":null}"#,
    )
    .unwrap();
    let before = fs::read(&session_file).unwrap();

    let outcome = load_session(ctx.data_dir(), &canonical).unwrap();

    assert!(matches!(outcome, LoadOutcome::RefusedNewer));
    let after = fs::read(&session_file).unwrap();
    assert_eq!(
        before, after,
        "a refused newer-version file must be left byte-identical"
    );
}
