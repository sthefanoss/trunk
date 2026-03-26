# Pitfalls Research

**Domain:** Adding E2E testing, performance benchmarks, code signing, and auto-updates to an existing Tauri 2 desktop Git GUI
**Researched:** 2026-03-26
**Confidence:** HIGH (verified against official Tauri v2 docs, GitHub issues, practitioner post-mortems, and analysis of existing CI/release pipeline)

---

## Context: What Is Changing in v1.0

v0.10 shipped CI quality gates and a cross-platform release pipeline (tag-triggered macOS ARM/Intel, Linux, Windows builds with .dmg/.AppImage/.msi installers, Homebrew tap auto-update). The builds are currently unsigned.

v1.0 adds four infrastructure features:
1. **E2E test harness** -- automated UI testing of the Tauri desktop app
2. **Performance benchmarks** -- Rust-side operation benchmarking with CI regression detection
3. **Code signing** -- macOS notarization + Windows signing for Gatekeeper/SmartScreen compliance
4. **Auto-updates** -- in-app update checking and installation via tauri-plugin-updater

Each feature has distinct pitfalls, and the interactions between them create additional failure modes.

---

## Critical Pitfalls

### Pitfall 1: Building auto-updates before code signing is complete

**What goes wrong:**
The Tauri updater requires two independent signing systems to function: (1) Tauri's own Ed25519 updater keypair that signs update artifacts, and (2) platform code signing (Apple Developer ID + notarization for macOS, EV/OV certificate for Windows). Developers often try to implement auto-updates first because the code is "more interesting," only to discover that unsigned builds cannot meaningfully test the update flow. macOS Gatekeeper blocks unsigned updates from installing, and the updater's signature verification is a separate layer on top of that.

**Why it happens:**
There are three distinct signing systems that look like one thing but are not:
- **Tauri updater signature** (Ed25519 keypair via `tauri signer generate`) -- verifies update integrity
- **macOS code signing** (Apple Developer ID certificate) -- satisfies Gatekeeper
- **macOS notarization** (Apple's server-side security scan) -- required for Developer ID apps

Developers conflate these. The updater pubkey in `tauri.conf.json` is NOT an Apple certificate. The Apple signing identity is NOT the updater key.

**How to avoid:**
Phase ordering must be: code signing first, then auto-updates. Code signing is a prerequisite for meaningful auto-update testing because:
1. Unsigned macOS apps get blocked by Gatekeeper, making update installation fail
2. The updater keypair is a separate task that belongs in the auto-update phase
3. Testing the full update cycle (check -> download -> install -> restart) requires a signed base build

**Warning signs:**
- Trying to test auto-updates with ad-hoc signed (`-`) builds
- Confusion about which key goes in `tauri.conf.json` pubkey field
- Building with `TAURI_SIGNING_PRIVATE_KEY` set but no Apple signing identity -- builds produce `.sig` files but the app itself is unsigned

**Phase to address:**
Code signing phase must complete before auto-update phase begins.

---

### Pitfall 2: E2E tests that cannot run on macOS

**What goes wrong:**
Tauri's official WebDriver testing uses `tauri-driver`, which wraps platform-native WebDriver servers. On Linux it uses WebKitWebDriver, on Windows it uses Edge WebDriver. But Apple does not ship a WKWebView WebDriver, and `safaridriver` only works with Safari itself, not with WKWebView inside a desktop app. Developers follow the official Tauri docs, get tests working in Linux CI, and then discover they cannot run or debug E2E tests locally on their Mac -- which is the primary development platform for Trunk.

**Why it happens:**
Apple simply does not provide a WebDriver implementation for WKWebView. The official Tauri docs acknowledge this: "On desktop, only Windows and Linux are supported due to macOS not having a WKWebView driver tool available." This is a hard platform limitation, not a configuration issue.

**How to avoid:**
Choose one of these approaches before writing any test infrastructure:

1. **Tauri WebDriver plugin** (recommended): Use `tauri-webdriver-automation` or `tauri-plugin-webdriver` which embed a WebDriver server directly inside the app (debug builds only). These expose a local HTTP endpoint compatible with WebdriverIO, working on all platforms including macOS. This is a newer approach (2025-2026) that solves the macOS gap.

2. **Two-layer testing**: (a) Rust integration tests using Tauri's mock runtime for command/IPC testing (extends the existing inner-fn pattern), and (b) WebDriver-based UI tests that run on Linux CI only, accepting that macOS UI automation is a CI-only activity.

3. **Skip WebDriver entirely**: Focus on Rust integration tests (which already exist) and TypeScript unit tests (vitest, already exists). Add a small set of manual smoke tests documented as a pre-release checklist.

**Warning signs:**
- E2E test setup docs that only mention Linux/Windows
- Tests requiring `WebKitWebDriver` or `msedgedriver` binary paths
- No story for running E2E tests locally on macOS

**Phase to address:**
E2E testing phase. The FIRST plan must evaluate and decide the WebDriver approach before writing any tests.

---

### Pitfall 3: Losing the updater private key permanently bricks update channel

**What goes wrong:**
The Tauri updater uses an Ed25519 keypair. The private key signs every update artifact (`.sig` files). Every installed copy of the app has the public key embedded and will reject any artifact signed with a different key. If the private key is lost, all existing installations become permanently unable to update. The only recovery is asking every user to manually download and reinstall -- which for a Git GUI used by developers is embarrassing at best.

**Why it happens:**
The key is generated once with `tauri signer generate`, stored as a GitHub Actions secret, and the original file is deleted or forgotten. If the repository is transferred, the organization changes, or the secret is accidentally deleted, the key is gone. Unlike Apple certificates which can be regenerated from Apple's portal, the updater key has no external authority -- it exists only where you put it.

**How to avoid:**
1. Generate locally: `tauri signer generate -w ~/.tauri/trunk.key`
2. Store the key in at least two durable locations outside GitHub (password manager + encrypted backup)
3. Add to GitHub Actions secrets as `TAURI_SIGNING_PRIVATE_KEY`
4. Document the key storage locations in a secure internal note
5. The public key goes in `tauri.conf.json` -- safe to commit
6. Add `*.key` to `.gitignore` as a safety net

**Warning signs:**
- Only one copy of the private key exists (in GitHub secrets)
- No documentation of where the key is stored
- `tauri signer generate` was run in CI and output was only captured as a secret
- Team member who generated the key has left (not applicable for solo project, but relevant for eventual open source)

**Phase to address:**
Auto-update phase, first task. Key generation and secure storage must happen before any updater code is written.

---

### Pitfall 4: Criterion benchmarks are meaningless in CI

**What goes wrong:**
Criterion measures wall-clock time. GitHub Actions runners are shared VMs with noisy neighbors, CPU throttling, and variable performance. Benchmark results fluctuate 10-30% between runs with identical code. Developers add benchmark CI gates, see phantom regressions on every PR, and eventually ignore or delete them. The Criterion FAQ explicitly warns about this: "The virtualization used by Cloud-CI providers introduces a great deal of noise into the benchmarking process."

**Why it happens:**
Criterion was designed for local developer machines with stable, dedicated hardware. Cloud CI runners share physical hosts, have unpredictable scheduling, and can be throttled at any time. The measurement noise exceeds the signal from typical code optimizations (which are often 1-10% improvements).

**How to avoid:**
Use `iai-callgrind` for CI benchmarks, Criterion for local development:

- **iai-callgrind** counts CPU instructions via Valgrind's Cachegrind. Instruction counts are deterministic -- they do not vary between CI runs regardless of VM noise. A 0.1% change in instruction count is reliably detectable. Requires `valgrind` installed on the runner (`sudo apt-get install valgrind` on Ubuntu).
- **Criterion** remains useful for local development where wall-clock time is meaningful and developers want human-readable output with statistical analysis.
- Platform limitation: iai-callgrind works on Linux (primary CI platform). Valgrind has limited macOS support and no Windows support. This aligns with running benchmarks only on Linux CI.

**Warning signs:**
- Benchmark CI jobs that fail intermittently without code changes
- Results varying more than 5% between identical commits
- Team ignoring benchmark failures as "CI noise"
- Using `criterion::Criterion::default().sample_size(10)` to speed up CI (reduces statistical power)

**Phase to address:**
Performance benchmarks phase. The tooling decision (Criterion vs iai-callgrind vs both) must be the first task.

---

### Pitfall 5: Apple notarization hanging or timing out in CI releases

**What goes wrong:**
Apple's notarization service (`notarytool`) can take 30 seconds to 2+ hours, and occasionally hangs indefinitely. Recent reports (March 2026) show submissions stuck "In Progress" for 72+ hours. GitHub Actions has a default 6-hour job timeout, so a hung notarization wastes CI minutes and blocks releases. Separately, the macOS keychain in CI requires careful setup -- if the temporary keychain is not properly configured, the signing step itself can hang waiting for a password prompt that never appears in a headless environment.

**Why it happens:**
Notarization is an external Apple service outside your control. Apple's servers have queue delays, especially during WWDC season or after major macOS releases. The `notarytool` CLI sometimes exits without reason on CI servers. Keychain-related hangs occur when the CI runner prompts for credentials in a headless environment where no user can respond.

**How to avoid:**
1. Add an explicit timeout (15-20 minutes) on the signing/notarization step with retry logic
2. Use App Store Connect API keys (not Apple ID + app-specific password) for CI -- they are more reliable and do not require 2FA handling
3. For keychain setup in CI: create a temporary keychain, set it as default, unlock it, configure `security set-keychain-settings -t 3600` to prevent premature locking
4. `tauri-action` handles notarization automatically when env vars are set, but add a job-level timeout as a safety net
5. Have a local release fallback: the ability to sign, notarize, and upload artifacts from a local Mac if CI notarization is broken

**Warning signs:**
- Release workflow macOS build step taking more than 30 minutes
- Intermittent "The request timed out" errors from Apple's notarization service
- Build appearing frozen at the signing step (keychain hang)
- No timeout configured on the notarization/signing steps

**Phase to address:**
Code signing phase. The CI integration plan must include timeout configuration and local fallback procedures.

---

### Pitfall 6: E2E test fixtures coupled to mutable git state

**What goes wrong:**
E2E tests for a Git GUI need test repositories with specific histories (merges, conflicts, stashes, branches, etc.). Tests become flaky because: (a) git operations are sensitive to timestamps, user.name/email config, and git version -- commits produce different SHAs on each run; (b) tests that modify shared fixtures corrupt state for subsequent tests; (c) test repos checked into the project tree accumulate cruft and `.git` directories cause nested-repo issues.

**Why it happens:**
Git repositories are inherently stateful and mutable. Unlike a JSON fixture, a git repo's behavior depends on system clock, user config, git version, and filesystem case sensitivity. The temptation is to create a test repo once and reuse it across tests, but any test that performs a write operation (commit, checkout, merge) mutates the shared state.

**How to avoid:**
1. Create fresh test repos per test using `tempfile` (Rust) or `os.tmpdir()` (Node)
2. Build a fixture library: deterministic functions that construct specific repo topologies (linear, branching, merge-conflict, stash, etc.)
3. Assert on structural properties (commit count, branch names, file contents, graph shape) not on SHAs or timestamps
4. Set `GIT_AUTHOR_DATE`, `GIT_COMMITTER_DATE`, `GIT_AUTHOR_NAME`, `GIT_COMMITTER_NAME` in test setup for reproducibility
5. Use `git2` crate (already in project) for fixture creation -- avoids dependence on system git version and provides consistent behavior across platforms

**Warning signs:**
- Tests that pass locally but fail in CI (or vice versa)
- Tests that fail when run in a different order
- Assertions on commit SHA values
- `.git` directories checked into the project tree
- Tests sharing a single test repository instance

**Phase to address:**
E2E testing phase. The test infrastructure plan must define the fixture strategy before writing individual tests.

---

### Pitfall 7: Updater endpoint misconfiguration with multi-platform matrix builds

**What goes wrong:**
The `latest.json` file must contain entries for all supported platforms (darwin-aarch64, darwin-x86_64, linux-x86_64, windows-x86_64). Trunk's release workflow uses a 4-platform matrix build. Each job generates its own artifacts. If `latest.json` does not merge all platform entries correctly, some platforms see "no update available" while others update fine. Additionally, the `{{current_version}}` variable in endpoint URLs is corrupted if the version contains a `+` character (e.g., `1.0.0+1`).

**Why it happens:**
Matrix builds run in parallel. The `tauri-action` with `includeUpdaterJson: true` handles merging platform entries into a single `latest.json` attached to the GitHub release. But Trunk's existing release workflow has a custom pipeline: build -> publish (remove draft) -> update Homebrew tap. Adding the updater JSON must not break this existing flow. If you customize the release creation/upload steps beyond what tauri-action expects, the JSON merging can fail silently.

**How to avoid:**
1. Add `includeUpdaterJson: true` to the existing `tauri-action` step in `release.yml`
2. Do NOT customize how the release is created/uploaded unless you understand how tauri-action merges platform entries across matrix jobs
3. After each release, validate `latest.json` structure: must contain entries for all four targets with valid URLs and signatures
4. Avoid `+` in version strings -- stick to plain semver (e.g., `0.2.0`, not `0.2.0+1`)
5. The updater endpoint URL should be `https://github.com/joaofnds/trunk/releases/latest/download/latest.json` -- this always points to the latest release
6. Ensure `createUpdaterArtifacts` is set to `true` (not `"v1Compatible"`) in `tauri.conf.json` since Trunk is a new Tauri 2 app with no v1 migration path

**Warning signs:**
- `latest.json` with fewer than 4 platform entries
- Update check returning 204 (no update) on some platforms but 200 on others
- Version string containing `+` characters
- Missing `.sig` signature files alongside the installer artifacts in the release

**Phase to address:**
Auto-update phase. The release workflow integration plan must address `latest.json` generation and include post-release validation.

---

### Pitfall 8: Benchmarking git2 operations on unrealistic test repos

**What goes wrong:**
Benchmarks run against tiny test repositories (5-10 commits) show sub-millisecond times for all operations with no discrimination between fast and slow paths. The numbers are meaningless for detecting regressions because measurement noise exceeds the actual operation time. Developers declare "everything is fast" and miss that `walk_commits` on a 100k-commit repo takes 500ms, or that commit graph rendering degrades quadratically with branch count.

**Why it happens:**
Creating realistic benchmark fixtures is tedious. The temptation is to use a tiny repo that is quick to set up. But Trunk's performance-critical operations (commit walking, graph lane algorithm, SVG coordinate computation) are O(n) or worse in commit count, and their real-world behavior only manifests at scale.

**How to avoid:**
1. Create benchmark fixtures at multiple scales: 100, 1k, 10k, 100k commits
2. Include topological variety: linear history, highly branched, octopus merges, many refs
3. For the Rust lane algorithm (the main performance concern): benchmark with repos that exercise max_columns growth, lane recycling, and merge edge routing
4. Use `git2` to programmatically generate fixtures (already available in the project) rather than cloning external repos (which adds network dependency and non-determinism)
5. Establish baselines before any optimization work so improvements are measurable

**Warning signs:**
- All benchmark results under 1ms
- No variety in commit counts across benchmark cases
- Benchmarks that create repos with `git init && git commit` in a loop (too simple topology)
- No benchmark for the frontend-side Active Lanes transformation or SVG coordinate computation

**Phase to address:**
Performance benchmarks phase. Fixture creation is part of the benchmark infrastructure, not an afterthought.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skip Windows code signing | Saves $200-400/year for certificate, simpler CI | SmartScreen blocks unsigned apps with scary warnings | During beta/personal use. Must sign before public open-source release. |
| Ad-hoc macOS signing (`-`) for dev builds | No Apple Developer account needed | Cannot notarize, Gatekeeper blocks on other machines, auto-updates will not work | Local development testing only. Never for distributed builds. |
| Criterion-only benchmarks (no iai-callgrind) | Familiar API, simpler setup, no Valgrind dependency | Unreliable CI results, phantom regressions, eventually ignored | Acceptable if benchmarks are local-only with no CI gate. |
| Hardcoded test timeouts in E2E | Quick to write | Tests fail on slow CI runners, pass on fast machines | Never. Use polling/waitFor patterns with reasonable upper bounds. |
| Single-platform E2E tests (Linux CI only) | 3x faster CI, simpler setup | macOS WebKit rendering differences and Windows path issues slip through | Acceptable for v1.0 if combined with manual pre-release testing on macOS/Windows. |
| Store updater key only in GitHub secrets | Simple key management | Key loss = bricked update channel for all users | Never. Always maintain a backup in a durable, separate location. |
| Skipping update UX (silent background update) | Less frontend code to write | App restarts without warning, user loses unsaved commit message | Never. Always show notification and let user control restart timing. |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| tauri-action + updater | Not adding `includeUpdaterJson: true` so `latest.json` is never attached to the release | Add the flag to the existing tauri-action step in `release.yml` |
| tauri-action + code signing | Setting `APPLE_SIGNING_IDENTITY` but forgetting `APPLE_API_KEY`/`APPLE_API_ISSUER` for notarization | Both signing AND notarization env vars must be set. Use App Store Connect API keys. |
| Updater + existing release workflow | Trunk's `release.yml` creates drafts, publishes, then updates Homebrew tap. Adding updater JSON must not break this pipeline. | The `latest.json` is attached to the release during the build job. The publish and tap-update jobs run after. Verify `latest.json` appears in the published release. |
| WebDriver + Tauri dev mode | Running E2E tests against `cargo tauri dev` which hot-reloads and recompiles on changes | Build a release/debug binary once, then run E2E tests against the built artifact. Dev mode hot-reload interferes with test stability. |
| iai-callgrind + GitHub Actions | Assuming Valgrind is pre-installed on runners | Ubuntu runners need `sudo apt-get install valgrind`. macOS/Windows have no Valgrind. Run iai benchmarks on Linux only. |
| Updater + `createUpdaterArtifacts` | Not adding this field to `tauri.conf.json`, so builds produce installers but no `.sig` files | Set `"createUpdaterArtifacts": true` (not `"v1Compatible"` -- Trunk is Tauri 2 native). |
| Updater + `TAURI_SIGNING_PRIVATE_KEY` | Not setting this env var before building, so `.sig` files are empty or missing | Set as environment variable in the release workflow. `.env` files do NOT work for this. |
| Code signing + Homebrew cask | Signed DMG has different SHA256 than unsigned DMG | The `update-tap` job already computes SHA256 from downloaded DMGs. No change needed, but verify the SHA in the cask matches the signed artifact. |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Benchmarking git2 on 10-commit repos | All results under 0.1ms, no discrimination | Use fixtures at 100, 1k, 10k, 100k commits | Immediately -- tiny repos produce meaningless numbers |
| E2E tests launching fresh app per test | Suite takes 10+ minutes, CI times out | Launch app once per suite, navigate states within session | At ~20 tests. App startup is 1-3s each. |
| Notarization in serial across platforms | Release takes 30+ min because platforms notarize one at a time | Matrix builds already parallelize. Notarization happens inside each matrix job. | Already handled by existing matrix structure. |
| Criterion benchmarks with debug builds | Results 10-100x slower than release, masking real characteristics | Always use `--release`. Criterion does this by default; custom harnesses may not. | Immediately if using non-Criterion bench framework. |
| Running all E2E tests on every push | CI time grows linearly with test count | Run E2E only on PRs and main branch, not on every push. Use `paths-ignore` for docs-only changes. | At ~50 E2E tests (~10 min runtime). |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Committing updater private key to repository | Anyone with repo access can sign malicious updates that all installed copies accept | Generate key locally, store in password manager, add to CI as secret. Add `*.key` to `.gitignore`. |
| Using Apple ID password instead of app-specific password for notarization | If CI secret leaks, attacker has full Apple account access including 2FA management | Create app-specific password at appleid.apple.com. Better: use App Store Connect API keys with scoped permissions. |
| Setting `dangerousInsecureTransportProtocol: true` in production | Man-in-the-middle can serve malicious updates over HTTP | Tauri enforces HTTPS by default. Only set this flag in development builds. Never commit it to production config. |
| Storing Apple signing certificate without password protection in CI | Certificate extractable from CI logs or artifacts | Export as `.p12` with a strong password. Create temporary keychains in CI and delete after signing. |
| E2E test fixtures that shell out to git CLI with interpolated strings | Command injection if test parameters contain special characters | Use `git2` crate for fixture creation (sandboxed, no shell). Never construct git commands via string interpolation. |
| Not rotating Apple API keys | Long-lived keys accumulate risk if CI environment is compromised | App Store Connect API keys cannot be rotated (download once). Guard the key path carefully. Document when it was created. |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Auto-update that restarts without warning | User loses in-progress commit message, staged hunks, merge resolution state | Show update-available notification. Let user choose when to restart. Apply on next launch. |
| Update dialog that blocks the entire UI | User cannot continue working while deciding | Non-modal toast/banner notification. App remains fully functional during download. |
| Silent update failures with no feedback | User never knows an update was available or that it failed | Log update check results. Show "Update failed, will retry" toast. Provide manual download link as fallback. |
| Forcing update on Windows (OS limitation) | Windows updater automatically quits the app during install | Warn user before installing: "The app will close and restart to install the update." Save any unsaved state first. |
| No update frequency control | App checks for updates on every launch, adding latency | Check once on launch, then every 24 hours. Cache the last-check timestamp. |

## "Looks Done But Isn't" Checklist

- [ ] **E2E tests:** Test teardown kills app processes and cleans temp repos -- zombie processes accumulate on CI runners otherwise
- [ ] **E2E tests:** Tests use `path.join()` not hardcoded `/` separators -- Windows paths use `\`
- [ ] **E2E tests:** Can run locally on macOS, not just Linux CI -- developer experience matters for debugging failures
- [ ] **Code signing:** Signed `.dmg` opens without Gatekeeper warning on a CLEAN Mac (not your dev machine which has exceptions stored)
- [ ] **Code signing:** CI temporary keychain is deleted after signing -- prevents keychain accumulation across builds
- [ ] **Code signing:** Notarization actually completes -- check the notarization log, not just the build exit code
- [ ] **Auto-updates:** App works normally when update endpoint is unreachable (no crashes, no blocking spinner, no frozen UI)
- [ ] **Auto-updates:** Downgrade prevention -- updater does not "update" to an older version if `latest.json` briefly points to a previous release
- [ ] **Auto-updates:** `latest.json` contains entries for ALL four targets (darwin-aarch64, darwin-x86_64, linux-x86_64, windows-x86_64)
- [ ] **Auto-updates:** Update notification is non-modal -- user can dismiss and continue working
- [ ] **Benchmarks:** Baseline numbers recorded before any optimization -- cannot measure improvement without a reference point
- [ ] **Benchmarks:** Fixtures use realistic repo sizes -- 10k+ commits, not 10-commit toy repos
- [ ] **Benchmarks:** CI benchmark gate uses iai-callgrind (instruction counts), not Criterion (wall-clock time)
- [ ] **Release workflow:** `latest.json` present in published release with correct URLs and valid signatures for all platforms

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Lost updater private key | HIGH | If truly lost: publish announcement, users must manually reinstall. Cannot recover. If old key exists somewhere: release a migration version signed with old key that updates the embedded pubkey to a new keypair. |
| Notarization hanging in CI | LOW | Cancel workflow, re-trigger. If persistent: sign/notarize locally, upload artifacts manually. Add timeout + retry to workflow. |
| E2E tests are flaky | MEDIUM | Quarantine flaky tests (mark skip), audit for timing dependencies, replace hardcoded waits with polling, add retry for known-flaky WebDriver behaviors. |
| Benchmark regression false positive (Criterion in CI) | LOW | Re-run benchmark job. Switch to iai-callgrind. Establish noise threshold (ignore changes under 2%). |
| Apple signing certificate expired | MEDIUM | Generate new certificate from Apple Developer portal, update CI secrets, re-sign and re-notarize. Existing installed copies still work; only new releases affected. |
| `latest.json` missing a platform | LOW | Manually construct correct `latest.json` with all platform entries and upload to GitHub release. Fix workflow for future releases. |
| Bad update pushed to users | MEDIUM | Immediately publish a hotfix version. Updater picks up new version on next check (default: app launch). Users who already updated get the fix on next check. |
| E2E test repo fixtures breaking across git versions | LOW | Switch from git CLI fixtures to `git2` crate (already in project). git2 has a stable API regardless of system git version. |
| Windows auto-update kills app without user consent | LOW | Cannot prevent (Windows installer limitation). Mitigate: warn user, save state before install, restore state on restart. |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Auto-updates before code signing | Phase ordering: code signing BEFORE auto-updates | Auto-update tests run against signed builds that pass Gatekeeper |
| E2E tests cannot run on macOS | E2E testing (first plan: tooling evaluation) | Run first E2E test locally on macOS before writing the full suite |
| Lost updater private key | Auto-updates (first task: key generation + backup) | Key exists in 2+ locations; backup verified accessible |
| Criterion flaky in CI | Performance benchmarks (first task: tooling decision) | Same benchmark run 3x in CI -- results vary less than 2% |
| Notarization hanging | Code signing (CI integration plan) | Trigger test release; notarization completes within 15 min |
| E2E fixtures coupled to git state | E2E testing (fixture strategy plan) | Tests pass in any order; pass on fresh CI with no git config |
| Updater endpoint misconfiguration | Auto-updates (release workflow integration) | Publish test pre-release; verify `latest.json` on all 4 platforms |
| Unrealistic benchmark fixtures | Performance benchmarks (fixture creation) | Benchmarks include 10k+ commit repos; results show meaningful differentiation |
| Auto-update UX blocking user | Auto-updates (UI integration plan) | User can continue working while update downloads; restart is user-initiated |

## Sources

- [Tauri v2 Updater Plugin docs](https://v2.tauri.app/plugin/updater/) -- configuration fields, signing requirements, endpoint format, platform behaviors, `createUpdaterArtifacts` options
- [Tauri v2 macOS Code Signing docs](https://v2.tauri.app/distribute/sign/macos/) -- certificate types, CI setup steps, notarization methods, env vars
- [Tauri v2 Testing docs](https://v2.tauri.app/develop/tests/) -- testing approaches, mock runtime, WebDriver overview
- [Tauri v2 WebDriver docs](https://v2.tauri.app/develop/tests/webdriver/) -- platform support matrix, macOS gap, tauri-driver usage
- [tauri-webdriver (danielraffel)](https://github.com/danielraffel/tauri-webdriver) -- open-source macOS WebDriver solution for WKWebView
- [tauri-webdriver-automation crate](https://crates.io/crates/tauri-webdriver-automation) -- embedded WebDriver plugin for cross-platform E2E
- [Ship Your Tauri v2 App Like a Pro: Code Signing (Part 1)](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-code-signing-for-macos-and-windows-part-12-3o9n) -- practitioner guide, certificate export, CI secrets
- [Shipping a Production macOS App with Tauri 2.0 (DEV Community)](https://dev.to/0xmassi/shipping-a-production-macos-app-with-tauri-20-code-signing-notarization-and-homebrew-mc3) -- end-to-end production deployment with notarization
- [Criterion.rs FAQ](https://bheisler.github.io/criterion.rs/book/faq.html) -- CI noise warning, `black_box` usage, optimizer pitfalls
- [iai-callgrind GitHub](https://github.com/iai-callgrind/iai-callgrind) -- deterministic instruction-count benchmarking, CI suitability
- [Bencher: Track Criterion benchmarks in CI](https://bencher.dev/learn/track-in-ci/rust/criterion/) -- continuous benchmarking practices
- [Tauri E2E testing discussion #10123](https://github.com/tauri-apps/tauri/discussions/10123) -- community approaches
- [Tauri notarization stuck bug #14579](https://github.com/tauri-apps/tauri/issues/14579) -- build stuck at notarizing
- [Tauri notarization discussion #8630](https://github.com/orgs/tauri-apps/discussions/8630) -- stuck for 1+ hours
- [tauri-action GitHub](https://github.com/tauri-apps/tauri-action) -- `includeUpdaterJson` config, release creation
- [Apple: Resolving common notarization issues](https://developer.apple.com/documentation/security/resolving-common-notarization-issues) -- official Apple troubleshooting
- [GitHub Action hanging on macOS code signing](https://www.codejam.info/2025/06/github-action-hanging-macos-app-code-signing.html) -- CI keychain hang diagnosis
- [macOS createUpdaterArtifacts v1Compatible bug #10217](https://github.com/tauri-apps/tauri/issues/10217) -- v1Compatible does not work on macOS
- [Tauri WebDriver docs issue #10670](https://github.com/tauri-apps/tauri/issues/10670) -- WebDriver testing docs not working
- [notarytool reliability issues](https://developer.apple.com/forums/thread/772619) -- notarytool taking 2+ hours on CI

---
*Pitfalls research for: Trunk v1.0 -- E2E testing, performance benchmarks, code signing, auto-updates*
*Researched: 2026-03-26*
