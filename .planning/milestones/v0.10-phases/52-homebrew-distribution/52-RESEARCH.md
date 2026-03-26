# Phase 52: Homebrew Distribution - Research

**Researched:** 2026-03-26
**Domain:** Homebrew cask distribution, GitHub Actions cross-repo automation
**Confidence:** HIGH

## Summary

Phase 52 adds a Homebrew cask formula so macOS users can install Trunk via `brew install --cask joaofnds/tap/trunk`. The cask lives in the existing `joaofnds/homebrew-tap` repository and references architecture-specific `.dmg` files from GitHub Releases. The release workflow (`.github/workflows/release.yml`) is extended with post-build jobs that publish the draft release and push an updated cask file to the tap repo.

The existing `astro.rb` cask in the tap provides a working template, though it uses `on_intel`/`on_arm` blocks while modern Homebrew prefers the `arch arm:, intel:` + `sha256 arm:, intel:` shorthand. Since astro.rb uses the older pattern and works, either approach is valid for a personal tap. The workflow needs a Personal Access Token (PAT) stored as a repository secret to push commits to the homebrew-tap repo, since `GITHUB_TOKEN` is scoped to the current repository only.

**Primary recommendation:** Extend release.yml with two new jobs: (1) a `publish` job that waits for all builds, publishes the draft release, and (2) an `update-tap` job that downloads the .dmg assets, computes SHA256, generates `Casks/trunk.rb`, and pushes to `joaofnds/homebrew-tap`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Formula update automated via release workflow -- a job in release.yml computes SHA256 of the .dmg files and pushes an updated trunk.rb to homebrew-tap after builds complete
- **D-02:** The release workflow auto-publishes the GitHub Release (removes draft status) after all builds complete, then updates the tap -- fully automated pipeline
- **D-03:** Tap repo already exists at joaofnds/homebrew-tap with established Casks/ and Formula/ directories
- **D-04:** README updated to list trunk alongside existing entries (astro, asdf-install-latest)
- **D-05:** Same pattern as existing astro.rb cask -- on_intel/on_arm blocks with GitHub Release .dmg URLs
- **D-06:** Hardcoded .dmg naming pattern (based on Tauri's productName "trunk") rather than dynamic discovery from release assets

### Claude's Discretion
- Exact .dmg filename pattern to hardcode (inspect tauri-action output to determine)
- How the release workflow authenticates to push to homebrew-tap (PAT, deploy key, or GitHub App)
- Cask metadata (desc, homepage, app stanza vs binary stanza for .dmg)
- SHA256 computation approach in the workflow job

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DIST-01 | Homebrew cask formula published to joaofnds/homebrew-tap for macOS installation via `brew install --cask joaofnds/tap/trunk` | Cask DSL documented, .dmg naming verified from actual release assets, workflow automation pattern established |
</phase_requirements>

## Architecture Patterns

### Workflow Extension Structure

The existing `release.yml` has a single `build` job using a matrix strategy. This phase adds two sequential jobs after builds complete:

```
build (matrix: 4 platforms) → publish (single job) → update-tap (single job)
```

**Job dependency chain:**
1. `build` -- existing matrix job, creates draft release with assets via tauri-action
2. `publish` -- depends on `build`, publishes the draft release using `gh release edit $TAG --draft=false`
3. `update-tap` -- depends on `publish`, downloads .dmg assets, computes SHA256, generates cask, pushes to homebrew-tap

### DMG Asset Naming Pattern (Verified from Release v0.10.0-rc1)

Actual filenames produced by tauri-action with `productName: "trunk"` and `version: "0.1.0"`:

| Architecture | DMG Filename | Arch Suffix |
|-------------|-------------|-------------|
| ARM (aarch64-apple-darwin) | `trunk_0.1.0_aarch64.dmg` | `aarch64` |
| Intel (x86_64-apple-darwin) | `trunk_0.1.0_x64.dmg` | `x64` |

**Confidence: HIGH** -- verified against actual release assets on the draft release `v0.10.0-rc1`.

Pattern: `trunk_{TAURI_VERSION}_{ARCH}.dmg`

Where `TAURI_VERSION` comes from `src-tauri/tauri.conf.json` `version` field (currently `"0.1.0"`), and `ARCH` is `aarch64` or `x64`.

### Version Mismatch Between Git Tag and Tauri Version

**Critical finding:** The git tag (e.g., `v0.10.0-rc1`) and the Tauri version (e.g., `0.1.0`) are currently different. This means:

- The release URL uses the **git tag**: `https://github.com/joaofnds/trunk/releases/download/v0.10.0-rc1/...`
- The asset filename uses the **Tauri version**: `trunk_0.1.0_aarch64.dmg`

The cask needs both values. Since the workflow generates the cask dynamically, it has access to both: the git tag from `${{ github.ref_name }}` and the Tauri version from `tauri.conf.json` (or parsed from the actual asset filenames).

**Recommendation:** The workflow should extract the Tauri version from `tauri.conf.json` using `jq` and template the cask file with both the tag and Tauri version.

### Cask Formula Structure

The cask should use the `app` stanza (not `binary`), because a .dmg from Tauri contains a `.app` bundle. Homebrew auto-mounts the .dmg and the `app` stanza moves the `.app` to `/Applications`.

The `.app` name inside the DMG is `trunk.app` (matching the lowercase `productName` in tauri.conf.json).

**Recommended cask template (using `on_intel`/`on_arm` per D-05):**

```ruby
cask "trunk" do
  version "TAG_VERSION"

  on_intel do
    sha256 "INTEL_SHA256"
    url "https://github.com/joaofnds/trunk/releases/download/v#{version}/trunk_TAURI_VERSION_x64.dmg"
  end
  on_arm do
    sha256 "ARM_SHA256"
    url "https://github.com/joaofnds/trunk/releases/download/v#{version}/trunk_TAURI_VERSION_aarch64.dmg"
  end

  name "Trunk"
  desc "Desktop Git GUI"
  homepage "https://github.com/joaofnds/trunk"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "trunk.app"

  zap trash: [
    "~/Library/Application Support/com.joaofnds.trunk",
    "~/Library/Caches/com.joaofnds.trunk",
    "~/Library/WebKit/com.joaofnds.trunk",
  ]
end
```

**Notes on this template:**
- `version` uses the tag without the `v` prefix (e.g., `0.10.0-rc1`), enabling `v#{version}` in the URL
- `TAURI_VERSION` is hardcoded at generation time (e.g., `0.1.0`) since it comes from tauri.conf.json
- The `livecheck` stanza uses `:github_latest` strategy to help `brew audit` -- though for a personal tap this is optional
- The `zap` stanza uses the Tauri identifier `com.joaofnds.trunk` for cleanup paths
- `app "trunk.app"` moves the .app from the mounted DMG to `/Applications`

### Authentication: PAT for Cross-Repo Push

**Recommendation: Personal Access Token (classic) stored as a repository secret.**

| Option | Pros | Cons | Verdict |
|--------|------|------|---------|
| PAT (classic) | Simple, well-documented, works immediately | User-scoped, needs periodic rotation | **Use this** |
| Fine-grained PAT | Repo-scoped, more secure | Newer, same simplicity as classic | Also good |
| Deploy key | Repo-scoped, no user dependency | Cannot be used with `gh` CLI easily | Not recommended |
| GitHub App | Most secure, installation-scoped | Complex setup for a personal tap | Overkill |

**Setup:**
1. Create a fine-grained PAT with `Contents: Write` permission scoped to `joaofnds/homebrew-tap`
2. Store as secret `HOMEBREW_TAP_TOKEN` in the `joaofnds/trunk` repository
3. Use in workflow: `git push` with the token, or `gh api` calls

**Workflow authentication pattern:**
```yaml
- name: Push cask to homebrew-tap
  env:
    GH_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
  run: |
    git clone https://x-access-token:${GH_TOKEN}@github.com/joaofnds/homebrew-tap.git tap
    # ... update cask file ...
    cd tap && git push
```

### SHA256 Computation

The workflow downloads .dmg files from the published release and computes SHA256:

```yaml
- name: Download DMGs and compute SHA256
  env:
    GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  run: |
    gh release download "$TAG" --repo joaofnds/trunk --pattern "*.dmg" --dir ./dmgs
    ARM_SHA256=$(shasum -a 256 ./dmgs/trunk_*_aarch64.dmg | awk '{print $1}')
    INTEL_SHA256=$(shasum -a 256 ./dmgs/trunk_*_x64.dmg | awk '{print $1}')
```

**Note:** `gh release download` works on published releases. The `publish` job must complete before `update-tap` runs. Using `--pattern "*.dmg"` downloads only the two DMG files.

### Draft Release URL Rewriting

**Verified finding:** When a draft release is published, GitHub rewrites asset URLs from `untagged-HASH` to the actual tag name. The current draft release `v0.10.0-rc1` shows URLs like:
```
https://github.com/joaofnds/trunk/releases/download/untagged-fe0ed3fb30e6403e8376/trunk_0.1.0_aarch64.dmg
```
After publishing with `gh release edit v0.10.0-rc1 --draft=false`, the URL becomes:
```
https://github.com/joaofnds/trunk/releases/download/v0.10.0-rc1/trunk_0.1.0_aarch64.dmg
```

**Confidence: HIGH** -- this is well-documented GitHub behavior and confirmed by multiple sources.

### Recommended Project Structure (Changes)

```
.github/workflows/
└── release.yml              # Extended with publish + update-tap jobs

# In joaofnds/homebrew-tap repo:
Casks/
├── astro.rb                 # Existing
└── trunk.rb                 # NEW - generated by workflow
README.md                    # Updated with trunk entry
```

### Anti-Patterns to Avoid
- **Using GITHUB_TOKEN for cross-repo push:** Will fail silently -- GITHUB_TOKEN is scoped to the current repo only
- **Downloading assets from draft release:** Draft release asset URLs use `untagged-HASH` -- must publish first
- **Hardcoding version in cask without templating:** The cask file is generated per-release, so all values must come from the workflow context
- **Using `sha256 :no_check`:** Defeats integrity verification -- always compute real checksums

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SHA256 computation | Custom download + hash script | `gh release download` + `shasum -a 256` | gh CLI handles auth, retries, asset matching |
| Release publishing | GitHub API calls with curl | `gh release edit TAG --draft=false` | gh CLI is pre-installed on runners, handles auth |
| Cross-repo file update | Complex API calls | `git clone` + `git push` with PAT | Standard git operations, easy to debug |
| Cask generation | Manual editing | Shell heredoc/sed templating in workflow | Generated per-release with correct values |

## Common Pitfalls

### Pitfall 1: Draft Release Asset URLs
**What goes wrong:** Cask URLs reference `untagged-HASH` instead of the tag
**Why it happens:** Assets uploaded to draft releases get temporary URLs
**How to avoid:** Always publish the release BEFORE generating the cask
**Warning signs:** 404 errors when running `brew install --cask`

### Pitfall 2: GITHUB_TOKEN Cannot Push to Other Repos
**What goes wrong:** Push to homebrew-tap silently fails or returns 403
**Why it happens:** GITHUB_TOKEN is scoped to the workflow's own repository
**How to avoid:** Use a PAT stored as `HOMEBREW_TAP_TOKEN` secret
**Warning signs:** Authentication errors in the update-tap job

### Pitfall 3: Version Mismatch in Cask URL
**What goes wrong:** Cask URL has wrong version in the asset filename
**Why it happens:** Git tag version and Tauri version in tauri.conf.json are different
**How to avoid:** Extract Tauri version from tauri.conf.json at workflow time, use it in the cask template
**Warning signs:** 404 when downloading DMG after `brew install`

### Pitfall 4: Race Condition Between Publish and Download
**What goes wrong:** update-tap job tries to download assets before publish job finishes
**Why it happens:** Jobs run in parallel unless `needs:` is specified
**How to avoid:** Chain jobs with `needs: [publish]`
**Warning signs:** "release not found" or 404 errors in update-tap job

### Pitfall 5: Cask app Stanza Wrong Name
**What goes wrong:** Homebrew can't find the .app inside the DMG
**Why it happens:** Using wrong app name (e.g., `"Trunk.app"` instead of `"trunk.app"`)
**How to avoid:** The app name matches Tauri's `productName` exactly: `"trunk"` -> `app "trunk.app"`
**Warning signs:** Error during `brew install --cask` about missing app

### Pitfall 6: Out-of-Scope Decision Override
**What goes wrong:** Planner hesitates on auto-publishing because REQUIREMENTS.md says "Automated GitHub Release creation" is out of scope
**Why it happens:** Phase 52 CONTEXT.md D-02 explicitly overrides this for the publish-draft step
**How to avoid:** D-02 is a locked decision -- the workflow auto-publishes (removes draft). This does NOT create a release from scratch; it publishes an existing draft created by tauri-action. The original out-of-scope item was about creating releases with auto-generated notes.
**Warning signs:** N/A -- planner awareness item

## Code Examples

### Complete Cask Formula (trunk.rb)

```ruby
# Generated by joaofnds/trunk release workflow. DO NOT EDIT.
cask "trunk" do
  version "0.10.0"

  on_intel do
    sha256 "abc123..."
    url "https://github.com/joaofnds/trunk/releases/download/v#{version}/trunk_0.1.0_x64.dmg"
  end
  on_arm do
    sha256 "def456..."
    url "https://github.com/joaofnds/trunk/releases/download/v#{version}/trunk_0.1.0_aarch64.dmg"
  end

  name "Trunk"
  desc "Desktop Git GUI"
  homepage "https://github.com/joaofnds/trunk"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "trunk.app"

  zap trash: [
    "~/Library/Application Support/com.joaofnds.trunk",
    "~/Library/Caches/com.joaofnds.trunk",
    "~/Library/WebKit/com.joaofnds.trunk",
  ]
end
```

### Workflow: Publish Job

```yaml
publish:
  needs: [build]
  runs-on: ubuntu-latest
  steps:
    - name: Publish release
      env:
        GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: gh release edit "${{ github.ref_name }}" --draft=false --repo "${{ github.repository }}"
```

### Workflow: Update Tap Job

```yaml
update-tap:
  needs: [publish]
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v6

    - name: Get Tauri version
      id: tauri
      run: echo "version=$(jq -r .version src-tauri/tauri.conf.json)" >> "$GITHUB_OUTPUT"

    - name: Download DMGs and compute checksums
      env:
        GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: |
        TAG="${{ github.ref_name }}"
        gh release download "$TAG" --pattern "*.dmg" --dir ./dmgs
        echo "arm_sha256=$(shasum -a 256 ./dmgs/*_aarch64.dmg | awk '{print $1}')" >> "$GITHUB_ENV"
        echo "intel_sha256=$(shasum -a 256 ./dmgs/*_x64.dmg | awk '{print $1}')" >> "$GITHUB_ENV"

    - name: Generate cask and push to tap
      env:
        HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
      run: |
        TAG="${{ github.ref_name }}"
        VERSION="${TAG#v}"
        TAURI_VERSION="${{ steps.tauri.outputs.version }}"

        git clone "https://x-access-token:${HOMEBREW_TAP_TOKEN}@github.com/joaofnds/homebrew-tap.git" tap

        cat > tap/Casks/trunk.rb << CASK
        # Generated by joaofnds/trunk release workflow. DO NOT EDIT.
        cask "trunk" do
          version "${VERSION}"
          ...
        end
        CASK

        cd tap
        git config user.name "github-actions[bot]"
        git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
        git add Casks/trunk.rb
        git commit -m "Update trunk to ${TAG}"
        git push
```

### README Update for Tap

Current README has a "Formulae" table. Add a "Casks" section:

```markdown
## Casks

| Repo | Cask | Description |
| ---- | ---- | ----------- |
| [trunk](https://github.com/joaofnds/trunk) | [cask](Casks/trunk.rb) | Desktop Git GUI |
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `on_intel`/`on_arm` blocks | `arch arm:, intel:` + `sha256 arm:, intel:` | Homebrew 2022+ | More concise for simple arch differences |
| `appcast` stanza | `livecheck` block | Homebrew 2020 | appcast removed entirely |
| Classic PAT | Fine-grained PAT | GitHub 2022+ | Better security with repo-scoping |
| Manual formula update | Automated via workflow | N/A | Eliminates human error |

**Note on `on_intel`/`on_arm` vs `arch`/`sha256` shorthand:** D-05 specifies the `on_intel`/`on_arm` pattern (matching `astro.rb`). Both are valid in Homebrew. The older pattern is slightly more verbose but works identically and maintains visual consistency with the existing cask in the tap. The shorthand is preferred for new submissions to homebrew-cask main, but for a personal tap either is fine.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Manual validation (no automated test framework for workflow + Homebrew) |
| Config file | N/A |
| Quick run command | `brew install --cask joaofnds/tap/trunk` |
| Full suite command | `brew install --cask joaofnds/tap/trunk && brew uninstall --cask trunk` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DIST-01 | Homebrew cask installs Trunk from .dmg | manual | `brew install --cask joaofnds/tap/trunk` | N/A |
| DIST-01 | Cask formula exists in tap repo | smoke | `curl -s https://raw.githubusercontent.com/joaofnds/homebrew-tap/main/Casks/trunk.rb` | N/A |
| DIST-01 | Release workflow publishes and updates tap | smoke | Trigger tag push, verify workflow succeeds | N/A |

### Sampling Rate
- **Per task commit:** Review generated cask syntax manually
- **Per wave merge:** Trigger a test release (tag push) and verify full pipeline
- **Phase gate:** `brew install --cask joaofnds/tap/trunk` succeeds on a clean machine

### Wave 0 Gaps
- [ ] PAT secret `HOMEBREW_TAP_TOKEN` must be created and stored in trunk repo before pipeline can work
- [ ] No automated unit tests possible -- this is workflow + external service integration

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| gh CLI | Workflow (publish, download) | Provided by runner | Pre-installed on ubuntu-latest | N/A |
| jq | Tauri version extraction | Provided by runner | Pre-installed on ubuntu-latest | N/A |
| shasum | SHA256 computation | Provided by runner | Pre-installed on ubuntu-latest | sha256sum on Linux |
| git | Push to tap repo | Provided by runner | Pre-installed on ubuntu-latest | N/A |
| HOMEBREW_TAP_TOKEN | Cross-repo push | Not yet created | N/A | None -- blocking |

**Missing dependencies with no fallback:**
- `HOMEBREW_TAP_TOKEN` secret must be created before the workflow can push to homebrew-tap

**Missing dependencies with fallback:**
- None

## Open Questions

1. **Should `shasum` or `sha256sum` be used?**
   - What we know: macOS runners have `shasum`, Linux runners have both `shasum` and `sha256sum`
   - What's unclear: The update-tap job runs on `ubuntu-latest` where both are available
   - Recommendation: Use `shasum -a 256` for consistency with Homebrew's own documentation

2. **Should the README update be part of the initial cask commit or a separate task?**
   - What we know: D-04 says README should be updated
   - What's unclear: Whether it should be updated on every release or just once
   - Recommendation: Update README once as a manual commit to homebrew-tap (not automated per-release), since the table entry is static

3. **Prerelease handling**
   - What we know: The workflow already detects prereleases via `contains(github.ref_name, '-')`
   - What's unclear: Should the cask be updated for prerelease versions?
   - Recommendation: Skip tap update for prereleases. The `livecheck` strategy `:github_latest` already ignores prereleases. Add a conditional: `if: "!contains(github.ref_name, '-')"` on the update-tap job.

## Sources

### Primary (HIGH confidence)
- Actual release assets from `v0.10.0-rc1` draft release -- verified .dmg naming pattern
- `src-tauri/tauri.conf.json` -- productName "trunk", version "0.1.0"
- `.github/workflows/release.yml` -- existing workflow structure
- `/Users/joaofnds/code/homebrew-tap/Casks/astro.rb` -- existing cask template
- [Cask Cookbook -- Homebrew Documentation](https://docs.brew.sh/Cask-Cookbook) -- DSL reference
- [gh release edit -- CLI manual](https://cli.github.com/manual/gh_release_edit) -- draft=false flag
- [gh release download](https://cli.github.com/manual/gh_release_download) -- pattern-based asset download
- [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) -- release creation, asset naming

### Secondary (MEDIUM confidence)
- [Automating Homebrew tap updates](https://josh.fail/2023/automate-updating-custom-homebrew-formulae-with-github-actions/) -- workflow pattern for cross-repo push
- [GitHub Actions cross-repo push](https://www.w3tutorials.net/blog/how-to-push-to-another-repository-in-github-actions/) -- PAT authentication pattern
- [Homebrew arch/sha256 shorthand PR](https://github.com/Homebrew/brew/pull/13703) -- modern cask DSL patterns

### Tertiary (LOW confidence)
- None -- all findings verified against primary sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- verified against actual release assets and existing workflow
- Architecture: HIGH -- based on existing astro.rb pattern and Homebrew official docs
- Pitfalls: HIGH -- version mismatch verified empirically, URL rewriting confirmed by multiple sources

**Research date:** 2026-03-26
**Valid until:** 2026-04-26 (stable domain, unlikely to change)
