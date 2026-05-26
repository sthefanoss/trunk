---
phase: 70-excerpt-resolution-markdown-render
plan: 02
subsystem: review
tags: [rust, tauri-command, ipc, review]

# Dependency graph
requires:
  - phase: 70-01
    provides: pub fn render(session, repo) -> String + pub(crate) classify_anchor
  - phase: 65-review-schema-keystone
    provides: ReviewSession (Clone), Comment, RepoState, ReviewSessionsState
  - phase: 69-comment-management
    provides: canonical_repo_path helper + clone-under-lock + spawn_blocking pattern
provides:
  - Tauri command generate_review_doc(path) -> Result<String, String>
  - D-11 zero-comment gate at IPC layer (no_comments TrunkError)
  - IPC entry point for DOC-01 (one markdown doc per Generate click)
affects:
  - 70-03 (Svelte preview UI calls safeInvoke<string>("generate_review_doc", {path}))
  - 71 (Copy/Save UX layers on the same preview)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Clone-under-lock then spawn_blocking — never hold ReviewSessionsState mutex across git2 work (Pitfall 6)"
    - "D-11 gate at command layer keeps the pure renderer assume->=1 / panic-free"
    - "Read-only command: no session-changed emit, no mutate_session_rmw, no _inner (RESEARCH Q1 Option B)"

key-files:
  created:
    - .planning/phases/70-excerpt-resolution-markdown-render/70-02-SUMMARY.md
  modified:
    - src-tauri/src/commands/review.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "D-11 gate lives in the command, not the renderer. Pure renderer assumes >=1 comment; this command is the only invocation path so a single layer enforces the invariant."
  - "Clone the FULL ReviewSession (not just .comments) — the renderer reads .commits for the D-07 short-sha refs list. ReviewSession derives Clone."
  - "No _inner wrapper: there is no disk I/O, so no testable wedge is warranted. The pure renderer in git::review::render is the testable surface (TDD'd to 30 tests in Plan 70-01)."
  - "Doc-comment explicitly calls out Pitfall 5 stance: markdown injection in comment text is a DELIBERATE non-mitigation (recipient is an AI agent; escaping would hide signal)."

patterns-established:
  - "generate_review_doc mirrors resolve_session_comments verbatim — same (path, state, sessions) signature, same Result<T, String> JSON-error contract, same clone-under-lock + spawn_blocking shape. Only divergence: clone the WHOLE session (not just .comments) and add the D-11 gate."

requirements-completed: [DOC-01]

# Metrics
duration: ~15min
completed: 2026-05-26
---

# Phase 70 Plan 02: generate_review_doc Tauri Command Summary

**Wires the pure renderer from Plan 70-01 into Tauri's IPC layer. Adds one read-only command `generate_review_doc(path: String) -> Result<String, String>` that clones the active `ReviewSession`, enforces the D-11 zero-comment gate, opens `git2::Repository` inside `spawn_blocking`, calls `crate::git::review::render`, and returns the markdown string. Registered in the invoke_handler block.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-26
- **Completed:** 2026-05-26
- **Tasks:** 2
- **Files modified:** 2 (`src-tauri/src/commands/review.rs`, `src-tauri/src/lib.rs`)
- **Lines added:** 65 (64 command + doc-comment + 1 registration)

## Accomplishments

- `generate_review_doc` command appended to `src-tauri/src/commands/review.rs` immediately after `resolve_session_comments` — same structural shape (read-only, no `_inner`, clone-under-lock, spawn_blocking)
- Registration added to `src-tauri/src/lib.rs` `tauri::generate_handler!` block, after `resolve_session_comments,`
- D-11 zero-comment gate enforced at the command layer with code `no_comments`; renderer's panic-free assume->=1 invariant preserved
- `git2::Repository::open` called inside `spawn_blocking` (Pitfall 6 — `git2::Repository` is not `Sync`)
- `ReviewSessionsState` mutex never held across `spawn_blocking` (full session cloned out under the short lock scope)
- No `session-changed` emit; no `mutate_session_rmw`; no `_inner` variant (RESEARCH Q1 Option B)
- Doc-comment annotates the three non-mitigations so future agents do not accidentally "fix" them: (a) skip `_inner` rationale, (b) Pitfall 5 markdown-injection stance, (c) read-only contract

## Task Commits

Neither task carried `tdd="true"` in the plan frontmatter. See `## TDD Gate Compliance` below for the rationale.

1. **Task 1: Implement `generate_review_doc` command** — `e700b6b` (feat)
2. **Task 2: Register in `invoke_handler`** — `c9ac63a` (feat)

## Files Created/Modified

- `src-tauri/src/commands/review.rs` — appended `generate_review_doc` after `resolve_session_comments` (64 lines incl. doc-comment)
- `src-tauri/src/lib.rs` — one-line registration `commands::review::generate_review_doc,` after `resolve_session_comments,`
- `.planning/phases/70-excerpt-resolution-markdown-render/70-02-SUMMARY.md` — this file

## Decisions Made

- **D-11 gate placement at the command layer, not the renderer.** Single-source-of-truth invariant: the pure renderer assumes >=1 and has no defensive zero-comment branch. A non-UI invocation that bypasses the UI's D-01 disabled-button gating surfaces as `Err({code: "no_comments", ...})` rather than producing an empty doc.
- **Clone the FULL `ReviewSession`** (not just `.comments`). The renderer reads `session.commits` for the D-07 short-sha refs list. `ReviewSession` derives `Clone` (`src-tauri/src/git/types.rs:339`), so the clone is shallow at the struct level and deep over the contained `Vec`s — well-bounded for the session sizes in scope.
- **No `_inner` wrapper.** Documented inline in the function's doc-comment with a citation to RESEARCH Q1 Option B. The pure renderer is the testable surface (already TDD'd to 30 tests in Plan 70-01); inventing a disk-testability wedge here would add indirection without adding coverage.
- **Markdown injection in comment text is a deliberate non-mitigation** (Pitfall 5). The recipient is an AI coding agent; escaping `` ``` `` or headings inside user comments would suppress signal the reviewer placed there intentionally. Stance documented in the function's doc-comment so future agents do not "fix" it.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] `cargo fmt` reflowed the `let doc = tauri::async_runtime::spawn_blocking(...)` block**

- **Found during:** Task 1 verification (`just check` → fmt diff)
- **Issue:** The hand-written body wrapped `tauri::async_runtime::spawn_blocking(...)` onto a new line under the binding; `cargo fmt` prefers keeping the opening call on the binding line and de-indenting the trailing `.await`/`.map_err` chain.
- **Fix:** Ran `cargo fmt --manifest-path src-tauri/Cargo.toml`. No semantic change.
- **Files modified:** `src-tauri/src/commands/review.rs` (formatting only)
- **Verification:** `just check` passes; all plan grep gates still satisfied (the format change moved no load-bearing tokens out of their grep windows).
- **Committed in:** `e700b6b` (Task 1 feat — fmt applied before the commit).

**2. [Rule 3 — Blocking] Worktree `node_modules` empty; `npm install` required**

- **Found during:** Task 1 verification (`just check` → vitest "Cannot find module '@testing-library/svelte/src/vitest.js'")
- **Issue:** Same condition Plan 70-01's SUMMARY documented (`Issues Encountered` section). Claude Code worktrees do not inherit the parent's `node_modules`; vitest fails on first run.
- **Fix:** `npm install` populated `node_modules`; the install touched `package-lock.json` as a registry side-effect; restored with `git checkout -- package-lock.json` (no actual lockfile drift, only the install's transient metadata).
- **Files modified:** None permanently. `node_modules/` is gitignored; `package-lock.json` restored.
- **Verification:** `just check` exits 0 after the install (`517 vitest tests passed`).
- **Committed in:** N/A — environment fix only, not task work.

**3. [Rule 3 — Blocking] Plan's package spec `trunk_lib` is the lib-target name, not the crate name**

- **Found during:** Task 1 verification (`cargo build -p trunk_lib` → "package ID specification `trunk_lib` did not match any packages")
- **Issue:** The plan's `<verify>` block runs `cargo build -p trunk_lib --manifest-path src-tauri/Cargo.toml`. In `src-tauri/Cargo.toml` the package is named `trunk`; `trunk_lib` is only the `[lib]` target name within that package.
- **Fix:** Used `cargo build -p trunk --manifest-path src-tauri/Cargo.toml` for verification. Build succeeds; the lib target compiles as part of the package. No code change.
- **Files modified:** None.
- **Verification:** `cargo build -p trunk --manifest-path src-tauri/Cargo.toml` exits 0; `just check` (which runs the right invocation already) exits 0.
- **Committed in:** N/A — verification command adjustment, not task work. Surfacing as a planner-side documentation note for future plans in this repo (the project uses `cargo build -p trunk` end-to-end, never `-p trunk_lib`).

---

**Total deviations:** 3 auto-fixed (all blocking environmental / planner-doc gaps). No scope creep. Plan executed as written for both Rust source-code tasks.

## Issues Encountered

- The plan's `<verify>` block uses `cargo build -p trunk_lib`; the actual package name is `trunk`. Adjusted at verification time. Recorded above as Deviation 3 for planner feedback (this is the third Phase-70 plan that references the wrong package spec — worth standardising in PATTERNS.md before Plan 70-03).
- Worktree's `node_modules` empty on spawn — same as 70-01. Cheap one-time fix (`npm install`), but recurring across worktrees. Worth a worktree-bootstrap hook entry if this pattern continues.

## Known Stubs

None — `generate_review_doc` is fully wired: input validation, D-11 gate, session clone, spawn_blocking, render call, error serialization, command registration. Plan 70-03 (UI) can call `safeInvoke<string>("generate_review_doc", { path })` immediately.

## TDD Gate Compliance

Neither task in this plan carries `tdd="true"`. The plan deliberately omits an `_inner` testable wedge (RESEARCH Q1 Option B — no disk I/O means no disk-testability indirection is warranted), and the underlying renderer was already TDD'd to 30 tests in Plan 70-01.

This plan is the **structural wrapper** that wires the renderer into Tauri's IPC layer: it does not introduce new behaviour beyond:
- input validation (mirrors `resolve_session_comments` verbatim),
- the D-11 gate (a single `is_empty()` check),
- the spawn_blocking shape (mirrors `resolve_session_comments` verbatim),
- registration in `invoke_handler` (one line).

Acceptance is structural (`grep` gates on signature, gate code, spawn_blocking, render call) + `just check` (which runs `cargo clippy -D warnings` and the full backend `cargo test` suite on the existing 30 renderer tests). Manufacturing a contentless `test(70-02): ...` RED commit would add noise without adding coverage.

If the phase-level TDD-mode gate flags this, the rationale above is the answer: tasks are wrapper/wiring, not behaviour-adding; the behavioural surface was TDD'd one layer down in Plan 70-01.

## Verification Results

- `cargo build -p trunk --manifest-path src-tauri/Cargo.toml` → **exits 0** (lib target compiles)
- `cargo clippy -p trunk --manifest-path src-tauri/Cargo.toml -- -D warnings` → **exits 0** (via `just check`)
- `just check` → **exits 0** (fmt + biome + svelte-check + clippy + cargo-test + 517 vitest tests)
- `grep -c 'pub async fn generate_review_doc' src-tauri/src/commands/review.rs` → **1**
- `grep -c 'commands::review::generate_review_doc' src-tauri/src/lib.rs` → **1**
- `grep -c 'session-changed' src-tauri/src/commands/review.rs` → **19** (unchanged from before plan; no new emit added)
- `grep -A 30 'pub async fn generate_review_doc' src-tauri/src/commands/review.rs | grep -c 'session.comments.is_empty'` → **1** (D-11 gate within 30 lines)
- `grep -A 30 'pub async fn generate_review_doc' src-tauri/src/commands/review.rs | grep -c 'no_comments'` → **1**
- `grep -A 40 'pub async fn generate_review_doc' src-tauri/src/commands/review.rs | grep -c 'spawn_blocking'` → **1**
- `grep -A 40 'pub async fn generate_review_doc' src-tauri/src/commands/review.rs | grep -c 'git::review::render'` → **1**

## User Setup Required

None — no external service configuration, no new dependencies.

## Next Phase Readiness

- Plan 70-03 (UI) can immediately call `safeInvoke<string>("generate_review_doc", { path: repoPath })` per RESEARCH Item 8.
- Error contract: caller may receive `not_open`, `no_session`, `no_comments`, `spawn_error`, or a serialized `git2::Error`-derived `TrunkError`. Each `TrunkError` is JSON-encoded into the `String` of `Result<String, String>` per the existing CONVENTIONS line 78.
- Behaviour to test from the UI side: (a) successful Generate returns the markdown string for preview; (b) `no_comments` returned when session has zero comments (UI should never hit this because D-01 disables the button, but the gate provides a hard backstop).

## Self-Check: PASSED

- `src-tauri/src/commands/review.rs` contains `pub async fn generate_review_doc` — FOUND
- `src-tauri/src/lib.rs` contains `commands::review::generate_review_doc,` registration — FOUND
- Commit hashes verified in git log:
  - `e700b6b feat(70-02): add generate_review_doc Tauri command` — FOUND
  - `c9ac63a feat(70-02): register generate_review_doc in invoke_handler` — FOUND
- No STATE.md / ROADMAP.md edits (worktree mode — orchestrator owns those)
- `just check` exits 0 — verified

---
*Phase: 70-excerpt-resolution-markdown-render*
*Completed: 2026-05-26*
