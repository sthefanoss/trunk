# Project Research Summary

**Project:** Trunk — Desktop Git GUI (Tauri 2 + Svelte 5 + Rust)
**Domain:** Production infrastructure for an existing desktop application
**Researched:** 2026-03-26
**Confidence:** HIGH (E2E macOS tooling MEDIUM due to ecosystem immaturity)

## Executive Summary

Trunk v1.0 is not a greenfield product build — it is a production-readiness upgrade to an already-shipping desktop Git GUI. v0.10 delivered the full release pipeline (4-platform matrix builds, Homebrew distribution, GitHub Actions CI with quality gates). What v1.0 adds is: code signing so macOS Gatekeeper doesn't block the app, auto-updates so users don't manually download every release, performance benchmarks so the lane algorithm doesn't regress silently, and E2E tests so IPC-layer regressions surface before users do.

The recommended implementation sequence is strict and dependency-driven. Benchmarks first because they have zero external dependencies and establish performance baselines. E2E tests second because they create validation infrastructure needed for the subsequent phases. Code signing third because it requires Apple Developer enrollment ($99/year) and must precede auto-updates — macOS Gatekeeper blocks unsigned update artifacts, making the auto-update flow untestable without prior signing. Auto-updates last because they depend on both the code-signing infrastructure and a separately generated Ed25519 updater keypair. Attempting a different order reliably produces dead ends.

The primary risks are: (1) conflating the three independent signing systems (Tauri Ed25519 updater key, Apple Developer ID certificate, Apple notarization) — treat each as a distinct task; (2) expecting Criterion to work as a CI regression gate — it measures wall-clock time which is too noisy on shared runners, so `iai-callgrind` (instruction counts, deterministic) must be used instead; (3) the macOS E2E gap — Apple provides no WKWebView WebDriver, so E2E runs on Linux CI only and macOS testing remains manual pre-release. These risks are well-understood and avoidable with the right tooling decisions made upfront.

## Key Findings

### Recommended Stack

The existing stack (Tauri 2, Svelte 5, Vite 6, TypeScript 5.6, Tailwind CSS 4, Rust with git2/notify/tokio, Vitest, GitHub Actions) needs no replacement — v1.0 adds targeted new pieces. The additions are: `tauri-plugin-updater` + `tauri-plugin-process` (Rust + JS) for auto-updates; `criterion 0.5.1` (dev-only) for local benchmarking with HTML reports; `iai-callgrind` for deterministic CI benchmark gates; `@wdio/cli` + `@wdio/local-runner` + `@wdio/mocha-framework` + `@wdio/spec-reporter` (all ^9.19) for E2E; and `tauri-driver` (cargo install) as the WebDriver bridge. Code signing requires no new software dependencies — `tauri-action@v0` (already in use) handles everything when the correct CI secrets are present.

**Core technologies:**
- `tauri-plugin-updater@2` + `@tauri-apps/plugin-updater@^2`: auto-update runtime — official Tauri 2 plugin, integrates with GitHub Releases static JSON, no custom server needed
- `criterion 0.5.1`: local Rust benchmarks — de facto standard, statistical analysis, HTML reports; pin to 0.5.x not 0.8.x (breaking API changes in 0.8.x require Rust 1.88+)
- `iai-callgrind`: CI benchmark gates — counts CPU instructions via Valgrind, deterministic across VM restarts, Linux-only (matches CI platform)
- `@wdio/cli@^9.19`: E2E test runner — official Tauri-recommended WebDriver framework, JavaScript-native, matches project toolchain
- `tauri-driver` (cargo install): WebDriver bridge — translates W3C WebDriver protocol to platform-native WebKit/Edge driver; Linux and Windows only

### Expected Features

**Must have (table stakes — v1.0):**
- macOS code signing + notarization — Gatekeeper blocks unsigned apps on macOS Sequoia entirely; required for any distributed build
- Update signing keypair — Tauri updater enforces signatures; the plugin refuses to function without a valid Ed25519 keypair
- Auto-updater (check + download + install + relaunch) — all major desktop Git GUIs (GitKraken, Fork, Tower, Sublime Merge) auto-update; users expect it
- Criterion benchmarks for critical Rust operations — `walk_commits_inner`, `list_refs_inner`, `diff_inner`; regressions in these are invisible without baselines
- E2E smoke tests on Linux CI — unit tests cover logic but not the IPC bridge; staging + commit flow must be validated end-to-end
- Update notification UI with download progress — non-blocking toast leveraging the existing toast system; silent restarts are unacceptable

**Should have (v1.x — add after baseline is stable):**
- Windows code signing — SmartScreen warnings are unprofessional; requires Azure Trusted Signing or OV cert ($200-400/year)
- CI benchmark regression gates (iai-callgrind) — once baseline data from multiple releases exists, add automated failure thresholds
- macOS E2E tests — blocked on `tauri-plugin-webdriver-automation` maturing (February 2026, experimental)

**Defer (v2+):**
- Windows E2E tests in CI — msedgedriver version matching is historically flaky
- Startup time tracking in CI — meaningful only with consistent self-hosted runners
- CrabNebula DevTools — useful debugging tool, not blocking

### Architecture Approach

All four features integrate cleanly into the existing architecture without restructuring it. The Svelte/Tauri IPC/Rust layer remains unchanged at the logical level. Benchmarks exploit the established inner-fn pattern (all Tauri commands already have `_inner` functions separated from Tauri state) — criterion calls them directly with no Tauri runtime. The updater registers as a standard Tauri plugin in `lib.rs` using the same builder-chain pattern as existing plugins (dialog, store, clipboard, window-state). Code signing is purely CI configuration: `tauri-action@v0` detects signing environment variables automatically. E2E tests launch a debug build (`--debug --no-bundle`) and drive it through the WebDriver protocol layer.

**Major components:**
1. `src-tauri/benches/` — Criterion benchmark suite calling `walk_commits_inner`, `diff_unstaged_inner`, `list_refs_inner` directly; NEW directory
2. `tests/e2e/` — WebdriverIO specs + fixture helpers; creates ephemeral test repos per suite; NEW directory
3. `src/lib/UpdateChecker.svelte` — non-modal update notification component reusing existing toast system; NEW component
4. Modified `release.yml` — adds signing env vars, `uploadUpdaterJson: true`; no structural changes to existing workflow
5. Modified `ci.yml` — adds Gate 3: E2E (Linux) and benchmarks after existing fast/heavy gates

### Critical Pitfalls

1. **Building auto-updates before code signing** — unsigned macOS apps get Gatekeeper-blocked on every update install, making the update flow untestable; phase ordering is a hard constraint: code signing must complete first

2. **Using Criterion as a CI regression gate** — GitHub Actions shared runners have 10-30% timing noise; Criterion benchmarks produce phantom regressions on every PR and get ignored; use `iai-callgrind` (instruction counts, deterministic) for CI gates, reserve Criterion for local development

3. **E2E tests that cannot run on macOS** — Apple provides no WKWebView WebDriver; the macOS E2E strategy (Linux-only CI vs. experimental plugin-based) must be decided before writing any test infrastructure

4. **Losing the updater Ed25519 private key** — recovery is impossible; every installed copy embeds the public key and will permanently reject updates signed with a different key; store in at least two durable locations outside GitHub before writing any updater code

5. **Apple notarization hanging in CI** — `notarytool` occasionally queues for 2+ hours; configure explicit step timeouts (15-20 min), use App Store Connect API keys (not Apple ID password), have a local signing fallback documented

## Implications for Roadmap

Based on research, the dependency graph dictates a strict 4-phase sequence with no reordering.

### Phase 1: Performance Benchmarks

**Rationale:** Zero external dependencies — no accounts, no certificates, no new CI secrets. Can start immediately with the existing codebase. The inner-fn pattern is already in place; this phase adds benchmark harnesses that call those functions directly. Establishes performance baselines before any signing or updater overhead is added to builds.

**Delivers:** Criterion benchmark suite for `walk_commits_inner` (lane algorithm), `diff_unstaged_inner`, `list_refs_inner` at multiple repo scales (100, 1k, 10k commits); baseline HTML reports locally; optional iai-callgrind setup for future CI gates.

**Addresses:** Criterion benchmarks for critical Rust operations (P1 feature); foundation for CI regression gates (P2 feature).

**Avoids:** Benchmarking tiny repos — use 100/1k/10k/100k fixtures; benchmarking through Tauri IPC — use `_inner` functions directly; Criterion in CI — reserve for local use.

**Research flag:** Standard patterns — Criterion is well-documented, inner-fn pattern already in place. Skip `/gsd:research-phase` unless iai-callgrind CI integration needs scoping.

### Phase 2: E2E Test Harness

**Rationale:** Creates validation infrastructure needed for Phases 3 and 4. When code signing and auto-updates are implemented, E2E tests provide automated verification that those changes didn't break core workflows. Independent of all external account requirements.

**Delivers:** WebdriverIO + tauri-driver setup on Linux CI; E2E specs for open-repo, stage-file, commit, verify-graph workflows; fixture library using `git2` for reproducible ephemeral repos; Gate 3 in `ci.yml`.

**Addresses:** E2E smoke tests (P1 feature).

**Avoids:** Shared mutable test repos (create ephemeral repos per suite); assertions on SHA values (assert on structural properties); hardcoded sleeps (use `waitFor` polling); testing against `tauri dev` hot-reload mode (use built binary).

**Research flag:** Needs `/gsd:research-phase` to decide the macOS E2E strategy. The Linux-only WebDriver approach is well-documented, but whether to add `tauri-plugin-webdriver-automation` for macOS support requires evaluating that plugin's maturity (Feb 2026, experimental). This decision affects test infrastructure design before the first line of test code is written.

### Phase 3: Code Signing

**Rationale:** Required before auto-updates. macOS Gatekeeper blocks unsigned update artifacts from installing; auto-update integration cannot be meaningfully tested without a signed base build. May have an external waiting period while Apple processes Developer Program enrollment ($99/year).

**Delivers:** macOS code signing + notarization in `release.yml` (Apple Developer ID certificate, Entitlements.plist, notarization via App Store Connect API keys); `TAURI_SIGNING_PRIVATE_KEY` Ed25519 keypair generated and backed up; `tauri.conf.json` signing config; CI step timeouts and local fallback procedure documented.

**Addresses:** macOS code signing + notarization (P1 feature); update signing keypair generation (P1 feature — logically belongs here as prerequisite for Phase 4).

**Avoids:** Apple ID password in CI — use App Store Connect API keys; signing secrets in config files — use GitHub Secrets only; no timeout on notarization step — Apple servers queue unpredictably; updater private key only in GitHub secrets — back up to password manager before proceeding.

**Research flag:** Largely standard patterns from official Tauri docs. However, the interaction between the existing draft-release workflow (build → publish → update Homebrew tap) and `uploadUpdaterJson: true` needs validation. Recommend a test pre-release in CI before the main implementation to verify `latest.json` appears correctly.

### Phase 4: Auto-Updates

**Rationale:** Last because it depends on Phase 3 (signed builds + updater keypair). All meaningful auto-update testing requires the signing chain to be in place. This phase is primarily frontend and configuration work once signing is done.

**Delivers:** `tauri-plugin-updater` + `tauri-plugin-process` integrated; `UpdateChecker.svelte` component with non-blocking toast notification; download progress feedback; user-controlled restart; `latest.json` generation in release pipeline with all 4 platform entries; `createUpdaterArtifacts: true` in `tauri.conf.json`.

**Addresses:** Auto-updater check + download + install (P1 feature); update notification UI (P1 feature); silent background update check (P1 feature).

**Avoids:** Auto-install without user consent — always show notification, let user control restart; blocking UI during download — non-modal toast; missing platforms in `latest.json` — verify all 4 targets (darwin-aarch64, darwin-x86_64, linux-x86_64, windows-x86_64) after first release; `+` characters in version strings — breaks updater endpoint URL interpolation.

**Research flag:** Standard patterns — official Tauri docs cover the full flow. No deep research needed unless the existing draft-release workflow requires custom `latest.json` handling.

### Phase Ordering Rationale

- **Benchmarks before everything** — self-contained, zero external dependencies, establishes baselines before infrastructure changes affect build characteristics
- **E2E before signing** — E2E tests become the automated validation layer for signing and updater changes; investing in the test infrastructure first makes subsequent phases safer to implement and verify
- **Code signing before auto-updates** — hard dependency; the Tauri updater requires signed artifacts to function on macOS; this ordering is not optional
- **All four phases in v1.0** — the features form a coherent production-readiness package; shipping code signing without auto-updates, or auto-updates without benchmarks, leaves the project incomplete in ways that degrade over time

### Research Flags

**Needs `/gsd:research-phase` during planning:**
- **Phase 2 (E2E):** macOS E2E strategy decision — evaluate `tauri-plugin-webdriver-automation` maturity vs. Linux-only approach; architecture differs significantly between the two paths and must be decided before infrastructure design

**Phases with standard patterns (skip research-phase):**
- **Phase 1 (Benchmarks):** Criterion + inner-fn pattern is fully documented; iai-callgrind setup is straightforward with official docs
- **Phase 3 (Code Signing):** Official Tauri docs cover the complete flow; recommend a test release to validate CI interaction with existing workflow before full implementation
- **Phase 4 (Auto-Updates):** Official Tauri docs cover the complete flow; plugin registration follows established patterns already in the codebase

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All additions sourced from official Tauri 2 docs; version pins validated against existing Rust toolchain; explicit "do not use" list backed by specific bugs and deprecation records |
| Features | HIGH | Feature set derived from official Tauri capabilities, competitor analysis, and existing codebase state; no speculation; defer decisions backed by cost/complexity rationale |
| Architecture | HIGH | All patterns follow established Tauri conventions; inner-fn pattern already in project; all integration points are additive not structural |
| Pitfalls | HIGH | Top pitfalls sourced from official Tauri docs, numbered GitHub issues with root-cause analysis, practitioner post-mortems, and specific failure modes from the existing CI pipeline |

**Overall confidence:** HIGH

### Gaps to Address

- **macOS E2E tooling:** `tauri-plugin-webdriver-automation` is from February 2026 with limited production track record; the Phase 2 plan must make a final decision before test infrastructure is designed. Linux-only E2E is the safe fallback if the plugin is too immature.

- **Apple notarization timing:** Notarization SLA is unpredictable (30 seconds to 2+ hours per recent GitHub issue reports). The local signing fallback procedure must be documented in Phase 3 before relying on CI-only notarization for releases.

- **`criterion` 0.5.x current patch version:** The pin to 0.5.1 is from training data. Verify the current 0.5.x latest on crates.io before `cargo add` — this is low risk, just confirm before Phase 1 begins.

- **`tauri-action` flag names:** Research notes both `uploadUpdaterJson: true` (workflow YAML input) and `createUpdaterArtifacts: true` (tauri.conf.json field) as distinct configuration. Validate exact field names against `tauri-action@v0` release notes during Phase 3 planning to avoid silent misconfiguration.

## Sources

### Primary (HIGH confidence)
- [Tauri 2 WebDriver docs](https://v2.tauri.app/develop/tests/webdriver/) — E2E platform support matrix, tauri-driver setup, macOS gap documentation
- [Tauri 2 WebdriverIO Example](https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/) — step-by-step wdio.conf.ts setup
- [Tauri 2 Updater Plugin](https://v2.tauri.app/plugin/updater/) — full configuration, signing, endpoints, createUpdaterArtifacts
- [Tauri 2 macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/) — certificate types, notarization, App Store Connect API keys
- [Tauri 2 Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) — Azure Trusted Signing setup
- [Tauri 2 GitHub Pipelines](https://v2.tauri.app/distribute/pipelines/github/) — tauri-action workflow, latest.json generation
- [criterion.rs docs](https://bheisler.github.io/criterion.rs/book/getting_started.html) — Rust benchmark harness, v0.5.x API, CI noise warning
- [iai-callgrind GitHub](https://github.com/iai-callgrind/iai-callgrind) — deterministic CI benchmarking via instruction counts
- [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) — includeUpdaterJson, signing integration

### Secondary (MEDIUM confidence)
- [Ship Tauri v2 Like a Pro: Code Signing](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-code-signing-for-macos-and-windows-part-12-3o9n) — practitioner certificate export walkthrough
- [Tauri auto-updater with GitHub Releases](https://thatgurjot.com/til/tauri-auto-updater/) — practical GitHub Releases updater setup
- [Shipping a Production macOS App with Tauri 2.0](https://dev.to/0xmassi/shipping-a-production-macos-app-with-tauri-20-code-signing-notarization-and-homebrew-mc3) — end-to-end notarization walkthrough

### Tertiary (LOW confidence — needs validation)
- [tauri-webdriver (danielraffel)](https://github.com/danielraffel/tauri-webdriver) — macOS WKWebView WebDriver alternative; 12 stars, Feb 2026, experimental; evaluate during Phase 2 planning before committing to this path
- [CrabNebula E2E Tests](https://docs.crabnebula.dev/plugins/tauri-e2e-tests/) — cross-platform tauri-driver with macOS support; commercial, rejected for personal project

---
*Research completed: 2026-03-26*
*Ready for roadmap: yes*
