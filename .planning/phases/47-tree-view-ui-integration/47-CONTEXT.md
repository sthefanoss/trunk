# Phase 47: Tree View UI Integration - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire the `buildTree` utility (Phase 46) into all file list contexts — staging panel, commit diffs, and merge editor — with flat/tree toggle, expand/collapse, keyboard navigation, and persisted view mode preference. Covers requirements TREE-01 through TREE-06.

</domain>

<decisions>
## Implementation Decisions

### Toggle placement & scope
- **D-01:** Single global toggle in the staging panel header ("N files changed on main" bar). One icon button (list/tree) controls the view mode for all file list contexts — staging panel sections, commit diff file lists, and merge editor file lists.
- **D-02:** View mode is a single persisted boolean (`tree_view_enabled`) in LazyStore. Toggle once, see tree view everywhere. Restored on app relaunch.
- **D-03:** The toggle applies uniformly — unstaged, staged, conflicted, commit diff file lists, and merge editor file lists all respect the same setting.

### Tree node visual design
- **D-04:** Indent + chevron only. Each nesting level adds ~16px left padding. Directories show a chevron (ChevronRight collapsed / ChevronDown expanded) before the directory name. Files show their existing status icon at the same indentation depth. No folder icons, no connector lines.
- **D-05:** In tree mode, files show filename only (e.g. "store.ts"), not the full relative path. The directory structure provides path context. Flat mode continues showing the full relative path as today.
- **D-06:** Directory rows have the same 26px height as FileRow. Clicking the row or chevron toggles expand/collapse.

### Expand/collapse state
- **D-07:** Expand/collapse state is per-section (unstaged, staged, conflicted each track their own `Set<string>` of expanded directory paths), keyed by the directory's full relative path. This allows independent tree navigation in each section.
- **D-08:** On status refresh (repo-changed event, stage/unstage operations), the expanded set is preserved — only the tree data re-derives from the new `FileStatus[]` array. This satisfies TREE-05 (state survives refreshes).
- **D-09:** When toggling from flat to tree mode, all directories start **collapsed**. User expands to drill in.
- **D-10:** Expand/collapse state is ephemeral (not persisted to disk). Switching tabs and back preserves it via keep-alive (Phase 45 D-08), but closing and reopening the app resets it.

### Keyboard navigation
- **D-11:** VS Code-style arrow key navigation. Up/Down moves focus between visible rows. Right on collapsed dir: expand. Right on expanded dir: move to first child. Left on file: jump to parent dir. Left on expanded dir: collapse. Left on collapsed dir: jump to parent. Enter on file: select (show diff). Enter on dir: no-op.
- **D-12:** Keyboard focus shows a visible highlight on the focused row (subtle background, similar to hover state). Active in both flat and tree modes.
- **D-13:** Focus is tracked per-section as an index into the visible (flattened) row list. Focus resets to 0 when the file list changes (status refresh), unless the previously focused path still exists.

### Claude's Discretion
- How to structure the TreeView component (single component wrapping FileRow, or new DirectoryRow + TreeList)
- Whether to create a shared `useTreeState` hook/factory or inline the state in each consumer
- How to render tree in DiffPanel and MergeEditor (these show `FileDiff[]` not `FileStatus[]` — may need a lightweight adapter to map FileDiff paths into buildTree-compatible input)
- Animation on expand/collapse (subtle slide or instant)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Tree data layer (Phase 46 output)
- `src/lib/build-tree.ts` — `buildTree()` function, `TreeNode`, `DirectoryNode`, `FileNode` types
- `src/lib/build-tree.test.ts` — Test patterns and edge cases covered

### Primary integration points
- `src/components/StagingPanel.svelte` — Main consumer; renders `FileStatus[]` via `{#each}` → `FileRow`. Three sections: unstaged, staged, conflicted. Each needs tree rendering.
- `src/components/FileRow.svelte` — Current flat file row component (26px height, status icon, path, hover action button). Tree mode reuses this for leaf nodes.
- `src/components/DiffPanel.svelte` — Commit diff file list; has its own `collapsedFiles` state and `FileDiff[]` rendering. Needs tree mode for its file list header area.
- `src/components/MergeEditor.svelte` — Merge conflict file list; renders file paths for conflict navigation.

### Persistence
- `src/lib/store.ts` — LazyStore pattern for persisting `tree_view_enabled` preference (add alongside existing zoom/pane/column persistence)

### Type contracts
- `src/lib/types.ts` — `FileStatus`, `FileStatusType`, `WorkingTreeStatus`, `FileDiff` interfaces
- `src/lib/tab-types.ts` — `TabInfo` (if tree state needs tab-level scoping)

### Requirements
- `.planning/REQUIREMENTS.md` — TREE-01 through TREE-06 acceptance criteria

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`buildTree()`** (`src/lib/build-tree.ts`): Ready to use — accepts `FileStatus[]`, returns `TreeNode[]` with path compression and sorting. Phase 46 output.
- **`FileRow.svelte`**: 26px-height file row with status icon, path text, hover action button. Can be reused directly for leaf (file) nodes in tree mode — just pass filename instead of full path.
- **ChevronDown / ChevronRight** from `@lucide/svelte`: Already imported in StagingPanel for section headers. Same icons work for directory expand/collapse.
- **LazyStore** (`src/lib/store.ts`): Established persistence pattern — add `tree_view_enabled` key alongside existing preferences.

### Established Patterns
- **Svelte 5 runes**: All state uses `$state()`, `$derived()`, `$effect()`. Tree state (expanded dirs, focus index) should follow this.
- **CSS custom properties**: All colors via `var(--color-*)`. Tree indentation and focus highlight must use theme variables.
- **`{#each ... as f (f.path)}`**: Keyed iteration for file lists. Tree mode needs keyed iteration by `node.path` for stable DOM updates.
- **Section expand/collapse**: StagingPanel already has `unstaged_expanded`, `staged_expanded`, `conflicted_expanded` booleans with chevron toggle — directory expand/collapse follows the same UX pattern.

### Integration Points
- **StagingPanel sections**: Each `{#each status?.unstaged ?? [] as f}` block needs to conditionally render tree or flat based on the global toggle.
- **DiffPanel file list**: Currently renders `fileDiffs` with its own collapse mechanism. Tree mode wraps the file list portion only.
- **MergeEditor**: Renders a single file at a time but has a file selector. Tree mode applies to the file selector if it lists multiple conflicted files.
- **Keyboard handler**: StagingPanel has no keyboard handlers currently. DiffPanel has `[`/`]` for hunk navigation. Tree keyboard nav is additive.

</code_context>

<specifics>
## Specific Ideas

- All directories start collapsed when switching from flat to tree (user drills into what they need)
- Keyboard focus with visible highlight works in both flat and tree modes — not tree-only
- VS Code explorer is the mental model for tree interaction (indent+chevron, arrow key behavior)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 47-tree-view-ui-integration*
*Context gathered: 2026-03-24*
