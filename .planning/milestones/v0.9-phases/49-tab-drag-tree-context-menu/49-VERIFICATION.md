---
phase: 49-tab-drag-tree-context-menu
verified: 2026-03-25T00:30:00Z
status: human_needed
score: 7/7 must-haves verified
re_verification: false
human_verification:
  - test: "Drag tab to new position"
    expected: "Tab moves with smooth animation (animation:150ms); the + button cannot be dragged; tab order updates immediately"
    why_human: "SortableJS manipulates the DOM directly — requires real browser + Tauri runtime; vitest cannot simulate pointer drag events"
  - test: "Tab reorder persists across app relaunch"
    expected: "After dragging tab 3 to position 1 and relaunching the app, the tab order matches the dragged order"
    why_human: "Requires full Tauri runtime with localStorage/store persistence"
  - test: "Tab bar auto-scrolls when dragging near edges"
    expected: "When dragging a tab near the left or right edge of the tab bar (overflow scenario), the bar scrolls"
    why_human: "Requires real browser layout with overflow + pointer events"
  - test: "Directory context menu in unstaged section"
    expected: "Right-clicking a directory node in the unstaged tree section shows a native menu with 'Stage All (N)' and 'Discard All (N)' items"
    why_human: "Tauri native menus (@tauri-apps/api/menu) are not available in vitest"
  - test: "Directory context menu in staged section"
    expected: "Right-clicking a directory node in the staged tree section shows a native menu with 'Unstage All (N)'"
    why_human: "Tauri native menus not available in vitest"
  - test: "Directory context menu in conflicted section"
    expected: "Right-clicking a conflicted directory shows 'Resolve All (N)' and 'Unresolve All (N)'"
    why_human: "Tauri native menus + merge conflict state required"
  - test: "Discard All shows confirmation dialog"
    expected: "Clicking 'Discard All' triggers a native warning dialog before any files are discarded"
    why_human: "@tauri-apps/plugin-dialog ask() requires Tauri runtime"
  - test: "Context menu appears on directory nodes only, not file nodes"
    expected: "Right-clicking a file row does not trigger the directory context menu"
    why_human: "Requires UI interaction in Tauri runtime"
---

# Phase 49: Tab Drag Reorder & Tree Context Menu — Verification Report

**Phase Goal:** Drag-and-drop tab reordering and right-click context menu on tree view directories for bulk stage/unstage/resolve/discard operations
**Verified:** 2026-03-25T00:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (from ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can drag tabs to reorder them, and the new order persists across app relaunch | ? HUMAN | `TabBar.svelte` has `Sortable.create` with `direction:'horizontal'`; `App.svelte` wires `onreorder` to `tabs = newTabs; persistTabs()`. DOM interaction and persistence require Tauri runtime. |
| 2 | User can right-click a directory in the tree view to access bulk actions (Stage All, Unstage All, Discard All, and resolve/unresolve for conflicted files) | ? HUMAN | `DirectoryRow.svelte` fires `oncontextmenu` on right-click with `e.preventDefault()`; `StagingPanel.svelte` has all three section handlers using `Menu.new()` / `menu.popup()`. Native Tauri menu requires runtime. |

**Score:** 7/7 must-haves code-verified (automated checks pass for all artifacts, key links, and data flow; runtime behavior requires human)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/components/TabBar.svelte` | SortableJS drag-and-drop integration with `{#key}` reconciliation | VERIFIED | `import Sortable from 'sortablejs'` at line 4; `Sortable.create(tabBarEl,` in `$effect` at line 28; `{#key tabs}` wrapper at line 51 |
| `src/App.svelte` | `onreorder` handler updating tabs and calling `persistTabs()` | VERIFIED | `onreorder={(newTabs) => { tabs = newTabs; persistTabs(); }}` at line 400 |
| `src/components/DirectoryRow.svelte` | `oncontextmenu` prop for directory right-click | VERIFIED | `oncontextmenu?: (e: MouseEvent) => void` in Props (line 14); handler on outer div at line 31 with `e.preventDefault()` |
| `src/components/TreeFileList.svelte` | `ondirectorycontextmenu` prop threaded to DirectoryRow | VERIFIED | `ondirectorycontextmenu?: (e: MouseEvent, dirPath: string) => void` in Props (line 19); passed to `DirectoryRow` at line 196 |
| `src/components/StagingPanel.svelte` | `showUnstagedDirContextMenu`, `showStagedDirContextMenu`, `showConflictedDirContextMenu` handlers | VERIFIED | All six functions present (lines 161, 203, 239, 285, 296, 307); all four TreeFileList instances wired (lines 745, 858, 872, 940) |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `TabBar.svelte` | `App.svelte` | `onreorder` callback prop | WIRED | `onreorder` in TabBar Props (line 14); passed and handled in App.svelte (line 400) |
| `App.svelte` | `src/lib/store.ts` | `persistTabs()` | WIRED | `persistTabs()` called in onreorder handler; pre-existing function serializing `tabs` array |
| `StagingPanel.svelte` | `TreeFileList.svelte` | `ondirectorycontextmenu` prop | WIRED | All 4 TreeFileList instances in StagingPanel have `ondirectorycontextmenu` wired to section-specific handlers |
| `TreeFileList.svelte` | `DirectoryRow.svelte` | `oncontextmenu` prop | WIRED | `oncontextmenu={ondirectorycontextmenu ? (e) => ondirectorycontextmenu!(e, row.node.path) : undefined}` at line 196 |
| `StagingPanel.svelte` | `@tauri-apps/api/menu` | `Menu.new()` / `menu.popup()` | WIRED | Dynamic import `await import('@tauri-apps/api/menu')` in all three dir context menu handlers |
| `StagingPanel.svelte` | `@tauri-apps/plugin-dialog` | `ask()` confirmation in Discard All | WIRED | `const { ask } = await import('@tauri-apps/plugin-dialog')` in `handleDiscardDirectory` (line 167) |

---

### Data-Flow Trace (Level 4)

Not applicable for this phase. The features are event-driven UI interactions (drag events, right-click context menus) rather than components that render data fetched from an API. The tab reorder propagates the existing in-memory `tabs` state array — no disconnected data source.

---

### Behavioral Spot-Checks (Step 7b)

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Test suite regression-free | `bun run test` | 170/170 passed (1.29s) | PASS |
| Type check — phase 49 files | `bun run check` (filter phase 49 files) | 0 errors in TabBar, App, DirectoryRow, TreeFileList, StagingPanel after `bun install` | PASS |
| `Sortable.create` pattern in TabBar | grep for `Sortable.create(tabBarEl` | Found at line 28 | PASS |
| `onreorder` wired in App | grep for `onreorder=` in App.svelte | Found at line 400 | PASS |
| All 4 TreeFileList instances wired | grep for `ondirectorycontextmenu` in StagingPanel | 4 occurrences at lines 745, 858, 872, 940 | PASS |

**Note on type check:** `bun run check` produces errors in `TabBar.svelte` (`Cannot find module 'sortablejs'`) when node_modules are not installed. The package is declared in `package.json` and `bun.lock` but `bun install` was not run as part of phase execution commits. After running `bun install`, all phase 49 files are type-clean. The lockfile (`bun.lock`) has uncommitted changes reflecting the missing install step — this is an environment setup gap, not a code defect.

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| TAB-11 | 49-01-PLAN.md | User can drag tabs to reorder them, and the new order persists across app relaunch | SATISFIED (code) / HUMAN (runtime) | `Sortable.create` with `direction:'horizontal'` + `onreorder` → `persistTabs()` wired end-to-end |
| TREE-11 | 49-02-PLAN.md | User can right-click a directory in the tree view to access bulk actions (Stage All, Unstage All, Discard All, and resolve/unresolve for conflicted files) | SATISFIED (code) / HUMAN (runtime) | All three dir context menu handlers present; discard confirmation via `ask()`; all TreeFileList instances wired |

No orphaned requirements — both TAB-11 and TREE-11 are the only IDs mapped to Phase 49 in REQUIREMENTS.md, and both are claimed by their respective plans.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/components/TabBar.svelte` | 4 | `Cannot find module 'sortablejs'` (type error without `bun install`) | Info | Type check fails in a fresh environment without running `bun install`; code is correct but deps must be installed |

No TODO/FIXME/placeholder comments, no empty implementations, no inline colors, no hardcoded empty data in any phase 49 modified file.

---

### Human Verification Required

The following behaviors require the full Tauri runtime and cannot be verified programmatically:

#### 1. Tab Drag Reorder — Visual Behavior

**Test:** Open 3+ repos as tabs. Grab a tab by clicking and holding, drag it to a different position.
**Expected:** Tab animates smoothly during drag (ghost opacity 0.4, chosen tab highlighted with `--color-selected-row`). Tab lands in the new position when released. The + button cannot be grabbed.
**Why human:** SortableJS manipulates DOM nodes directly; requires real browser pointer events in Tauri WebView.

#### 2. Tab Order Persists After Relaunch

**Test:** Drag tab 3 to position 1. Quit the app (Cmd+Q). Relaunch.
**Expected:** The tab order from before quit is restored.
**Why human:** Requires `persistTabs()` to complete write to localStorage and Tauri store before quit; full runtime needed.

#### 3. Tab Bar Auto-Scroll at Edges

**Test:** Open enough tabs to overflow the tab bar. Drag a tab toward the left or right edge of the bar.
**Expected:** The bar scrolls to reveal hidden tabs during the drag.
**Why human:** Requires real browser layout with CSS overflow and SortableJS `scroll: true` + `scrollSensitivity: 50` interaction.

#### 4. Directory Context Menu — Unstaged Section

**Test:** Create 2+ unstaged files in a directory. Switch to tree view. Right-click the directory.
**Expected:** Native OS context menu appears with "Stage All (2)" and "Discard All (2)".
**Why human:** `@tauri-apps/api/menu` Menu.new() / menu.popup() require Tauri IPC; unavailable in vitest.

#### 5. Directory Context Menu — Staged Section

**Test:** Stage files in a directory. Right-click the directory in the staged section (tree view).
**Expected:** Native OS context menu appears with "Unstage All (N)".
**Why human:** Same as above.

#### 6. Directory Context Menu — Conflicted Section

**Test:** Create a merge conflict with conflicted files in a single directory. Right-click the directory.
**Expected:** Native menu shows "Resolve All (N)" and "Unresolve All (N)".
**Why human:** Requires merge conflict state + Tauri runtime.

#### 7. Discard All Confirmation Dialog

**Test:** Right-click an unstaged directory, click "Discard All".
**Expected:** A native warning dialog appears with message including the file count. Changes are discarded only after confirming.
**Why human:** `@tauri-apps/plugin-dialog` `ask()` requires Tauri runtime.

#### 8. Context Menu Scope — Directory Nodes Only

**Test:** Right-click a file row (not a directory) in tree view.
**Expected:** The directory context menu does NOT appear (file context menu may appear, but directory bulk actions should not).
**Why human:** Requires DOM interaction to confirm the `oncontextmenu` prop is absent from FileRow for this feature.

---

### Gaps Summary

No code gaps found. All artifacts exist, are substantive, and are wired end-to-end. The phase 49 code is complete per plan specifications.

One environment note: `sortablejs` and `@types/sortablejs` are declared in `package.json` but the phase 49 commits did not include a `bun install` run or lockfile commit. The type check fails with `Cannot find module 'sortablejs'` in a fresh environment. Running `bun install` resolves this. This is not a code defect but should be noted for CI/onboarding.

All blocking verification items are UI interaction behaviors requiring the full Tauri runtime, appropriately flagged as manual-only in the phase 49 VALIDATION.md contract.

---

_Verified: 2026-03-25T00:30:00Z_
_Verifier: Claude (gsd-verifier)_
