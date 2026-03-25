# Phase 46: Tree View Data Layer - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-24
**Phase:** 46-tree-view-data-layer
**Areas discussed:** Input/output contract, Path compression rules, Sort order, File location
**Mode:** auto (all decisions auto-selected with recommended defaults)

---

## Input/Output Contract

| Option | Description | Selected |
|--------|-------------|----------|
| Accept FileStatus[], return TreeNode[] with discriminated union | Leaves carry full FileStatus for Phase 47 rendering; directories carry path/name/children | ✓ |
| Accept string[] paths only, return minimal tree | Would require re-lookup of FileStatus in Phase 47 | |
| Accept generic T[] with path accessor | Over-engineered for the use case | |

**User's choice:** [auto] Accept FileStatus[], return discriminated union TreeNode[]
**Notes:** Recommended because Phase 47 needs FileStatus on leaves for FileRow rendering

## Path Compression Rules

| Option | Description | Selected |
|--------|-------------|----------|
| Compress directory-only chains (VS Code style) | Single-child dirs that contain only another dir get merged names | ✓ |
| Compress any single-child node including files | Would hide single-file directories | |
| No compression (full tree) | Does not satisfy TREE-07 | |

**User's choice:** [auto] Compress directory-only chains
**Notes:** Matches TREE-07 requirement and industry standard (VS Code, GitKraken)

## Sort Order

| Option | Description | Selected |
|--------|-------------|----------|
| Dirs before files, case-insensitive alphabetical | Standard Git GUI convention | ✓ |
| Dirs before files, case-sensitive | Unusual, may confuse users | |
| Mixed (no dir/file separation) | Non-standard for file trees | |

**User's choice:** [auto] Dirs before files, case-insensitive alphabetical
**Notes:** Matches every major Git GUI (VS Code, GitKraken, Fork, Tower)

## File Location

| Option | Description | Selected |
|--------|-------------|----------|
| src/lib/build-tree.ts + src/lib/build-tree.test.ts | Follows existing pure-function pattern in src/lib/ | ✓ |
| src/lib/tree/build-tree.ts | New subdirectory, unnecessary for single utility | |
| src/utils/build-tree.ts | No existing utils/ directory | |

**User's choice:** [auto] src/lib/build-tree.ts
**Notes:** Consistent with overlay-paths.ts, active-lanes.ts, merge-parser.ts

## Claude's Discretion

- Internal algorithm choice (trie-based, recursive, iterative)
- Type export location (build-tree.ts vs types.ts)
- Test fixture structure

## Deferred Ideas

None — analysis stayed within phase scope
