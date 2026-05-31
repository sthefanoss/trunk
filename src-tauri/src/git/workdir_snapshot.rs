//! Capture the current working tree as a REAL but DANGLING commit so it can be
//! reviewed exactly like a hand-picked commit. The snapshot commit's own tree is
//! the working tree (staged + unstaged + untracked-not-ignored); its parent is
//! HEAD. The existing review pipeline then resolves `Side::Old` against the
//! parent tree (= HEAD) and `Side::New` against the snapshot tree (= working
//! tree) — precisely "before vs after" for uncommitted work, with no new
//! Source/Side variant.

use crate::error::TrunkError;

/// Snapshot the current working tree into a dangling commit (parent = HEAD) and
/// return its Oid.
///
/// No-clobber rationale: the working tree is captured through a THROWAWAY index
/// built with `git2::Index::new()` + `repo.set_index(..)`. `set_index` only
/// associates the in-memory index with the repo so `add_all` can resolve the
/// workdir; `write_tree_to` only writes tree objects to the ODB. Neither calls
/// `index.write()` — the ONLY call that persists `.git/index` to disk. We never
/// call `index.write()`, so the user's real index is byte-for-byte untouched.
pub fn snapshot_working_tree(repo: &git2::Repository) -> Result<git2::Oid, TrunkError> {
    // 1. Associate an EMPTY in-memory index with the repo. Starting from empty +
    //    add_all("*") captures the full current workdir (staged + unstaged +
    //    untracked-not-ignored) in one shot, independent of what is staged.
    let mut idx = git2::Index::new()?;
    repo.set_index(&mut idx)?;

    // 2. Re-fetch the now-associated index and add the whole workdir.
    //    IndexAddOption::DEFAULT respects .gitignore: it includes
    //    untracked-but-not-ignored files and excludes ignored ones (identical to
    //    the shipped call at commands/staging.rs:344). NEVER call idx.write().
    let mut idx = repo.index()?;
    idx.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;

    // 3. Write the tree objects to the ODB (does NOT persist the on-disk index).
    let tree = repo.find_tree(idx.write_tree_to(repo)?)?;

    // 4. Resolve the parent: HEAD's commit, or none when HEAD is unborn (a fresh
    //    repo with zero commits still snapshots fine — a parent-less commit).
    let head_commit = if is_head_unborn(repo) {
        None
    } else {
        Some(repo.head()?.peel_to_commit()?)
    };
    let parents: Vec<&git2::Commit> = head_commit.iter().collect();

    // 5. Descriptive message only — the snapshot is tracked by OID in the session
    //    field, never by parsing this string.
    let sig = git2::Signature::now("Trunk", "review@trunk.local")?;
    let msg = format!("Uncommitted changes — {}", sig.when().seconds());

    // 6. `None` target ref => DANGLING commit (no ref litter; GC degradation is
    //    the accepted locked tradeoff).
    let oid = repo.commit(None, &sig, &sig, &msg, &tree, &parents)?;
    Ok(oid)
}

/// HEAD is unborn when the repo has no commits yet (freshly init'd). Mirrors the
/// probe at commands/diff.rs:25.
fn is_head_unborn(repo: &git2::Repository) -> bool {
    match repo.head() {
        Err(e) => e.code() == git2::ErrorCode::UnbornBranch,
        Ok(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn sig() -> git2::Signature<'static> {
        git2::Signature::new("Test", "test@example.com", &git2::Time::new(0, 0)).unwrap()
    }

    /// Init a repo and write one committed file on HEAD so the snapshot has a
    /// real parent. Returns the TempDir (keep alive) and the open repo.
    fn repo_with_initial_commit() -> (TempDir, git2::Repository) {
        let dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("committed.txt"), b"original\n").unwrap();
        {
            let mut index = repo.index().unwrap();
            index
                .add_path(std::path::Path::new("committed.txt"))
                .unwrap();
            index.write().unwrap();
            let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
            let s = sig();
            repo.commit(Some("HEAD"), &s, &s, "initial", &tree, &[])
                .unwrap();
        }
        (dir, repo)
    }

    fn tree_contains(repo: &git2::Repository, oid: git2::Oid, path: &str) -> bool {
        let commit = repo.find_commit(oid).unwrap();
        let tree = commit.tree().unwrap();
        tree.get_path(std::path::Path::new(path)).is_ok()
    }

    // Test A: an untracked-but-not-ignored workdir file IS present in the snapshot.
    #[test]
    fn untracked_file_is_included_in_snapshot() {
        let (dir, repo) = repo_with_initial_commit();
        fs::write(dir.path().join("new.txt"), b"uncommitted\n").unwrap();

        let oid = snapshot_working_tree(&repo).unwrap();

        assert!(
            tree_contains(&repo, oid, "new.txt"),
            "untracked-not-ignored file must appear in the snapshot tree"
        );
    }

    // Test B: a file matching a .gitignore pattern is NOT present in the snapshot.
    #[test]
    fn ignored_file_is_excluded_from_snapshot() {
        let (dir, repo) = repo_with_initial_commit();
        fs::write(dir.path().join(".gitignore"), b"secret.txt\n").unwrap();
        fs::write(dir.path().join("secret.txt"), b"do not capture\n").unwrap();

        let oid = snapshot_working_tree(&repo).unwrap();

        assert!(
            !tree_contains(&repo, oid, "secret.txt"),
            "a .gitignore-matched file must NOT appear in the snapshot tree"
        );
        assert!(
            tree_contains(&repo, oid, ".gitignore"),
            "the .gitignore itself (not ignored) should still be captured"
        );
    }

    // Test C: the user's real .git/index is byte-for-byte unchanged by the call.
    #[test]
    fn real_index_is_untouched() {
        let (dir, repo) = repo_with_initial_commit();
        // The initial commit wrote a real .git/index. Capture its bytes.
        let index_path = dir.path().join(".git").join("index");
        let before = fs::read(&index_path).expect("repo with a commit has a .git/index");

        // Stage nothing extra; just snapshot a dirty workdir.
        fs::write(dir.path().join("new.txt"), b"uncommitted\n").unwrap();
        snapshot_working_tree(&repo).unwrap();

        let after = fs::read(&index_path).expect(".git/index must still exist");
        assert_eq!(
            before, after,
            "snapshot must NOT persist anything to the real .git/index"
        );
    }

    // Test D: a freshly init'd repo with zero commits (unborn HEAD) snapshots
    // without error and yields a parent-less commit reflecting the workdir.
    #[test]
    fn unborn_head_snapshots_without_parent() {
        let dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("first.txt"), b"hello\n").unwrap();

        let oid = snapshot_working_tree(&repo).unwrap();

        let commit = repo.find_commit(oid).unwrap();
        assert_eq!(
            commit.parent_count(),
            0,
            "unborn-HEAD snapshot must have no parent"
        );
        assert!(
            tree_contains(&repo, oid, "first.txt"),
            "the snapshot tree must reflect the workdir even with no commits yet"
        );
    }
}
