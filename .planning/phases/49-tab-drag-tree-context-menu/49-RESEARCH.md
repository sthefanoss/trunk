# Phase 49: Tab Drag Reorder & Tree Context Menu - Research

**Researched:** 2026-03-25
**Domain:** SortableJS drag-and-drop, Tauri native context menus, Svelte 5 integration
**Confidence:** HIGH

## Summary

This phase adds two independent features: (1) drag-to-reorder tabs using SortableJS, and (2) right-click context menus on directory nodes in the tree view for bulk operations. Both features build on established patterns already in the codebase.

SortableJS is already installed and used in RebaseEditor.svelte with a proven Svelte 5 integration pattern (`$effect` for lifecycle, `{#key}` block to prevent Svelte/SortableJS DOM conflicts). The tab bar is a simple flex container with `flex-shrink: 0` items -- a straightforward SortableJS target. The `direction: 'horizontal'` option handles horizontal reorder. The `persistTabs()` function already exists and is called after every tab mutation, so calling it from the SortableJS `onEnd` handler completes persistence.

The Tauri native menu API (`@tauri-apps/api/menu`) is used extensively across the app (11+ call sites) with a consistent pattern: dynamic import, `Menu.new()` with items array, `menu.popup()`. Directory bulk operations (stage, unstage) already exist in StagingPanel. Discard for individual files exists with a confirmation dialog pattern via `@tauri-apps/plugin-dialog`. The new work is: (a) adding an `oncontextmenu` prop to DirectoryRow/TreeFileList, (b) creating directory-level context menu handlers in StagingPanel for each section, and (c) implementing directory-scoped discard and resolve/unresolve operations.

**Primary recommendation:** Follow the existing RebaseEditor SortableJS pattern for tabs (with `{#key}` reconciliation), and follow the existing file-level context menu pattern for directory menus. No new libraries needed.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use SortableJS for tab drag-and-drop (already a project dependency, used in RebaseEditor). Tabs animate/swap in place as you drag.
- **D-02:** Auto-scroll when dragging near the left/right edge of the tab bar, for overflow scenarios with many open tabs.
- **D-03:** The + (new tab) button is excluded from SortableJS -- always pinned at the far right of the tab bar.
- **D-04:** After reorder, call existing `persistTabs()` (which calls `setOpenTabs()`) to save the new tab order. Order persists across app relaunch.
- **D-05:** Context menu appears on directory nodes only. Individual files keep their existing per-file context menus from StagingPanel.
- **D-06:** Uses native Tauri menus (`@tauri-apps/api/menu`), consistent with all other context menus in the app (graph rows, branches, tabs, file rows).
- **D-07:** Unstaged section menu: "Stage All" + "Discard All". Stage All stages all files in the directory recursively. Discard All discards all changes in the directory.
- **D-08:** "Discard All" always shows a confirmation dialog before executing. Destructive operation -- consistent with existing single-file discard confirmation.
- **D-09:** Staged section menu: "Unstage All" only. Unstages all files in the directory recursively.
- **D-10:** Conflicted section menu: "Resolve All" + "Unresolve All". Resolve All marks all conflicted files in the directory as resolved (stages them). Unresolve All marks resolved files back as conflicted.

### Claude's Discretion
- Drag styling (opacity, cursor, drop indicator) -- use existing CSS custom properties, keep it subtle and consistent with the app aesthetic
- SortableJS configuration details (animation duration, ghost class, handle vs full-tab drag)
- How to wire the `oncontextmenu` handler through TreeFileList to DirectoryRow (callback prop pattern)
- Whether directory bulk operations use sequential or parallel IPC calls (Promise.all vs sequential)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TAB-11 | User can drag tabs to reorder them, and the new order persists across app relaunch | SortableJS already installed and proven in RebaseEditor; `persistTabs()` exists for persistence; `{#key}` pattern handles Svelte reconciliation |
| TREE-11 | User can right-click a directory in the tree view to access bulk actions (Stage All, Unstage All, Discard All, and resolve/unresolve for conflicted files) | Tauri menu API used in 11+ locations; `stageDirectory`/`unstageDirectory` already exist; `handleDiscardFile` + confirmation dialog pattern established; DirectoryRow needs `oncontextmenu` prop addition |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **Never inline colors** -- always use CSS custom properties from the theme
- **Never fight layout with positioning hacks** -- use grid/flexbox so elements flow naturally
- **All git operations go through git2 crate** -- no shelling out
- **Stack:** Svelte 5 runes (`$state`, `$derived`), Vite 6, TypeScript 5.6 strict, Tailwind CSS 4
- **Frontend-Backend:** `invoke("command_name", args)` calls Rust `#[tauri::command]` fns
- **CSS custom properties must use semantic names** (e.g. `--color-success`), not purpose-specific or color-named

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sortablejs | ^1.15.7 | Tab drag-and-drop reorder | Already installed and used in RebaseEditor |
| @tauri-apps/api | 2.10.1 | Native context menus via `Menu`, `MenuItem`, `popup()` | Already used in 11+ locations across the app |
| @tauri-apps/plugin-dialog | 2.6.0 | Confirmation dialog for destructive discard | Already used for discard confirmations |
| @types/sortablejs | ^1.15.9 | TypeScript types for SortableJS | Already installed |

### Supporting
No additional libraries needed. Everything required is already in the project.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| SortableJS | HTML5 Drag API directly | SortableJS handles cross-platform edge cases, animation, auto-scroll; already proven in the codebase |
| Native Tauri menus | Custom HTML context menu | Would break consistency with all other context menus in the app; violates D-06 |

## Architecture Patterns

### Pattern 1: SortableJS + Svelte 5 Reconciliation (from RebaseEditor)
**What:** Wrap sortable container in `{#key stateArray}` to force DOM recreation after SortableJS reorders elements. SortableJS manipulates the DOM directly, which conflicts with Svelte's keyed `{#each}` block. The `{#key}` block makes Svelte discard and re-create the DOM when the state array changes.
**When to use:** Any time SortableJS reorders items managed by Svelte reactive state.
**Example:**
```typescript
// Source: src/components/RebaseEditor.svelte lines 170-197, 458-460
// In <script>:
let tabBarEl: HTMLDivElement | undefined = $state();

$effect(() => {
  if (!tabBarEl) return;
  const sortable = Sortable.create(tabBarEl, {
    animation: 150,
    direction: 'horizontal',
    forceFallback: true,
    ghostClass: 'tab-ghost',
    dragClass: 'tab-drag',
    filter: '.new-tab-btn',   // D-03: exclude + button
    preventOnFilter: false,
    scroll: true,             // D-02: auto-scroll
    scrollSensitivity: 50,
    onEnd: (e) => {
      if (e.oldIndex == null || e.newIndex == null || e.oldIndex === e.newIndex) return;
      // Reorder the tabs array, then persist
      const updated = [...tabs];
      const [moved] = updated.splice(e.oldIndex, 1);
      updated.splice(e.newIndex, 0, moved);
      onreorder(updated); // App.svelte updates tabs array + calls persistTabs()
    },
  });
  return () => sortable.destroy();
});

// In template:
{#key tabs}
<div class="tab-bar" bind:this={tabBarEl}>
  {#each tabs as tab (tab.id)}
    <!-- tab items -->
  {/each}
  <button class="new-tab-btn">+</button>
</div>
{/key}
```

**Critical detail:** The `{#key tabs}` block wraps the entire sortable container. When `onEnd` fires and updates the `tabs` array, Svelte destroys the old DOM and creates new DOM reflecting the new order. The `$effect` then re-creates the SortableJS instance on the new DOM. This is the proven pattern from RebaseEditor.

### Pattern 2: Context Menu via Callback Prop Chain
**What:** StagingPanel creates the context menu handler, passes it through TreeFileList as a prop, TreeFileList passes it to DirectoryRow which binds it to the `oncontextmenu` DOM event.
**When to use:** Directory context menus in tree view.
**Example:**
```typescript
// Source: existing FileRow oncontextmenu pattern
// DirectoryRow.svelte - add prop:
interface Props {
  // ... existing props
  oncontextmenu?: (e: MouseEvent) => void;
}

// In template:
<div
  role="treeitem"
  oncontextmenu={(e) => { if (oncontextmenu) { e.preventDefault(); oncontextmenu(e); } }}
  // ... rest of attributes
>

// TreeFileList.svelte - add prop:
interface Props {
  // ... existing props
  ondirectorycontextmenu?: (e: MouseEvent, dirPath: string) => void;
}

// In DirectoryRow rendering:
<DirectoryRow
  oncontextmenu={ondirectorycontextmenu
    ? (e) => ondirectorycontextmenu!(e, row.node.path)
    : undefined}
/>

// StagingPanel.svelte - create handler + pass through:
<TreeFileList
  ondirectorycontextmenu={(e, dirPath) => showUnstagedDirContextMenu(e, dirPath)}
/>
```

### Pattern 3: Native Tauri Menu with Dynamic Import
**What:** Dynamically import `@tauri-apps/api/menu` and `@tauri-apps/plugin-dialog` at call time, build menu items, call `.popup()`.
**When to use:** All context menus in the app.
**Example:**
```typescript
// Source: StagingPanel.svelte lines 161-178 (showUnstagedContextMenu)
async function showUnstagedDirContextMenu(e: MouseEvent, dirPath: string) {
  const { Menu, MenuItem } = await import('@tauri-apps/api/menu');
  const files = (status?.unstaged ?? []).filter(
    f => f.path.startsWith(dirPath + '/') || f.path === dirPath
  );
  if (files.length === 0) return;

  const menu = await Menu.new({
    items: [
      await MenuItem.new({
        text: `Stage All (${files.length} files)`,
        action: () => { stageDirectory(dirPath); },
      }),
      await MenuItem.new({
        text: `Discard All (${files.length} files)`,
        action: () => { handleDiscardDirectory(dirPath).catch(() => {}); },
      }),
    ],
  });
  await menu.popup();
}
```

### Pattern 4: Directory Prefix Matching for Bulk Operations
**What:** Filter the flat file list by path prefix to find all files in a directory recursively. Already used by `stageDirectory` and `unstageDirectory`.
**When to use:** Any directory-scoped bulk operation.
**Example:**
```typescript
// Source: StagingPanel.svelte lines 107-118 (stageDirectory)
const directMatches = (status?.unstaged ?? []).filter(
  f => f.path.startsWith(dirPath + '/') || f.path === dirPath
);
```
This pattern handles both files directly in the directory and files in subdirectories. The `|| f.path === dirPath` handles the edge case of a file whose path exactly matches the directory path (unusual but safe).

### Anti-Patterns to Avoid
- **Modifying Svelte state inside SortableJS onEnd without {#key}:** SortableJS manipulates the DOM directly, and Svelte's diffing algorithm will fight it. Always use `{#key}` to force DOM recreation.
- **Using static imports for menu API in StagingPanel:** The existing pattern uses dynamic imports to keep the menu API tree-shakeable and lazy-loaded. Follow the `await import(...)` pattern.
- **Building custom HTML context menus:** Violates D-06 and breaks consistency with the rest of the app that uses native Tauri menus.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Drag-and-drop reorder | Custom HTML5 DnD implementation | SortableJS `Sortable.create()` | Handles cross-platform, touch, animation, auto-scroll, ghost elements |
| Native context menus | Custom HTML/CSS dropdown menus | `@tauri-apps/api/menu` Menu/MenuItem | Native look and feel, consistent with all other app menus |
| Confirmation dialogs | Custom modal component | `@tauri-apps/plugin-dialog` `ask()` | Native OS dialog, consistent with existing discard confirmations |
| Directory file collection | Tree traversal on TreeNode[] | Flat prefix matching on FileStatus[] | Already proven pattern in stageDirectory/unstageDirectory |

**Key insight:** Every building block for this phase already exists in the codebase. The work is wiring them together, not creating new patterns.

## Common Pitfalls

### Pitfall 1: SortableJS DOM Manipulation vs Svelte Reactivity
**What goes wrong:** SortableJS reorders DOM elements directly. If Svelte's `{#each}` block tries to reconcile the DOM afterward, it can produce duplicate or missing elements.
**Why it happens:** Svelte tracks DOM order via keyed `{#each}` blocks. SortableJS bypasses this entirely.
**How to avoid:** Wrap the sortable container in `{#key items}` (exactly as RebaseEditor does). When `onEnd` fires and updates state, the `{#key}` block destroys old DOM and Svelte creates fresh DOM from the updated array.
**Warning signs:** Duplicate tabs appearing, tabs disappearing after drag, or tabs snapping back to original position.

### Pitfall 2: SortableJS Filtering the New Tab Button
**What goes wrong:** The "+" new tab button gets dragged along with tab items, or gets repositioned during drag.
**Why it happens:** SortableJS treats all direct children of the container as draggable items by default.
**How to avoid:** Use `filter: '.new-tab-btn'` in SortableJS options. This tells SortableJS to ignore elements matching that selector. Combined with `preventOnFilter: false` to allow click events on filtered elements.
**Warning signs:** The + button moves when dragging tabs, or clicking + stops working.

### Pitfall 3: Tab Bar Auto-Scroll Direction
**What goes wrong:** Auto-scroll doesn't work or scrolls vertically instead of horizontally.
**Why it happens:** The tab bar has `overflow-x: auto` and `overflow-y: hidden`. SortableJS needs `scroll: true` and correct `scrollSensitivity` to detect the scroll container.
**How to avoid:** Set `scroll: true` and `scrollSensitivity: 50` (or similar) in SortableJS options. The `direction: 'horizontal'` option also helps SortableJS understand the layout.
**Warning signs:** Dragging a tab to the edge of a scrolled tab bar doesn't scroll to reveal hidden tabs.

### Pitfall 4: Discard Directory Without Confirmation
**What goes wrong:** Accidentally discarding all changes in a directory with no way to undo.
**Why it happens:** Forgetting to add the confirmation dialog, or the dialog not awaiting the user's response.
**How to avoid:** Always use `const { ask } = await import('@tauri-apps/plugin-dialog')` and `await ask(...)` before calling `discard_file` for each file. Follow the exact pattern from `handleDiscardFile` (line 143-159 in StagingPanel).
**Warning signs:** Changes disappearing without a confirmation prompt.

### Pitfall 5: Conflicted Section Missing ondirectoryaction
**What goes wrong:** The conflicted section TreeFileList has no `ondirectoryaction` prop, so directory nodes show no hover action button.
**Why it happens:** The conflicted section was originally designed without directory-level actions (unlike unstaged/staged).
**How to avoid:** For the context menu feature, this is fine -- context menus don't require `ondirectoryaction`. But if a hover action button is also wanted for conflicted directories, the prop must be added.
**Warning signs:** Directory hover buttons appear in unstaged/staged but not conflicted.

### Pitfall 6: Resolve All vs Unresolve All -- Different Mechanisms
**What goes wrong:** Implementing Resolve All and Unresolve All the same way.
**Why it happens:** They seem symmetrical but use different backend commands.
**How to avoid:** "Resolve All" stages all conflicted files via `stage_file` (which marks them resolved). "Unresolve All" needs to unstage previously resolved files, using `unstage_file`. The existing `markAllResolved` function (StagingPanel line 242-247) stages all conflicted files via `stage_file` -- the directory version filters by prefix first.
**Warning signs:** Unresolve All not actually un-staging files, or resolve not marking files as resolved.

## Code Examples

### SortableJS Tab Reorder Integration
```typescript
// TabBar.svelte -- add SortableJS to existing component
import Sortable from 'sortablejs';

interface Props {
  tabs: TabInfo[];
  activeTabId: string;
  onactivate: (tabId: string) => void;
  onclose: (tabId: string, force: boolean) => void;
  onnew: () => void;
  oncontextmenu: (tabId: string, event: MouseEvent) => void;
  onauxclose: (tabId: string) => void;
  onreorder: (newTabs: TabInfo[]) => void;  // NEW
}

let tabBarEl: HTMLDivElement;

$effect(() => {
  if (!tabBarEl) return;
  const sortable = Sortable.create(tabBarEl, {
    animation: 150,
    direction: 'horizontal',
    forceFallback: true,
    ghostClass: 'tab-ghost',
    chosenClass: 'tab-chosen',
    dragClass: 'tab-drag',
    filter: '.new-tab-btn',
    preventOnFilter: false,
    scroll: true,
    scrollSensitivity: 50,
    onEnd: (e) => {
      if (e.oldIndex == null || e.newIndex == null || e.oldIndex === e.newIndex) return;
      const updated = [...tabs];
      const [moved] = updated.splice(e.oldIndex, 1);
      updated.splice(e.newIndex, 0, moved);
      onreorder(updated);
    },
  });
  return () => sortable.destroy();
});
```

### Directory Context Menu Handler
```typescript
// StagingPanel.svelte -- new handler for unstaged directory context menu
async function showUnstagedDirContextMenu(e: MouseEvent, dirPath: string) {
  const { Menu, MenuItem } = await import('@tauri-apps/api/menu');
  const files = (status?.unstaged ?? []).filter(
    f => f.path.startsWith(dirPath + '/') || f.path === dirPath
  );
  if (files.length === 0) return;

  const menu = await Menu.new({
    items: [
      await MenuItem.new({
        text: `Stage All (${files.length})`,
        action: () => { stageDirectory(dirPath); },
      }),
      await MenuItem.new({
        text: `Discard All (${files.length})`,
        action: () => { handleDiscardDirectory(dirPath).catch(() => {}); },
      }),
    ],
  });
  await menu.popup();
}
```

### Directory Discard with Confirmation
```typescript
// StagingPanel.svelte -- discard all files in a directory with confirmation
async function handleDiscardDirectory(dirPath: string) {
  const files = (status?.unstaged ?? []).filter(
    f => f.path.startsWith(dirPath + '/') || f.path === dirPath
  );
  if (files.length === 0) return;

  const { ask } = await import('@tauri-apps/plugin-dialog');
  const confirmed = await ask(
    `Discard all changes in ${dirPath}/ (${files.length} file${files.length === 1 ? '' : 's'})? This cannot be undone.`,
    { title: 'Discard Directory Changes', kind: 'warning' }
  );
  if (!confirmed) return;

  try {
    await Promise.all(files.map(f => safeInvoke('discard_file', { path: repoPath, filePath: f.path })));
    await loadStatus();
    showToast(`Discarded ${files.length} files in ${dirPath}/`, 'success');
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Discard failed', 'error');
  }
}
```

### DirectoryRow with Context Menu
```typescript
// DirectoryRow.svelte -- add oncontextmenu prop
interface Props {
  node: DirectoryNode;
  depth: number;
  expanded: boolean;
  focused: boolean;
  ontoggle: () => void;
  actionLabel?: string;
  onaction?: () => void;
  oncontextmenu?: (e: MouseEvent) => void;  // NEW
}

// In template div:
oncontextmenu={(e) => { if (oncontextmenu) { e.preventDefault(); oncontextmenu(e); } }}
```

### SortableJS CSS Classes (Theme-Aware)
```css
/* TabBar.svelte -- use :global() for SortableJS-applied classes */
:global(.tab-ghost) {
  opacity: 0.4;
}
:global(.tab-chosen) {
  background: var(--color-selected-row);
}
:global(.tab-drag) {
  opacity: 0;
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| svelte-sortablejs wrapper | Direct SortableJS + $effect | Svelte 5 (2024) | Wrappers are mostly Svelte 3/4; direct integration with $effect is cleaner |
| HTML5 Drag API for simple reorder | SortableJS | N/A | SortableJS handles edge cases (touch, auto-scroll, animation) that raw HTML5 API doesn't |

**Deprecated/outdated:**
- svelte-sortablejs packages: Most are built for Svelte 3/4 and don't support runes. Use SortableJS directly.

## Open Questions

1. **Tab drag indicator styling**
   - What we know: RebaseEditor uses `ghostClass`, `chosenClass`, `dragClass` with opacity/background changes
   - What's unclear: Whether the horizontal tab bar needs different visual treatment (e.g., a vertical drop indicator line between tabs)
   - Recommendation: Start with the same RebaseEditor pattern (ghost opacity + chosen background). If the visual feedback is unclear for horizontal layout, add a subtle border indicator.

2. **Conflicted directory operations -- what does "Unresolve All" actually do?**
   - What we know: "Resolve All" stages conflicted files (existing `markAllResolved` calls `stage_file` for each). The context menu version is the same but filtered by directory prefix.
   - What's unclear: "Unresolve All" needs to un-stage previously staged conflicted files. The question is whether `unstage_file` on a previously-conflicted file returns it to the conflicted state or to unstaged.
   - Recommendation: Use `unstage_file` -- in git, unstaging a file that was conflicted returns it to the conflicted state. Test this behavior during implementation.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest (via Vite config) |
| Config file | vite.config.ts (test section) |
| Quick run command | `bun run test` |
| Full suite command | `bun run test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TAB-11 | Tab drag reorder persists order | manual-only | N/A -- requires SortableJS DOM interaction, not unit-testable | N/A |
| TREE-11 | Directory context menu bulk operations | manual-only | N/A -- requires Tauri menu API (native), not available in vitest | N/A |

**Justification for manual-only:** Both features are primarily UI interaction (drag-and-drop, native context menus) that require the full Tauri runtime and DOM environment. SortableJS manipulates the DOM directly, and Tauri native menus require the native webview runtime. The underlying logic (array reordering, directory prefix matching) is already tested via existing build-tree and flatten-tree tests.

### Sampling Rate
- **Per task commit:** `bun run test` (ensures no regressions in existing tests)
- **Per wave merge:** `bun run test && bun run check` (full type check + tests)
- **Phase gate:** Full suite green + manual verification of drag reorder and context menu

### Wave 0 Gaps
None -- existing test infrastructure covers the pure-logic components. The new code is primarily UI wiring.

## Sources

### Primary (HIGH confidence)
- `src/components/RebaseEditor.svelte` lines 170-197 -- SortableJS + Svelte 5 integration pattern with `{#key}` block
- `src/components/TabBar.svelte` -- current tab bar structure (149 lines)
- `src/components/StagingPanel.svelte` lines 107-131 -- `stageDirectory`/`unstageDirectory` bulk operations
- `src/components/StagingPanel.svelte` lines 143-220 -- file context menu handlers and discard confirmation pattern
- `src/components/DirectoryRow.svelte` -- current directory row (no context menu prop yet)
- `src/components/TreeFileList.svelte` -- tree rendering with `ondirectoryaction` and `onfilecontextmenu` props
- `src/App.svelte` lines 83-103, 193-203 -- `showTabContextMenu` and `persistTabs` functions

### Secondary (MEDIUM confidence)
- [SortableJS GitHub](https://github.com/SortableJS/Sortable) -- `direction: 'horizontal'`, `scroll`, `filter` options
- [Tauri v2 menu API reference](https://v2.tauri.app/reference/javascript/api/namespacemenu/) -- `Menu.new()`, `MenuItem.new()`, `.popup()`
- [Svelte 5 and SortableJS - DEV Community](https://dev.to/jdgamble555/svelte-5-and-sortablejs-5h6j) -- Svelte 5 runes integration patterns

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already installed and used in the codebase
- Architecture: HIGH -- all patterns already established with working examples in the same codebase
- Pitfalls: HIGH -- identified from direct code analysis of existing SortableJS integration and context menu patterns

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable -- no external dependency changes expected)
