# Feature Research

**Domain:** Production infrastructure for a Tauri 2 desktop Git GUI -- E2E testing, performance benchmarks, code signing, auto-updates
**Researched:** 2026-03-26
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features that any production-quality Tauri 2 desktop app shipping to real users must have. Missing these means the app either cannot be distributed safely, cannot be validated automatically, or regresses silently.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| macOS code signing + notarization | Without signing, Gatekeeper blocks the app entirely on macOS Sequoia. Users see "damaged" or "unidentified developer" dialogs. No workaround except right-click > Open, which erodes trust. | HIGH | Requires Apple Developer Program ($99/year). Environment variables: `APPLE_CERTIFICATE` (base64 P12), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_TEAM_ID`, `APPLE_ID`, `APPLE_PASSWORD` (app-specific). Notarization adds 2-5 min per build. Entitlements.plist needed for JIT. tauri-action handles notarization when env vars are present. |
| Update signature keypair generation | Tauri's updater requires cryptographic signatures -- this is enforced, not optional. Without signing keys, the updater plugin refuses to function at all. | LOW | Run `npx tauri signer generate -w ~/.tauri/trunk.key`. Produces public key (embed in tauri.conf.json) and private key (CI secret). Set `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` during builds. Loss of private key = cannot publish updates ever again. |
| Updater endpoint (latest.json) | The updater needs a URL to check for new versions. Without it, the check() call has nowhere to query. | LOW | tauri-action auto-generates `latest.json` when `createUpdaterArtifacts: true` is set and uploads it to GitHub Releases. Endpoint URL: `https://github.com/joaofnds/trunk/releases/latest/download/latest.json`. The JSON contains version, platform URLs, and signatures. |
| Auto-update check + install flow | Every major desktop Git GUI (GitKraken, Fork, Tower, Sublime Merge) auto-updates. Users do not want to manually download new versions. | MEDIUM | `tauri-plugin-updater` (Rust + JS). Frontend calls `check()` on launch, shows update dialog, calls `downloadAndInstall()`. Requires `tauri-plugin-process` for `relaunch()`. Permissions: `updater:default` in capabilities. |
| Rust backend benchmarks (Criterion) | The lane algorithm processes 10k commits in ~5ms. Without benchmarks, regressions are invisible until users complain about lag. Performance is a core value prop vs. Electron-based competitors. | MEDIUM | `criterion` crate for `walk_commits`, `list_refs_inner`, `diff_inner`, hunk staging. Benchmarks live in `src-tauri/benches/`. Run via `cargo bench`. Output: HTML reports with statistical analysis, regression detection. |
| E2E smoke tests for critical workflows | Unit tests (14 vitest + Rust tests) cover logic but not the IPC bridge, UI rendering, or user workflows. A commit flow that works in unit tests can break when the webview talks to Rust. | HIGH | Multiple approaches available. Recommended: WebdriverIO + tauri-driver (Linux/Windows) with tauri-plugin-webdriver-automation for macOS. Minimum viable: test open repo, stage file, create commit, verify graph updates. |

### Differentiators (Competitive Advantage)

Features that go beyond minimum expectations and signal a mature, well-engineered desktop app.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Windows code signing | Eliminates SmartScreen "Windows protected your PC" warning. Professional appearance. Required for serious Windows distribution. | HIGH | Requires Azure Key Vault + OV certificate ($200-400/year) OR Azure Trusted Signing. Environment variables: `AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, `AZURE_CLIENT_SECRET`. `relic` or `trusted-signing-cli` for sign command. More complex CI setup than macOS. |
| Frontend performance benchmarks (IPC round-trip) | Measures the Tauri IPC bridge latency (~0.5ms per invoke). Detects serialization overhead regressions when GraphCommit structures grow. No competitor open-sources their perf metrics. | MEDIUM | Custom timing harness: measure `invoke()` round-trip for key commands (`get_graph`, `get_diff`, `list_refs`). Record in CI, compare across runs. Could use Vitest bench mode or custom script. |
| Update progress UI with download percentage | Fork and GitKraken show download progress bars. A sudden restart without user feedback feels broken. | LOW | `downloadAndInstall()` accepts a callback with `Started`, `Progress` (chunk_length, content_length), `Finished` events. Wire to a toast or modal showing percentage. |
| Benchmark regression gates in CI | Fail CI if critical operations (lane algorithm, ref listing) regress beyond a threshold. Prevents slow-creep performance degradation. | MEDIUM | `criterion` supports `--save-baseline` and comparisons. Alternatives: `bencher.dev` (SaaS), `github-action-benchmark` (stores results in gh-pages). |
| Silent background update check | Check for updates on launch without blocking the UI. Show a non-intrusive notification only when an update is available. User clicks to install at their convenience. | LOW | Spawn update check in `setTimeout` or Svelte `onMount`. Store "skip this version" preference in LazyStore. Never auto-install without user consent. |
| Cross-platform E2E in CI (all 3 OS) | Validates that the app works identically on macOS, Linux, and Windows. Most Tauri projects only test on Linux CI. | HIGH | Requires matrix CI runners. Linux: `xvfb-run` for headless display. macOS: needs tauri-plugin-webdriver-automation (no native WKWebView driver). Windows: needs msedgedriver matching Edge version. Flaky in CI -- use retries. |
| Startup time measurement | Track cold-start time from app launch to first meaningful paint. Desktop apps that take >2 seconds to show content feel sluggish. | MEDIUM | Rust: measure time from `main()` to first `invoke` received. Frontend: `performance.now()` from DOMContentLoaded to first graph render. Log to console or structured output for CI capture. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems. Explicitly choosing NOT to build these in v1.0.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| CrabNebula Cloud for distribution | "Managed update hosting and macOS testing" | Commercial subscription required. Vendor lock-in for core distribution. Adds a dependency on a third party's uptime for your users to receive updates. Overkill for a personal open-source project. | GitHub Releases + static latest.json. Free, reliable, no vendor dependency. |
| Auto-install updates without user consent | "Just keep me on the latest version" | Users hate surprise restarts. Data loss risk if user has uncommitted work. Some users pin versions intentionally. Desktop apps are not mobile apps. | Show notification, let user choose when to install. Never auto-restart during active work. |
| Electron-style Playwright E2E tests | "Playwright has better DX than WebDriver" | Playwright does not support WKWebView or WebView2 natively. Tauri apps are not Chromium -- Playwright literally cannot connect to the rendering engine. Some workarounds mock the entire backend, defeating the purpose of E2E. | WebdriverIO + tauri-driver for real E2E. Vitest + mockIPC for frontend-only integration tests. Two-layer approach covers both needs. |
| Full UI visual regression testing | "Screenshot every state and diff" | Enormous maintenance burden. Pixel-level comparisons break on OS updates, font rendering changes, DPI differences. Flaky by nature. | Test behavior, not pixels. E2E tests verify elements exist and interactions work. Visual bugs are caught during manual QA before release. |
| Performance profiling in CI with flamegraphs | "Generate flamegraphs on every commit" | Flamegraphs require instrumented builds (debug symbols, no optimizations), which are 10x slower. CI timing is unreliable (shared runners, noisy neighbors). Flamegraphs are diagnostic tools, not regression tests. | Criterion benchmarks with statistical analysis detect regressions. Flamegraphs used locally when investigating specific slowdowns. |
| Nightly update channel | "Power users want bleeding edge" | Doubles signing/build infrastructure. Two update streams to maintain. Nightly builds may contain breaking changes. No user base yet to justify the cost. | Single release channel. Contributors build from source for bleeding edge. |
| App Store distribution (macOS/Windows) | "Reach more users via the store" | Apple App Store requires sandbox compliance, review process, 15-30% revenue cut (N/A for free app but adds constraints). Microsoft Store requires MSIX packaging. Both restrict functionality (file system access, subprocess spawning for git CLI). Trunk's git CLI subprocess usage would likely violate sandbox rules. | Direct download from GitHub Releases. Full system access. No review delays. |

## Feature Dependencies

```
[Apple Developer Program enrollment]
    └──required by──> [macOS code signing + notarization]
                          └──required by──> [Auto-update signature verification on macOS]

[Update signing keypair]
    └──required by──> [createUpdaterArtifacts in tauri-action]
                          └──produces──> [latest.json + .sig files]
                                             └──required by──> [Auto-update check]

[macOS code signing]
    └──should precede──> [Auto-updater integration]
        (unsigned updates would re-trigger Gatekeeper on every update)

[tauri-plugin-updater (Rust)]
    └──required by──> [Frontend update check/install UI]
    └──requires──> [tauri-plugin-process] (for relaunch())
    └──requires──> [Update signing keypair]

[Criterion benchmarks (Rust)]
    (no dependencies -- can be built independently)
    └──enhances──> [CI benchmark regression gates]

[E2E test harness]
    └──requires──> [Built application binary] (tests run against compiled app)
    └──requires──> [tauri-driver or plugin-webdriver] (WebDriver protocol layer)
    └──optional──> [tauri-plugin-webdriver-automation] (macOS support only)
    └──enhances──> [CI quality gates]

[Windows code signing]
    └──requires──> [Azure Key Vault setup OR Azure Trusted Signing]
    └──independent of──> [macOS code signing] (can be done in parallel)
```

### Dependency Notes

- **Code signing must precede auto-updates:** Tauri's updater verifies signatures on downloaded artifacts. If the app itself is unsigned, macOS Gatekeeper will block the updated binary after download, creating a broken update experience. Sign first, then enable the updater.
- **Update signing keypair is separate from code signing:** The `tauri signer generate` key is for verifying update artifact integrity. The Apple/Windows certificates are for OS-level trust. Both are needed but serve different purposes.
- **E2E tests need a built binary:** Unlike unit tests that run against source, E2E tests launch the actual compiled `.app`/`.AppImage`/`.exe`. This means E2E in CI requires a full Tauri build step before tests run (~8-15 min), making them expensive.
- **Criterion benchmarks are fully independent:** They test pure Rust functions (inner-fn pattern already isolates from Tauri runtime). Can be built and run before any other feature.
- **macOS and Windows signing are independent:** Different certificate authorities, different CI secrets, different tools. Can be implemented in parallel or sequentially.

## MVP Definition

### Launch With (v1.0)

The minimum infrastructure to ship a production-quality, self-updating desktop app.

- [ ] **macOS code signing + notarization** -- eliminates Gatekeeper blocking for all macOS users. Apple Developer enrollment, certificate export, CI secrets, Entitlements.plist, notarization in release workflow.
- [ ] **Update signing keypair** -- generate and store securely. Public key in tauri.conf.json, private key as CI secret.
- [ ] **Auto-updater integration** -- `tauri-plugin-updater` + `tauri-plugin-process`. Frontend update check on launch, download progress UI, user-initiated install + relaunch. `createUpdaterArtifacts: true` in config.
- [ ] **Criterion benchmarks for critical Rust operations** -- `walk_commits` (lane algorithm), `list_refs_inner` (branch/tag listing), `diff_inner` (diff generation). Baseline established, run in CI.
- [ ] **E2E smoke test harness** -- WebdriverIO + tauri-driver on Linux CI. Minimum: open repo, verify commit list renders, stage a file, create a commit. Foundation for expanding coverage.

### Add After Validation (v1.x)

Features to add once the core infrastructure is proven stable.

- [ ] **Windows code signing** -- when Windows user base warrants the $200-400/year certificate cost and Azure Key Vault complexity
- [ ] **macOS E2E tests** -- when tauri-plugin-webdriver-automation matures (very new, Feb 2026) or CrabNebula's cross-platform driver stabilizes
- [ ] **CI benchmark regression gates** -- once baseline data exists from multiple releases, set thresholds for automated failure
- [ ] **Frontend IPC benchmarks** -- measure invoke() round-trip times for key commands in CI

### Future Consideration (v2+)

Features to defer until the project has an established user base.

- [ ] **Windows E2E tests in CI** -- requires msedgedriver version matching, historically flaky
- [ ] **Startup time tracking in CI** -- meaningful only with consistent runner hardware (self-hosted)
- [ ] **CrabNebula DevTools integration** -- IPC inspection, tracing spans, useful for debugging but not blocking

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| macOS code signing + notarization | HIGH | HIGH | P1 |
| Update signing keypair | HIGH | LOW | P1 |
| Auto-updater (check + download + install) | HIGH | MEDIUM | P1 |
| Criterion benchmarks (Rust backend) | HIGH | MEDIUM | P1 |
| E2E smoke tests (Linux CI) | HIGH | HIGH | P1 |
| Update progress UI | MEDIUM | LOW | P1 |
| Silent background update check | MEDIUM | LOW | P1 |
| Windows code signing | MEDIUM | HIGH | P2 |
| CI benchmark regression gates | MEDIUM | MEDIUM | P2 |
| Frontend IPC benchmarks | MEDIUM | MEDIUM | P2 |
| macOS E2E tests | MEDIUM | HIGH | P3 |
| Cross-platform E2E (all 3 OS) | LOW | HIGH | P3 |
| Startup time measurement | LOW | MEDIUM | P3 |

**Priority key:**
- P1: Must have for v1.0 launch -- these features make the app distributable and maintainable
- P2: Should have, add when cost/benefit makes sense
- P3: Nice to have, defer until ecosystem tooling matures or user base demands it

## Competitor Feature Analysis

| Feature | GitKraken | Fork | Sublime Merge | Tower | Our Approach (Trunk v1.0) |
|---------|-----------|------|---------------|-------|---------------------------|
| Code signing | Yes (all platforms) | Yes (macOS + Windows) | Yes (all platforms) | Yes (macOS + Windows) | macOS first (P1), Windows deferred (P2) |
| Auto-update | Yes, silent background | Yes, check on launch | Yes, check on launch | Yes, check on launch | Check on launch, user-initiated install |
| Update channel | Stable + Preview | Single channel | Stable + Dev | Single channel | Single channel (simplicity) |
| E2E testing | Private/unknown | Private/unknown | Private/unknown | Private/unknown | WebdriverIO + tauri-driver (open-source advantage) |
| Performance benchmarks | Private/unknown | Private/unknown | Private/unknown | Private/unknown | Criterion (open-source, published results) |
| Distribution | Direct + Homebrew + Snap + Flatpak | Direct + Homebrew | Direct + apt + pacman + Homebrew | Direct + Homebrew | Direct + Homebrew (already done in v0.10) |

**Key insight:** All competitors are closed-source commercial products. We cannot know their internal testing or benchmarking practices. Our advantage is being open-source: public CI, published benchmarks, transparent quality gates. This is a differentiator for the contributor community.

## Existing Project State

Key facts about the current codebase that affect feature implementation.

| Aspect | Current State | Implication for v1.0 |
|--------|---------------|----------------------|
| Release workflow | Tag-triggered, 4-platform matrix (macOS ARM/Intel, Linux, Windows), tauri-action@v0, draft releases | Modify to add signing env vars, createUpdaterArtifacts, latest.json upload. Core workflow structure is solid. |
| Homebrew cask | Auto-generated, pushed to joaofnds/homebrew-tap on release | Cask needs to point to signed .dmg. SHA256 computation already in workflow. |
| CI workflow | 5 gates: Biome, cargo fmt, svelte-check, clippy, cargo test, vitest | Add E2E and benchmark steps as new gates. Existing structure supports it. |
| Inner-fn pattern | All Tauri commands use inner functions separated from Tauri state | Criterion benchmarks can call inner functions directly without Tauri runtime. Huge advantage. |
| Tauri config | `bundle.targets: "all"`, no signing config, no updater config | Add `bundle.macOS.signingIdentity`, `bundle.macOS.entitlements`, `plugins.updater`, `bundle.createUpdaterArtifacts`. |
| Capabilities | `default.json` exists with current permissions | Add `updater:default` and `process:relaunch` permissions. |
| LazyStore | Used for UI state persistence (column widths, view preferences) | Reuse for "skip this version" update preference. |
| Toast system | Auto-dismiss toasts with per-kind styling | Reuse for update notifications: "Update available: v1.1.0" toast with action button. |
| Identifier | `com.joaofnds.trunk` | Apple bundle identifier for code signing. Already correct format. |
| Version | `0.1.0` across all three files | Must be consistent for updater version comparison to work. |

## Sources

- [Tauri 2 WebDriver Testing](https://v2.tauri.app/develop/tests/webdriver/) -- official E2E testing guide, platform support matrix (HIGH confidence)
- [Tauri 2 Updater Plugin](https://v2.tauri.app/plugin/updater/) -- complete updater configuration, signing, endpoints, API (HIGH confidence)
- [Tauri 2 macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/) -- Apple Developer requirements, CI secrets, notarization (HIGH confidence)
- [Tauri 2 Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) -- Azure Key Vault, OV certificates, relic/trusted-signing-cli (HIGH confidence)
- [Tauri 2 GitHub Actions Pipeline](https://v2.tauri.app/distribute/pipelines/github/) -- tauri-action workflow, latest.json generation (HIGH confidence)
- [Tauri 2 Mock APIs](https://v2.tauri.app/develop/tests/mocking/) -- mockIPC, mockWindows for frontend testing (HIGH confidence)
- [tauri-webdriver (macOS)](https://github.com/danielraffel/tauri-webdriver) -- open-source macOS WebDriver, Feb 2026, plugin + CLI architecture (MEDIUM confidence -- very new)
- [CrabNebula E2E Tests](https://docs.crabnebula.dev/plugins/tauri-e2e-tests/) -- cross-platform tauri-driver fork with macOS support, commercial (MEDIUM confidence)
- [Ship Tauri v2 Like a Pro: Code Signing](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-code-signing-for-macos-and-windows-part-12-3o9n) -- practical step-by-step for macOS + Windows signing (MEDIUM confidence)
- [Tauri v2 Auto-Updater Setup](https://thatgurjot.com/til/tauri-auto-updater/) -- practical GitHub Releases updater setup (MEDIUM confidence)
- [Criterion.rs](https://github.com/bheisler/criterion.rs) -- statistics-driven Rust benchmarking library (HIGH confidence)
- [Criterion How-to on Bencher](https://bencher.dev/learn/benchmarking/rust/criterion/) -- Criterion integration guide (MEDIUM confidence)
- [WebdriverIO Tauri Example](https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/) -- official WebdriverIO integration guide (HIGH confidence)
- [Tauri WebDriver CI](https://v2.tauri.app/develop/tests/webdriver/ci/) -- GitHub Actions E2E setup with xvfb (HIGH confidence)
- [tauri-apps/tauri#7068](https://github.com/tauri-apps/tauri/issues/7068) -- open issue for macOS tauri-driver support (HIGH confidence)
- [Tauri 2 Testing Overview](https://v2.tauri.app/develop/tests/) -- unit, integration, E2E strategy (HIGH confidence)

---
*Feature research for: Production infrastructure -- E2E testing, performance benchmarks, code signing, auto-updates (Tauri 2 desktop app)*
*Researched: 2026-03-26*
