# Phase 47: Tree View UI Integration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-24
**Phase:** 47-tree-view-ui-integration
**Areas discussed:** Toggle placement & style, Tree node visual design, Expand/collapse state scope, Keyboard navigation

---

## Toggle placement & style

| Option | Description | Selected |
|--------|-------------|----------|
| Per-section header | Small icon toggle in each section header (Unstaged, Staged, Conflicted). Independent per section. | |
| Single global toggle | One toggle in the panel header that applies to ALL sections at once. | ✓ |
| You decide | Claude picks the best approach. | |

**User's choice:** Single global toggle
**Notes:** Toggle in the "N files changed" panel header bar.

### Follow-up: Toggle scope across contexts

| Option | Description | Selected |
|--------|-------------|----------|
| One global preference | Single persisted setting applies everywhere — staging, commit diffs, merge editor. | ✓ |
| Independent per context | Each context (staging, diffs, merge) has its own toggle. | |

**User's choice:** One global preference

---

## Tree node visual design

| Option | Description | Selected |
|--------|-------------|----------|
| Indent + chevron only | Each level adds ~16px padding. Chevrons for dirs, status icons for files. | ✓ |
| Indent + chevron + folder icon | Same plus Lucide Folder/FolderOpen icons on directories. | |
| Indent + connector lines | Tree lines (│ ├─ └─) connecting parent to children. | |

**User's choice:** Indent + chevron only

### Follow-up: File label in tree mode

| Option | Description | Selected |
|--------|-------------|----------|
| Filename only | Tree mode shows just "store.ts", flat mode shows full path. | ✓ |
| Filename + dimmed parent path | Show "store.ts" with dimmed "src/lib/" prefix. | |

**User's choice:** Filename only

---

## Expand/collapse state scope

| Option | Description | Selected |
|--------|-------------|----------|
| Per-section, keyed by path | Unstaged and Staged track their own Set<string> of expanded dirs. Preserved on refresh. | ✓ |
| Shared across sections | One Set<string> for the whole panel. | |
| You decide | Claude picks. | |

**User's choice:** Per-section, keyed by path

### Follow-up: Default expand state on toggle

| Option | Description | Selected |
|--------|-------------|----------|
| All expanded | Everything open on first toggle to tree. | |
| All collapsed | Only top-level visible. User expands to drill in. | ✓ |
| You decide | Claude picks. | |

**User's choice:** All collapsed

---

## Keyboard navigation

| Option | Description | Selected |
|--------|-------------|----------|
| VS Code style | Up/Down move, Right expand/enter, Left collapse/parent, Enter select. | ✓ |
| Simple up/down only | Up/Down moves, Left/Right ignored. Click-only expand/collapse. | |
| You decide | Claude picks. | |

**User's choice:** VS Code style

### Follow-up: Focus visibility and scope

| Option | Description | Selected |
|--------|-------------|----------|
| Visible focus, tree mode only | Highlight on focused row, tree mode only. | |
| Visible focus, both modes | Add keyboard navigation to both flat and tree modes. | ✓ |
| You decide | Claude picks. | |

**User's choice:** Visible focus, both modes

---

## Claude's Discretion

- TreeView component structure (single component vs DirectoryRow + TreeList)
- Shared hook/factory vs inline state for tree management
- FileDiff → buildTree adapter approach for DiffPanel/MergeEditor
- Expand/collapse animation (slide vs instant)

## Deferred Ideas

None — discussion stayed within phase scope
