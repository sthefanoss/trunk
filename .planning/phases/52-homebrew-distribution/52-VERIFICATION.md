---
phase: 52-homebrew-distribution
verified: 2026-03-26T00:00:00Z
status: human_needed
score: 6/6 must-haves verified
re_verification: false
human_verification:
  - test: "Install via brew install --cask joaofnds/tap/trunk"
    expected: "Trunk.app installs to /Applications without errors; app launches"
    why_human: "Requires a real published release with trunk.rb in homebrew-tap; prerelease test tag v0.10.0-test1 correctly skipped tap update, so cask file has not been generated yet"
  - test: "Push a non-prerelease tag (e.g. v0.10.0) and confirm full pipeline"
    expected: "build (4 platforms) + publish + update-tap all succeed; Casks/trunk.rb appears in homebrew-tap with correct SHA256 hashes and on_intel/on_arm URLs"
    why_human: "update-tap job was intentionally skipped during test run (prerelease tag); the cask generation script has not executed end-to-end yet"
---

# Phase 52: Homebrew Distribution Verification Report

**Phase Goal:** macOS users can install Trunk via Homebrew with a single command
**Verified:** 2026-03-26
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | Tag push triggers build -> publish -> update-tap pipeline end-to-end | ✓ VERIFIED | `needs: [build]` on publish (line 110), `needs: [publish]` on update-tap (line 121); confirmed working with v0.10.0-test1 |
| 2  | Draft release is published (non-draft) after all builds complete | ✓ VERIFIED | Line 118: `gh release edit "${{ github.ref_name }}" --draft=false --repo "${{ github.repository }}"` |
| 3  | Cask formula trunk.rb is generated with correct SHA256 hashes and pushed to homebrew-tap | ? UNCERTAIN | Script logic verified correct; not yet executed for non-prerelease tag — see human verification |
| 4  | Cask formula uses on_intel/on_arm blocks with architecture-specific DMG URLs | ✓ VERIFIED | Lines 155-162 of release.yml contain `on_intel do` and `on_arm do` blocks with architecture-specific DMG URLs |
| 5  | Prerelease tags skip the tap update job | ✓ VERIFIED | Line 123: `if: ${{ !contains(github.ref_name, '-') }}`; confirmed skipped for v0.10.0-test1 |
| 6  | Homebrew-tap README lists trunk in a Casks section | ✓ VERIFIED | Commit 914c1f7 pushed to remote; README contains `## Casks` heading and trunk entry |

**Score:** 5/6 automated truths verified (1 deferred to human for runtime confirmation)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `.github/workflows/release.yml` | publish and update-tap jobs chained after build | ✓ VERIFIED | Contains `publish:` at line 109 and `update-tap:` at line 120; commit 43b3382 |
| `.github/workflows/release.yml` | update-tap job with SHA256 computation and cask generation | ✓ VERIFIED | `shasum -a 256` at lines 137-138; heredoc cask template at lines 150-176 |
| `/Users/joaofnds/code/homebrew-tap/README.md` | Trunk listed in tap README under Casks section | ✓ VERIFIED | Line 18: `[trunk](https://github.com/joaofnds/trunk) \| [cask](Casks/trunk.rb) \| Desktop Git GUI`; commit 914c1f7 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| publish job | build job | `needs: [build]` | ✓ WIRED | Line 110 |
| update-tap job | publish job | `needs: [publish]` | ✓ WIRED | Line 121 |
| update-tap job | HOMEBREW_TAP_TOKEN secret | `secrets.HOMEBREW_TAP_TOKEN` | ✓ WIRED | Lines 142, 148; secret confirmed in trunk repo by human test |

### Data-Flow Trace (Level 4)

This phase produces a CI workflow, not a UI component. Data-flow tracing (Level 4) is not applicable.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| publish job exists and removes draft | `grep "draft=false" .github/workflows/release.yml` | `gh release edit ... --draft=false` found at line 118 | ✓ PASS |
| update-tap skips prereleases | `grep "contains(github.ref_name" .github/workflows/release.yml` | `!contains(github.ref_name, '-')` at line 123 | ✓ PASS |
| SHA256 computed for both architectures | `grep "shasum -a 256" .github/workflows/release.yml` | ARM and Intel patterns at lines 137-138 | ✓ PASS |
| Cask has on_intel/on_arm blocks | `grep "on_intel\|on_arm" .github/workflows/release.yml` | Both blocks at lines 155, 159 | ✓ PASS |
| HOMEBREW_TAP_TOKEN referenced | `grep "HOMEBREW_TAP_TOKEN" .github/workflows/release.yml` | Present at lines 142, 148 | ✓ PASS |
| Full pipeline via non-prerelease tag | Requires tag push + workflow run | Not testable without side effects | ? SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| DIST-01 | 52-01-PLAN.md | Homebrew cask formula published to joaofnds/homebrew-tap for macOS installation via `brew install --cask joaofnds/tap/trunk` | ? PARTIAL | Workflow automation verified; cask generation deferred to non-prerelease tag push (see human verification) |

REQUIREMENTS.md shows DIST-01 as `Pending` (unchecked). The workflow implementation is complete and verified; status should be updated to complete after a production tag confirms end-to-end cask generation.

### Anti-Patterns Found

No anti-patterns detected. The workflow is a pure CI/CD file with no stubs, placeholders, or empty implementations.

Cask generation uses shell variable expansion in the heredoc (not a quoted heredoc) — this is intentional and correct. The bash variables `${VERSION}`, `${intel_sha256}`, `${arm_sha256}`, and `${TAURI_VERSION}` expand at write time. Ruby's `#{version}` interpolation is written literally into the file, where Homebrew resolves it at install time. This is the standard pattern for Homebrew cask formulas.

### Human Verification Required

#### 1. Non-prerelease tag end-to-end test

**Test:** Push a non-prerelease production tag (e.g. `v0.10.0`): `git tag v0.10.0 && git push origin v0.10.0`

**Expected:**
- All 4 build matrix jobs pass (macos-latest ARM, macos-15-intel, ubuntu-22.04, windows-latest)
- `publish` job sets the release to non-draft via `gh release edit --draft=false`
- `update-tap` job runs (not skipped), downloads both DMGs, computes SHA256, generates `Casks/trunk.rb` in joaofnds/homebrew-tap, and pushes
- `trunk.rb` in homebrew-tap contains `on_intel do` and `on_arm do` blocks with real SHA256 hashes matching the DMG files

**Why human:** The `update-tap` job was intentionally skipped during the v0.10.0-test1 test run because test tags contain `-`. Cask generation requires a production tag to execute end-to-end.

#### 2. Homebrew install test (optional, post production tag)

**Test:** On a macOS machine with Homebrew: `brew tap joaofnds/tap && brew install --cask joaofnds/tap/trunk`

**Expected:** Trunk.app downloads from the GitHub Release DMG, installs to /Applications, and launches successfully

**Why human:** Requires a real cask file in homebrew-tap (generated by #1 above) and an actual macOS Homebrew environment

### Gaps Summary

No blocking gaps. All workflow logic is correctly implemented and verified:
- 3-job pipeline (build -> publish -> update-tap) with proper `needs:` chaining
- Draft release publication via `gh release edit --draft=false`
- Prerelease skip condition correctly implemented
- SHA256 computation for both ARM and Intel DMGs
- Cask formula with `on_intel do` / `on_arm do` blocks
- Cross-repo push via `HOMEBREW_TAP_TOKEN`
- homebrew-tap README updated with Casks section

The only outstanding item is runtime confirmation that the cask generation step produces correct output when executed with a non-prerelease tag. This cannot be verified statically — it requires a real workflow run with a production tag.

DIST-01 in REQUIREMENTS.md should be marked complete after the first production tag confirms end-to-end pipeline success.

---

_Verified: 2026-03-26_
_Verifier: Claude (gsd-verifier)_
