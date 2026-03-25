# Phase 51: Cross-Platform Release Pipeline - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

A git tag push (`v*`) triggers a GitHub Actions workflow that builds Trunk for macOS ARM, macOS Intel, Linux x64, and Windows x64 — producing platform-specific installers (.dmg, .AppImage, .msi) and portable .tar.gz archives. All artifacts are uploaded to the workflow run for manual attachment to GitHub Releases.

</domain>

<decisions>
## Implementation Decisions

### Workflow Structure
- **D-01:** Separate `release.yml` workflow (not part of ci.yml) triggered by tag push matching `v*`
- **D-02:** Matrix strategy with per-platform jobs running in parallel (macOS ARM, macOS Intel, Linux, Windows)
- **D-03:** No CI gate dependency — tag push implies code on main is already CI-validated; keeps release workflow simple and fast

### Build Tool
- **D-04:** Use `tauri-apps/tauri-action@v0` for building — handles Tauri-specific bundling, signing prep, and artifact output
- **D-05:** Each matrix entry configures tauri-action with appropriate target and runner

### macOS Strategy
- **D-06:** Separate ARM (aarch64-apple-darwin) and Intel (x86_64-apple-darwin) builds — not universal binary
- **D-07:** Rationale: REL-01 explicitly lists "macOS ARM, macOS Intel" as separate targets; separate builds are simpler and more explicit than universal binaries

### Portable Archives
- **D-08:** Post-build step wraps tauri-action installer output into .tar.gz archives per platform
- **D-09:** .tar.gz contains the platform-appropriate binary/app bundle for users who prefer portable installs

### Artifact Upload
- **D-10:** All artifacts (installers + .tar.gz) uploaded as GitHub Actions workflow artifacts via `actions/upload-artifact`
- **D-11:** No automated GitHub Release creation — release created manually with AI-assisted notes from GSD context (per STATE.md decision)

### Runner Selection
- **D-12:** Linux builds on `ubuntu-22.04` (REL-05 requirement for AppImage glibc compatibility)
- **D-13:** macOS builds on `macos-latest` (ARM) and `macos-13` (Intel, last Intel runner)
- **D-14:** Windows builds on `windows-latest`

### Claude's Discretion
- Exact matrix configuration format and variable naming
- Concurrency controls (cancel-in-progress for same tag, or allow all)
- Whether to cache Rust builds in release workflow (tradeoff: cache size vs build time for infrequent releases)
- Specific tauri-action configuration options and version pinning
- Artifact naming convention (e.g., `trunk-v{version}-{platform}-{arch}.{ext}`)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — REL-01 through REL-05 define success criteria for this phase

### Existing CI
- `.github/workflows/ci.yml` — Established GHA patterns (checkout@v6, setup-bun@v2, rust-toolchain@stable, rust-cache@v2, system deps for Linux)

### Tauri Configuration
- `src-tauri/tauri.conf.json` — Bundle config (`"targets": "all"`), app identifier, icon paths
- `src-tauri/Cargo.toml` — Rust dependencies, vendored-libgit2 feature, Tauri plugin list

### Prior Phase Context
- `.planning/phases/50-ci-quality-gates/50-CONTEXT.md` — CI patterns, runner config, caching strategy established in Phase 50

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `.github/workflows/ci.yml` — Linux system dependencies block can be reused for release Linux builds
- `src-tauri/icons/` — Full icon set for all platforms (icns, ico, png variants) already present
- `tauri.conf.json` bundle config — `"targets": "all"` already enables all installer formats

### Established Patterns
- GHA action versions: checkout@v6, setup-bun@v2, dtolnay/rust-toolchain@stable, Swatinem/rust-cache@v2
- `bun install --frozen-lockfile` for reproducible frontend builds
- `--manifest-path src-tauri/Cargo.toml` for cargo commands
- Concurrency groups: `ci-${{ github.ref }}` with cancel-in-progress

### Integration Points
- New file: `.github/workflows/release.yml` (separate from ci.yml)
- tauri-action reads `src-tauri/tauri.conf.json` for bundle configuration
- `bun run build` (beforeBuildCommand in tauri.conf.json) must succeed on all platforms

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 51-cross-platform-release-pipeline*
*Context gathered: 2026-03-25*
