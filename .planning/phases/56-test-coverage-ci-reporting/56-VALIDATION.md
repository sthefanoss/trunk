---
phase: 56
slug: test-coverage-ci-reporting
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-27
---

# Phase 56 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + vitest (TypeScript) |
| **Config file** | `src-tauri/Cargo.toml` + `vite.config.ts` |
| **Quick run command** | `cargo test --manifest-path src-tauri/Cargo.toml && bun run test` |
| **Full suite command** | `cargo test --manifest-path src-tauri/Cargo.toml && bun run test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --manifest-path src-tauri/Cargo.toml && bun run test`
- **After every plan wave:** Run full suite
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 56-01-01 | 01 | 1 | UNIT-04 | config | `bun run test -- --coverage.enabled 2>&1 \| head -5` | ❌ W0 | ⬜ pending |
| 56-01-02 | 01 | 1 | UNIT-04 | config | `cargo llvm-cov --version 2>&1` | ❌ W0 | ⬜ pending |
| 56-02-01 | 02 | 1 | UNIT-04 | ci | `cat .github/workflows/ci.yml \| grep -c coverage` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `@vitest/coverage-v8` — install as dev dependency for TypeScript coverage
- [ ] `cargo-llvm-cov` — CI installs via `taiki-e/install-action`; local install optional

*Existing test infrastructure covers all phase requirements — only coverage tooling additions needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| PR comment renders correctly | UNIT-04 | Requires actual PR on GitHub | Create test PR, verify coverage comment appears |
| HTML artifact downloadable | UNIT-04 | Requires CI run completion | Check Actions tab for coverage artifacts |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
