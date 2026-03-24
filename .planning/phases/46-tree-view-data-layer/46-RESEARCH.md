# Phase 46: Tree View Data Layer - Research

**Researched:** 2026-03-24
**Domain:** Pure TypeScript data transformation (flat file paths to nested tree)
**Confidence:** HIGH

## Summary

Phase 46 is a pure data-transformation utility with no UI, no backend, and no external dependencies. The `buildTree` function converts a flat `FileStatus[]` array into a nested `TreeNode[]` tree structure with directory-before-file sorting and single-child directory chain compression. This is a well-understood algorithmic problem with zero ambiguity.

The project has a mature vitest testing infrastructure (v4.1.0, 12 test files, 139 tests all passing) with established patterns for pure-function testing including factory helpers, edge case coverage, and co-located test files. The implementation follows the same pattern as `overlay-paths.ts`, `active-lanes.ts`, and `merge-parser.ts` -- pure functions in `src/lib/` with co-located `*.test.ts` files.

**Primary recommendation:** Use an iterative trie-insert algorithm. For each `FileStatus`, split the path by `/`, walk/create directory nodes along the path, and attach the `FileStatus` as a leaf. Then post-process to compress single-child directory chains and apply sorting. This is the simplest correct approach and matches how VS Code and other editors build their file trees.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** `buildTree` accepts `FileStatus[]` (from `src/lib/types.ts`) -- carries `path`, `status`, and `is_binary` through to leaf nodes so Phase 47 can render file rows directly from tree leaves.
- **D-02:** Returns a `TreeNode[]` where each node is either a `directory` (with `name`, `path`, `children: TreeNode[]`) or a `file` (with `name`, `path`, `file: FileStatus`). Use a discriminated union with a `type` field (`'directory' | 'file'`).
- **D-03:** The `path` field on directory nodes stores the full relative path prefix (e.g. `src/lib/`) for use by Phase 48's directory staging feature (TREE-08).
- **D-04:** Compress chains where a directory has exactly one child that is also a directory. The compressed node's `name` becomes the joined path segments (e.g. `src/lib` instead of `src > lib`). This matches VS Code / GitKraken behavior and satisfies TREE-07.
- **D-05:** A directory with one child that is a file is NOT compressed -- only directory-only chains collapse.
- **D-06:** Directories sort before files at every level of the tree.
- **D-07:** Within each group (directories, files), sort alphabetically case-insensitive using `localeCompare`.
- **D-08:** Utility lives at `src/lib/build-tree.ts` with tests at `src/lib/build-tree.test.ts`. Follows the established pattern of pure functions in `src/lib/` with co-located tests.

### Claude's Discretion
- Internal algorithm for building the tree (trie-based, recursive insert, or iterative)
- Whether to export helper types from `build-tree.ts` or from `types.ts`
- Test fixture structure and helper factories

### Deferred Ideas (OUT OF SCOPE)
None -- analysis stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TREE-07 | Single-child directory paths are compressed (e.g. src/lib/ instead of src > lib) | Decisions D-04 and D-05 define exact compression rules. Algorithm section below covers implementation. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| TypeScript | ~5.6.2 | Language (strict mode) | Already configured in project |
| vitest | 4.1.0 | Test runner | Already configured, 139 tests passing |

### Supporting
No additional libraries needed. This is a pure algorithmic utility with zero external dependencies.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom buildTree | npm tree-building library | Overkill -- the algorithm is ~50 lines, no reason to add a dependency |

## Architecture Patterns

### Recommended Project Structure
```
src/lib/
├── build-tree.ts        # buildTree() + TreeNode types
├── build-tree.test.ts   # Co-located vitest tests
└── types.ts             # FileStatus (input type, already exists)
```

### Pattern 1: Discriminated Union for TreeNode

**What:** Use a `type` discriminant field to distinguish directory nodes from file (leaf) nodes.
**When to use:** Whenever a node can be one of two distinct shapes with different properties.
**Why:** TypeScript's control flow narrowing works automatically with discriminated unions. Downstream consumers get type-safe access after a simple `if (node.type === 'directory')` check.

```typescript
// Discriminated union -- matches project convention (EdgeType, DiffOrigin, etc.)
export type TreeNode = DirectoryNode | FileNode;

export interface DirectoryNode {
  type: 'directory';
  name: string;       // Display name (may be compressed: "src/lib")
  path: string;       // Full relative path prefix ("src/lib/")
  children: TreeNode[];
}

export interface FileNode {
  type: 'file';
  name: string;       // Filename only ("App.svelte")
  path: string;       // Full relative path ("src/App.svelte")
  file: FileStatus;   // Original FileStatus for downstream rendering
}
```

### Pattern 2: Trie-Insert then Post-Process

**What:** Build the tree in two phases: (1) insert all paths into an intermediate trie structure, (2) compress and sort into the final `TreeNode[]`.
**When to use:** When you need to transform flat paths into a tree with post-processing (compression, sorting).
**Why it works:** Separating construction from transformation keeps each step simple and testable.

**Algorithm outline:**
1. For each `FileStatus`, split `path` by `/` into segments.
2. Walk/create intermediate directory nodes along the segments.
3. At the final segment, attach the file as a leaf.
4. Post-process: recursively compress single-child directory chains.
5. Post-process: recursively sort children (directories first, then alphabetical).

**Intermediate structure vs direct TreeNode:** Using an intermediate `Map<string, IntermediateNode>` for children during construction avoids O(n) scans when inserting. Convert to `TreeNode[]` arrays during the final pass.

### Pattern 3: Export Types from Source File

**What:** Export `TreeNode`, `DirectoryNode`, and `FileNode` from `build-tree.ts` rather than `types.ts`.
**Why:** These types are specific to the tree utility and only consumed by the tree view UI. Keeping them co-located with the function reduces coupling. If Phase 47+ needs them broadly, they can be re-exported from a barrel file later.

### Anti-Patterns to Avoid
- **Sorting during insertion:** Sort only once at the end, not on every insert. Sorting during insertion is O(n^2 log n) vs O(n log n) at the end.
- **String concatenation for path building:** Use array segments and `join('/')` to avoid separator bugs.
- **Mutating input array:** `buildTree` must be pure -- never modify the input `FileStatus[]`.
- **Forgetting trailing slash on directory paths:** D-03 requires directory `path` to include the trailing prefix (e.g. `src/lib/`). This is important for Phase 48's directory staging, which will pass the path to `git2` to stage all files under that prefix.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| N/A | N/A | N/A | This IS the hand-rolled utility -- it's the deliverable |

**Key insight:** The algorithm is simple enough (~60-80 lines) that a library would add unnecessary complexity. The value is in the type contract and edge case testing, not algorithmic sophistication.

## Common Pitfalls

### Pitfall 1: Path Separator Assumptions
**What goes wrong:** Code assumes `/` but some systems or git implementations might use `\`.
**Why it happens:** Windows paths use backslashes.
**How to avoid:** Git status paths from git2/libgit2 always use forward slashes (`/`), even on Windows. The `FileStatus.path` values in this project come from git2 via Tauri, so they are always `/`-separated. No normalization needed, but document this assumption.
**Warning signs:** Test failures on Windows (not currently a target, but worth noting).

### Pitfall 2: Compression Consuming Directory with File Child
**What goes wrong:** A directory with exactly one child that is a file gets compressed, hiding the directory level.
**Why it happens:** Compression logic checks `children.length === 1` without checking child type.
**How to avoid:** D-05 explicitly says only compress when the single child is also a directory. The compression condition must be: `children.length === 1 && children[0].type === 'directory'`.
**Warning signs:** Test case `["src/only-file.ts"]` should show `src/ > only-file.ts`, not just `src/only-file.ts` as a flat file.

### Pitfall 3: Empty Input
**What goes wrong:** Function throws on empty array.
**Why it happens:** Missing base case.
**How to avoid:** Return `[]` immediately for empty input. Trivial but must be tested.
**Warning signs:** Crash when staging panel has no files.

### Pitfall 4: Path with Only Directory Components
**What goes wrong:** A path like `src/lib/` (trailing slash, no filename) creates a directory node with an empty-string child.
**Why it happens:** `"src/lib/".split("/")` produces `["src", "lib", ""]`.
**How to avoid:** Filter empty segments from the split result. In practice, `FileStatus.path` from git2 never has trailing slashes (it always points to a file), so this is defensive only.
**Warning signs:** Ghost empty-name nodes in the tree.

### Pitfall 5: Directory Path Field Missing Trailing Context
**What goes wrong:** Phase 48 uses directory `path` to stage all files under a prefix, but the path is stored without proper context (e.g. `src/lib` instead of `src/lib/`).
**Why it happens:** Confusion about whether `path` includes trailing `/`.
**How to avoid:** Per D-03, directory `path` stores the full relative path prefix. For consistency, store it WITHOUT trailing slash (e.g. `src/lib`) since all child paths will start with `src/lib/`. Phase 48 can append `/` if needed for prefix matching. Alternatively, store WITH trailing slash. The key is to be consistent -- pick one and test it.
**Warning signs:** Directory staging misses files or includes wrong files.

### Pitfall 6: Unicode and Special Characters in Filenames
**What goes wrong:** `localeCompare` behaves differently across locales for non-ASCII characters.
**Why it happens:** Default `localeCompare()` uses the system locale.
**How to avoid:** Use `localeCompare(other, undefined, { sensitivity: 'base' })` or `localeCompare(other, 'en', { sensitivity: 'base' })` for consistent cross-platform behavior. The `sensitivity: 'base'` option makes it case-insensitive while handling accented characters correctly.
**Warning signs:** Sort order differs between dev machine and CI.

## Code Examples

### Type Definitions

```typescript
// src/lib/build-tree.ts
import type { FileStatus } from './types.js';

export interface DirectoryNode {
  type: 'directory';
  name: string;
  path: string;
  children: TreeNode[];
}

export interface FileNode {
  type: 'file';
  name: string;
  path: string;
  file: FileStatus;
}

export type TreeNode = DirectoryNode | FileNode;
```

### Core Algorithm Sketch

```typescript
export function buildTree(files: FileStatus[]): TreeNode[] {
  if (files.length === 0) return [];

  // Phase 1: Build intermediate trie
  // Use Map<string, ...> for O(1) child lookup during insertion
  interface IntermediateDir {
    children: Map<string, IntermediateDir>;
    files: FileStatus[];
    path: string; // accumulated path prefix
  }

  const root: IntermediateDir = { children: new Map(), files: [], path: '' };

  for (const file of files) {
    const segments = file.path.split('/');
    let current = root;
    // Walk directory segments (all but last)
    for (let i = 0; i < segments.length - 1; i++) {
      const seg = segments[i];
      if (!current.children.has(seg)) {
        const childPath = segments.slice(0, i + 1).join('/');
        current.children.set(seg, { children: new Map(), files: [], path: childPath });
      }
      current = current.children.get(seg)!;
    }
    // Last segment is the file
    current.files.push(file);
  }

  // Phase 2: Convert to TreeNode[], compress, sort
  function convert(dir: IntermediateDir): TreeNode[] {
    const result: TreeNode[] = [];

    // Convert subdirectories
    for (const [name, child] of dir.children) {
      let dirNode: DirectoryNode = {
        type: 'directory',
        name,
        path: child.path,
        children: convert(child),
      };
      // Compress single-child directory chains (D-04, D-05)
      dirNode = compress(dirNode);
      result.push(dirNode);
    }

    // Convert files
    for (const file of dir.files) {
      const name = file.path.split('/').pop()!;
      result.push({ type: 'file', name, path: file.path, file });
    }

    // Sort: directories first, then alphabetical case-insensitive (D-06, D-07)
    return sortNodes(result);
  }

  return convert(root);
}

function compress(node: DirectoryNode): DirectoryNode {
  // Only compress when single child is also a directory
  while (
    node.children.length === 1 &&
    node.children[0].type === 'directory'
  ) {
    const child = node.children[0] as DirectoryNode;
    node = {
      type: 'directory',
      name: node.name + '/' + child.name,
      path: child.path,
      children: child.children,
    };
  }
  return node;
}

function sortNodes(nodes: TreeNode[]): TreeNode[] {
  return nodes.sort((a, b) => {
    // Directories before files
    if (a.type !== b.type) return a.type === 'directory' ? -1 : 1;
    // Alphabetical case-insensitive within same type
    return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
  });
}
```

### Test Factory Pattern (following project conventions)

```typescript
// src/lib/build-tree.test.ts
import { describe, it, expect } from 'vitest';
import { buildTree } from './build-tree.js';
import type { FileStatus } from './types.js';
import type { TreeNode, DirectoryNode, FileNode } from './build-tree.js';

/** Factory: minimal FileStatus */
function makeFile(path: string, status: FileStatus['status'] = 'Modified'): FileStatus {
  return { path, status, is_binary: false };
}

/** Helper: extract all file paths from tree (depth-first) */
function collectPaths(nodes: TreeNode[]): string[] {
  const paths: string[] = [];
  for (const node of nodes) {
    if (node.type === 'file') paths.push(node.path);
    else paths.push(...collectPaths(node.children));
  }
  return paths;
}
```

### Key Edge Case Tests to Cover

```typescript
describe('buildTree', () => {
  // Empty input
  it('returns empty array for empty input', () => {
    expect(buildTree([])).toEqual([]);
  });

  // Single file at root
  it('handles single file at root level', () => {
    const result = buildTree([makeFile('README.md')]);
    expect(result).toHaveLength(1);
    expect(result[0].type).toBe('file');
  });

  // Directory compression
  it('compresses single-child directory chains', () => {
    const result = buildTree([makeFile('src/lib/utils/helper.ts')]);
    expect(result).toHaveLength(1);
    expect(result[0].type).toBe('directory');
    expect((result[0] as DirectoryNode).name).toBe('src/lib/utils');
  });

  // Does NOT compress directory with single file child
  it('does not compress directory with single file child', () => {
    const result = buildTree([makeFile('src/index.ts')]);
    expect(result).toHaveLength(1);
    expect(result[0].type).toBe('directory');
    expect((result[0] as DirectoryNode).name).toBe('src');
  });

  // Sort order: directories before files
  it('sorts directories before files', () => {
    const result = buildTree([
      makeFile('zebra.ts'),
      makeFile('src/app.ts'),
    ]);
    expect(result[0].type).toBe('directory');
    expect(result[1].type).toBe('file');
  });

  // Unicode filenames
  it('handles unicode filenames', () => {
    const result = buildTree([
      makeFile('docs/resume.md'),
      makeFile('docs/resume.md'),
    ]);
    expect(result).toBeDefined();
  });

  // Deeply nested paths
  it('compresses deeply nested single-child chains', () => {
    const result = buildTree([makeFile('a/b/c/d/e/f.ts')]);
    expect(result).toHaveLength(1);
    expect((result[0] as DirectoryNode).name).toBe('a/b/c/d/e');
  });
});
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Recursive string parsing | Trie-insert + post-process | Standard practice | Cleaner separation of concerns |
| Enum for node types | String literal union discriminants | TypeScript 4+ | Better tree shaking, matches project conventions |

**Deprecated/outdated:**
- None relevant -- this is a simple data structure transformation, not a library-dependent feature.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest 4.1.0 |
| Config file | `vite.config.ts` (inline `test` block) |
| Quick run command | `bun run test` |
| Full suite command | `bun run test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TREE-07 | Single-child directory paths are compressed | unit | `bunx vitest run src/lib/build-tree.test.ts` | Wave 0 (to be created) |

### Sampling Rate
- **Per task commit:** `bunx vitest run src/lib/build-tree.test.ts`
- **Per wave merge:** `bun run test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/lib/build-tree.ts` -- the utility itself (new file)
- [ ] `src/lib/build-tree.test.ts` -- covers TREE-07 and all edge cases (new file)

*(No framework install needed -- vitest 4.1.0 already configured and working.)*

## Open Questions

1. **Directory `path` trailing slash convention**
   - What we know: D-03 says "full relative path prefix (e.g. `src/lib/`)" with trailing slash in the example.
   - What's unclear: Whether to include the trailing `/` in the stored `path` field.
   - Recommendation: Include trailing slash to match D-03's example literally. Phase 48 can use it directly for prefix matching (`file.path.startsWith(dirNode.path)`).

## Sources

### Primary (HIGH confidence)
- `src/lib/types.ts` -- `FileStatus` interface definition (lines 80-84)
- `src/lib/overlay-paths.test.ts` -- Factory helper pattern, edge case test structure
- `src/lib/active-lanes.test.ts` -- Minimal factory pattern with `Partial<T> & required`
- `src/lib/merge-parser.test.ts` -- Pure function testing pattern
- `vite.config.ts` -- vitest configuration (inline test block)
- `package.json` -- vitest 4.1.0 dependency, `bun run test` script
- `tsconfig.json` -- strict mode, ESNext module, bundler resolution

### Secondary (MEDIUM confidence)
- VS Code source code tree view -- compression behavior reference (widely documented)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, all tooling already in place
- Architecture: HIGH -- discriminated union pattern is established in project; algorithm is textbook trie construction
- Pitfalls: HIGH -- all pitfalls are well-understood edge cases of path manipulation and tree compression

**Research date:** 2026-03-24
**Valid until:** Indefinite (pure algorithm, no external dependency drift)

## Project Constraints (from CLAUDE.md)

- **Test command:** `bun run test` (vitest, `src/**/*.test.ts`)
- **TypeScript strict mode:** All types must be explicit
- **Paths:** `$lib` maps to `src/lib`
- **No inline colors:** Not applicable (no UI in this phase)
- **No positioning hacks:** Not applicable (no UI in this phase)
- **Git operations via git2 only:** Not applicable (no git operations in this phase)
