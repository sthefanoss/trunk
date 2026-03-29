# Phase 62: UI Refactor & Component Structure - Research

**Researched:** 2026-03-29
**Domain:** Svelte 5 component decomposition, diff viewer architecture, UI controls
**Confidence:** HIGH

## Summary

This phase decomposes the 667-line `DiffPanel.svelte` monolith into focused components: `DiffToolbar` (toolbar with view mode toggle and file actions), `DiffViewer` (view mode dispatcher rendering HunkView/FullFileView/SplitView), and a `DiffLineRenderer` (per-line rendering with gutter, origin symbol, and merged spans). It also adds a two-column line number gutter and a persisted `ViewMode` type.

The refactor is structural only -- existing hunk view behavior must remain pixel-identical. All 378 existing tests pass today. The `DiffPanel.test.ts` tests render `DiffPanel` directly, so the public Props interface cannot change. The new child components are internal implementation details that DiffPanel orchestrates.

**Primary recommendation:** Use a `diff/` subdirectory under `src/components/` (following the `virtual-list/` precedent) to co-locate the new components. Use separate `.svelte` files for DiffToolbar and DiffViewer. Use a `{#snippet}` for DiffLineRenderer since it needs access to parent scope (hunk context, line selection state) and has no independent state.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Three-level split: `DiffToolbar` (view mode toggle, file-level actions, close button), `DiffViewer` (view mode dispatcher -- renders HunkView, FullFileView, or SplitView based on current mode), and a `DiffLineRenderer` snippet/component for individual line rendering (origin symbol, merged spans, line backgrounds).
- **D-02:** `DiffPanel.svelte` becomes a thin shell: owns state (selectedHunkKey, line selection, collapsedFiles), passes props down. All staging operation handlers stay in DiffPanel -- they need access to `repoPath`, `diffKind`, `onhunkaction` from RepoView.
- **D-03:** For this phase, only HunkView is functional (existing behavior). FullFileView and SplitView are stub components that show a placeholder message -- they get implemented in Phases 63 and 64.
- **D-04:** Three view modes: `"hunk"` (default, current behavior), `"full"` (Phase 63), `"split"` (Phase 64). Represented as a TypeScript union type `ViewMode = "hunk" | "full" | "split"`.
- **D-05:** Segmented control in DiffToolbar with three buttons. Active mode has a highlighted background. Persisted to LazyStore via `diff_view_mode` key.
- **D-06:** View mode state lives in DiffPanel (loaded from LazyStore on mount, saved on change). Passed to DiffViewer as prop.
- **D-07:** Two-column gutter showing old line number (left) and new line number (right), positioned before the origin symbol (+/-/space). Context lines show both numbers, Add lines show only new, Delete lines show only old. GitHub/GitKraken style.
- **D-08:** Gutter columns use fixed-width `min-width` based on max line number digits. Color matches `--color-text-muted` to avoid visual competition with diff content.
- **D-09:** Line numbers come from existing `DiffLine.old_lineno` and `DiffLine.new_lineno` fields (already populated by Rust backend -- these are `Option<u32>` / `number | null` in TS).
- **D-10:** Toolbar contains: view mode segmented control (left), filename (center, overflow ellipsis), file-level actions + close button (right). Replaces the current 24px header bar.
- **D-11:** Stage File / Unstage File buttons remain in the toolbar (same position as now -- right side). Hunk-level and line-level action buttons remain inline in the hunk toolbar row (unchanged).
- **D-12:** Context lines dropdown, whitespace toggle, word wrap, show invisibles are NOT added in this phase -- they belong to Phase 63.

### Claude's Discretion
- Exact file organization (separate files vs `DiffPanel/` subdirectory)
- Whether DiffLineRenderer is a separate `.svelte` component or a `{#snippet}` within DiffViewer
- CSS class naming for gutter elements
- Transition/animation on view mode switch (if any)

### Deferred Ideas (OUT OF SCOPE)
None -- analysis stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DISP-01 | Line numbers shown in diff gutter (old lineno + new lineno) | DiffLine already has `old_lineno: number \| null` and `new_lineno: number \| null` populated by Rust backend. Gutter columns render conditionally based on DiffOrigin. Fixed-width via `ch` units or `min-width` calculated from max line number. |
| VIEW-01 | User can toggle diff between hunk view, full file view, and split (side-by-side) view | `ViewMode = "hunk" \| "full" \| "split"` union type. Segmented control in DiffToolbar. Persisted to LazyStore `diff_view_mode` key. Only hunk mode is functional; full and split are stubs. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **Never inline colors** -- always use CSS custom properties from the theme
- **Never fight layout with positioning hacks** -- use grid/flexbox so elements flow naturally
- **All git operations through git2 crate** -- no shelling out (not relevant for this frontend phase)
- **Stack:** Svelte 5 (runes: `$state`, `$derived`), TypeScript 5.6 strict, Tailwind CSS 4
- **Paths:** `$lib` -> `src/lib`, commands in `src-tauri/src/commands/`
- **Tests:** `bun run test` (vitest, src/**/*.test.ts)
- **Checks:** `bun run check` (svelte-check)

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Svelte 5 | 5.x | Component framework (runes, snippets) | Project stack |
| TypeScript | 5.6 strict | Type safety | Project stack |
| @tauri-apps/plugin-store | 2.4.2 | LazyStore for preference persistence | Already used for all diff prefs |
| @lucide/svelte | 0.577.0 | Icons for toolbar controls | Already used throughout |
| vitest | (project config) | Unit testing | All 378 tests pass |

### No New Dependencies
This phase requires zero new packages. All building blocks exist in the project already.

## Architecture Patterns

### Recommended Project Structure
```
src/components/
  DiffPanel.svelte              # Thin shell: state owner, staging handlers, delegates to children
  DiffPanel.test.ts             # Existing tests -- Props interface unchanged
  diff/
    DiffToolbar.svelte          # View mode segmented control, filename, file actions, close
    DiffViewer.svelte           # View mode dispatcher: HunkView / FullFileView / SplitView
    HunkView.svelte             # Current hunk rendering (extracted from DiffPanel)
    FullFileView.svelte         # Stub: "Full file view -- coming soon"
    SplitView.svelte            # Stub: "Split view -- coming soon"
```

**Rationale:** The project already uses subdirectory organization for complex components (`src/components/virtual-list/`). A `diff/` subdirectory keeps the 5 new files co-located without polluting the flat `src/components/` namespace. DiffPanel.svelte stays at the top level since RepoView imports it there.

### Pattern 1: Thin Shell Parent (DiffPanel)
**What:** DiffPanel becomes a state-owning shell that delegates rendering to child components.
**When to use:** When decomposing monoliths where the outer component is the integration point for consumers.
**Example:**
```svelte
<script lang="ts">
  // DiffPanel.svelte -- thin shell
  import DiffToolbar from "./diff/DiffToolbar.svelte";
  import DiffViewer from "./diff/DiffViewer.svelte";
  import { getDiffViewMode, setDiffViewMode } from "../lib/store.js";
  import type { ViewMode } from "../lib/types.js";

  // Props interface UNCHANGED from current
  interface Props { /* same as today */ }
  let { fileDiffs, commitDetail, selectedPath, onclose, diffKind, repoPath, onhunkaction, loading }: Props = $props();

  // State owned by shell
  let viewMode = $state<ViewMode>("hunk");
  let selectedHunkKey = $state<string | null>(null);
  let selectedLineIndices = $state<Set<number>>(new Set());
  let collapsedFiles = $state<Set<string>>(new Set());

  // Load persisted view mode on mount
  $effect(() => {
    getDiffViewMode().then((m) => { viewMode = m; });
  });

  // All staging handlers stay here (handleStageFile, handleUnstageFile, etc.)
</script>

<div style="height: 100%; display: flex; flex-direction: column; overflow: hidden; background: var(--color-bg);">
  <DiffToolbar {viewMode} onviewmodechange={handleViewModeChange} {selectedPath} {diffKind} ... />
  <DiffViewer {viewMode} {fileDiffs} ... />
</div>
```

### Pattern 2: View Mode Dispatcher (DiffViewer)
**What:** A component that renders one of several view components based on a mode prop.
**When to use:** When the same data can be presented in multiple ways with a toggle.
**Example:**
```svelte
<script lang="ts">
  // DiffViewer.svelte
  import HunkView from "./HunkView.svelte";
  import FullFileView from "./FullFileView.svelte";
  import SplitView from "./SplitView.svelte";
  import type { ViewMode } from "../../lib/types.js";

  interface Props {
    viewMode: ViewMode;
    // ... pass through all rendering props
  }
  let { viewMode, ...rest }: Props = $props();
</script>

{#if viewMode === "hunk"}
  <HunkView {...rest} />
{:else if viewMode === "full"}
  <FullFileView />
{:else}
  <SplitView />
{/if}
```

### Pattern 3: DiffLineRenderer as {#snippet}
**What:** A Svelte 5 snippet for per-line rendering within HunkView.
**When to use:** When the rendering logic needs access to parent scope (hunk context, line selection state, click handlers) and has no independent state of its own.
**Why snippet over component:** DiffLineRenderer needs access to `selectedHunkKey`, `selectedLineIndices`, `handleLineClick`, `isSelectable` context. Passing all of these as props to a separate component creates a large, fragile interface. A snippet captures parent scope naturally. It also avoids component instantiation overhead for potentially thousands of lines.
**Example:**
```svelte
<!-- Inside HunkView.svelte -->
{#snippet renderLine(line: DiffLine, lineIdx: number, fd: FileDiff, hunkIdx: number, maxDigits: number)}
  {@const isSelectable = diffKind !== 'commit' && line.origin !== 'Context'}
  {@const hunkKey = `${fd.path}-${hunkIdx}`}
  {@const isSelected = selectedHunkKey === hunkKey && selectedLineIndices.has(lineIdx)}
  <div class="diff-line ...">
    <span class="gutter gutter-old">{line.old_lineno ?? ''}</span>
    <span class="gutter gutter-new">{line.new_lineno ?? ''}</span>
    <span class="origin">{originSymbol(line.origin)}</span>
    <span class="content"><!-- spans or plain text --></span>
  </div>
{/snippet}
```

### Pattern 4: Segmented Control
**What:** A group of buttons where exactly one is active, styled as a connected control.
**When to use:** For mutually exclusive mode toggles (view modes, display options).
**Example:**
```svelte
<!-- Inside DiffToolbar.svelte -->
<div class="segmented-control">
  {#each modes as mode}
    <button
      class="segment"
      class:active={viewMode === mode.value}
      onclick={() => onviewmodechange(mode.value)}
    >
      {mode.label}
    </button>
  {/each}
</div>

<style>
  .segmented-control {
    display: inline-flex;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    overflow: hidden;
  }
  .segment {
    background: none;
    border: none;
    border-right: 1px solid var(--color-border);
    color: var(--color-text-muted);
    font-size: 11px;
    padding: 2px 8px;
    cursor: pointer;
  }
  .segment:last-child {
    border-right: none;
  }
  .segment.active {
    background: var(--color-accent-bg);
    color: var(--color-accent);
  }
</style>
```

### Pattern 5: Line Number Gutter
**What:** Two fixed-width columns showing old and new line numbers before the diff content.
**When to use:** DISP-01 requirement.
**Key implementation detail:** Calculate `maxDigits` from the maximum line number across all hunks in a file, then use `ch` units or `min-width` to ensure columns don't shift.
**Example:**
```svelte
<script lang="ts">
  // Calculate max digits for consistent gutter width
  function maxLineNumber(fileDiff: FileDiff): number {
    let max = 0;
    for (const hunk of fileDiff.hunks) {
      for (const line of hunk.lines) {
        if (line.old_lineno !== null && line.old_lineno > max) max = line.old_lineno;
        if (line.new_lineno !== null && line.new_lineno > max) max = line.new_lineno;
      }
    }
    return max;
  }

  function gutterWidth(maxNum: number): string {
    const digits = Math.max(String(maxNum).length, 1);
    return `${digits + 1}ch`; // +1ch for padding
  }
</script>

<!-- Per line -->
<span class="gutter" style="min-width: {width}; text-align: right; color: var(--color-text-muted);">
  {line.old_lineno ?? ''}
</span>
<span class="gutter" style="min-width: {width}; text-align: right; color: var(--color-text-muted);">
  {line.new_lineno ?? ''}
</span>
```

### Anti-Patterns to Avoid
- **Breaking DiffPanel's Props interface:** RepoView passes 8 props to DiffPanel in 2 places. Changing this interface breaks the integration. The refactor is internal only.
- **Moving staging handlers to child components:** Staging handlers need `repoPath`, `diffKind`, and `onhunkaction` which come from RepoView. Keep them in DiffPanel and pass callbacks down.
- **Using a separate component for DiffLineRenderer:** Creating a `.svelte` component that needs 10+ props (line, lineIdx, hunkIdx, filePath, selectedHunkKey, selectedLineIndices, maxDigits, diffKind, handleLineClick, lineBackground, lineColor, originSymbol) is worse than a snippet that captures parent scope.
- **Inline color values in gutter:** Per CLAUDE.md, use `var(--color-text-muted)` not `#8b949e`.
- **Position hacks for gutter alignment:** Use CSS Grid or `display: flex` with fixed-width gutter columns. Never use `position: absolute` to align gutters.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| View mode persistence | Custom localStorage wrapper | LazyStore (`@tauri-apps/plugin-store`) | Already used for 8+ diff/layout prefs. Same get/set/save pattern. |
| Icons | SVG strings or custom icon components | `@lucide/svelte` | Already imported in 13 components. Consistent icon set. |
| State management | Custom pub/sub or context stores | Svelte 5 `$state` + `$derived` runes | Project standard. No Svelte stores needed. |

## Common Pitfalls

### Pitfall 1: Breaking Existing Tests
**What goes wrong:** DiffPanel.test.ts imports and renders `DiffPanel` directly. If the Props interface changes or the rendered DOM structure changes incompatibly, 14 tests break.
**Why it happens:** Refactoring components often inadvertently changes the public API.
**How to avoid:** Keep DiffPanel's Props interface identical. Test after every structural change. The tests check for specific text content ("@@ -1,3 +1,4 @@", "+const x = 2;", "Stage Hunk") and CSS classes (".word-delete", ".syn-keyword", ".diff-line-add"). All of these must remain in the rendered output.
**Warning signs:** `bun run test` failures on DiffPanel.test.ts.

### Pitfall 2: LazyStore Async Initialization Race
**What goes wrong:** View mode loads from LazyStore asynchronously. If the component renders before the value resolves, it flashes the default mode then switches.
**Why it happens:** `$effect` runs after the first render. LazyStore.get() is async.
**How to avoid:** Default to `"hunk"` (the current behavior) so the initial render is correct even before the stored value loads. The update from stored value to a different mode (if any) happens seamlessly since it just changes which view component renders.
**Warning signs:** Visual flash on component mount.

### Pitfall 3: Line Selection State Across View Mode Switch
**What goes wrong:** User selects lines in hunk mode, switches to full/split mode (stubs), switches back -- selection state is lost or stale.
**Why it happens:** View mode switch unmounts/remounts HunkView, losing transient state.
**How to avoid:** Line selection state (`selectedHunkKey`, `selectedLineIndices`, `lastClickedIndex`) lives in DiffPanel (the shell), not in HunkView. Clear selection on view mode change (same as current behavior on fileDiffs change).
**Warning signs:** Stale selection highlights after switching modes.

### Pitfall 4: Gutter Width Recalculation on Large Files
**What goes wrong:** For files with 10,000+ lines (5 digits), the gutter must be wide enough. If calculated per-hunk instead of per-file, different hunks have different gutter widths.
**Why it happens:** Each hunk has different line number ranges.
**How to avoid:** Calculate `maxLineNumber` across ALL hunks in a FileDiff, not per-hunk. Pass the computed `maxDigits` to HunkView.
**Warning signs:** Gutter columns jumping width between hunks in the same file.

### Pitfall 5: Keyboard Navigation After Decomposition
**What goes wrong:** Hunk navigation ([ and ] keys) and Escape to clear selection stop working.
**Why it happens:** The keyboard event listener in DiffPanel needs `hunkElements` ref map which moves to HunkView.
**How to avoid:** Keep keyboard event listener in DiffPanel (shell), but `hunkElements` must be accessible. Either: (a) pass a `bind:this` ref from HunkView back to DiffPanel, or (b) keep hunkElements in DiffPanel and pass it down for HunkView to `bind:this` into, or (c) use `scrollToHunk` via a callback. The simplest approach: keep `hunkElements` as a `$state` object in DiffPanel, pass it to HunkView which binds elements into it.
**Warning signs:** [ and ] keys don't navigate hunks, Escape doesn't clear selection.

## Code Examples

### ViewMode Type Addition (types.ts)
```typescript
// Add to src/lib/types.ts
export type ViewMode = "hunk" | "full" | "split";
```

### Store Functions (store.ts)
```typescript
// Add to src/lib/store.ts
const DIFF_VIEW_MODE_KEY = "diff_view_mode";

export async function getDiffViewMode(): Promise<ViewMode> {
  const stored = await store.get<string>(DIFF_VIEW_MODE_KEY);
  if (stored === "hunk" || stored === "full" || stored === "split") return stored;
  return "hunk";
}

export async function setDiffViewMode(mode: ViewMode): Promise<void> {
  await store.set(DIFF_VIEW_MODE_KEY, mode);
  await store.save();
}
```
Note: The `ViewMode` type import must use `import type { ViewMode } from "./types.js"`.

### Stub View Component
```svelte
<!-- FullFileView.svelte -->
<div style="
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-muted);
  font-size: 13px;
">
  Full file view -- coming soon
</div>
```

### Line Number Rendering in Snippet
```svelte
{#snippet renderLine(line, lineIdx, fd, hunkIdx, gutterW)}
  {@const isSelectable = diffKind !== 'commit' && line.origin !== 'Context'}
  {@const hunkKey = `${fd.path}-${hunkIdx}`}
  {@const isSelected = selectedHunkKey === hunkKey && selectedLineIndices.has(lineIdx)}
  <div
    class="diff-line {line.origin === 'Add' ? 'diff-line-add' : line.origin === 'Delete' ? 'diff-line-delete' : 'diff-line-context'}"
    style="
      font-family: monospace;
      font-size: 12px;
      line-height: 1.5;
      padding: 0 8px;
      white-space: pre;
      overflow-x: auto;
      background: {lineBackground(line.origin, isSelected)};
      color: {lineColor(line.origin)};
      cursor: {isSelectable ? 'pointer' : 'default'};
      display: flex;
    "
    onclick={(e) => isSelectable && handleLineClick(fd.path, hunkIdx, lineIdx, line.origin, hunk.lines, e)}
  >
    <span style="min-width: {gutterW}; text-align: right; color: var(--color-text-muted); user-select: none; flex-shrink: 0;">{line.old_lineno ?? ''}</span>
    <span style="min-width: {gutterW}; text-align: right; color: var(--color-text-muted); padding-right: 8px; user-select: none; flex-shrink: 0;">{line.new_lineno ?? ''}</span>
    <span style="flex-shrink: 0; width: 1ch; user-select: none;">{originSymbol(line.origin)}</span>
    <span style="flex: 1;">{#if line.spans.length > 0}{#each line.spans as span}<span
      class="{span.syntax_class}{span.emphasized ? (line.origin === 'Add' ? ' word-add' : ' word-delete') : ''}"
    >{line.content.slice(span.start, span.end)}</span>{/each}{:else}{line.content}{/if}</span>
  </div>
{/snippet}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Svelte slots | Svelte 5 snippets | Svelte 5.0 (Oct 2024) | Snippets replace slots for content injection; this project already uses Svelte 5 |
| Svelte stores | `$state` / `$derived` runes | Svelte 5.0 (Oct 2024) | No writable/readable stores needed; this project uses runes throughout |
| `export let` props | `$props()` destructuring | Svelte 5.0 (Oct 2024) | All components in project use `$props()` |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest (via vite config) |
| Config file | `vite.config.ts` (test section) |
| Quick run command | `bun run test -- --run DiffPanel` |
| Full suite command | `bun run test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| VIEW-01 | View mode toggle renders segmented control with 3 options | unit | `bun run test -- --run DiffPanel` | Needs new tests |
| VIEW-01 | Clicking a view mode button changes the active mode | unit | `bun run test -- --run DiffPanel` | Needs new tests |
| VIEW-01 | Non-hunk modes render stub placeholder text | unit | `bun run test -- --run DiffPanel` | Needs new tests |
| DISP-01 | Line numbers render in gutter for context/add/delete lines | unit | `bun run test -- --run DiffPanel` | Needs new tests |
| DISP-01 | Add lines show only new line number, Delete only old | unit | `bun run test -- --run DiffPanel` | Needs new tests |
| REFACTOR | All 14 existing DiffPanel tests still pass | regression | `bun run test -- --run DiffPanel` | Yes (DiffPanel.test.ts) |

### Sampling Rate
- **Per task commit:** `bun run test -- --run DiffPanel`
- **Per wave merge:** `bun run test && bun run check`
- **Phase gate:** Full suite green + svelte-check clean before `/gsd:verify-work`

### Wave 0 Gaps
- New test cases for VIEW-01 (segmented control rendering, mode switching, stub display)
- New test cases for DISP-01 (line number gutter rendering)
- Existing 14 DiffPanel tests must pass unchanged (regression)

## Sources

### Primary (HIGH confidence)
- Direct code reading: `src/components/DiffPanel.svelte` (667 lines, current implementation)
- Direct code reading: `src/components/DiffPanel.test.ts` (14 tests, 346 lines)
- Direct code reading: `src/lib/types.ts` (DiffLine with old_lineno/new_lineno fields)
- Direct code reading: `src/lib/store.ts` (LazyStore pattern, 8 existing preference keys)
- Direct code reading: `src/components/RepoView.svelte` (DiffPanel usage at lines 597-604, 650-663)
- Direct code reading: `src/app.css` (CSS custom properties for diff colors, syntax colors)
- [Svelte 5 snippet docs](https://svelte.dev/docs/svelte/snippet) - snippet syntax, limitations, when to use

### Secondary (MEDIUM confidence)
- [Svelte 5 migration guide](https://svelte.dev/docs/svelte/v5-migration-guide) - runes and snippets patterns
- [Svelte best practices 2026](https://onehorizon.ai/blog/svelte-best-practices-in-2026-scaling-with-runes-snippets-and-pure-reactivity) - component vs snippet decision guidelines

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies, everything already in project
- Architecture: HIGH - based on direct code reading of 667-line monolith and project patterns
- Pitfalls: HIGH - identified from analyzing existing test suite, state management, and keyboard navigation code

**Research date:** 2026-03-29
**Valid until:** 2026-04-28 (stable -- Svelte 5 runes API is settled, project code is the primary research target)
