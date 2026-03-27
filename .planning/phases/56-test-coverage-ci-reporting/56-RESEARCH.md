# Phase 56: Test Coverage & CI Reporting - Research

**Researched:** 2026-03-27
**Domain:** CI/CD coverage tooling (cargo-llvm-cov, vitest coverage-v8, GitHub Actions)
**Confidence:** HIGH

## Summary

This phase adds test coverage measurement and reporting to the existing CI pipeline. Both codebases (Rust and TypeScript) already have comprehensive test suites -- 41 TypeScript test files with 364 tests, and 15+ Rust integration test files. The tooling choices are locked: cargo-llvm-cov for Rust and @vitest/coverage-v8 for TypeScript, both producing lcov output for unified reporting.

The implementation is straightforward CI-only work. The existing `cargo-test` and `vitest` jobs in `ci.yml` get modified to produce coverage reports, and new steps upload HTML artifacts and post PR comments. No application code changes are needed -- only `ci.yml` and `vite.config.ts` are modified, plus one dev dependency added.

**Primary recommendation:** Extend the existing Gate 2 jobs in ci.yml rather than creating new jobs. Use `zgosalvez/github-actions-report-lcov@v4` for PR comments since it natively handles lcov files, generates HTML artifacts, and posts sticky PR comments in a single action -- avoiding the complexity of piping output between separate actions.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use cargo-llvm-cov (not cargo-tarpaulin) -- source-based instrumentation is more accurate than tarpaulin's ptrace approach, faster execution, better maintained, and produces standard lcov output for tooling integration.
- **D-02:** Use vitest's built-in coverage with @vitest/coverage-v8 provider -- already using vitest for tests, native integration via `vitest run --coverage`, produces lcov/text/html reports.
- **D-03:** Both HTML artifacts and PR comment summary -- HTML uploaded as CI artifacts for detailed line-by-line browsing, lightweight text/table summary posted as PR comment for quick visibility without downloading artifacts.
- **D-04:** Report-only, no enforcement -- this is the first time coverage is being measured. Establish a baseline first. Thresholds can be added in a future phase once the team knows what realistic targets look like.

### Claude's Discretion
- Choice of GitHub Action for PR comment posting (e.g., marocchino/sticky-pull-request-comment or similar)
- lcov merge strategy if combining Rust + TS into a single report
- Whether to show per-file coverage or summary-only in PR comments

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UNIT-04 | Test coverage metrics are measured and reported in CI | cargo-llvm-cov generates Rust coverage lcov; @vitest/coverage-v8 generates TS coverage lcov; both uploaded as HTML artifacts and summarized in PR comments |

</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **Stack:** Tauri 2 + Svelte 5 + Rust -- frontend tests via vitest, backend tests via cargo test (integration tests in `src-tauri/tests/`)
- **Test command:** `bun run test` runs `vitest run`
- **CI runs on:** ubuntu-latest with system deps for webkit/tauri compilation
- **Existing CI pattern:** Gate 1 (fast checks) gates Gate 2 (heavy checks); coverage fits in Gate 2
- **Package manager:** bun with `bun install --frozen-lockfile`
- No application code changes needed for this phase

## Standard Stack

### Core
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| cargo-llvm-cov | 0.8.4 | Rust source-based coverage | LLVM instrumentation, faster/more accurate than tarpaulin, lcov output |
| @vitest/coverage-v8 | 4.1.2 | TypeScript/Svelte coverage | Native vitest integration, V8's built-in coverage, zero-config with vitest |
| taiki-e/install-action | @cargo-llvm-cov | Install cargo-llvm-cov in CI | Official installation method, installs prebuilt binary (fast) |

### Supporting
| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| zgosalvez/github-actions-report-lcov | @v4 | PR comment + HTML artifact from lcov | Post coverage summary on PRs, upload HTML report as artifact |
| actions/upload-artifact | @v4 | Upload HTML coverage reports | Store detailed coverage HTML for download from CI |
| dtolnay/rust-toolchain | @stable | Rust toolchain with llvm-tools | cargo-llvm-cov requires llvm-tools-preview component |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| zgosalvez/github-actions-report-lcov | marocchino/sticky-pull-request-comment | sticky-comment is generic (needs separate lcov parsing step); zgosalvez does lcov parsing + HTML + PR comment in one action |
| zgosalvez/github-actions-report-lcov | davelosert/vitest-coverage-report-action | vitest-specific only -- cannot handle Rust lcov files; would need two different comment actions |
| Separate coverage jobs | Extending existing jobs | Separate jobs add overhead (checkout, deps install, cache restore); extending existing jobs reuses build artifacts |

**Recommendation for Claude's Discretion (PR comment action):** Use `zgosalvez/github-actions-report-lcov@v4` for both Rust and TypeScript coverage comments. Call it twice with different `title-prefix` values ("Rust" and "TypeScript") so each language gets its own sticky comment. This avoids merging lcov files across languages (which can produce confusing paths) and keeps reports clearly separated.

**Recommendation for Claude's Discretion (lcov merge):** Do NOT merge Rust and TypeScript lcov files. They cover completely different codebases with different path roots (`src-tauri/src/` vs `src/`). Separate reports are more useful than a combined one with mixed paths.

**Recommendation for Claude's Discretion (per-file vs summary):** Use per-file coverage in PR comments. The zgosalvez action includes both a summary line and a file-level breakdown by default. Since this is a small-to-medium codebase, per-file detail is manageable and much more actionable than summary-only.

**Installation:**
```bash
bun add -D @vitest/coverage-v8
```

No installation needed for cargo-llvm-cov in the project itself -- it is installed via `taiki-e/install-action` in CI only.

## Architecture Patterns

### CI Workflow Structure (extended ci.yml)
```
Gate 1: biome, cargo-fmt, svelte-check (unchanged)
    |
Gate 2: cargo-clippy, cargo-test*, vitest* (coverage added to starred jobs)
    |
    +-- cargo-test: now runs cargo-llvm-cov, uploads lcov + HTML artifact, posts PR comment
    +-- vitest: now runs vitest --coverage, uploads lcov + HTML artifact, posts PR comment
```

### Pattern 1: Extend Existing cargo-test Job
**What:** Add cargo-llvm-cov installation and invocation to the existing `cargo-test` job instead of creating a new job.
**When to use:** Always -- avoids duplicate system deps install and cache restore.
**Example:**
```yaml
# In cargo-test job, replace `cargo test` with:
- name: Install cargo-llvm-cov
  uses: taiki-e/install-action@cargo-llvm-cov
- name: Run tests with coverage
  run: cargo llvm-cov --manifest-path src-tauri/Cargo.toml --lcov --output-path rust-lcov.info
- name: Generate HTML report
  run: cargo llvm-cov report --manifest-path src-tauri/Cargo.toml --html --output-dir rust-coverage-html
```

### Pattern 2: Extend Existing vitest Job
**What:** Add coverage configuration and run vitest with coverage enabled.
**When to use:** Always -- the vitest job already has bun and node_modules ready.
**Example:**
```yaml
# In vitest job, replace `bun run test` with:
- run: bun run test -- --coverage.enabled --coverage.provider=v8 --coverage.reporter=lcov --coverage.reporter=text --coverage.reporter=html
```

### Pattern 3: Separate PR Comments per Language
**What:** Post two distinct sticky PR comments -- one for Rust, one for TypeScript.
**When to use:** When coverage tools produce separate lcov files.
**Why:** Avoids confusing merged reports; each comment is independently updateable.

### Anti-Patterns to Avoid
- **Creating new CI jobs for coverage:** Wastes time reinstalling system deps, Rust toolchain, and restoring cache. Extend existing jobs instead.
- **Merging Rust + TS lcov files:** Different path roots make merged reports confusing. Keep separate.
- **Enforcing thresholds on first run:** D-04 explicitly says report-only. Do not add `minimum-coverage` thresholds.
- **Using `cargo tarpaulin`:** D-01 explicitly chose cargo-llvm-cov. Tarpaulin uses ptrace which is slower and less accurate.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| lcov parsing for PR comments | Custom shell script to parse lcov.info | zgosalvez/github-actions-report-lcov@v4 | Handles parsing, HTML generation, sticky PR comments, artifact upload |
| Installing cargo-llvm-cov | `cargo install cargo-llvm-cov` | taiki-e/install-action@cargo-llvm-cov | Downloads prebuilt binary (seconds vs minutes for source compile) |
| Coverage report formatting | Bash awk/sed to format coverage text | Action's built-in formatters | Edge cases in lcov parsing, HTML report generation is non-trivial |

**Key insight:** Coverage reporting has enough moving parts (lcov parsing, HTML generation, PR comment management, artifact upload) that using purpose-built actions saves significant effort and avoids subtle bugs in report formatting.

## Common Pitfalls

### Pitfall 1: cargo-llvm-cov Requires llvm-tools-preview Component
**What goes wrong:** `cargo llvm-cov` fails with "llvm-tools-preview is not installed" error.
**Why it happens:** The Rust toolchain needs the `llvm-tools-preview` component installed. `taiki-e/install-action` does NOT install this -- it only installs the cargo-llvm-cov binary.
**How to avoid:** Add `llvm-tools-preview` to the `dtolnay/rust-toolchain` components, or cargo-llvm-cov will auto-install it on first run (but this adds time). Better to be explicit:
```yaml
- uses: dtolnay/rust-toolchain@stable
  with:
    components: llvm-tools-preview
```
**Warning signs:** CI job fails with "Failed to find llvm-tools" in the cargo-llvm-cov step.

### Pitfall 2: cargo-llvm-cov Cleans Build Artifacts
**What goes wrong:** Running cargo-llvm-cov invalidates the Rust cache, causing a full rebuild.
**Why it happens:** cargo-llvm-cov instruments the code differently than regular `cargo test`, requiring recompilation with `-C instrument-coverage`.
**How to avoid:** This is expected behavior. The instrumented build is cached by Swatinem/rust-cache. Since cargo-test already runs in a separate job from clippy, the cache key will naturally separate instrumented vs non-instrumented builds. Accept the rebuild cost -- it only happens once per cache miss.
**Warning signs:** Unexpectedly long CI times on the first run with coverage.

### Pitfall 3: vitest Coverage Include/Exclude Mismatch
**What goes wrong:** Coverage report includes test files themselves, or excludes source files that should be measured.
**Why it happens:** Default vitest coverage includes only files imported by tests. Files not imported by any test get 0% but may not appear in reports at all.
**How to avoid:** Configure `coverage.include` to match source files: `['src/**/*.ts', 'src/**/*.svelte']` and `coverage.exclude` to skip test files and setup: `['src/**/*.test.ts', 'vitest-setup.ts']`.
**Warning signs:** Coverage percentage looks suspiciously high because untested files are invisible.

### Pitfall 4: PR Comment Permissions
**What goes wrong:** PR comment step fails with 403 Forbidden.
**Why it happens:** GitHub Actions workflows triggered by `pull_request` from forks have read-only permissions by default.
**How to avoid:** Add explicit `permissions: pull-requests: write` to the jobs that post comments. For fork PRs, this is a known limitation -- the comment step should use `if: github.event.pull_request.head.repo.full_name == github.repository` to skip on fork PRs gracefully.
**Warning signs:** CI fails on external contributor PRs.

### Pitfall 5: Artifact Name Conflicts
**What goes wrong:** Upload-artifact step fails because two jobs upload with the same artifact name.
**Why it happens:** Both Rust and TypeScript jobs try to upload artifacts.
**How to avoid:** Use distinct artifact names: `rust-coverage-report` and `typescript-coverage-report`.
**Warning signs:** "Artifact name already exists" error in CI.

## Code Examples

### vite.config.ts Coverage Configuration
```typescript
// Source: https://vitest.dev/config/coverage
// Add to existing test section in vite.config.ts
test: {
  include: ["src/**/*.test.ts"],
  environment: "jsdom",
  setupFiles: ["./vitest-setup.ts"],
  coverage: {
    provider: "v8",
    reporter: ["text", "lcov", "html"],
    reportsDirectory: "./coverage",
    include: ["src/**/*.ts", "src/**/*.svelte"],
    exclude: ["src/**/*.test.ts"],
  },
},
```

### cargo-llvm-cov CI Steps
```yaml
# Source: https://github.com/taiki-e/cargo-llvm-cov README
- uses: dtolnay/rust-toolchain@stable
  with:
    components: llvm-tools-preview
- uses: taiki-e/install-action@cargo-llvm-cov
- name: Run tests with coverage
  run: cargo llvm-cov --manifest-path src-tauri/Cargo.toml --lcov --output-path rust-lcov.info
- name: Generate HTML coverage report
  run: cargo llvm-cov report --manifest-path src-tauri/Cargo.toml --html --output-dir rust-coverage-html
```

### PR Comment via zgosalvez Action
```yaml
# Source: https://github.com/zgosalvez/github-actions-report-lcov
- name: Report Rust coverage
  if: github.event_name == 'pull_request'
  uses: zgosalvez/github-actions-report-lcov@v4
  with:
    coverage-files: rust-lcov.info
    artifact-name: rust-coverage-report
    github-token: ${{ secrets.GITHUB_TOKEN }}
    title-prefix: "Rust"
    update-comment: true
```

### vitest CI Steps with Coverage
```yaml
- uses: oven-sh/setup-bun@v2
- run: bun install --frozen-lockfile
- name: Run tests with coverage
  run: bun run test -- --coverage.enabled --coverage.provider=v8
- name: Upload TypeScript coverage HTML
  uses: actions/upload-artifact@v4
  with:
    name: typescript-coverage-report
    path: coverage/
- name: Report TypeScript coverage
  if: github.event_name == 'pull_request'
  uses: zgosalvez/github-actions-report-lcov@v4
  with:
    coverage-files: coverage/lcov.info
    artifact-name: ""
    github-token: ${{ secrets.GITHUB_TOKEN }}
    title-prefix: "TypeScript"
    update-comment: true
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| cargo-tarpaulin (ptrace) | cargo-llvm-cov (LLVM instrumentation) | 2022+ | More accurate, faster, better maintained |
| @vitest/coverage-istanbul | @vitest/coverage-v8 (default in vitest 4.x) | vitest 2.x+ | V8's native coverage is faster, zero-config default |
| codecov/coveralls third-party services | Self-hosted via GitHub Actions artifacts + PR comments | 2024+ | No external service dependency, no token management |
| actions/upload-artifact@v3 | actions/upload-artifact@v4 | 2024 | Immutable artifacts, better deduplication |

**Deprecated/outdated:**
- `cargo-tarpaulin`: Still maintained but inferior to llvm-cov for accuracy and speed
- `@vitest/coverage-istanbul`: Still works but V8 is the default and recommended provider
- `actions/upload-artifact@v3`: Deprecated in favor of v4

## Open Questions

1. **cargo-llvm-cov report vs single command**
   - What we know: `cargo llvm-cov --lcov --output-path` runs tests AND generates lcov in one command. A separate `cargo llvm-cov report --html` can generate HTML from the same profiling data without re-running tests.
   - What's unclear: Whether the two-command approach (lcov first, then HTML report) works correctly with `--manifest-path` when not in the workspace root.
   - Recommendation: Try the two-command approach first. If it fails, fall back to running `cargo llvm-cov` twice (once for lcov, once for HTML) -- the instrumented binary is cached so the second run is fast.

2. **zgosalvez/github-actions-report-lcov version stability**
   - What we know: Latest is v7.0.7 but examples reference v4. The action has been actively maintained.
   - What's unclear: Whether v4 pinning is stable enough or if we should use a newer version.
   - Recommendation: Start with v4 as shown in official README examples. The major version pin provides stability. Can upgrade later if needed.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest 4.1.x (TypeScript) + cargo test (Rust) |
| Config file | `vite.config.ts` (vitest), `src-tauri/Cargo.toml` (Rust) |
| Quick run command | `bun run test` |
| Full suite command | `bun run test && cd src-tauri && cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UNIT-04 | Coverage metrics measured and reported in CI | CI workflow validation | Push to PR branch, verify CI run produces coverage artifacts and PR comments | N/A -- validated by CI run, not by unit tests |

### Sampling Rate
- **Per task commit:** `bun run test` (verify vitest config changes don't break tests)
- **Per wave merge:** Full CI pipeline run on a PR (the actual validation target)
- **Phase gate:** Open a test PR, verify both coverage reports appear as artifacts and PR comments

### Wave 0 Gaps
None -- this phase modifies CI config and vitest config only. No new test files needed. The validation is the CI pipeline itself producing coverage output.

## Sources

### Primary (HIGH confidence)
- [cargo-llvm-cov GitHub repository](https://github.com/taiki-e/cargo-llvm-cov) - Installation, CLI flags, CI examples
- [Vitest coverage configuration](https://vitest.dev/config/coverage) - V8 provider options, reporters, include/exclude
- [zgosalvez/github-actions-report-lcov](https://github.com/zgosalvez/github-actions-report-lcov) - PR comment action, inputs, usage
- [marocchino/sticky-pull-request-comment](https://github.com/marocchino/sticky-pull-request-comment) - Alternative PR comment action (evaluated, not recommended)
- [davelosert/vitest-coverage-report-action](https://github.com/davelosert/vitest-coverage-report-action) - Vitest-specific action (evaluated, not recommended due to Rust coverage limitation)

### Secondary (MEDIUM confidence)
- [Vitest coverage guide](https://vitest.dev/guide/coverage.html) - General coverage setup guidance
- [taiki-e/install-action](https://github.com/taiki-e/install-action) - Prebuilt binary installation for CI
- npm registry: @vitest/coverage-v8@4.1.2, vitest@4.1.2 -- verified current versions

### Tertiary (LOW confidence)
- cargo-llvm-cov version 0.8.4 from crates.io search results -- needs verification at execution time via `cargo install --list` or crates.io

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all tools verified via official repos and docs
- Architecture: HIGH - extending existing CI jobs is straightforward; existing ci.yml fully analyzed
- Pitfalls: HIGH - well-documented issues with cargo-llvm-cov component requirements and GitHub Actions permissions

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable tooling, monthly cadence sufficient)
