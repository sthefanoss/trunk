---
phase: 68
slug: full-file-source-anchor-capture
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-25
---

# Phase 68 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from RESEARCH.md §6 Validation Architecture (observables V1–V10).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (frontend), cargo test (backend round-trip) |
| **Config file** | `vite.config.ts` (vitest block) — existing |
| **Quick run command** | `bun run test` (or scoped: `bunx vitest run src/lib/full-file-anchor.test.ts`) |
| **Full suite command** | `just check` (fmt, biome, svelte-check, clippy, cargo-test, vitest) |
| **Estimated runtime** | quick ~5s; full suite ~60–120s |

---

## Sampling Rate

- **After every task commit:** Run `bun run test` (or the scoped adapter test during adapter work)
- **After every plan wave:** Run `just check`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds (quick run)

---

## Per-Task Verification Map

> Task IDs are assigned by gsd-planner. Rows below are seeded from RESEARCH.md §6
> observables and are bound to concrete Task IDs during planning. The pure adapter
> (V1–V4) is the TDD core.

| Obs | Behavior | Requirement | Test Type | Automated Command | Status |
|-----|----------|-------------|-----------|-------------------|--------|
| V1 | `buildFullFileAnchor` returns `{source:FullFile, side:New, start/end = min/max new_lineno}` | ANCH-02 | unit (tdd) | `bunx vitest run src/lib/full-file-anchor.test.ts` | ⬜ pending |
| V2 | Delete lines (`new_lineno=null`) excluded from range + excerpt (D-02) | ANCH-02 | unit (tdd) | `bunx vitest run src/lib/full-file-anchor.test.ts` | ⬜ pending |
| V3 | Excerpt is plain new-side content, no `+`/`-`/space prefix (D-04) | ANCH-02 | unit (tdd) | `bunx vitest run src/lib/full-file-anchor.test.ts` | ⬜ pending |
| V4 | Gap-crossing span keeps correct range + inserts `… N lines unchanged …` (D-03) | ANCH-02 | unit (tdd) | `bunx vitest run src/lib/full-file-anchor.test.ts` | ⬜ pending |
| V5 | Empty/zero-hunk file shows no affordance, no throw | ANCH-02 | component | `bunx vitest run src/components/diff/FullFileView.test.ts` | ⬜ pending |
| V6 | Click + shift-click → contiguous selection; only new-side lines selectable (D-01/D-02) | ANCH-02 | component | `bunx vitest run src/components/diff/FullFileView.test.ts` | ⬜ pending |
| V7 | Submit calls `add_comment` with FullFile anchor + plain excerpt | ANCH-02 | component | `bun run test` (mock safeInvoke) | ⬜ pending |
| V8 | Draft persists via `save_draft_comment` on 300ms debounce (L-03) | ANCH-02 | component | `bun run test` (fake timers) | ⬜ pending |
| V9 | Full-file comment survives restart (round-trip persistence) | ANCH-02 | backend integration | `cargo test --manifest-path src-tauri/Cargo.toml full_file` | ⬜ pending |
| V10 | Merge commits keep the full-file affordance enabled — no `isMerge` disable (L-05) | ANCH-02 | component (negative) | `bunx vitest run src/components/diff/FullFileView.test.ts` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/lib/full-file-anchor.test.ts` — new test file (clone fixture builders from `src/lib/diff-anchor.test.ts`)
- [ ] `src/components/diff/FullFileView.test.ts` — new component test file (mock-`safeInvoke` pattern from `CommentComposer.test.ts`)
- [ ] Framework: vitest + cargo test already installed — no install needed

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual selection highlight uses theme color, looks correct in full-file view | ANCH-02 | Visual fidelity not assertable in unit/component tests | Open a commit, switch to full-file content mode, click + shift-click a range, confirm contiguous highlight uses a `--color-*` var (not inline color) |
| End-to-end attach survives an actual app restart | ANCH-02 | Real restart cycle (V9 covers the store round-trip; this confirms the full app path) | Attach a full-file comment, quit and relaunch the app, confirm the comment is still present |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
