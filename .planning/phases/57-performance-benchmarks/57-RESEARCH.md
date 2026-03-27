# Phase 57: Performance Benchmarks - Research

**Researched:** 2026-03-27
**Domain:** Rust benchmarking with Criterion, CI regression detection
**Confidence:** HIGH

## Summary

Phase 57 establishes Criterion-based benchmarks for critical Rust backend operations and integrates regression detection into CI. The scope is Rust-only -- BENCH-03 (frontend IPC) and BENCH-04 (startup time) are deferred to Phase 58 per user decision D-01.

The primary benchmark targets are: `walk_commits` (graph lane algorithm at 100/1k/10k commit scales), `list_refs_inner`, `diff_unstaged_inner`, and `stage_hunk_inner`. All are public functions accessible through the `trunk_lib` rlib crate. The `walk_commits` function takes `&mut Repository` directly, while the `_inner` command functions follow a `(path: &str, ..., state_map: &HashMap<String, PathBuf>)` pattern that requires a filesystem-backed repo with a state_map lookup.

CI integration uses `benchmark-action/github-action-benchmark@v1` with `actions/cache` for baseline storage (not gh-pages), configured to fail on 30% regression (130% alert threshold). A separate `benchmarks.yml` workflow triggers on push to main only, while `ci.yml` gets a compile-check (`cargo test --benches`) to catch build breaks.

**Primary recommendation:** Use Criterion 0.8.2 with `html_reports` feature, `BenchmarkGroup` + `BenchmarkId` for parameterized benchmarks, `OnceLock` for fixture caching, and git2's `blob()` + `treebuilder()` + `commit()` for in-memory repo generation at scale.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Rust-only this phase -- Criterion benchmarks for `_inner` functions. BENCH-03 (frontend IPC round-trips) and BENCH-04 (application startup time) are deferred to Phase 58 where tauri-driver infrastructure will already exist.
- **D-02:** Required benchmark targets: `walk_commits` (graph.rs) at 100, 1k, 10k commit scales; `list_refs_inner`, `diff_unstaged_inner`, `stage_hunk_inner`.
- **D-03:** Additional benchmark targets at Claude's discretion based on call frequency and complexity (e.g., `get_status_inner`, `search_commits_inner`).
- **D-04:** Use `benchmark-action/github-action-benchmark` for automated regression detection. Compares each run against previous stored results.
- **D-05:** Alert threshold of 130% (30% regression tolerance) -- sweet spot for GitHub Actions shared runners. Catches real regressions while ignoring CI noise.
- **D-06:** `fail-on-alert: true` -- workflow fails if any benchmark exceeds the threshold.
- **D-07:** Store baselines in CI cache (`actions/cache`), not gh-pages branch.
- **D-08:** Use git2's `treebuilder()` + `blob()` to create benchmark repos entirely in-memory (no filesystem writes). Fast even at 10k commits.
- **D-09:** Generate fixtures once per benchmark group via `OnceLock`. Fixture creation runs outside the timed section.
- **D-10:** Separate `benchmarks.yml` workflow -- triggers on push to main only. Runs `cargo bench`, compares via benchmark-action, fails on regression.
- **D-11:** Add `cargo test --benches` to existing `ci.yml` as a compile-check only (ensures benchmarks build without running them).
- **D-12:** Use Criterion 0.8 with `html_reports` feature. Each benchmark file gets a `[[bench]]` entry with `harness = false`.

### Claude's Discretion
- Which additional `_inner` functions to benchmark beyond the required set (D-03)
- Exact benchmark group structure and file organization in `benches/`
- Sample size and measurement time configuration per benchmark
- Whether to add branch topology (merges, forks) to benchmark fixtures or keep them linear
- `warm_up_time` and `measurement_time` tuning per benchmark scale

### Deferred Ideas (OUT OF SCOPE)
- BENCH-03 (Frontend IPC round-trip benchmarks) -- deferred to Phase 58
- BENCH-04 (Application startup time measurement) -- deferred to Phase 58
- Visual benchmark dashboard (gh-pages) -- can be added later
- iai-callgrind instruction counting -- Linux-only, deterministic but doesn't measure wall-clock time
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| BENCH-01 | Criterion benchmarks for graph lane algorithm (walk_commits) with varying repo sizes | Criterion 0.8.2 BenchmarkGroup + BenchmarkId parameterized benchmarks at 100/1k/10k scales; `walk_commits` takes `&mut Repository` directly |
| BENCH-02 | Criterion benchmarks for ref listing, diff computation, and hunk staging | `list_refs_inner`, `diff_unstaged_inner`, `stage_hunk_inner` all accessible via `trunk_lib::commands::*`; each needs filesystem-backed temp repo with state_map |
| BENCH-03 | Frontend IPC round-trip benchmarks (DEFERRED to Phase 58) | User decision D-01 defers to Phase 58 |
| BENCH-04 | Application startup time measurement (DEFERRED to Phase 58) | User decision D-01 defers to Phase 58 |
| BENCH-05 | CI pipeline detects performance regressions with threshold-based gates | `benchmark-action/github-action-benchmark@v1` with `actions/cache`, 130% alert threshold, `fail-on-alert: true` |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- All git operations go through git2 crate, no shelling out
- `$lib` -> `src/lib`, commands in `src-tauri/src/commands/`
- Backend: Tauri 2, git2 0.19 (libgit2), tokio 1
- Library crate `trunk_lib` with `crate-type = ["staticlib", "cdylib", "rlib"]`
- Run all checks (test, check, format, lint) before every push

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| criterion | 0.8.2 | Statistical micro-benchmarking | De facto Rust benchmark framework; 132M+ downloads, statistics-driven, HTML reports |
| benchmark-action/github-action-benchmark | v1 | CI regression detection | Parses Criterion output natively via `tool: 'cargo'`; supports external JSON + cache |
| actions/cache | v4 | Persistent benchmark baselines | Stores benchmark-data.json across workflow runs without gh-pages branch |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| git2 | 0.19 (already in deps) | Benchmark fixture generation | `blob()` + `treebuilder()` + `commit()` for in-memory repo creation |
| tempfile | 3 (already in dev-deps) | Filesystem-backed test repos | For `_inner` functions that need `state_map` with real path |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| criterion | divan | Newer, simpler API, but less ecosystem support and no `benchmark-action` integration |
| benchmark-action | bencher.dev | More features but commercial SaaS, overkill for personal project |
| actions/cache baselines | gh-pages branch | More persistent but adds branch noise, harder to manage |

**Installation:**
```bash
# In src-tauri/Cargo.toml [dev-dependencies]
criterion = { version = "0.8", features = ["html_reports"] }
```

**Version verification:** `cargo search criterion` on 2026-03-27 returns `criterion = "0.8.2"`. This is the current latest stable release.

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/
├── benches/
│   ├── bench_graph.rs       # walk_commits benchmarks (BENCH-01)
│   ├── bench_commands.rs    # list_refs, diff_unstaged, stage_hunk (BENCH-02)
│   └── fixtures.rs          # Shared fixture generation (OnceLock-cached repos)
.github/
└── workflows/
    ├── benchmarks.yml       # New: benchmark + regression detection
    └── ci.yml               # Modified: add cargo test --benches
```

### Pattern 1: In-Memory Repo Generation with git2
**What:** Create benchmark repos using git2's low-level API (blob + treebuilder + commit) to avoid filesystem I/O overhead
**When to use:** For `walk_commits` benchmarks where only the ODB matters (no working directory needed)
**Example:**
```rust
// Source: git2 docs (docs.rs/git2/latest)
use std::sync::OnceLock;

struct BenchRepo {
    _dir: tempfile::TempDir,
    path: std::path::PathBuf,
}

fn make_repo_with_n_commits(n: usize) -> BenchRepo {
    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();
    let sig = git2::Signature::now("Bench", "bench@test.com").unwrap();

    let mut parent_oid: Option<git2::Oid> = None;

    for i in 0..n {
        // Write blob directly to ODB (no filesystem write)
        let blob_oid = repo.blob(format!("content {}", i).as_bytes()).unwrap();

        // Build tree in memory
        let mut tb = repo.treebuilder(None).unwrap();
        tb.insert(
            format!("file{}.txt", i),
            blob_oid,
            0o100644,
        ).unwrap();
        let tree_oid = tb.write().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        // Create commit with parent chain
        let parents: Vec<git2::Commit> = parent_oid
            .map(|oid| repo.find_commit(oid).unwrap())
            .into_iter()
            .collect();
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

        let oid = repo.commit(
            Some("refs/heads/main"),
            &sig, &sig,
            &format!("Commit {}", i),
            &tree,
            &parent_refs,
        ).unwrap();
        parent_oid = Some(oid);
    }

    BenchRepo {
        path: dir.path().to_path_buf(),
        _dir: dir,
    }
}

// Cache with OnceLock -- fixture created once, reused across iterations
static REPO_100: OnceLock<BenchRepo> = OnceLock::new();
static REPO_1K: OnceLock<BenchRepo> = OnceLock::new();
static REPO_10K: OnceLock<BenchRepo> = OnceLock::new();
```

### Pattern 2: Parameterized Benchmark Groups
**What:** Use `BenchmarkGroup` + `BenchmarkId` to benchmark the same function at different input scales
**When to use:** For `walk_commits` at 100/1k/10k commit scales
**Example:**
```rust
// Source: Criterion docs (bheisler.github.io/criterion.rs/book)
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_walk_commits(c: &mut Criterion) {
    let mut group = c.benchmark_group("walk_commits");

    for &(label, size, repo_lock) in &[
        ("100", 100, &REPO_100),
        ("1k", 1_000, &REPO_1K),
        ("10k", 10_000, &REPO_10K),
    ] {
        let bench_repo = repo_lock.get_or_init(|| make_repo_with_n_commits(size));
        group.bench_with_input(
            BenchmarkId::from_parameter(label),
            &bench_repo.path,
            |b, path| {
                b.iter(|| {
                    let mut repo = git2::Repository::open(path).unwrap();
                    trunk_lib::git::graph::walk_commits(&mut repo, 0, usize::MAX).unwrap();
                });
            },
        );
    }
    group.finish();
}
```

### Pattern 3: _inner Function Benchmarks with state_map
**What:** Benchmark command `_inner` functions that require `(path: &str, ..., state_map: &HashMap<String, PathBuf>)` signature
**When to use:** For BENCH-02 targets: `list_refs_inner`, `diff_unstaged_inner`, `stage_hunk_inner`
**Example:**
```rust
use std::collections::HashMap;
use std::path::PathBuf;

fn bench_list_refs(c: &mut Criterion) {
    // Create a repo with branches for list_refs to enumerate
    let bench_repo = REPO_REFS.get_or_init(|| make_repo_with_branches(50));
    let path = bench_repo.path.display().to_string();
    let mut state_map: HashMap<String, PathBuf> = HashMap::new();
    state_map.insert(path.clone(), bench_repo.path.clone());

    c.bench_function("list_refs_inner", |b| {
        b.iter(|| {
            trunk_lib::commands::branches::list_refs_inner(&path, &state_map).unwrap();
        });
    });
}
```

### Pattern 4: CI Benchmark Workflow
**What:** GitHub Actions workflow that runs benchmarks and detects regressions
**When to use:** On every push to main
**Example:**
```yaml
# Source: benchmark-action/github-action-benchmark README
name: Benchmarks

on:
  push:
    branches: [main]

permissions:
  contents: read

jobs:
  benchmark:
    name: Criterion Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev build-essential curl wget file \
            libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: "src-tauri -> target"
          save-if: true
      - name: Run benchmarks
        run: cd src-tauri && cargo bench -- --output-format bencher | tee output.txt
      - name: Download previous benchmark data
        uses: actions/cache@v4
        with:
          path: ./cache
          key: ${{ runner.os }}-benchmark
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: src-tauri/output.txt
          external-data-json-path: ./cache/benchmark-data.json
          alert-threshold: '130%'
          fail-on-alert: true
```

### Anti-Patterns to Avoid
- **Filesystem I/O in the timed loop:** Never create repos inside `b.iter()`. Use `OnceLock` to create once, reopen with `Repository::open()` inside the loop.
- **Using `--output-format bencher` without Criterion:** The `tool: 'cargo'` setting in benchmark-action expects Criterion's bencher-compatible output. Standard `cargo bench` without Criterion uses a different format.
- **Running benchmarks on PRs:** Shared CI runners have variable performance. Running on push-to-main only (with cached baselines) provides more consistent comparisons.
- **Amending tree per commit:** Don't build a cumulative tree that grows with each commit in the loop. Create a minimal single-file tree per commit -- the benchmark measures `walk_commits`, not tree size.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Statistical analysis of timings | Custom mean/stddev calculations | Criterion's built-in statistics | Handles outlier detection, confidence intervals, regression detection |
| CI regression detection | Shell script comparing JSON outputs | benchmark-action/github-action-benchmark | Handles threshold comparison, caching, alerting with tested edge cases |
| Benchmark output parsing | Custom JSON parser for cargo bench output | benchmark-action `tool: 'cargo'` | Criterion output format changes between versions; action handles this |
| Fixture caching across iterations | `lazy_static!` or manual `Mutex<Option<T>>` | `std::sync::OnceLock` | stdlib since Rust 1.70, zero-cost after init, no external crate needed |

**Key insight:** Criterion handles the entire statistical pipeline (warmup, sampling, outlier detection, comparison). The only custom code needed is fixture generation and the benchmark function bodies.

## Common Pitfalls

### Pitfall 1: Criterion Output Format for benchmark-action
**What goes wrong:** benchmark-action can't parse the output, reports zero benchmarks
**Why it happens:** Criterion 0.8 defaults to its own verbose output format, not bencher-compatible. The `--output-format bencher` flag is needed for benchmark-action to parse results. However, this flag changed behavior across versions.
**How to avoid:** Use `cargo bench -- --output-format bencher` and pipe through `tee` to capture output. Verify the output file contains lines like `test bench_name ... bench: X ns/iter (+/- Y)`.
**Warning signs:** benchmark-action step succeeds but reports "0 benchmarks found"

### Pitfall 2: OnceLock Fixtures and Benchmark Isolation
**What goes wrong:** Benchmarks that modify repo state (stage_hunk) corrupt the shared fixture
**Why it happens:** `OnceLock` creates one repo shared across all iterations. If a benchmark mutates the repo (staging, committing), subsequent iterations see different state.
**How to avoid:** For mutating operations like `stage_hunk_inner`, create a fresh fixture per iteration OR use `iter_batched` with setup/teardown. For read-only operations (`walk_commits`, `list_refs`, `diff_unstaged`), OnceLock is safe.
**Warning signs:** Benchmark results vary wildly across iterations, or "No unstaged changes" errors

### Pitfall 3: Tauri System Dependencies in CI
**What goes wrong:** `cargo bench` fails to compile on ubuntu-latest
**Why it happens:** `trunk_lib` depends on Tauri which requires `libwebkit2gtk-4.1-dev` and other system packages even for benchmarks (the `rlib` crate type still links against Tauri types)
**How to avoid:** Copy the same `apt-get install` block from the existing `ci.yml` cargo-test job into `benchmarks.yml`. The system deps list is already proven in CI.
**Warning signs:** Compilation errors mentioning `webkit2gtk` or `glib`

### Pitfall 4: 10k Commit Fixture Generation Time
**What goes wrong:** Benchmark setup takes 30+ seconds, masking actual benchmark time
**Why it happens:** Creating 10k commits with filesystem writes is slow. Even with git2's treebuilder, the ODB writes accumulate.
**How to avoid:** Use the in-memory blob+treebuilder pattern (D-08). This bypasses filesystem writes entirely -- only the ODB gets written. With OnceLock (D-09), this cost is paid exactly once per benchmark run, not per iteration.
**Warning signs:** First benchmark iteration takes 10x longer than subsequent ones (OnceLock working correctly shows this pattern, but it should only happen once)

### Pitfall 5: `walk_commits` Opens All Refs
**What goes wrong:** Benchmark for `walk_commits` shows unexpected scaling because revwalk pushes all refs
**Why it happens:** `walk_commits` calls `revwalk.push_glob("refs/heads")`, `push_glob("refs/remotes")`, and `push_glob("refs/tags")`. A repo with only `refs/heads/main` walks all commits once. A repo with N branches walks overlapping commit sets.
**How to avoid:** For the linear scaling benchmark (100/1k/10k), create repos with a single `refs/heads/main` branch. This isolates the lane algorithm scaling from ref enumeration overhead.
**Warning signs:** 1k commits benchmark is slower than expected compared to 100 commits

### Pitfall 6: cargo test --benches vs cargo bench
**What goes wrong:** `cargo test --benches` runs the benchmarks instead of just compile-checking
**Why it happens:** Confusion between compile-check and execution. `cargo test --benches` compiles and runs benchmark binaries in test mode (which Criterion handles by running a single iteration). This is actually fine for compile-checking but takes a few seconds.
**How to avoid:** `cargo test --benches --no-run` if you want pure compile-check without any execution. However, `cargo test --benches` (with run) is also acceptable as Criterion's test mode is fast. Decision D-11 says "compile-check only" so use `--no-run` for clarity.
**Warning signs:** CI gate 2 takes longer than expected due to benchmark execution

## Code Examples

### Complete Benchmark File Template
```rust
// Source: Criterion docs + project-specific patterns
// benches/bench_graph.rs

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::OnceLock;
use std::time::Duration;

struct BenchRepo {
    _dir: tempfile::TempDir,
    path: std::path::PathBuf,
}

// In-memory repo generation using git2 treebuilder (D-08)
fn make_linear_repo(n: usize) -> BenchRepo {
    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();
    let sig = git2::Signature::now("Bench", "bench@test.com").unwrap();
    let mut parent: Option<git2::Oid> = None;

    for i in 0..n {
        let blob_oid = repo.blob(format!("content-{}", i).as_bytes()).unwrap();
        let mut tb = repo.treebuilder(None).unwrap();
        tb.insert(format!("file{}.txt", i), blob_oid, 0o100644).unwrap();
        let tree_oid = tb.write().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        let parents: Vec<git2::Commit> = parent
            .map(|oid| repo.find_commit(oid).unwrap())
            .into_iter()
            .collect();
        let refs: Vec<&git2::Commit> = parents.iter().collect();

        let oid = repo
            .commit(Some("refs/heads/main"), &sig, &sig, &format!("Commit {}", i), &tree, &refs)
            .unwrap();
        parent = Some(oid);
    }

    BenchRepo { path: dir.path().to_path_buf(), _dir: dir }
}

// OnceLock caching (D-09)
static REPO_100: OnceLock<BenchRepo> = OnceLock::new();
static REPO_1K: OnceLock<BenchRepo> = OnceLock::new();
static REPO_10K: OnceLock<BenchRepo> = OnceLock::new();

fn bench_walk_commits(c: &mut Criterion) {
    let mut group = c.benchmark_group("walk_commits");
    // Adjust measurement time for larger repos
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(5));

    let configs: &[(&str, usize, &OnceLock<BenchRepo>)] = &[
        ("100", 100, &REPO_100),
        ("1k", 1_000, &REPO_1K),
        ("10k", 10_000, &REPO_10K),
    ];

    for &(label, size, lock) in configs {
        let bench_repo = lock.get_or_init(|| make_linear_repo(size));
        group.bench_with_input(
            BenchmarkId::from_parameter(label),
            &bench_repo.path,
            |b, path| {
                b.iter(|| {
                    let mut repo = git2::Repository::open(path).unwrap();
                    trunk_lib::git::graph::walk_commits(&mut repo, 0, usize::MAX).unwrap()
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_walk_commits);
criterion_main!(benches);
```

### Cargo.toml [[bench]] Entry
```toml
# Source: Criterion getting started guide
[dev-dependencies]
criterion = { version = "0.8", features = ["html_reports"] }

[[bench]]
name = "bench_graph"
harness = false

[[bench]]
name = "bench_commands"
harness = false
```

### stage_hunk_inner Benchmark with iter_batched
```rust
// For mutating benchmarks: create fresh state per iteration
use criterion::BatchSize;

fn bench_stage_hunk(c: &mut Criterion) {
    c.bench_function("stage_hunk_inner", |b| {
        b.iter_batched(
            || {
                // Setup: create repo with unstaged change and return (path, state_map)
                let dir = tempfile::tempdir().unwrap();
                let repo = git2::Repository::init(dir.path()).unwrap();
                // ... create initial commit, then modify file to produce unstaged diff ...
                let path = dir.path().display().to_string();
                let mut state_map = std::collections::HashMap::new();
                state_map.insert(path.clone(), dir.path().to_path_buf());
                (dir, path, state_map)  // dir must live until iteration ends
            },
            |(dir, path, state_map)| {
                trunk_lib::commands::staging::stage_hunk_inner(
                    &path, "README.md", 0, &state_map
                ).unwrap();
                drop(dir); // explicit drop for clarity
            },
            BatchSize::SmallInput,
        );
    });
}
```

### benchmarks.yml Workflow
```yaml
# Source: benchmark-action/github-action-benchmark README
name: Benchmarks

on:
  push:
    branches: [main]

concurrency:
  group: benchmarks-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read

jobs:
  benchmark:
    name: Criterion Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            build-essential \
            curl wget file \
            libxdo-dev \
            libssl-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: "src-tauri -> target"
          save-if: true

      - name: Run benchmarks
        run: cd src-tauri && cargo bench -- --output-format bencher | tee output.txt

      - name: Download previous benchmark data
        uses: actions/cache@v4
        with:
          path: ./cache
          key: ${{ runner.os }}-benchmark

      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: src-tauri/output.txt
          external-data-json-path: ./cache/benchmark-data.json
          alert-threshold: '130%'
          fail-on-alert: true
```

## Discretionary Recommendations

### Additional Benchmark Targets (D-03)
Based on call frequency from the invoke_handler registration and complexity:

| Function | Module | Rationale | Priority |
|----------|--------|-----------|----------|
| `get_status_inner` | staging.rs | Called on every poll cycle, iterates all files | HIGH -- add |
| `search_commits_inner` | history.rs | Searches cached graph, O(n) over all commits | MEDIUM -- skip (operates on in-memory GraphResult, not git2) |

**Recommendation:** Add `get_status_inner` as it exercises `repo.statuses()` which is a real git2 operation. Skip `search_commits_inner` as it only searches an in-memory vector (not a meaningful benchmark of git performance).

### File Organization
**Recommendation:** Two benchmark files.
- `bench_graph.rs` -- `walk_commits` parameterized benchmarks (BENCH-01). Separate because it has unique fixture needs (large repos at multiple scales).
- `bench_commands.rs` -- `list_refs_inner`, `diff_unstaged_inner`, `stage_hunk_inner`, `get_status_inner` (BENCH-02 + D-03). These share similar fixture patterns (moderate repo with branches/changes).

A shared `fixtures.rs` module is unnecessary -- Criterion benches are standalone binaries. Use inline helper functions within each bench file. If duplication becomes a problem, extract to a `benches/support/` module later.

### Measurement Configuration
**Recommendation:**
- `walk_commits` 100 commits: default (5s measurement, 3s warmup)
- `walk_commits` 1k commits: measurement_time 10s (each iteration is slower)
- `walk_commits` 10k commits: measurement_time 15s, sample_size 20 (each iteration may take seconds)
- Command benchmarks: default settings (operations are fast, ~ms range)

### Fixture Topology
**Recommendation:** Keep linear for the primary scaling benchmark. Linear repos isolate the lane algorithm's O(n) behavior. Add a separate benchmark with branch topology (5 branches, some merges) at fixed 1k commits to test the lane allocation path. This gives both scaling data and real-world-topology data.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| criterion 0.5 + `lazy_static!` | criterion 0.8 + `std::sync::OnceLock` | Rust 1.70 (2023) + Criterion 0.6+ | No external crate for fixture caching; cleaner API |
| gh-pages benchmark storage | actions/cache + external-data-json-path | benchmark-action v1 | Simpler setup, no extra branch |
| `criterion_group!` with config | `BenchmarkGroup::measurement_time()` | Criterion 0.5+ | Per-group config instead of global |

**Deprecated/outdated:**
- `lazy_static!` for benchmark fixtures: Use `OnceLock` (stdlib since Rust 1.70)
- Criterion 0.3 API: `Benchmark` struct replaced by `BenchmarkGroup`
- `#[bench]` attribute (nightly only): Use Criterion with `harness = false` on stable

## Open Questions

1. **Criterion --output-format bencher compatibility with 0.8**
   - What we know: Criterion 0.5 supported `--output-format bencher`. Criterion 0.8 documentation still references it.
   - What's unclear: Whether the output format changed between versions in any way that breaks benchmark-action parsing
   - Recommendation: Test locally with `cargo bench -- --output-format bencher` and verify output matches expected `test X ... bench: Y ns/iter (+/- Z)` format before committing CI workflow

2. **10k commit fixture generation time**
   - What we know: git2 blob+treebuilder is fast (avoids filesystem), but 10k ODB writes still take measurable time
   - What's unclear: Exact generation time on CI runners (could be 2s or 20s)
   - Recommendation: Measure locally, add a comment in the fixture code documenting expected generation time. If >15s, consider reducing to 5k.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable | All benchmarks | Yes | 1.93.1 | -- |
| cargo | Benchmark runner | Yes | 1.93.1 | -- |
| criterion (crate) | Benchmark framework | Yes (crates.io) | 0.8.2 | -- |
| git2 (crate) | Fixture generation | Yes (already dep) | 0.19 | -- |
| tempfile (crate) | Temp directories | Yes (already dev-dep) | 3 | -- |
| GitHub Actions ubuntu-latest | CI benchmarks | Yes | ubuntu-24.04 | -- |
| libwebkit2gtk-4.1-dev | trunk_lib compilation | Yes (apt) | -- | -- |

**Missing dependencies with no fallback:** None
**Missing dependencies with fallback:** None

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Criterion 0.8.2 |
| Config file | `src-tauri/Cargo.toml` (dev-dependency + [[bench]] entries) |
| Quick run command | `cd src-tauri && cargo bench -- --test` |
| Full suite command | `cd src-tauri && cargo bench` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BENCH-01 | walk_commits benchmarks at 100/1k/10k | benchmark | `cd src-tauri && cargo bench --bench bench_graph` | Wave 0 |
| BENCH-02 | list_refs, diff_unstaged, stage_hunk benchmarks | benchmark | `cd src-tauri && cargo bench --bench bench_commands` | Wave 0 |
| BENCH-03 | DEFERRED to Phase 58 | -- | -- | -- |
| BENCH-04 | DEFERRED to Phase 58 | -- | -- | -- |
| BENCH-05 | CI regression detection | integration | `cargo test --benches --no-run --manifest-path src-tauri/Cargo.toml` (compile-check) | Wave 0 |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test --benches --no-run` (compile-check, ~30s)
- **Per wave merge:** `cd src-tauri && cargo bench --bench bench_graph -- --test` (single-iteration smoke test)
- **Phase gate:** Full `cargo bench` run + verify benchmark-action YAML is valid

### Wave 0 Gaps
- [ ] `src-tauri/benches/bench_graph.rs` -- covers BENCH-01 (walk_commits scaling benchmarks)
- [ ] `src-tauri/benches/bench_commands.rs` -- covers BENCH-02 (command inner function benchmarks)
- [ ] `src-tauri/Cargo.toml` -- add criterion dev-dependency + [[bench]] entries
- [ ] `.github/workflows/benchmarks.yml` -- covers BENCH-05 (CI regression detection)
- [ ] `.github/workflows/ci.yml` -- add `cargo test --benches --no-run` compile-check

## Sources

### Primary (HIGH confidence)
- cargo search criterion (local, 2026-03-27) - version 0.8.2 confirmed
- [Criterion.rs Getting Started](https://bheisler.github.io/criterion.rs/book/getting_started.html) - Cargo.toml setup, macro usage
- [Criterion.rs Advanced Configuration](https://bheisler.github.io/criterion.rs/book/user_guide/advanced_configuration.html) - measurement_time, sample_size, BenchmarkGroup API
- [Criterion.rs Benchmarking With Inputs](https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_with_inputs.html) - BenchmarkId, bench_with_input, parameterized benchmarks
- [git2 Repository docs](https://docs.rs/git2/latest/git2/struct.Repository.html) - blob(), treebuilder(), commit() API
- [git2 TreeBuilder docs](https://docs.rs/git2/latest/git2/struct.TreeBuilder.html) - insert(), write() API
- [benchmark-action/github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark) - CI integration, external-data-json-path, alert-threshold, tool: 'cargo'

### Secondary (MEDIUM confidence)
- [Criterion crates.io page](https://crates.io/crates/criterion) - download stats, version history
- Existing project CI workflow (`.github/workflows/ci.yml`) - system dependency list, rust-cache pattern, Gate 1/Gate 2 pattern

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Criterion 0.8.2 version confirmed via cargo search, benchmark-action well-documented
- Architecture: HIGH - Based on actual codebase analysis (function signatures, crate structure, CI patterns)
- Pitfalls: HIGH - Derived from understanding of actual function signatures and git2 behavior
- Discretionary recommendations: MEDIUM - Additional targets based on code analysis, measurement tuning based on general heuristics

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable ecosystem, Criterion unlikely to break)
