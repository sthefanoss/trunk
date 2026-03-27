# Phase 58: E2E Test Harness - Research

**Researched:** 2026-03-27
**Domain:** WebdriverIO + tauri-driver E2E testing for Tauri 2 desktop application
**Confidence:** HIGH

## Summary

Tauri 2 supports E2E testing via the W3C WebDriver standard using `tauri-driver`, a cross-platform wrapper that delegates to the native platform's WebDriver server (WebKitWebDriver on Linux, msedgedriver on Windows). WebdriverIO v9 is the official recommended test framework, with Mocha as the test runner. The official Tauri v2 webdriver-example repository provides a complete reference implementation.

The main challenge for this project is that the app uses native file dialogs (`@tauri-apps/plugin-dialog`) to open repositories, which WebDriver cannot interact with. The solution is to use WebdriverIO's `browser.execute()` to call `window.__TAURI_INTERNALS__.invoke("open_repo", { path })` directly, bypassing the dialog. This is the standard pattern for E2E testing Tauri apps that use native dialogs.

**Primary recommendation:** Use the official Tauri v2 WebdriverIO pattern with `wdio.conf.js` managing tauri-driver lifecycle. Build the debug binary in `onPrepare`, spawn tauri-driver in `beforeSession`, run Mocha-based specs. Use `browser.execute()` with Tauri IPC to programmatically open fixture repos. Add `data-testid` attributes to key Svelte components incrementally.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use `data-testid` attributes on key interactive elements and verification targets. E2E tests select elements via `[data-testid="..."]` -- explicit, stable, decoupled from styling and DOM structure.
- **D-02:** Add `data-testid` attributes incrementally -- only on elements needed by E2E test scenarios, not exhaustively across the entire UI.
- **D-03:** macOS E2E documented as manual pre-release validation (E2E-05). The experimental Tauri WebDriver plugin for WKWebView is not reliable enough for CI. A manual validation checklist covers the same core workflows tested on Linux.
- **D-04:** Each E2E test creates its own fixture repository at runtime using git CLI commands (init, add, commit, branch, etc.). Tests get fresh isolated repos -- mirrors the Rust test pattern from Phase 53.
- **D-05:** A helper/utility module provides fixture builders for common repo shapes (linear history, branches, conflicts) to keep test code DRY.
- **D-06:** Separate `e2e.yml` GitHub Actions workflow -- E2E tests are long-running and should not block the fast CI gates in `ci.yml`.
- **D-07:** Linux CI uses Xvfb (virtual framebuffer) for headless display. The workflow installs webkit2gtk system deps, builds a debug Tauri binary, starts tauri-driver, and runs WebdriverIO tests.
- **D-08:** E2E workflow triggers on push to main and pull requests (same as ci.yml), but runs in parallel rather than gating on ci.yml completion.

### Claude's Discretion
- WebdriverIO configuration details (wdio.conf.ts structure, reporters, timeouts)
- tauri-driver launch and lifecycle management
- Exact data-testid naming conventions
- Which additional edge-case scenarios to cover beyond the required core workflows
- Test file organization within the E2E test directory
- Whether to use Page Object pattern or flat test helpers

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| E2E-01 | E2E test harness using WebdriverIO + tauri-driver runs on Linux CI | Official Tauri v2 WebdriverIO pattern verified; wdio.conf.js with tauri-driver lifecycle management; GitHub Actions workflow with Xvfb |
| E2E-02 | E2E tests cover core workflow: open repo, browse commit history, view diffs | `browser.execute()` + Tauri IPC for opening fixture repos; `data-testid` selectors on CommitGraph, CommitRow for verifying history |
| E2E-03 | E2E tests cover staging workflow: stage/unstage files, create commits | `data-testid` on StagingPanel file rows, CommitForm subject input and submit button; verify commit appears in history after creation |
| E2E-04 | E2E tests cover branch operations: checkout, create, delete | `data-testid` on BranchSidebar section, BranchRow elements, create branch input; verify sidebar updates reflect operations |
| E2E-05 | macOS E2E tests via experimental WebDriver plugin | Documented as manual validation checklist per D-03; macOS has no native WKWebView driver for CI |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Never inline colors -- always use CSS custom properties from the theme
- Never fight layout with positioning hacks -- use grid/flexbox
- All git operations go through git2 crate, no shelling out (except GIT_EDITOR)
- Frontend uses Svelte 5 runes (`$state`, `$derived`)
- Backend is Tauri 2, git2 0.19
- Frontend->Backend via `invoke("command_name", args)` calling `#[tauri::command]` fns
- Commands in `src-tauri/src/commands/`, frontend in `src/lib` and `src/components`

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| @wdio/cli | 9.27.0 | WebdriverIO test runner CLI | Official Tauri-recommended E2E framework |
| @wdio/local-runner | 9.27.0 | Local WebDriver execution | Runs tests against local tauri-driver |
| @wdio/mocha-framework | 9.27.0 | Mocha test framework integration | Official Tauri example uses Mocha BDD |
| @wdio/spec-reporter | 9.27.0 | Spec-style test output | Clean CI output, matches official example |
| tauri-driver | latest (cargo) | Cross-platform WebDriver proxy for Tauri | Required intermediary between WDIO and native WebDriver |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| WebKitWebDriver | system | Native Linux WebDriver for webkit2gtk | Automatically used by tauri-driver on Linux |
| xvfb | system | Virtual framebuffer for headless Linux CI | Required for GUI testing without display |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Mocha | Jest | Mocha is the official Tauri example; Jest would require extra config |
| @wdio/spec-reporter | @wdio/allure-reporter | Allure adds complexity; spec reporter is sufficient for CI |
| tauri-driver (cargo) | @crabnebula/tauri-driver (npm) | CrabNebula adds macOS support but is a third-party fork; D-03 defers macOS to manual validation |

**Installation:**
```bash
# E2E test dependencies (separate from main project)
cd e2e && bun install

# tauri-driver (Rust binary -- installed in CI via cargo)
cargo install tauri-driver --locked
```

**Version verification:** All WDIO packages verified at 9.27.0 via `npm view` on 2026-03-27.

## Architecture Patterns

### Recommended Project Structure
```
e2e/
  package.json          # Separate package for E2E deps
  wdio.conf.js          # WebdriverIO configuration
  helpers/
    fixture.js          # Fixture repo builder (git CLI)
    app.js              # App interaction helpers (open repo, wait for load)
  specs/
    history.e2e.js      # E2E-02: browse commit history
    staging.e2e.js      # E2E-03: staging workflow
    branches.e2e.js     # E2E-04: branch operations
```

### Pattern 1: Flat Test Helpers (recommended over Page Objects)

**What:** Shared helper functions for common actions, no class hierarchy.
**When to use:** Small-to-medium test suites with straightforward UI interactions.
**Why over Page Objects:** The app has a single-page layout with three panels (sidebar, graph, staging). Page Objects add indirection without clear benefit here. Flat helpers are simpler and more discoverable.

**Example:**
```javascript
// Source: Pattern derived from official Tauri v2 WebdriverIO example
// e2e/helpers/app.js

import { execSync } from 'child_process';
import { mkdtempSync, writeFileSync } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';

/**
 * Open a repo in the app by calling the Tauri IPC command directly.
 * This bypasses the native file dialog which WebDriver cannot interact with.
 */
export async function openRepo(repoPath) {
  await browser.execute(async (path) => {
    await window.__TAURI_INTERNALS__.invoke('open_repo', { path });
  }, repoPath);
}

/**
 * Create a fixture repo with linear history.
 */
export function createLinearRepo(commitCount = 3) {
  const dir = mkdtempSync(join(tmpdir(), 'trunk-e2e-'));
  execSync('git init', { cwd: dir });
  execSync('git config user.email "test@test.com"', { cwd: dir });
  execSync('git config user.name "Test"', { cwd: dir });

  for (let i = 1; i <= commitCount; i++) {
    writeFileSync(join(dir, `file-${i}.txt`), `content ${i}`);
    execSync('git add .', { cwd: dir });
    execSync(`git commit -m "commit ${i}"`, { cwd: dir });
  }
  return dir;
}
```

### Pattern 2: data-testid Selector Convention

**What:** Kebab-case `data-testid` attributes on interactive and verification elements.
**When to use:** All E2E-targeted elements.

**Naming convention:** `{component}-{element}` or `{component}-{element}-{qualifier}`

```
commit-row                     # Each commit row in the graph
commit-row-summary             # The summary text within a commit row
commit-form-subject            # Subject input field
commit-form-submit             # Commit button
staging-unstaged-section       # Unstaged files section header
staging-unstaged-file          # Individual unstaged file row
staging-staged-section         # Staged files section header
staging-staged-file            # Individual staged file row
branch-sidebar                 # Branch sidebar container
branch-local-section           # Local branches section
branch-row                     # Individual branch row
branch-create-input            # New branch name input
```

### Pattern 3: Fixture Repo Builder

**What:** Helper functions that create git repos with specific shapes using git CLI.
**When to use:** Every E2E test creates its own fixture for isolation.

```javascript
// e2e/helpers/fixture.js

export function createBranchRepo() {
  const dir = mkdtempSync(join(tmpdir(), 'trunk-e2e-'));
  execSync('git init', { cwd: dir });
  execSync('git config user.email "test@test.com"', { cwd: dir });
  execSync('git config user.name "Test"', { cwd: dir });

  writeFileSync(join(dir, 'README.md'), 'initial');
  execSync('git add .', { cwd: dir });
  execSync('git commit -m "initial commit"', { cwd: dir });

  execSync('git branch feature-a', { cwd: dir });
  execSync('git branch feature-b', { cwd: dir });

  return dir;
}
```

### Pattern 4: Opening Repos via Tauri IPC (bypassing native dialog)

**What:** Use `browser.execute()` to invoke Tauri commands directly from WebDriver context.
**When to use:** Any time you need to call a Tauri command that normally goes through native UI (file dialogs, context menus).
**Why:** WebDriver cannot interact with native OS dialogs. The Tauri IPC bridge (`window.__TAURI_INTERNALS__`) is available in the webview and allows direct command invocation.

**Critical detail:** After calling `open_repo` via IPC, the frontend state also needs to be updated. The app normally handles this through `openRepoInTab()` in App.svelte. From E2E, after the IPC call, the app needs to be notified. The simplest approach: inject JavaScript that dispatches a custom event or directly manipulates the store, OR wait for the app's built-in `repo-changed` event listener to refresh the UI.

**Recommended approach:** Call `open_repo` IPC, then wait for the UI to reflect the opened repo (e.g., wait for `[data-testid="commit-row"]` to appear), rather than trying to also trigger frontend state changes from outside.

### Anti-Patterns to Avoid
- **Selecting by CSS class or tag name:** Fragile, breaks on styling changes. Always use `data-testid`.
- **Using `browser.pause(N)`:** Hard-coded sleeps. Use `waitForExist()` or `waitUntil()` instead.
- **Building debug binary inside each test:** Build once in `onPrepare`, reuse across all specs.
- **Sharing state between tests:** Each test creates its own fixture repo. Never rely on state from a previous test.
- **Using `@crabnebula/tauri-driver` npm package:** We use the official `cargo install tauri-driver` per D-03 (macOS is manual validation).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WebDriver protocol | Custom HTTP client | WebdriverIO + tauri-driver | W3C standard implementation with Tauri-specific extensions |
| Test fixture repos | In-memory git mocks | Real git CLI in tmpdir | E2E tests must exercise the full stack including git2 |
| Wait/retry logic | Custom polling loops | WDIO `waitForExist()`, `waitUntil()` | Built-in with configurable timeouts, error messages |
| Process lifecycle | Manual spawn/kill | wdio.conf.js hooks | `onPrepare`/`beforeSession`/`afterSession` handle lifecycle cleanly |
| CI virtual display | Custom Xvfb scripts | `xvfb-run` command | Standard approach, one-liner in CI |

**Key insight:** The official Tauri WebdriverIO example is minimal and battle-tested. Follow it closely rather than inventing custom patterns.

## Common Pitfalls

### Pitfall 1: Native Dialog Interaction
**What goes wrong:** Tests try to click the "Open Repository" button which opens a native file dialog that WebDriver cannot interact with.
**Why it happens:** WebDriver only controls the webview, not native OS dialogs.
**How to avoid:** Use `browser.execute()` to call `window.__TAURI_INTERNALS__.invoke('open_repo', { path })` directly.
**Warning signs:** Test hangs waiting for a dialog to appear or close.

### Pitfall 2: Binary Not Built Before Tests
**What goes wrong:** Tests fail immediately because the Tauri binary doesn't exist at the expected path.
**Why it happens:** `wdio.conf.js` `onPrepare` hook doesn't build, or build fails silently.
**How to avoid:** Build in `onPrepare` with `stdio: 'inherit'` so build errors are visible. Also verify binary exists before proceeding.
**Warning signs:** "ENOENT" or "application not found" errors from tauri-driver.

### Pitfall 3: tauri-driver Not Installed
**What goes wrong:** `beforeSession` fails because `tauri-driver` binary is not in PATH.
**Why it happens:** CI doesn't include `cargo install tauri-driver --locked` step.
**How to avoid:** Explicit installation step in CI workflow before running tests.
**Warning signs:** "spawn ENOENT" error for tauri-driver path.

### Pitfall 4: Missing Xvfb on Linux CI
**What goes wrong:** App fails to create a window because there's no display server.
**Why it happens:** Linux CI runners are headless -- no X11 display.
**How to avoid:** Use `xvfb-run` to wrap the test command: `xvfb-run bun run wdio`.
**Warning signs:** "Failed to open display" or similar X11 errors.

### Pitfall 5: Race Conditions After Repo Open
**What goes wrong:** Test tries to interact with commit graph before it's loaded.
**Why it happens:** `open_repo` IPC returns before the frontend finishes rendering the graph.
**How to avoid:** After opening a repo, use `waitForExist('[data-testid="commit-row"]')` before proceeding.
**Warning signs:** Element not found errors that pass on retry.

### Pitfall 6: Stale State Between Tests
**What goes wrong:** Second test sees leftover repo from first test in the tab.
**Why it happens:** App persists open tabs in `trunk-prefs.json` (LazyStore).
**How to avoid:** Each test should either (1) close the repo before ending, or (2) the app should be restarted fresh for each spec file.
**Warning signs:** Wrong repo data appears in tests, tests pass individually but fail together.

### Pitfall 7: Context Menus in E2E
**What goes wrong:** Tests try to use context menus (right-click on branches) but these are native Tauri menus, not DOM elements.
**Why it happens:** Branch delete, checkout via double-click are context menu actions.
**How to avoid:** For branch operations, prefer using the UI interactions that don't require context menus (e.g., double-click for checkout, the create branch "+" button). For delete, may need to use IPC directly or find a DOM-based alternative.
**Warning signs:** Context menu never appears in WebDriver session.

### Pitfall 8: WebKitWebDriver Package Name on Ubuntu
**What goes wrong:** CI fails because `WebKitWebDriver` binary is not found.
**Why it happens:** Wrong package name used in apt-get install.
**How to avoid:** Install `webkit2gtk-driver` package (not just `libwebkit2gtk-4.1-dev`).
**Warning signs:** `which WebKitWebDriver` returns empty.

## Code Examples

### Complete wdio.conf.js for Trunk

```javascript
// Source: Adapted from https://github.com/tauri-apps/webdriver-example/blob/main/v2/webdriver/webdriverio/wdio.conf.js
import os from 'os';
import path from 'path';
import { spawn, spawnSync } from 'child_process';
import { fileURLToPath } from 'url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

let tauriDriver;
let exit = false;

export const config = {
  host: '127.0.0.1',
  port: 4444,
  specs: ['./specs/**/*.e2e.js'],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        application: path.resolve(__dirname, '../src-tauri/target/debug/trunk'),
      },
    },
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  // Build the debug binary before any test runs
  onPrepare: () => {
    spawnSync(
      'bun',
      ['run', 'tauri', 'build', '--', '--debug', '--no-bundle'],
      {
        cwd: path.resolve(__dirname, '..'),
        stdio: 'inherit',
        shell: true,
      }
    );
  },

  // Start tauri-driver before each session
  beforeSession: () => {
    tauriDriver = spawn(
      path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
      [],
      { stdio: [null, process.stdout, process.stderr] }
    );

    tauriDriver.on('error', (error) => {
      console.error('tauri-driver error:', error);
      process.exit(1);
    });
    tauriDriver.on('exit', (code) => {
      if (!exit) {
        console.error('tauri-driver exited with code:', code);
        process.exit(1);
      }
    });
  },

  afterSession: () => {
    closeTauriDriver();
  },
};

function closeTauriDriver() {
  exit = true;
  tauriDriver?.kill();
}

function onShutdown(fn) {
  const cleanup = () => {
    try { fn(); } finally { process.exit(); }
  };
  process.on('exit', cleanup);
  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('SIGHUP', cleanup);
  process.on('SIGBREAK', cleanup);
}

onShutdown(() => { closeTauriDriver(); });
```

### Opening a Fixture Repo from Test

```javascript
// Source: Pattern derived from Tauri IPC documentation + WebdriverIO execute API
import { createLinearRepo } from '../helpers/fixture.js';

describe('Commit History', () => {
  let repoDir;

  before(async () => {
    // Create fixture with 5 commits
    repoDir = createLinearRepo(5);

    // Open repo via Tauri IPC (bypasses native file dialog)
    await browser.execute(async (path) => {
      await window.__TAURI_INTERNALS__.invoke('open_repo', { path });
    }, repoDir);

    // Wait for the UI to render the commit graph
    const commitRow = await $('[data-testid="commit-row"]');
    await commitRow.waitForExist({ timeout: 10000 });
  });

  it('should display correct number of commits', async () => {
    const rows = await $$('[data-testid="commit-row"]');
    // 5 commits + 1 WIP row at top
    expect(rows.length).toBeGreaterThanOrEqual(5);
  });
});
```

### data-testid Attribute in Svelte Component

```svelte
<!-- Source: Pattern for adding testid to CommitRow.svelte -->
<div
  data-testid="commit-row"
  class="relative flex items-center cursor-pointer text-[13px]"
  onclick={() => onselect?.(commit.oid)}
>
  <!-- Column 3: Message -->
  <div data-testid="commit-row-summary" class="flex-1 ...">
    {commit.summary}
  </div>
</div>
```

### GitHub Actions E2E Workflow

```yaml
# Source: Adapted from https://github.com/tauri-apps/webdriver-example/.github/workflows/webdriver-v2.yml
name: E2E

on:
  push:
    branches: [main]
  pull_request:

concurrency:
  group: e2e-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read

jobs:
  e2e:
    name: E2E Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            build-essential \
            curl \
            wget \
            file \
            libxdo-dev \
            libssl-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev \
            webkit2gtk-driver \
            xvfb

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: "src-tauri -> target"
          save-if: ${{ github.ref == 'refs/heads/main' }}

      - uses: oven-sh/setup-bun@v2

      - name: Install frontend dependencies
        run: bun install --frozen-lockfile

      - name: Install E2E dependencies
        run: cd e2e && bun install --frozen-lockfile

      - name: Install tauri-driver
        run: cargo install tauri-driver --locked

      - name: Run E2E tests
        run: xvfb-run bun --cwd e2e run test
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| tauri-driver v1 (Tauri 1) | tauri-driver v2 (Tauri 2) | 2024 | Config uses `tauri:options` capability key |
| WebdriverIO v7-v8 | WebdriverIO v9 | 2024-2025 | ESM-first config, updated API |
| @wdio/cli as dependency | @wdio/cli as devDependency | WDIO v9 | Matches official Tauri example |
| CJS wdio.conf.js | ESM wdio.conf.js | WDIO v9 | `type: "module"` in package.json |

**Deprecated/outdated:**
- `tauri:options.binary` property -- use `tauri:options.application` instead
- WebdriverIO v7 TypeScript typings workarounds -- resolved in v9
- `--bundle` flag for debug builds -- use `--no-bundle` for faster debug builds

## Open Questions

1. **App State Isolation Between Test Files**
   - What we know: The app persists open tabs via `trunk-prefs.json` (LazyStore). If one test file opens a repo, the next test file may see stale state.
   - What's unclear: Whether tauri-driver restarts the app between spec files, or if it persists.
   - Recommendation: WDIO spawns a fresh session per spec file by default (`maxInstances: 1`). Each `beforeSession` kills and restarts tauri-driver. Verify this behavior works correctly. If state leaks, add cleanup in `beforeSession` to delete the store file before starting the app.

2. **Tauri IPC Access from browser.execute()**
   - What we know: `window.__TAURI_INTERNALS__` is the internal IPC bridge in Tauri 2 webviews.
   - What's unclear: Whether `browser.execute()` can call async functions with `await` on `__TAURI_INTERNALS__.invoke()`.
   - Recommendation: Use `browser.executeAsync()` if `browser.execute()` doesn't handle the Promise. Test this pattern early in Wave 0.

3. **WIP Row in Commit Graph**
   - What we know: The commit graph includes a synthetic `__wip__` row at the top when there are dirty files.
   - What's unclear: Whether the WIP row appears in a clean repo fixture (it should not).
   - Recommendation: Account for WIP row in commit count assertions. If fixture is clean, WIP should not appear.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| bun | E2E test execution | (check in CI) | -- | npm/pnpm |
| cargo | tauri-driver install | (check in CI) | -- | -- |
| tauri-driver | WebDriver proxy | Not installed locally | -- | `cargo install tauri-driver --locked` |
| git | Fixture repo creation | (check in CI) | -- | -- |
| WebKitWebDriver | Linux native WebDriver | (CI only, via webkit2gtk-driver) | -- | -- |
| xvfb | Headless display | (CI only, via xvfb package) | -- | -- |

**Missing dependencies with no fallback:**
- tauri-driver: Must be installed via cargo in CI. Not needed locally if running tests only in CI.
- WebKitWebDriver: Linux-only. Must be installed via `webkit2gtk-driver` apt package.
- xvfb: Linux CI only. Required for headless GUI testing.

**Missing dependencies with fallback:**
- None. All dependencies are standard CI tools with well-known installation paths.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | WebdriverIO v9.27.0 + Mocha |
| Config file | `e2e/wdio.conf.js` (Wave 0 -- does not exist yet) |
| Quick run command | `cd e2e && bun run test` |
| Full suite command | `xvfb-run bun --cwd e2e run test` (Linux CI) |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| E2E-01 | WebdriverIO + tauri-driver runs on Linux CI | infrastructure | `xvfb-run bun --cwd e2e run test` (CI workflow validates) | Wave 0 |
| E2E-02 | Open repo, browse commit history, verify graph | e2e | `bun --cwd e2e run wdio -- --spec specs/history.e2e.js` | Wave 0 |
| E2E-03 | Stage file, enter message, create commit, verify in history | e2e | `bun --cwd e2e run wdio -- --spec specs/staging.e2e.js` | Wave 0 |
| E2E-04 | Checkout branch, create branch, delete branch, verify sidebar | e2e | `bun --cwd e2e run wdio -- --spec specs/branches.e2e.js` | Wave 0 |
| E2E-05 | macOS manual validation checklist | manual-only | N/A -- checklist document | Wave 0 |

### Sampling Rate
- **Per task commit:** N/A (E2E tests are too slow for per-commit runs)
- **Per wave merge:** Full suite via `xvfb-run bun --cwd e2e run test` (Linux only)
- **Phase gate:** All E2E specs pass on Linux CI before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `e2e/package.json` -- E2E-specific dependencies (WDIO v9)
- [ ] `e2e/wdio.conf.js` -- WebdriverIO configuration
- [ ] `e2e/helpers/fixture.js` -- Fixture repo builders
- [ ] `e2e/helpers/app.js` -- App interaction helpers
- [ ] `e2e/specs/history.e2e.js` -- E2E-02 commit history test
- [ ] `e2e/specs/staging.e2e.js` -- E2E-03 staging workflow test
- [ ] `e2e/specs/branches.e2e.js` -- E2E-04 branch operations test
- [ ] `.github/workflows/e2e.yml` -- CI workflow
- [ ] `data-testid` attributes added to Svelte components

## Key Component -> data-testid Map

Components that need `data-testid` attributes for E2E scenarios:

| Component | Element | Proposed testid | Required By |
|-----------|---------|-----------------|-------------|
| CommitRow.svelte | Root div | `commit-row` | E2E-02 |
| CommitRow.svelte | Summary span | `commit-row-summary` | E2E-02 |
| CommitForm.svelte | Subject input | `commit-form-subject` | E2E-03 |
| CommitForm.svelte | Submit button | `commit-form-submit` | E2E-03 |
| StagingPanel.svelte | Unstaged section | `staging-unstaged-section` | E2E-03 |
| StagingPanel.svelte | Staged section | `staging-staged-section` | E2E-03 |
| FileRow.svelte | Root element | `staging-file` | E2E-03 |
| BranchSidebar.svelte | Container | `branch-sidebar` | E2E-04 |
| BranchSection.svelte | Local section | `branch-section-local` | E2E-04 |
| BranchRow.svelte | Root div | `branch-row` | E2E-04 |
| BranchSidebar.svelte | Create input | `branch-create-input` | E2E-04 |

## Sources

### Primary (HIGH confidence)
- [Tauri v2 WebdriverIO Example](https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/) -- Official configuration and test patterns
- [Tauri v2 WebDriver Overview](https://v2.tauri.app/develop/tests/webdriver/) -- Platform support, tauri-driver installation
- [Tauri v2 WebDriver CI](https://v2.tauri.app/develop/tests/webdriver/ci/) -- GitHub Actions workflow patterns
- [tauri-apps/webdriver-example v2](https://github.com/tauri-apps/webdriver-example) -- Official reference implementation (fetched wdio.conf.js and CI workflow via GitHub API)
- [WebdriverIO Best Practices](https://webdriver.io/docs/bestpractices/) -- data-testid selectors, wait patterns

### Secondary (MEDIUM confidence)
- [WebdriverIO execute API](https://webdriver.io/docs/api/browser/execute/) -- `browser.execute()` for injecting JS into webview context
- npm registry -- WDIO v9.27.0 version verified via `npm view`

### Tertiary (LOW confidence)
- `window.__TAURI_INTERNALS__.invoke()` pattern for bypassing native dialogs -- verified that the object exists in Tauri 2 webviews, but exact async behavior in `browser.execute()` context needs validation in Wave 0

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- Official Tauri documentation and example repository provide exact packages and versions
- Architecture: HIGH -- wdio.conf.js pattern is directly from the official example; data-testid is industry standard
- Pitfalls: HIGH -- Native dialog, Xvfb, and WebKitWebDriver issues are well-documented in Tauri E2E testing community
- IPC bypass pattern: MEDIUM -- `__TAURI_INTERNALS__` is the documented internal API, but async execution from `browser.execute()` needs validation

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable ecosystem, 30 days)
