# Phase 51: Cross-Platform Release Pipeline - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-25
**Phase:** 51-cross-platform-release-pipeline
**Areas discussed:** macOS architecture, portable archives, CI gate dependency, workflow organization
**Mode:** --auto (all areas auto-selected, recommended defaults chosen)

---

## macOS Architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Separate ARM + Intel builds | Build separately on aarch64-apple-darwin and x86_64-apple-darwin runners | ✓ |
| Universal binary | Single fat binary combining both architectures | |

**User's choice:** [auto] Separate ARM and Intel builds
**Notes:** REL-01 explicitly lists "macOS ARM, macOS Intel" as separate targets. Separate builds are simpler and avoid cross-compilation complexity.

---

## Portable Archives

| Option | Description | Selected |
|--------|-------------|----------|
| Post-build tar.gz wrapping | Shell step after tauri-action wraps output into .tar.gz | ✓ |
| tauri-action native output | Rely on tauri-action to produce .tar.gz directly | |
| Manual cargo build + bundle | Skip tauri-action, manual build and archive | |

**User's choice:** [auto] Post-build step wrapping tauri-action output
**Notes:** tauri-action produces installers natively. A simple shell step to create .tar.gz from the output is straightforward and reliable.

---

## CI Gate Dependency

| Option | Description | Selected |
|--------|-------------|----------|
| Independent (no CI gate) | Release workflow runs on tag push without waiting for CI | ✓ |
| Require CI passing | Release workflow depends on CI workflow completing successfully | |

**User's choice:** [auto] Independent (no CI gate)
**Notes:** Tag push implies code is on main and already CI-validated. Adding a dependency would complicate the workflow and slow releases.

---

## Workflow Organization

| Option | Description | Selected |
|--------|-------------|----------|
| Matrix strategy | Single workflow with matrix for platform/arch combinations | ✓ |
| Separate per-platform jobs | Individual jobs for each platform without matrix | |
| Reusable workflow calls | Separate workflow files called from orchestrator | |

**User's choice:** [auto] Matrix strategy with per-platform jobs
**Notes:** Matrix keeps the workflow DRY while each platform builds in parallel. tauri-action supports matrix configuration.

---

## Claude's Discretion

- Exact matrix configuration format and variable naming
- Concurrency controls
- Rust build caching strategy for release workflow
- tauri-action configuration options and version pinning
- Artifact naming convention

## Deferred Ideas

None — discussion stayed within phase scope
