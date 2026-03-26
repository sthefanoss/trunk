---
phase: 51
slug: cross-platform-release-pipeline
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 51 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | GitHub Actions (workflow syntax validation via `actionlint` or manual push) |
| **Config file** | `.github/workflows/release.yml` |
| **Quick run command** | `cat .github/workflows/release.yml` (syntax review) |
| **Full suite command** | `git tag v0.0.0-test && git push origin v0.0.0-test` (live trigger test) |
| **Estimated runtime** | ~15 minutes (full cross-platform build) |

---

## Sampling Rate

- **After every task commit:** Review workflow YAML for syntax correctness
- **After every plan wave:** Validate workflow structure against requirements
- **Before `/gsd:verify-work`:** Full tag-push test must produce all expected artifacts
- **Max feedback latency:** 900 seconds (cross-platform builds are inherently slow)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 51-01-01 | 01 | 1 | REL-01 | integration | `gh workflow view release.yml` | ❌ W0 | ⬜ pending |
| 51-01-02 | 01 | 1 | REL-02 | integration | `grep -c '\.dmg\|\.AppImage\|\.msi' .github/workflows/release.yml` | ❌ W0 | ⬜ pending |
| 51-01-03 | 01 | 1 | REL-03 | integration | `grep -c 'tar.*gz\|\.tar\.gz' .github/workflows/release.yml` | ❌ W0 | ⬜ pending |
| 51-01-04 | 01 | 1 | REL-04 | integration | `grep -c 'upload-artifact' .github/workflows/release.yml` | ❌ W0 | ⬜ pending |
| 51-01-05 | 01 | 1 | REL-05 | integration | `grep 'ubuntu-22.04' .github/workflows/release.yml` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements — this phase creates a new workflow file, no test framework needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tag push triggers builds | REL-01 | Requires actual GitHub Actions execution | Push a `v*` tag and verify workflow starts |
| .dmg/.AppImage/.msi produced | REL-02 | Requires real cross-platform compilation | Check workflow artifacts after completion |
| .tar.gz archives produced | REL-03 | Requires real build output to wrap | Check workflow artifacts after completion |
| Artifacts downloadable | REL-04 | Requires GitHub UI interaction | Navigate to workflow run and download artifacts |
| Linux glibc compat | REL-05 | Requires checking runner OS version | Verify workflow YAML uses ubuntu-22.04 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 900s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
