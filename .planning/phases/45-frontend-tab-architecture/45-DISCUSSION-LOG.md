# Phase 45: Frontend Tab Architecture - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 45-frontend-tab-architecture
**Areas discussed:** Shared vs per-tab settings, Dirty indicator details, Tab overflow handling, Tab switching confirmation

---

## Shared vs Per-tab Settings

### Zoom level

| Option | Description | Selected |
|--------|-------------|----------|
| Global (Recommended) | Cmd+/- changes zoom for the whole app. Simpler, consistent — matches browser behavior. | ✓ |
| Per-tab | Each tab remembers its own zoom level. More flexible but unusual. | |

**User's choice:** Global
**Notes:** None

### Pane widths

| Option | Description | Selected |
|--------|-------------|----------|
| Global (Recommended) | Resizing sidebar/panel applies to all tabs. Matches VS Code, GitKraken. | ✓ |
| Per-tab | Each tab remembers its own sidebar/panel widths. | |

**User's choice:** Global
**Notes:** None

### Pane collapsed state

| Option | Description | Selected |
|--------|-------------|----------|
| Global (Recommended) | Collapsing the sidebar collapses it for all tabs. Simple, predictable. | ✓ |
| Per-tab | Each tab can independently show/hide sidebar and right panel. | |

**User's choice:** Global
**Notes:** None

---

## Dirty Indicator Details

### What counts as dirty

| Option | Description | Selected |
|--------|-------------|----------|
| Staged + unstaged (Recommended) | Any modified/added/deleted file (staged or unstaged) shows the dot. Most intuitive. | ✓ |
| Staged only | Only files explicitly staged show the dot. | |
| Unstaged only | Only working tree modifications (not yet staged). | |

**User's choice:** Staged + unstaged
**Notes:** None

### Detection method

| Option | Description | Selected |
|--------|-------------|----------|
| Watcher events (Recommended) | fs watcher already emits repo-changed events per repo path. Listen and update dirty flag. | ✓ |
| Poll on interval | Periodically call get_dirty_counts for background tabs. | |

**User's choice:** Watcher events
**Notes:** None

---

## Tab Overflow Handling

### Overflow behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Shrink then scroll (Recommended) | Tabs shrink to ~100px min, then tab bar scrolls. Matches Chrome/VS Code. | |
| Scroll only | Tabs keep fixed width, tab bar scrolls horizontally. Simpler. | ✓ |
| Overflow dropdown | N visible tabs + dropdown for the rest. Compact but hides tabs. | |

**User's choice:** Scroll only
**Notes:** User preferred fixed-width tabs with scroll over shrinking

### Max tab limit

| Option | Description | Selected |
|--------|-------------|----------|
| No limit (Recommended) | Let users open as many as they want. | ✓ |
| Soft limit with warning | Warn at ~10 tabs but allow more. | |

**User's choice:** No limit
**Notes:** None

---

## Tab Switching Confirmation

### Switching strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Destroy/recreate (Recommended) | Unmount inactive tab, remount on switch. Simple, low memory. Loses scroll position. | |
| Keep-alive (hidden) | All tabs stay mounted with display:none. Preserves scroll position and open diffs. | ✓ |

**User's choice:** Keep-alive (hidden)
**Notes:** User wants zero context loss when switching between repos — preserving scroll position and open diffs is important

### Restore selected commit

| Option | Description | Selected |
|--------|-------------|----------|
| No restore (Recommended) | Tab switches to clean state. Simple, consistent. | |
| Restore selected commit | Remember last selected commit OID per tab, re-select on switch. | ✓ |

**User's choice:** Restore selected commit
**Notes:** Natural consequence of keep-alive — state is preserved automatically

---

## Claude's Discretion

- Per-tab state structure (component extraction pattern, context API vs props)
- remoteState and undoRedoState isolation approach
- Tab bar implementation details
- Keyboard shortcut handler architecture

## Deferred Ideas

None — discussion stayed within phase scope
