# Architecture Research: E2E Tests, Performance Benchmarks, Code Signing, and Auto-Updates

**Domain:** Production-readiness infrastructure for Tauri 2 desktop Git GUI
**Researched:** 2026-03-26
**Confidence:** HIGH

## System Overview

```
                         Existing Architecture
                               |
     +-------------------------+-------------------------+
     |                         |                         |
  Svelte 5 Frontend      Tauri IPC Layer          Rust Backend
  (TypeScript)          (invoke/listen)          (git2, state)
     |                         |                         |
     +-------------------------+-------------------------+
                               |
              +----------------+----------------+
              |                |                |
         GitHub CI        Release Pipeline   Homebrew Tap
         (ci.yml)         (release.yml)      (update-tap)
              |                |
              v                v
     +--------+--------+   +--+------------------+
     | NEW: E2E tests  |   | NEW: Code signing   |
     | NEW: Benchmarks |   | NEW: Updater JSON   |
     +--+-----------+--+   | NEW: Signing keys   |
        |           |       +---------------------+
        v           v
    CI Gate 3    CI Gate 3      +-------------------+
    (E2E)        (Bench)        | NEW: Auto-updater |
                                | (plugin + UI)     |
                                +-------------------+
```

### Integration Summary: What Is New vs Modified

| Component | Status | Description |
|-----------|--------|-------------|
| `src-tauri/benches/` | **NEW** | Criterion benchmark suite for Rust backend |
| `tests/e2e/` | **NEW** | WebdriverIO E2E test suite |
| `src-tauri/Cargo.toml` | **MODIFIED** | Add criterion, tauri-plugin-updater deps |
| `src-tauri/src/lib.rs` | **MODIFIED** | Register updater plugin |
| `src-tauri/tauri.conf.json` | **MODIFIED** | Add updater config, signing identity, entitlements |
| `src-tauri/capabilities/default.json` | **MODIFIED** | Add updater permissions |
| `.github/workflows/ci.yml` | **MODIFIED** | Add E2E test gate, benchmark gate |
| `.github/workflows/release.yml` | **MODIFIED** | Add signing env vars, updater JSON, signing keys |
| `src/lib/UpdateChecker.svelte` | **NEW** | Frontend update notification UI |
| `package.json` | **MODIFIED** | Add @tauri-apps/plugin-updater, @tauri-apps/plugin-process, wdio deps |
| `wdio.conf.ts` | **NEW** | WebdriverIO configuration |
| `src-tauri/Entitlements.plist` | **NEW** | macOS entitlements for notarization |

## Component Architecture

### 1. E2E Test Harness

```
┌─────────────────────────────────────────────────────────┐
│                    E2E Test Layer                         │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ WebdriverIO  │  │ Test Specs   │  │ tauri-driver  │  │
│  │ (test runner)│  │ (.test.ts)   │  │ (WebDriver    │  │
│  │              │  │              │  │  proxy)       │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
│         │                 │                  │          │
│         └─────────────────┴──────────────────┘          │
│                           │                             │
│                    WebDriver Protocol                    │
│                           │                             │
│              ┌────────────┴─────────────┐               │
│              │  Platform WebDriver      │               │
│              │  Linux: WebKitWebDriver  │               │
│              │  Windows: msedgedriver   │               │
│              │  macOS: NOT SUPPORTED    │               │
│              └────────────┬─────────────┘               │
│                           │                             │
│              ┌────────────┴─────────────┐               │
│              │  Trunk App (debug build) │               │
│              │  --debug --no-bundle     │               │
│              └──────────────────────────┘               │
└─────────────────────────────────────────────────────────┘
```

**How it integrates with existing architecture:**

- Tests interact with the Svelte frontend via WebDriver selectors (CSS selectors, text content)
- The app is built with `--debug --no-bundle` for fast iteration (no packaging step)
- `tauri-driver` acts as a proxy between WebdriverIO and the platform's native WebDriver
- Tests exercise the full stack: click a button in Svelte -> invoke fires -> Rust processes -> events emit -> UI updates
- A test git repository is created in a temp directory before each test suite (using `git init` + scripted commits)

**Platform constraint:** Official `tauri-driver` does NOT support macOS. Apple provides no WKWebView WebDriver. E2E tests run on Linux in CI. The third-party `tauri-webdriver` project (February 2026) exists for macOS but is experimental. Recommendation: run E2E on Linux only in CI; use local manual testing on macOS.

**Component responsibilities:**

| Component | Responsibility | Location |
|-----------|----------------|----------|
| `wdio.conf.ts` | Test runner config: binary path, tauri-driver spawn, timeout | Project root |
| `tests/e2e/*.test.ts` | Test specs: open repo, stage file, commit, etc. | `tests/e2e/` |
| `tests/e2e/fixtures/` | Helper scripts to create test git repos | `tests/e2e/fixtures/` |
| `tauri-driver` | WebDriver proxy (cargo-installed binary) | CI PATH |

### 2. Performance Benchmarks

```
┌─────────────────────────────────────────────────────────┐
│                  Benchmark Layer                         │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────────────┐     │
│  │ Criterion.rs     │  │ Benchmark Suites         │     │
│  │ (harness)        │  │                          │     │
│  └────────┬─────────┘  │ graph_bench.rs           │     │
│           │            │   walk_commits (10/100/  │     │
│           │            │   1k/10k commits)        │     │
│           │            │                          │     │
│           │            │ diff_bench.rs            │     │
│           │            │   diff computation       │     │
│           │            │                          │     │
│           │            │ search_bench.rs          │     │
│           │            │   commit search          │     │
│           │            └──────────────────────────┘     │
│           │                                             │
│      cargo bench                                        │
│           │                                             │
│  ┌────────┴─────────────────────────────────────────┐   │
│  │  Existing Rust Backend (no changes needed)       │   │
│  │  git/graph.rs  walk_commits_inner()              │   │
│  │  git/repository.rs  diff/search functions        │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

**How it integrates with existing architecture:**

- Benchmarks call the `_inner` functions directly (the established inner-fn pattern for testable Tauri commands)
- No Tauri runtime needed -- pure Rust functions operating on `git2::Repository`
- Test repositories are created programmatically using `git2` (not CLI) for reproducibility
- Criterion produces HTML reports and statistical comparison against previous runs
- The existing `walk_commits_inner()` in `git/graph.rs` is the primary benchmark target (~5ms for 10k commits per PROJECT.md)

**Key benchmark targets (by existing codebase):**

| Function | File | Why Benchmark |
|----------|------|---------------|
| `walk_commits_inner()` | `git/graph.rs` | Core algorithm, O(n), regression detection |
| `diff_unstaged_inner()` | `commands/diff.rs` | Large repos can have many changed files |
| `search_commits_inner()` | `commands/history.rs` | Full-text search over commit graph |
| `stage_hunk_inner()` | `commands/staging.rs` | Patch application performance |
| `list_refs_inner()` | `commands/branches.rs` | Branch listing with ahead/behind |

**Cargo.toml additions:**

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "graph_bench"
harness = false

[[bench]]
name = "diff_bench"
harness = false
```

### 3. Code Signing

```
┌─────────────────────────────────────────────────────────┐
│               Code Signing Layer                         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────── macOS ────────────┐                       │
│  │ Developer ID Application     │                       │
│  │ Certificate (.p12)           │                       │
│  │           │                  │                       │
│  │  APPLE_CERTIFICATE (base64)  │                       │
│  │  APPLE_CERTIFICATE_PASSWORD  │                       │
│  │  APPLE_SIGNING_IDENTITY      │                       │
│  │           │                  │                       │
│  │  tauri-action imports cert   │                       │
│  │  into temp keychain          │                       │
│  │           │                  │                       │
│  │  codesign + Entitlements     │                       │
│  │           │                  │                       │
│  │  xcrun notarytool submit     │                       │
│  │  (APPLE_ID + APPLE_PASSWORD  │                       │
│  │   + APPLE_TEAM_ID)           │                       │
│  │           │                  │                       │
│  │  xcrun stapler staple .dmg   │                       │
│  └──────────────────────────────┘                       │
│                                                         │
│  ┌─────────── Windows ──────────┐                       │
│  │ OV/EV cert (Azure Key Vault  │                       │
│  │ or local PFX)                │                       │
│  │           │                  │                       │
│  │  signtool sign /f cert.pfx   │                       │
│  │  (tauri handles via env vars)│                       │
│  └──────────────────────────────┘                       │
│                                                         │
│  ┌─────────── Linux ────────────┐                       │
│  │ No code signing standard.    │                       │
│  │ AppImage is distributed      │                       │
│  │ unsigned.                    │                       │
│  └──────────────────────────────┘                       │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**How it integrates with existing architecture:**

- `tauri-action@v0` (already used in `release.yml`) handles ALL signing/notarization when env vars are present
- Zero code changes in the Rust or Svelte source -- this is purely CI/CD configuration
- The `tauri.conf.json` gets a `macOS.signingIdentity` field and `Entitlements.plist` reference
- GitHub Secrets provide credentials; `release.yml` passes them as env vars to tauri-action
- Notarization is automatic: tauri-action submits to Apple, polls for completion, staples the ticket to the .dmg

**GitHub Secrets required:**

| Secret | Platform | Purpose |
|--------|----------|---------|
| `APPLE_CERTIFICATE` | macOS | Base64-encoded .p12 certificate |
| `APPLE_CERTIFICATE_PASSWORD` | macOS | .p12 export password |
| `APPLE_SIGNING_IDENTITY` | macOS | "Developer ID Application: Name (TEAMID)" |
| `APPLE_ID` | macOS | Apple account email for notarization |
| `APPLE_PASSWORD` | macOS | App-specific password for notarization |
| `APPLE_TEAM_ID` | macOS | 10-character team identifier |

**Windows signing** requires an OV/EV certificate. Since June 2023, new code signing certificates must use HSMs (Azure Key Vault is the most accessible option). This is optional for initial release -- Windows SmartScreen warnings are annoying but not blocking for a personal-use tool. Can be added later.

### 4. Auto-Updates

```
┌─────────────────────────────────────────────────────────┐
│                Auto-Update Layer                         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌────── Build Time ──────────────────────────────┐     │
│  │                                                │     │
│  │  TAURI_SIGNING_PRIVATE_KEY (env var)            │     │
│  │           │                                    │     │
│  │  tauri build produces:                         │     │
│  │    trunk_0.2.0_aarch64.dmg                     │     │
│  │    trunk_0.2.0_aarch64.dmg.sig  <-- signature  │     │
│  │           │                                    │     │
│  │  tauri-action generates:                       │     │
│  │    latest.json  (version, platforms, sigs)     │     │
│  │           │                                    │     │
│  │  All uploaded to GitHub Release                │     │
│  └────────────────────────────────────────────────┘     │
│                                                         │
│  ┌────── Runtime ─────────────────────────────────┐     │
│  │                                                │     │
│  │  Rust: tauri-plugin-updater registered         │     │
│  │           │                                    │     │
│  │  Frontend:                                     │     │
│  │    check() -> fetches latest.json from GitHub  │     │
│  │           │                                    │     │
│  │    update.version > current_version?           │     │
│  │           │                                    │     │
│  │    YES -> show toast/banner with "Update       │     │
│  │           available: v0.3.0"                   │     │
│  │           │                                    │     │
│  │    User clicks "Update" ->                     │     │
│  │      downloadAndInstall(progressCallback)      │     │
│  │           │                                    │     │
│  │    relaunch() via @tauri-apps/plugin-process   │     │
│  │                                                │     │
│  └────────────────────────────────────────────────┘     │
│                                                         │
│  ┌────── Endpoint ────────────────────────────────┐     │
│  │                                                │     │
│  │  https://github.com/joaofnds/trunk/releases/   │     │
│  │    latest/download/latest.json                 │     │
│  │                                                │     │
│  │  Static file served by GitHub Releases.        │     │
│  │  No custom server needed.                      │     │
│  │  tauri-action@v0 with uploadUpdaterJson: true  │     │
│  │  generates and uploads this automatically.     │     │
│  └────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────┘
```

**How it integrates with existing architecture:**

- **Rust backend (`lib.rs`):** One line added -- register `tauri_plugin_updater::Builder::new().build()` in the plugin chain (same pattern as existing dialog, store, clipboard plugins)
- **Frontend:** New `UpdateChecker.svelte` component imported in `App.svelte`. Uses `@tauri-apps/plugin-updater` to check on app launch, shows toast notification via existing toast system
- **Capabilities:** Add `"updater:default"` to `default.json` permissions array
- **Release workflow:** Add `TAURI_SIGNING_PRIVATE_KEY` env var and `uploadUpdaterJson: true` to tauri-action
- **Config:** `tauri.conf.json` gets `plugins.updater` block with public key and GitHub endpoint

**Signing key relationship:** The updater signing key is SEPARATE from the macOS code signing certificate. The updater key is a Tauri-specific Ed25519 keypair that signs the update binary so the client can verify integrity. The macOS certificate signs the .app bundle so macOS trusts it. Both are needed for a fully signed + auto-updating release.

## Data Flow

### E2E Test Flow

```
Test Runner (WebdriverIO)
    |
    | WebDriver Protocol (HTTP)
    v
tauri-driver (proxy)
    |
    | Platform WebDriver Protocol
    v
WebKitWebDriver (Linux) / msedgedriver (Windows)
    |
    | DOM Interaction
    v
Trunk App (Svelte frontend)
    |
    | invoke("open_repo", { path })
    v
Rust Backend (RepoState, CommitCache)
    |
    | emit("repo-changed")
    v
Trunk App (event listener updates UI)
    |
    | WebDriver reads updated DOM
    v
Test Assertion (expect text/element)
```

### Auto-Update Flow

```
App Launch
    |
    v
check() -> GET https://github.com/joaofnds/trunk/releases/latest/download/latest.json
    |
    v
Compare latest.json.version with tauri.conf.json version
    |
    +-- No update -> silent, do nothing
    |
    +-- Update available ->
            |
            v
        Show toast: "Update v0.3.0 available"
            |
            +-- User ignores -> dismiss after timeout
            |
            +-- User clicks "Update" ->
                    |
                    v
                downloadAndInstall(progress callback)
                    |
                    v
                Show download progress in toast/bar
                    |
                    v
                relaunch() via plugin-process
```

### Benchmark Flow

```
cargo bench --manifest-path src-tauri/Cargo.toml
    |
    v
Criterion harness
    |
    v
Setup: create temp git repo with N commits (git2 API)
    |
    v
Run: walk_commits_inner(&repo_path) x 100 iterations
    |
    v
Measure: mean, std dev, confidence interval
    |
    v
Compare: against saved baseline (target/criterion/)
    |
    v
Output: HTML report + terminal summary
    |
    v
CI: fail if regression > 10% (criterion threshold)
```

### Code Signing Flow (CI)

```
Tag push (v*)
    |
    v
release.yml triggers
    |
    v
tauri-action@v0
    |
    +-- macOS runner:
    |       Import APPLE_CERTIFICATE into temp keychain
    |       Build aarch64 / x86_64 targets
    |       codesign with APPLE_SIGNING_IDENTITY + Entitlements.plist
    |       xcrun notarytool submit (APPLE_ID + APPLE_PASSWORD + APPLE_TEAM_ID)
    |       Poll notarization status (2-5 minutes)
    |       xcrun stapler staple .dmg
    |       Generate .dmg.sig (TAURI_SIGNING_PRIVATE_KEY)
    |       Upload to GitHub Release
    |
    +-- Windows runner:
    |       (Optional) Import certificate
    |       Build x86_64
    |       signtool sign (if cert available)
    |       Generate .msi.sig
    |       Upload to GitHub Release
    |
    +-- Linux runner:
    |       Build x86_64
    |       Generate .AppImage.sig
    |       Upload to GitHub Release
    |
    +-- After all builds:
            Generate latest.json (merges all platform sigs + URLs)
            Upload latest.json to GitHub Release
```

## Recommended Project Structure Changes

```
trunk/
├── src-tauri/
│   ├── benches/                    # NEW: Criterion benchmarks
│   │   ├── graph_bench.rs          # walk_commits performance
│   │   └── diff_bench.rs           # diff computation performance
│   ├── src/
│   │   └── lib.rs                  # MODIFIED: add updater plugin
│   ├── Cargo.toml                  # MODIFIED: add criterion, tauri-plugin-updater
│   ├── tauri.conf.json             # MODIFIED: updater config, signing identity
│   ├── capabilities/
│   │   └── default.json            # MODIFIED: add updater:default permission
│   └── Entitlements.plist          # NEW: macOS entitlements for notarization
├── tests/
│   └── e2e/
│       ├── specs/                  # NEW: E2E test specs
│       │   ├── open-repo.test.ts   # Open repo and verify graph loads
│       │   ├── staging.test.ts     # Stage/unstage/commit flow
│       │   └── navigation.test.ts  # Branch checkout, search, etc.
│       └── fixtures/
│           └── create-test-repo.sh # Script to generate test git repos
├── src/
│   └── lib/
│       └── UpdateChecker.svelte    # NEW: update notification component
├── wdio.conf.ts                    # NEW: WebdriverIO configuration
├── .github/
│   └── workflows/
│       ├── ci.yml                  # MODIFIED: add E2E + benchmark gates
│       └── release.yml             # MODIFIED: add signing + updater JSON
└── package.json                    # MODIFIED: add wdio + updater deps
```

### Structure Rationale

- **`src-tauri/benches/`:** Standard Rust convention; Criterion requires `[[bench]]` entries in `Cargo.toml` pointing to files in `benches/`
- **`tests/e2e/`:** Separates E2E from unit tests (`src/lib/*.test.ts`); WebdriverIO expects a dedicated spec directory
- **`Entitlements.plist` in `src-tauri/`:** Tauri expects it relative to the Tauri project root
- **`UpdateChecker.svelte` in `src/lib/`:** Follows existing component pattern; imported by `App.svelte`

## Architectural Patterns

### Pattern 1: Plugin Registration Chain

**What:** Tauri 2 plugins are registered as a builder chain in `lib.rs`. The updater follows the same pattern as existing plugins (dialog, store, clipboard, window-state).

**When to use:** Adding any new Tauri capability.

**Example (existing + new):**

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_store::Builder::default().build())
    .plugin(tauri_plugin_window_state::Builder::new().build())
    .plugin(tauri_plugin_clipboard_manager::init())
    // NEW: updater plugin (desktop only)
    .setup(|app| {
        #[cfg(desktop)]
        app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
        Ok(())
    })
```

**Trade-offs:** The updater must be registered in `.setup()` (not `.plugin()`) because it needs the app handle. This is the official pattern from Tauri docs. The `#[cfg(desktop)]` guard prevents compilation issues on mobile targets.

### Pattern 2: Inner-fn Benchmarking

**What:** The existing inner-fn pattern (`walk_commits_inner`, `diff_unstaged_inner`, etc.) separates pure Rust logic from Tauri state extraction. Benchmarks call these inner functions directly.

**When to use:** Writing benchmarks for any Tauri command.

**Example:**

```rust
// In benches/graph_bench.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use trunk_lib::git::graph::walk_commits_inner;

fn bench_walk_commits(c: &mut Criterion) {
    let repo_path = setup_test_repo(1000); // helper creates temp repo
    c.bench_with_input(
        BenchmarkId::new("walk_commits", "1000_commits"),
        &repo_path,
        |b, path| b.iter(|| walk_commits_inner(path)),
    );
}
```

**Trade-offs:** Requires inner functions to be `pub` (they already are for unit testing). The benchmark measures pure algorithm time without IPC overhead, which is what matters for regression detection.

### Pattern 3: Static Update Endpoint

**What:** The updater endpoint is a static `latest.json` file hosted on GitHub Releases, not a dynamic server. `tauri-action` generates it automatically.

**When to use:** Projects distributed via GitHub Releases (which Trunk already does).

**Example config:**

```json
{
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ...",
      "endpoints": [
        "https://github.com/joaofnds/trunk/releases/latest/download/latest.json"
      ]
    }
  }
}
```

**Trade-offs:** No server to maintain. GitHub Releases has high availability. The downside is no usage analytics or staged rollouts -- but those are overkill for a personal-use tool.

### Pattern 4: Toast-Based Update Notification

**What:** Reuse the existing toast notification system to show update availability. No new UI paradigm needed.

**When to use:** Non-intrusive update notifications.

**Example flow:**

```typescript
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { addToast } from '$lib/toast.svelte';

const update = await check();
if (update) {
    addToast({
        kind: 'info',
        message: `Update ${update.version} available`,
        action: {
            label: 'Update',
            onClick: async () => {
                await update.downloadAndInstall();
                await relaunch();
            }
        }
    });
}
```

**Trade-offs:** Simple and consistent with existing UX. However, the existing toast system may need a minor extension to support an action button (currently toasts are display-only with auto-dismiss). This is a small addition, not a rewrite.

## CI/CD Integration

### Modified CI Pipeline (ci.yml)

```
Gate 1 (fast, ~10-30s):          Gate 2 (heavy, needs Gate 1):
  biome                            cargo-clippy
  cargo-fmt                        cargo-test
  svelte-check                     vitest

                                 Gate 3 (NEW, needs Gate 2):
                                   e2e-tests (Linux only)
                                   benchmarks (Linux only)
```

**E2E tests as Gate 3:** They require a full debug build (~2-3 minutes) plus test execution (~1-2 minutes). Placing them after Gate 2 ensures we don't waste CI time on E2E if basic checks fail. Running on Linux only because macOS WebDriver is unsupported.

**Benchmarks as Gate 3:** `cargo bench` compiles in release mode (slower than debug). Running after Gate 2 gates prevents wasted compute. The benchmark job reports results but does NOT fail the build initially -- establish baselines first, then add regression thresholds.

### Modified Release Pipeline (release.yml)

The existing `release.yml` needs these additions to the `build` job's `tauri-apps/tauri-action@v0` step:

```yaml
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  # NEW: macOS code signing
  APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
  APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
  APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
  APPLE_ID: ${{ secrets.APPLE_ID }}
  APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
  APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
  # NEW: updater signing
  TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
with:
  tagName: ${{ github.ref_name }}
  releaseName: ${{ github.ref_name }}
  releaseDraft: true
  prerelease: ${{ contains(github.ref_name, '-') }}
  args: ${{ matrix.args }}
  tauriScript: bunx tauri
  # NEW: generate latest.json for auto-updater
  uploadUpdaterJson: true
```

`tauri-action` automatically:
1. Detects macOS env vars and runs codesign + notarytool
2. Generates `.sig` files for each platform bundle using `TAURI_SIGNING_PRIVATE_KEY`
3. Creates `latest.json` with all platform signatures and download URLs
4. Uploads everything to the GitHub Release

## Anti-Patterns

### Anti-Pattern 1: Testing E2E on macOS CI

**What people do:** Try to run `tauri-driver` E2E tests on macOS GitHub Actions runners.
**Why it's wrong:** macOS has no WKWebView WebDriver. Tests will hang or crash. The official docs explicitly state "macOS does not provide a desktop WebDriver client."
**Do this instead:** Run E2E on Linux (ubuntu-22.04) only. Use manual testing for macOS-specific issues. Consider the experimental `tauri-webdriver` project only if macOS-specific E2E becomes critical.

### Anti-Pattern 2: Storing Signing Keys in Config Files

**What people do:** Put the updater private key or Apple certificate in `tauri.conf.json` or commit them to git.
**Why it's wrong:** Private keys in version control are a security breach.
**Do this instead:** Store in GitHub Secrets. The public key goes in `tauri.conf.json` (that's safe). The private key is only available via `TAURI_SIGNING_PRIVATE_KEY` env var at build time.

### Anti-Pattern 3: Benchmarking Through Tauri IPC

**What people do:** Write benchmarks that invoke Tauri commands through the full IPC pipeline.
**Why it's wrong:** Measures IPC serialization overhead, not algorithm performance. Results are noisy and non-reproducible.
**Do this instead:** Benchmark the `_inner` functions directly. The inner-fn pattern exists precisely for this purpose.

### Anti-Pattern 4: Blocking UI During Update Download

**What people do:** Show a modal dialog during update download, blocking the user.
**Why it's wrong:** Users are in the middle of git work. Blocking them loses their context.
**Do this instead:** Non-blocking toast notification. User clicks "Update" only when ready. Download happens in background. Relaunch only after download completes.

### Anti-Pattern 5: Running Benchmarks on Every Push

**What people do:** Add `cargo bench` to the fast CI gate.
**Why it's wrong:** Benchmarks compile in release mode (~3-5 minutes). Running on every push wastes CI time and money.
**Do this instead:** Run benchmarks in Gate 3 (after all fast checks pass) or only on `main` branch pushes.

## Build Order (Dependency-Driven)

Based on dependencies between the four features:

```
Phase 1: Performance Benchmarks
    ├── No external dependencies (pure Rust, Criterion only)
    ├── No CI changes needed initially (run locally first)
    ├── Validates inner-fn pattern works for benchmarking
    └── Creates baseline metrics before adding signing overhead

Phase 2: E2E Test Harness
    ├── Depends on: nothing (independent of benchmarks)
    ├── Requires: tauri-driver install, WebdriverIO setup
    ├── Modifies: ci.yml (adds Gate 3)
    └── Creates test infrastructure for validating future features

Phase 3: Code Signing
    ├── Depends on: Apple Developer Account ($99/year)
    ├── Requires: certificate generation, GitHub Secrets setup
    ├── Modifies: release.yml, tauri.conf.json
    ├── Can be validated by: E2E tests (from Phase 2) + manual testing
    └── Required BEFORE auto-updates (updates must be signed)

Phase 4: Auto-Updates
    ├── Depends on: Code signing (Phase 3) -- updates MUST be signed
    ├── Depends on: TAURI_SIGNING_PRIVATE_KEY (generated in Phase 3)
    ├── Modifies: lib.rs, tauri.conf.json, capabilities, release.yml
    ├── Adds: UpdateChecker.svelte, @tauri-apps/plugin-updater
    └── Can be validated by: E2E tests (from Phase 2)
```

**Rationale for ordering:**
1. **Benchmarks first** because they have zero external dependencies and establish performance baselines
2. **E2E second** because they create validation infrastructure for Phases 3 and 4
3. **Code signing third** because auto-updates require signed binaries (Tauri updater REQUIRES a signature -- this cannot be disabled)
4. **Auto-updates last** because they depend on both code signing (for the update signature) AND the updater signing key (generated as part of signing setup)

## Sources

- [Tauri 2 WebDriver Testing](https://v2.tauri.app/develop/tests/webdriver/) - Official docs, HIGH confidence
- [Tauri 2 WebdriverIO Example](https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/) - Official example, HIGH confidence
- [Tauri 2 Updater Plugin](https://v2.tauri.app/plugin/updater/) - Official docs, HIGH confidence
- [Tauri 2 macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/) - Official docs, HIGH confidence
- [Tauri 2 Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) - Official docs, HIGH confidence
- [Tauri 2 GitHub Pipelines](https://v2.tauri.app/distribute/pipelines/github/) - Official docs, HIGH confidence
- [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) - Official action, HIGH confidence
- [CrabNebula Tauri E2E Tests](https://docs.crabnebula.dev/plugins/tauri-e2e-tests/) - Third-party (paid for macOS), MEDIUM confidence
- [tauri-webdriver (macOS)](https://github.com/danielraffel/tauri-webdriver) - Third-party, experimental, LOW confidence
- [Criterion.rs](https://bheisler.github.io/criterion.rs/book/getting_started.html) - Standard Rust benchmarking, HIGH confidence
- [Shipping Tauri with Code Signing](https://dev.to/0xmassi/shipping-a-production-macos-app-with-tauri-20-code-signing-notarization-and-homebrew-mc3) - Practical guide, MEDIUM confidence

---
*Architecture research for: E2E tests, benchmarks, code signing, auto-updates in Tauri 2 desktop app*
*Researched: 2026-03-26*
