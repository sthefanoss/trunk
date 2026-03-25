# Phase 46: Tree View Data Layer - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Build and test a pure `buildTree` utility that transforms flat file paths into a compressed directory tree structure. Directories sorted before files, single-child directory chains compressed. This is a pure data-transformation utility with no UI and no backend changes. Covers requirement TREE-07.

</domain>

<decisions>
## Implementation Decisions

### Input/output contract
- **D-01:** `buildTree` accepts `FileStatus[]` (from `src/lib/types.ts`) — carries `path`, `status`, and `is_binary` through to leaf nodes so Phase 47 can render file rows directly from tree leaves.
- **D-02:** Returns a `TreeNode[]` where each node is either a `directory` (with `name`, `path`, `children: TreeNode[]`) or a `file` (with `name`, `path`, `file: FileStatus`). Use a discriminated union with a `type` field (`'directory' | 'file'`).
- **D-03:** The `path` field on directory nodes stores the full relative path prefix (e.g. `src/lib/`) for use by Phase 48's directory staging feature (TREE-08).

### Path compression
- **D-04:** Compress chains where a directory has exactly one child that is also a directory. The compressed node's `name` becomes the joined path segments (e.g. `src/lib` instead of `src > lib`). This matches VS Code / GitKraken behavior and satisfies TREE-07.
- **D-05:** A directory with one child that is a file is NOT compressed — only directory-only chains collapse.

### Sort order
- **D-06:** Directories sort before files at every level of the tree.
- **D-07:** Within each group (directories, files), sort alphabetically case-insensitive using `localeCompare`.

### File location
- **D-08:** Utility lives at `src/lib/build-tree.ts` with tests at `src/lib/build-tree.test.ts`. Follows the established pattern of pure functions in `src/lib/` with co-located tests (e.g. `overlay-paths.ts`, `active-lanes.ts`, `merge-parser.ts`).

### Claude's Discretion
- Internal algorithm for building the tree (trie-based, recursive insert, or iterative)
- Whether to export helper types from `build-tree.ts` or from `types.ts`
- Test fixture structure and helper factories

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Type contracts
- `src/lib/types.ts` — `FileStatus` interface (the input type), `FileStatusType` union, `WorkingTreeStatus` (how FileStatus arrays are grouped)

### Existing pure-function test patterns
- `src/lib/overlay-paths.test.ts` — Example of thorough vitest testing with factory helpers for pure functions
- `src/lib/merge-parser.test.ts` — Another pure-function test example
- `src/lib/active-lanes.test.ts` — Edge-case testing pattern

### Integration point (Phase 47 consumer)
- `src/components/FileRow.svelte` — Current flat file renderer; takes `FileStatus` as prop. Tree view will wrap this.
- `src/components/StagingPanel.svelte` — Primary consumer in Phase 47; currently renders `FileStatus[]` as flat lists

### Requirements
- `.planning/REQUIREMENTS.md` — TREE-07 acceptance criteria (path compression)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`FileStatus` type** (`src/lib/types.ts`): `{ path: string; status: FileStatusType; is_binary: boolean }` — the input data. File paths are relative to repo root, use `/` as separator.
- **vitest infrastructure**: Already configured (`bun run test`), used extensively in `src/lib/` for pure function testing.
- **Factory pattern in tests**: `overlay-paths.test.ts` uses `makeConn()` and `makeNode()` factory helpers — same pattern works for tree test fixtures.

### Established Patterns
- **Pure functions in `src/lib/`**: All graph/overlay utilities are pure functions with no side effects. `buildTree` follows this pattern.
- **TypeScript strict mode**: All types must be explicit. Discriminated unions with `type` field are used elsewhere (e.g. `EdgeType`, `DiffOrigin`).
- **Co-located tests**: `*.test.ts` files sit next to their source files in `src/lib/`.

### Integration Points
- **Phase 47** will import `buildTree` and `TreeNode` from `src/lib/build-tree.ts` and render the tree in `StagingPanel.svelte`, `DiffPanel.svelte`, and merge editor contexts.
- **Phase 48** will use directory node `path` field for directory-level staging operations (TREE-08) and `children.length` for count badges (TREE-09).

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — analysis stayed within phase scope

</deferred>

---

*Phase: 46-tree-view-data-layer*
*Context gathered: 2026-03-24*
