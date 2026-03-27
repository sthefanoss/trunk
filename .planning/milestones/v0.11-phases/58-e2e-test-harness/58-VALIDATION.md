---
phase: 58
slug: e2e-test-harness
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-27
---

# Phase 58 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | WebdriverIO 9.x + tauri-driver |
| **Config file** | e2e/wdio.conf.ts (Wave 0 installs) |
| **Quick run command** | `npx wdio run e2e/wdio.conf.ts --spec e2e/specs/smoke.spec.ts` |
| **Full suite command** | `npx wdio run e2e/wdio.conf.ts` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx wdio run e2e/wdio.conf.ts --spec e2e/specs/smoke.spec.ts`
- **After every plan wave:** Run `npx wdio run e2e/wdio.conf.ts`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 58-01-01 | 01 | 1 | E2E-01 | e2e infra | `npx wdio run e2e/wdio.conf.ts --spec e2e/specs/smoke.spec.ts` | ❌ W0 | ⬜ pending |
| 58-02-01 | 02 | 2 | E2E-02 | e2e | `npx wdio run e2e/wdio.conf.ts --spec e2e/specs/history.spec.ts` | ❌ W0 | ⬜ pending |
| 58-03-01 | 03 | 2 | E2E-03 | e2e | `npx wdio run e2e/wdio.conf.ts --spec e2e/specs/staging.spec.ts` | ❌ W0 | ⬜ pending |
| 58-04-01 | 04 | 2 | E2E-04 | e2e | `npx wdio run e2e/wdio.conf.ts --spec e2e/specs/branches.spec.ts` | ❌ W0 | ⬜ pending |
| 58-05-01 | 05 | 3 | E2E-05 | manual | Manual macOS validation | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/wdio.conf.ts` — WebdriverIO configuration with tauri-driver lifecycle
- [ ] `e2e/specs/smoke.spec.ts` — Minimal smoke test (app launches, window exists)
- [ ] `package.json` — devDependencies for @wdio/cli, @wdio/mocha-framework, @wdio/spec-reporter
- [ ] `cargo install tauri-driver --locked` — tauri-driver binary

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| macOS E2E validation | E2E-05 | WKWebView WebDriver unreliable for CI | Run core workflow tests manually on macOS: open repo, browse history, stage/commit, branch ops |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
