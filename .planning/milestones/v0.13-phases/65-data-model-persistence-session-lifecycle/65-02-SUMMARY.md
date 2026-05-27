---
phase: 65-data-model-persistence-session-lifecycle
plan: 02
subsystem: persistence
tags: [rust, std-fs, atomic-write, fnv-1a, serde-json, recovery, review-session]

# Dependency graph
requires:
  - "ReviewSession / Comment / Anchor / DraftComment Rust DTOs (Serialize + Deserialize) from 65-01"
provides:
  - "review_store::save_session — atomic tmp+sync_all+rename per-repo JSON write (D-10)"
  - "review_store::load_session — schema-peek recovery state machine returning LoadOutcome (D-15/D-16)"
  - "review_store::delete_session — idempotent hard-delete (D-13 / SESS-03 primitive)"
  - "review_store::session_exists — resume-detection predicate (D-14)"
  - "LoadOutcome enum { Loaded, None, RecoveredCorrupt, RefusedNewer }"
  - "private session_filename — build-stable FNV-1a hash of canonical path (D-11 / path-traversal mitigation)"
  - "TestContext::data_dir() — TempDir accessor standing in for app_data_dir (testability wedge)"
affects: [65-03-session-lifecycle-commands, review-commands]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Atomic local write: tmp-in-same-dir + sync_all + fs::rename (rename atomic only within one filesystem)"
    - "Build-stable on-disk identifier via inline FNV-1a (NEVER std::hash::DefaultHasher — not version-stable)"
    - "serde_json::Value schema_version peek before full deserialize (distinguishes newer-schema from corrupt)"
    - "Testability wedge: I/O fn takes data_dir: &Path so tests inject a tempfile::TempDir"
    - "Private fn tested via in-module #[cfg(test)] mod tests { use super::*; } (encapsulation preserved)"

key-files:
  created:
    - src-tauri/src/git/review_store.rs
    - src-tauri/tests/test_review.rs
  modified:
    - src-tauri/src/git/mod.rs
    - src-tauri/tests/common/context.rs

key-decisions:
  - "Round-trip test compares via serde_json::to_value structural equality (no PartialEq on ReviewSession), continuing the 65-01 decision rather than forcing a new derive onto production structs."
  - "Integration tests discover the on-disk file via read_dir(sessions/) instead of reaching for the private session_filename — keeping the hash function encapsulated while the file path stays observable."
  - "Filename layout locked: app_data_dir/sessions/<16-hex FNV-1a>.json, with .json.tmp (transient) and .json.corrupt (quarantine) sidecars deriving via Path::with_extension."

patterns-established:
  - "Recovery state machine returns a LoadOutcome enum so the caller (65-03) decides fresh-session vs. warn-and-leave per D-15/D-16 — the store never crashes or silently destroys a file."
  - "delete_session treats NotFound as idempotent success so SESS-03 end-and-clear is safe to call repeatedly."

requirements-completed: [SESS-02]

# Metrics
duration: ~5min
completed: 2026-05-25
---

# Phase 65 Plan 02: Review-Session Persistence Store Summary

**`review_store` persists a per-repo review session atomically to `app_data_dir/sessions/<FNV-1a hash>.json`, loads it back equal, quarantines a corrupt file to a `.corrupt` sidecar without data loss, refuses a newer-schema file byte-untouched, and creates its directory on first write — the one phase-65 primitive with no codebase analog.**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-05-25T09:20:32Z
- **Completed:** 2026-05-25
- **Tasks:** 3 (1 auto harness + TDD RED + TDD GREEN)
- **Files created:** 2 (`review_store.rs`, `test_review.rs`)
- **Files modified:** 2 (`git/mod.rs`, `tests/common/context.rs`)

## Accomplishments
- Built the net-new persistence primitive: atomic write (D-10), build-stable FNV-1a filename hash that is also the path-traversal mitigation (D-11), and a load state machine that recovers corrupt files (D-15) and refuses newer-schema files untouched (D-16).
- Added the `data_dir()` testability wedge to the existing `TestContext` harness so all disk behavior is proven by `tempfile::TempDir`-injecting integration tests — no Tauri state, no real `app_data_dir`.
- Kept `session_filename` private and proved its build-stability with an in-module test (the only place that can legally call it), so encapsulation is preserved while the security-relevant hash is still under test.

## Task Commits

Each task was committed atomically:

1. **Task 1: data_dir() test-harness accessor (auto)** - `e28eff7` (test)
2. **Task 2: persistence tests (TDD RED)** - `ec07f18` (test)
3. **Task 3: review_store.rs implementation (TDD GREEN)** - `9642eb6` (feat)

**Plan metadata:** (this docs commit)

_TDD gate sequence: test (RED) → feat (GREEN). No REFACTOR commit — the module follows the RESEARCH code examples directly and reads cleanly._

## Files Created/Modified
- `src-tauri/src/git/review_store.rs` (NEW) - `atomic_write_json`, private `session_filename` (FNV-1a), `save_session`, `load_session` (recovery state machine), `quarantine_corrupt`, `delete_session`, `session_exists`, `LoadOutcome` enum, and the in-module `same_canonical_path_same_file` stability test.
- `src-tauri/tests/test_review.rs` (NEW) - five public-API integration tests: `session_round_trips`, `first_write_creates_dir`, `atomic_write_clean`, `corrupt_quarantined`, `newer_version_refused`.
- `src-tauri/src/git/mod.rs` - registered `pub mod review_store;`.
- `src-tauri/tests/common/context.rs` - added a second `TempDir` field + `data_dir()` accessor (initialized in `new_empty()` and `from_parts`).

## Decisions Made
- **Structural-equality round-trip (no PartialEq):** `session_round_trips` compares `serde_json::to_value(&loaded)` against `serde_json::to_value(&session)`, continuing the 65-01 decision not to add a `PartialEq` derive that production code does not need.
- **File discovery via read_dir, not session_filename:** Integration tests live in the separate `src-tauri/tests/` crate and cannot reach the private `session_filename`. They locate the on-disk file by scanning `sessions/` for the single `.json`, keeping the hash function encapsulated while the file path remains an observable behavior. Filename-stability is verified separately by the in-module unit test.
- **Filename + sidecar layout locked:** `app_data_dir/sessions/<16-hex>.json`; `.json.tmp` transient during write; `.json.corrupt` on quarantine. All derive cleanly via `Path::with_extension("json.tmp"|"json.corrupt")`.

## Deviations from Plan
None - plan executed exactly as written. No deviation rules triggered.

## Threat Model Compliance
All five STRIDE register entries from the plan are mitigated and tested:
- **T-65-02-PT (path traversal):** FNV-1a `{:016x}.json` token — no `..`/separators/verbatim prefix can reach the filename. Filename derivation proven build-stable by `same_canonical_path_same_file`.
- **T-65-02-DOS (corrupt JSON):** `Result`-based load, no `unwrap`; unparseable → quarantine + `RecoveredCorrupt`. Proven by `corrupt_quarantined`.
- **T-65-02-DL (downgrade data loss):** `schema_version` peek before deserialize; `> 1` → `RefusedNewer`, file left byte-identical. Proven by `newer_version_refused` (read-before/read-after `assert_eq`).
- **T-65-02-AV (durability):** tmp + `sync_all` + `rename`; no `.tmp` residue. Proven by `atomic_write_clean`.
- **T-65-02-SC (supply chain):** No new packages — `serde`, `serde_json`, `std`, `tempfile` already present. No install task.

## Known Stubs
None. `delete_session` and `session_exists` are fully implemented and tested-by-construction (used by the recovery flow / locked API); they are wired into Tauri commands in Plan 65-03 as designed — not stubs.

## Issues Encountered
- macOS tempdirs resolve through a `/var` → `/private/var` symlink, so tests canonicalize `ctx.repo_path()` before passing it as the `canonical` arg, ensuring save and load agree on the same hash. No code impact — this is exactly the canonicalization the production caller (65-03) will perform.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plan 65-03 (lifecycle commands) can now build `commands/review.rs` thin commands over `_inner(data_dir: &Path, ...)` that call `save_session` / `load_session` / `delete_session` / `session_exists`, resolving `data_dir` from `app.path().app_data_dir()` and keying `ReviewSessionsState` by the canonical `PathBuf`.
- No blockers. The recovery contract (`LoadOutcome`) is the seam between the store and the command layer's D-15/D-16 user-facing behavior (toast on corrupt, warn-and-leave on newer).

## TDD Gate Compliance
- RED gate: `ec07f18` (test) — five integration tests fail to compile (`unresolved import trunk_lib::git::review_store`).
- GREEN gate: `9642eb6` (feat) — all five integration tests + the in-module stability test pass.
- Sequence verified: test commit precedes feat commit.

## Self-Check: PASSED
- Files verified present: `src-tauri/src/git/review_store.rs`, `src-tauri/tests/test_review.rs`, `src-tauri/src/git/mod.rs`, `src-tauri/tests/common/context.rs`, `65-02-SUMMARY.md`.
- Commits verified in git log: `e28eff7` (harness), `ec07f18` (RED), `9642eb6` (GREEN).
- Symbols verified: `pub fn save_session`, `pub fn load_session`, `pub enum LoadOutcome`, private `fn session_filename`, `fn session_round_trips`, `fn same_canonical_path_same_file`.
- `just check` green (Rust tests + clippy + fmt + svelte-check + 456 vitest).

---
*Phase: 65-data-model-persistence-session-lifecycle*
*Completed: 2026-05-25*
