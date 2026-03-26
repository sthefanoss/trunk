# Stack Research: E2E Testing, Performance Benchmarks, Code Signing & Auto-Updates

**Domain:** Desktop app production infrastructure (Tauri 2 + Svelte 5 + Rust)
**Researched:** 2026-03-26
**Confidence:** HIGH (E2E testing MEDIUM due to macOS WebDriver ecosystem immaturity)

## Existing Stack (DO NOT RE-ADD)

Already validated in v0.10: Tauri 2, Svelte 5, Vite 6, TypeScript 5.6, Tailwind CSS 4, Rust (git2 0.19, notify 7, tokio 1), Vitest, CI/CD with GitHub Actions (quality gates + cross-platform release builds), Homebrew cask distribution.

---

## Recommended Stack Additions

### 1. E2E Test Harness

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `@wdio/cli` | ^9.19 | WebdriverIO test runner CLI | Official Tauri-recommended E2E framework. W3C WebDriver protocol support. Tauri docs provide step-by-step setup with `wdio.conf.ts` examples. |
| `@wdio/local-runner` | ^9.19 | Local test runner for WebdriverIO | Executes tests on the local machine against the built Tauri app binary. |
| `@wdio/mocha-framework` | ^9.19 | Mocha test framework adapter | Standard choice from Tauri docs. Familiar describe/it syntax. 60s default timeout handles app startup latency. |
| `@wdio/spec-reporter` | ^9.19 | Console test output | Clear pass/fail output in terminal and CI logs. |
| `tauri-driver` | latest (cargo install) | WebDriver-to-Tauri bridge | Translates WebDriver protocol to platform-native webview driver. Required glue layer. Linux uses WebKitWebDriver, Windows uses Edge WebDriver. |

**Platform support matrix:**

| Platform | WebDriver Backend | CI Support | Status |
|----------|-------------------|------------|--------|
| Linux | WebKitWebDriver (webkit2gtk-driver) | Yes (xvfb-run) | Fully supported |
| Windows | Microsoft Edge WebDriver (msedgedriver) | Yes | Fully supported |
| macOS | None (Apple provides no WKWebView driver) | No | NOT SUPPORTED by official tooling |

**macOS E2E gap:** The official `tauri-driver` does not work on macOS because Apple does not provide a WebDriver for WKWebView. Three alternatives exist, none production-ready:

1. **tauri-plugin-webdriver-automation** (danielraffel) -- Open-source, 12 GitHub stars, 27 commits. Embeds Axum HTTP server in debug builds, translates WebDriver commands via JavaScript injection. macOS-only focus. Too immature for production use.
2. **CrabNebula tauri-driver fork** (`@crabnebula/tauri-driver`) -- Requires paid subscription. Not viable for a personal/open-source project.
3. **tauri-plugin-webdriver** (third-party) -- Claims cross-platform support but 404s on GitHub. Unverifiable.

**Recommendation:** Run E2E tests on Linux in CI (primary) and Windows (secondary). Accept macOS E2E gap for now. The existing unit test coverage (vitest + cargo test) combined with Linux E2E provides sufficient confidence. macOS-specific rendering bugs are rare in Tauri since WebKit behavior is consistent across the same engine version.

### 2. Performance Benchmarks

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `criterion` | 0.5.1 | Rust micro-benchmark harness | De facto standard for Rust benchmarking. Statistics-driven with automatic regression detection. Produces HTML reports. Measures wall-clock time with statistical significance testing. Use 0.5.1 (last stable in the 0.5.x line) rather than 0.8.x which requires Rust 1.88+ and is a major API change. |
| `vitest bench` | (bundled with vitest ^4.1) | TypeScript benchmark runner | Already have vitest. Built-in `bench()` function uses Tinybench underneath. Experimental but sufficient for comparing TypeScript transformation performance. No new dependency needed. |

**What to benchmark (Rust, criterion):**

| Benchmark Target | Why | Expected Baseline |
|------------------|-----|-------------------|
| `walk_commits` (lane algorithm) | Core graph computation, O(n). Regression here breaks UX. | ~5ms for 10k commits |
| `list_refs_inner` | Called on every sidebar refresh. Includes ahead/behind computation. | <10ms for typical repo |
| `diff_file` / `diff_stat` | Diff computation for staging panel. Large files can regress. | <50ms for 1MB file |
| Serde serialization of `GraphResponse` | IPC bottleneck for large commit histories. | <5ms for 500 commits |

**What to benchmark (TypeScript, vitest bench):**

| Benchmark Target | Why | Expected Baseline |
|------------------|-----|-------------------|
| Active Lanes transformation | O(lanes + merge_edges), called on every graph render. | <2ms for 10k commits |
| Trie flat-to-tree conversion | Called when staging panel refreshes. | <1ms for 500 files |

### 3. Code Signing

| Technology | Config/Tool | Purpose | Why Recommended |
|------------|-------------|---------|-----------------|
| Apple Developer ID certificate | Developer ID Application | macOS code signing + notarization | Required to distribute outside App Store without Gatekeeper warnings. Costs $99/year (Apple Developer Program). `tauri-action` handles signing + notarization automatically when environment variables are set. |
| Azure Trusted Signing | `trusted-signing-cli` (cargo install) | Windows code signing | Eliminates SmartScreen "unknown publisher" warning. Azure Trusted Signing is the modern approach (replaces deprecated OV certificates). Requires Azure account. |

**macOS signing -- environment variables for CI:**

| Variable | Purpose | How to Obtain |
|----------|---------|---------------|
| `APPLE_CERTIFICATE` | Base64-encoded .p12 certificate file | Export from Keychain Access, base64 encode |
| `APPLE_CERTIFICATE_PASSWORD` | Password for .p12 file | Set during export |
| `APPLE_SIGNING_IDENTITY` | Certificate common name (e.g., "Developer ID Application: Name (TEAMID)") | `security find-identity -v -p codesigning` |
| `APPLE_API_ISSUER` | App Store Connect API issuer ID | App Store Connect > Users and Access > Integrations > Keys |
| `APPLE_API_KEY` | API key ID | Same location as issuer |
| `APPLE_API_KEY_PATH` | Path to .p8 private key file | Downloaded once when key is created |

**Windows signing -- environment variables for CI:**

| Variable | Purpose |
|----------|---------|
| `AZURE_CLIENT_ID` | Azure App Registration client ID |
| `AZURE_CLIENT_SECRET` | App Registration secret |
| `AZURE_TENANT_ID` | Azure directory tenant ID |

**tauri.conf.json additions for signing:**

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": null
    },
    "windows": {
      "signCommand": "trusted-signing-cli -e https://wus2.codesigning.azure.net -a <Account> -c <Profile> %1"
    }
  }
}
```

Note: `signingIdentity` is set to `null` in config because the `APPLE_SIGNING_IDENTITY` environment variable takes precedence. This keeps secrets out of the config file.

**Integration with existing release workflow:** The current `release.yml` already uses `tauri-apps/tauri-action@v0`. Adding signing requires only setting the environment variables as GitHub Actions secrets -- tauri-action detects them automatically and applies signing + notarization. No YAML changes needed beyond `env:` blocks.

### 4. Auto-Updates

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `tauri-plugin-updater` | 2 | Rust plugin for update checking, downloading, and installing | Official Tauri 2 updater plugin. Handles signature verification, platform-aware downloads, and atomic replacement. Works with static JSON endpoint (GitHub Releases) or dynamic server. |
| `@tauri-apps/plugin-updater` | ^2 | JavaScript API for update checking and UI | Frontend API: `check()` returns update metadata, `downloadAndInstall()` handles the download. Pairs with plugin-process for relaunch. |
| `@tauri-apps/plugin-process` | ^2 | App relaunch after update | Provides `relaunch()` to restart the app after update installation. Required companion for the updater plugin. |
| `tauri-plugin-process` | 2 | Rust side of process plugin | Backend for the relaunch capability. |

**Update signing (separate from code signing):**

The updater requires its own keypair for verifying update integrity. This is NOT the Apple/Windows code signing certificate -- it's Tauri's own Ed25519 signature system.

```bash
# Generate once, store securely
bun run tauri signer generate -- -w ~/.tauri/trunk.key
```

This produces a public key (goes in `tauri.conf.json`) and a private key (goes in CI secrets as `TAURI_SIGNING_PRIVATE_KEY`).

**tauri.conf.json additions for updater:**

```json
{
  "bundle": {
    "createUpdaterArtifacts": true
  },
  "plugins": {
    "updater": {
      "pubkey": "<GENERATED_PUBLIC_KEY>",
      "endpoints": [
        "https://github.com/joaofnds/trunk/releases/latest/download/latest.json"
      ]
    }
  }
}
```

**How it works with GitHub Releases:**

1. During build, `tauri-action` creates platform bundles + `.sig` signature files + `latest.json` manifest
2. `latest.json` is uploaded to the GitHub Release alongside binaries
3. App calls `check()` which fetches `latest.json` from the endpoint
4. If a newer version exists, `downloadAndInstall()` downloads the binary, verifies the signature, and replaces the running app
5. `relaunch()` restarts the app

**Capabilities (permissions) to add:**

```json
// src-tauri/capabilities/default.json
{
  "permissions": [
    "updater:default",
    "process:default"
  ]
}
```

**Build-time environment variables (CI):**

| Variable | Purpose |
|----------|---------|
| `TAURI_SIGNING_PRIVATE_KEY` | Ed25519 private key content or path -- signs update bundles |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for the private key (can be empty) |

---

## Installation

### Rust dependencies (src-tauri/Cargo.toml)

```toml
[dependencies]
tauri-plugin-updater = "2"
tauri-plugin-process = "2"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

### Benchmark harness setup (src-tauri/Cargo.toml)

```toml
[[bench]]
name = "graph"
harness = false

[[bench]]
name = "diff"
harness = false
```

### JavaScript dependencies

```bash
# Auto-update plugins
bun add @tauri-apps/plugin-updater @tauri-apps/plugin-process

# E2E test harness (dev only)
bun add -D @wdio/cli @wdio/local-runner @wdio/mocha-framework @wdio/spec-reporter
```

### CLI tools (CI)

```bash
# WebDriver bridge for E2E tests
cargo install tauri-driver --locked

# Windows code signing (Windows CI only)
cargo install trusted-signing-cli
```

### New files to create

```
e2e/wdio.conf.ts                           # WebdriverIO configuration
e2e/specs/*.ts                              # E2E test specs
src-tauri/benches/graph.rs                  # Criterion benchmarks for graph algorithm
src-tauri/benches/diff.rs                   # Criterion benchmarks for diff operations
.github/workflows/e2e.yml                   # E2E test workflow (Linux + Windows)
src-tauri/capabilities/default.json         # Add updater + process permissions
```

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| WebdriverIO (via `@wdio/cli`) | Selenium | If you need Java ecosystem integration. WebdriverIO is JavaScript-native, matches the project's toolchain, and has first-class Tauri examples in official docs. |
| WebdriverIO | Playwright | Not viable for Tauri. Playwright requires Chromium/Firefox/WebKit via its own browser engine, not WKWebView/WebView2. It cannot drive Tauri's native webview. |
| `tauri-driver` (official) | `tauri-plugin-webdriver-automation` | If macOS E2E is critical and you accept using an immature (12-star) project. For now, Linux-only E2E with `tauri-driver` is the safer choice. |
| `criterion` 0.5.1 | `criterion` 0.8.x | If your minimum Rust version is 1.88+. 0.8.x has breaking API changes (renamed macros, new group API). 0.5.1 is proven stable and widely used. |
| `criterion` | `divan` | If you want simpler syntax (`#[divan::bench]` attribute macros). divan is newer, less ecosystem tooling (no CI regression detection out of box). criterion's statistical analysis and HTML reports are more mature. |
| `criterion` | Rust nightly `#[bench]` | Never. Unstable, nightly-only, no statistics, no regression detection. |
| `vitest bench` | Dedicated JS benchmark lib (e.g., `benny`) | If vitest bench is too experimental. In practice, we only need relative comparisons for TS transforms, not publication-grade benchmarks. vitest bench is sufficient and adds zero dependencies. |
| GitHub Releases static JSON | CrabNebula Cloud | If you need a managed update server with analytics, staged rollouts, and CDN. Overkill and paid for a personal project. GitHub Releases is free and sufficient. |
| GitHub Releases static JSON | Self-hosted update server | If you need conditional updates (A/B testing, staged rollout). Adds operational burden. Not needed for a personal desktop app. |
| Apple Developer ID + Azure Trusted Signing | Skip signing entirely | Acceptable during development. Users get Gatekeeper/SmartScreen warnings but app still works. However, auto-updates REQUIRE code signing (macOS will reject unsigned updates). |
| App Store Connect API (notarization) | Apple ID + password | If you don't want to create API keys. Apple ID auth is simpler but less secure (requires app-specific password in CI secrets). API key approach is recommended by Apple. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Playwright for Tauri E2E | Cannot drive native WKWebView/WebView2. Playwright bundles its own browser engines. | WebdriverIO + tauri-driver |
| Cypress for Tauri E2E | Same issue as Playwright -- runs in Electron, cannot interact with native webview. | WebdriverIO + tauri-driver |
| `criterion` 0.8.x | Requires Rust 1.88+; major breaking API changes from 0.5.x. Project doesn't need the new features. | `criterion` 0.5.1 |
| `tauri-plugin-webdriver-automation` (danielraffel) in production | 12 stars, 27 commits, documented as "code written with Claude Code". Not production-ready. | Official `tauri-driver` on Linux/Windows |
| CrabNebula Cloud for updates | Paid service. GitHub Releases provides the same static JSON approach for free. | `tauri-plugin-updater` + GitHub Releases |
| `APPLE_ID` + `APPLE_PASSWORD` for notarization | Less secure than API key approach. Apple may deprecate password-based auth. | App Store Connect API (`APPLE_API_ISSUER` + `APPLE_API_KEY` + `APPLE_API_KEY_PATH`) |
| Manual `.sig` file management | Error-prone. `tauri-action` + `createUpdaterArtifacts: true` generates signatures automatically. | Let tauri-action handle it |
| `tauri signer generate` with no password | Private key at rest should be encrypted. Even if password is simple, it adds a layer of protection if key leaks. | Always set a password, store in `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` |

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `tauri-plugin-updater = "2"` | `tauri = "2"` | Must match major version. Plugin follows Tauri's release cadence. |
| `@tauri-apps/plugin-updater@^2` | `@tauri-apps/api@^2` | Frontend plugins must match the Tauri API major version. |
| `tauri-plugin-process = "2"` | `tauri-plugin-updater = "2"` | Process plugin provides `relaunch()` needed after update installation. Both are v2 plugins. |
| `criterion = "0.5"` | Rust stable 1.70+ | 0.5.x works on current stable. Only in `[dev-dependencies]`, no production impact. |
| `@wdio/*@^9.19` | Node 18+ / Bun | WebdriverIO 9.x is the current major. All @wdio packages must be same major version. |
| `tauri-driver` (cargo install) | `tauri = "2"` | tauri-driver from the tauri-apps org. Install latest via `cargo install tauri-driver --locked`. |
| `createUpdaterArtifacts: true` | `tauri-apps/tauri-action@v0` | tauri-action reads this config flag and generates `latest.json` + `.sig` files automatically. |
| Code signing env vars | `tauri-apps/tauri-action@v0` | tauri-action detects `APPLE_CERTIFICATE`, `APPLE_SIGNING_IDENTITY`, etc. and applies signing during bundle step. No extra YAML needed. |
| `TAURI_SIGNING_PRIVATE_KEY` | `createUpdaterArtifacts: true` | Build fails if `createUpdaterArtifacts` is true but signing key is not set. Both must be configured together. |

## Integration with Existing CI

### Current release.yml changes needed

The existing `release.yml` needs these additions:

1. **Signing environment variables** -- Add `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH` as secrets to the tauri-action step's `env:` block.
2. **Updater signing** -- Add `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` to the same `env:` block.
3. **Windows signing** -- Add `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID` for Windows matrix entries.
4. **No structural YAML changes** -- tauri-action reads all config from `tauri.conf.json` and environment variables.

### New e2e.yml workflow

Separate workflow from CI. E2E tests are slow (app build + WebDriver + test execution) and should not block fast lint/test feedback.

```
Trigger: push to main, pull_request
Platforms: ubuntu-22.04 (primary), windows-latest (secondary)
Steps:
  1. Checkout
  2. Install system deps (Linux: webkit2gtk-driver, xvfb)
  3. Setup Bun + Rust
  4. Build debug app: bun run tauri build --debug --no-bundle
  5. Install tauri-driver
  6. Run WebdriverIO: xvfb-run bun run wdio (Linux), bun run wdio (Windows)
```

### Benchmark workflow (optional, CI integration)

Criterion benchmarks can run in CI to detect regressions. Two approaches:

1. **Manual:** Run `cargo bench` locally, review HTML reports in `target/criterion/`.
2. **CI gate:** Use `bencher.dev` or `github-action-benchmark` to track benchmark results across commits and fail CI on regressions exceeding a threshold.

Recommendation: Start with local-only benchmarks. Add CI integration later if regression detection becomes necessary.

## Sources

- [Tauri 2 WebDriver docs](https://v2.tauri.app/develop/tests/webdriver/) -- Official E2E testing guide (HIGH confidence)
- [Tauri 2 WebdriverIO example](https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/) -- Step-by-step setup (HIGH confidence)
- [Tauri 2 CI WebDriver workflow](https://v2.tauri.app/develop/tests/webdriver/ci/) -- GitHub Actions example (HIGH confidence)
- [Tauri 2 Updater plugin](https://v2.tauri.app/plugin/updater/) -- Full configuration guide (HIGH confidence)
- [@tauri-apps/plugin-updater npm](https://www.npmjs.com/package/@tauri-apps/plugin-updater) -- v2.10.0, published February 2026 (HIGH confidence)
- [tauri-plugin-updater docs.rs](https://docs.rs/crate/tauri-plugin-updater/latest) -- Rust API reference (HIGH confidence)
- [Tauri 2 macOS code signing](https://v2.tauri.app/distribute/sign/macos/) -- Certificate and notarization setup (HIGH confidence)
- [Tauri 2 Windows code signing](https://v2.tauri.app/distribute/sign/windows/) -- Azure Trusted Signing setup (HIGH confidence)
- [criterion.rs repository](https://github.com/bheisler/criterion.rs) -- v0.5.1 stable (HIGH confidence)
- [criterion docs.rs](https://docs.rs/crate/criterion/latest) -- v0.8.1 latest, v0.5.1 last stable 0.5.x (MEDIUM confidence -- version pinned from training data)
- [Tauri auto-updater with GitHub Releases guide](https://thatgurjot.com/til/tauri-auto-updater/) -- Practical setup walkthrough (MEDIUM confidence)
- [danielraffel/tauri-webdriver](https://github.com/danielraffel/tauri-webdriver) -- macOS WebDriver alternative, evaluated and rejected (HIGH confidence)
- [CrabNebula tauri-driver docs](https://docs.crabnebula.dev/plugins/tauri-e2e-tests/) -- Paid macOS E2E solution, evaluated and rejected (HIGH confidence)
- [Vitest benchmarking](https://vitest.dev/guide/features) -- Experimental bench() feature (MEDIUM confidence)

---
*Stack research for: E2E Testing, Performance Benchmarks, Code Signing & Auto-Updates (Trunk v1.0)*
*Researched: 2026-03-26*
