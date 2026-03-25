---
phase: 50
slug: ci-quality-gates
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 50 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest 4.1.0 (frontend), cargo test (backend), biome ci (lint/format) |
| **Config file** | `vite.config.ts` (vitest), `#[cfg(test)]` modules (Rust), `biome.json` (Biome) |
| **Quick run command** | `bun run test && cd src-tauri && cargo test` |
| **Full suite command** | `biome ci . && cargo fmt --manifest-path src-tauri/Cargo.toml --check && bun run check && cd src-tauri && cargo clippy -- -D warnings && cargo test && cd .. && bun run test` |
| **Estimated runtime** | ~60-90 seconds (local) |

---

## Sampling Rate

- **After every task commit:** Run the specific check for that task (e.g., `cargo clippy -- -D warnings`, `biome ci .`)
- **After every plan wave:** Push to branch, verify CI workflow triggers and all jobs pass
- **Before `/gsd:verify-work`:** Full suite must be green; CI must pass on a pushed commit
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | CI-01 | smoke | `cargo fmt --manifest-path src-tauri/Cargo.toml --check && cd src-tauri && cargo clippy -- -D warnings && cargo test` | ✅ | ⬜ pending |
| TBD | TBD | TBD | CI-02 | smoke | `bun run check && bun run test` | ✅ | ⬜ pending |
| TBD | TBD | TBD | CI-03 | smoke | `biome ci .` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CI-04 | manual | `biome ci .` returns exit 0 | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CI-05 | manual | Check CI logs for cache hit/miss | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Install `@biomejs/biome` as devDependency
- [ ] Create `biome.json` configuration with Svelte overrides
- [ ] Fix `cargo fmt` issues (run `cargo fmt` — 15 files)
- [ ] Fix `cargo clippy` warnings (29 mechanical fixes)
- [ ] Fix `svelte-check` failures (exclude vendored JS, fix 1 CommitGraph error)
- [ ] Run `biome format --write .` to format all frontend files

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CI triggers on push/PR | CI-01, CI-02, CI-03 | Requires actual GitHub Actions run | Push a commit to any branch, verify workflow runs in Actions tab |
| Rust caching active | CI-05 | Requires inspecting CI logs | Check two consecutive CI runs — second should show "Restored cache" |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
