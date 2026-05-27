---
phase: 66
slug: commit-selection
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-25
---

# Phase 66 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust: `cargo test --lib` (in-process tempfile/test-repo unit tests). Frontend: `vitest` + `@testing-library/svelte` (component tests, e.g. existing `ReviewPanel.test.ts`). |
| **Config file** | Rust: Cargo defaults. Frontend: existing vitest config. |
| **Quick run command** | `cd src-tauri && cargo test --lib review` / `npx vitest run ReviewPanel CommitRow` |
| **Full suite command** | `just check` (fmt, biome, svelte-check, clippy, cargo-test, vitest) |
| **Estimated runtime** | ~60 seconds (quick); ~3 min (`just check`) |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test --lib review` (or the touched test file's vitest subset)
- **After every plan wave:** Run `just check`
- **Before `/gsd:verify-work`:** `just check` must be green
- **Max feedback latency:** ~60 seconds

---

## Per-Task Verification Map

Task IDs are assigned at planning. The rows below map each phase requirement / locked
decision to its proving test (from RESEARCH.md §Validation Architecture). The planner must
attach each test to a concrete task `<automated>` block or a Wave 0 dependency.

| Req / Decision | Behavior | Threat Ref | Test Type | Automated Command | File Exists | Status |
|----------------|----------|------------|-----------|-------------------|-------------|--------|
| SEL-01 | `[base..tip]` inclusive — both endpoints in result | — | unit (Rust) | `cargo test --lib seed_range_inclusive` | ❌ W0 | ⬜ pending |
| SEL-01 | root-commit base — walk without hiding | — | unit (Rust) | `cargo test --lib seed_range_root_base` | ❌ W0 | ⬜ pending |
| SEL-01 | `base == tip` → single-commit set | — | unit (Rust) | `cargo test --lib seed_range_base_eq_tip` | ❌ W0 | ⬜ pending |
| SEL-01 | base not ancestor of tip → Err, no mutation | T-66-01 | unit (Rust) | `cargo test --lib seed_range_rejects_non_ancestor` | ❌ W0 | ⬜ pending |
| SEL-01 | unrelated histories → Err (merge_base NotFound) | T-66-01 | unit (Rust) | `cargo test --lib seed_range_rejects_unrelated` | ❌ W0 | ⬜ pending |
| SEL-01 / D-03 | range unions (never replaces); dedup | — | unit (Rust) | `cargo test --lib seed_range_unions_dedups` | ❌ W0 | ⬜ pending |
| SEL-02 | add appends, idempotent (no dup) | — | unit (Rust) | `cargo test --lib add_commit_idempotent` | ❌ W0 | ⬜ pending |
| SEL-03 | remove deletes exactly one; missing oid is no-op | — | unit (Rust) | `cargo test --lib remove_commit` | ❌ W0 | ⬜ pending |
| SEL-02/03 | concurrent add/remove serialized (no lost write) | T-66-02 | unit (Rust) | `cargo test --lib selection_rmw_serialized` | ❌ W0 | ⬜ pending |
| SEL-04 | list returned in graph order, dedup'd | — | unit (Rust) | `cargo test --lib list_session_commits_graph_order` | ❌ W0 | ⬜ pending |
| SEL-04 | selected OID absent from cache → fallback resolves/appends | — | unit (Rust) | `cargo test --lib list_session_commits_orphan_fallback` | ❌ W0 | ⬜ pending |
| D-08 | merge commit IS selectable | — | unit (Rust) | `cargo test --lib merge_commit_selectable` | ❌ W0 | ⬜ pending |
| D-05 / D-07 | panel renders list + × triggers remove | — | component (vitest) | `npx vitest run ReviewPanel` | ⚠️ extend existing | ⬜ pending |
| D-04 | `inSession` prop tints row via theme var | — | component (vitest) | `npx vitest run CommitRow` | ❌ W0 | ⬜ pending |
| D-06 | toggle label flips Add/Remove on membership | — | component (vitest) / manual | `npx vitest run CommitGraph` | ⚠️ if harness exists | ⬜ pending |
| SEL-04 / sync | `session-changed` triggers list reload | — | component (vitest) | `npx vitest run ReviewPanel` | ⚠️ pattern exists | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Rust selection unit tests in `commands/review.rs` `#[cfg(test)]` — need an in-process test repo with a known linear+merge topology. Grep for `tempfile` / `Repository::init` in `src-tauri` tests for an existing helper before hand-rolling one (the revwalk tests need real commits).
- [ ] `src/components/CommitRow.test.ts` — covers D-04 `inSession` marker (none exists today).
- [ ] Extend `src/components/ReviewPanel.test.ts` — list rendering (D-05) + × remove (D-07) + reload-on-`session-changed`.
- [ ] (Optional) CommitGraph context-menu toggle test — Tauri `Menu` mocking is heavier; manual verification acceptable for the D-06 label flip if no harness exists.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Two-right-click range gesture + transient base highlight (D-01) | SEL-01 | Native Tauri `Menu.popup()` + transient frontend state are hard to drive in vitest | Right-click a commit → "Set as review base" (base highlights); right-click a later commit → "Add range to review"; confirm `[base..tip]` appears in panel, highlight clears. |
| D-06 context-menu toggle label flip (if no CommitGraph harness) | SEL-02 / SEL-03 | Tauri `Menu`/`MenuItem` mocking is heavy | Right-click a non-session commit → shows "Add to review"; right-click a session commit → shows "Remove from review". |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
