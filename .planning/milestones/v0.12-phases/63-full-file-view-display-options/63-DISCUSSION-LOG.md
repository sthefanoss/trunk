# Phase 63: Full File View & Display Options - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 63-full-file-view-display-options
**Areas discussed:** Full file view structure, Toolbar controls layout, Invisible characters, Word wrap behavior

---

## Full File View Structure

### Presentation Style

| Option | Description | Selected |
|--------|-------------|----------|
| Continuous document | Remove hunk headers, show whole file as one scrollable block with changed lines highlighted | ✓ |
| Expanded hunks with headers | Keep @@ hunk headers but fill in all context between them | |
| VS Code style | Full file with subtle separator lines between change regions | |

**User's choice:** Continuous document
**Notes:** Sequential line numbers, no section dividers

### Staging in Full File View

| Option | Description | Selected |
|--------|-------------|----------|
| No staging in full file view | Read-only for reviewing, switch to hunk view to stage. VIEW-05 is Phase 64 | ✓ |
| Hunk staging only | Detect hunk boundaries and allow staging whole hunks | |
| You decide | Claude picks simplest approach | |

**User's choice:** No staging in full file view

### Line Numbers for Deleted Lines

| Option | Description | Selected |
|--------|-------------|----------|
| Show both columns always | Old/new columns follow same rules as hunk view. Consistent behavior | ✓ |
| Sequential new-only column | Single column showing new file line numbers. Deleted lines get no number | |

**User's choice:** Show both columns always

### New Files in Full File View

| Option | Description | Selected |
|--------|-------------|----------|
| Show complete file with add backgrounds | All lines with green add background, sequential line numbers | ✓ |
| Same as hunk view | No special behavior | |
| You decide | Claude picks simplest | |

**User's choice:** Show complete file with add backgrounds

---

## Toolbar Controls Layout

### Control Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Inline after view mode | Toggle buttons after segmented control with separator. All visible, no menus | ✓ |
| Overflow dropdown | New controls in a gear dropdown menu | |
| Second toolbar row | Display options on a dedicated row below | |

**User's choice:** Inline after view mode

### Toggle Active State

| Option | Description | Selected |
|--------|-------------|----------|
| Highlighted background | Active toggles get subtle filled background, matching segmented control pattern | ✓ |
| Icon color change | Active toggles change icon color | |
| You decide | Claude picks consistent approach | |

**User's choice:** Highlighted background

### Context Lines Dropdown

| Option | Description | Selected |
|--------|-------------|----------|
| Hide in full file mode | Dropdown disappears when view mode is "full" | ✓ |
| Show but disabled | Grayed out in full file mode | |
| Always show | Always visible and functional | |

**User's choice:** Hide in full file mode
**Notes:** User subsequently clarified: NO context lines dropdown at all. Context lines is config-file-only. All user configurables stored in trunk-prefs.json. Future settings page will provide UI.

### Context Lines Values

| Option | Description | Selected |
|--------|-------------|----------|
| 3 / 5 / 10 / 25 | Four presets, no 0 option | |
| 0 / 3 / 5 / 10 / 25 | Five presets including 0 for changes-only view | ✓ |
| 3 / 5 / 10 | Three presets only | |

**User's choice:** 0 / 3 / 5 / 10 / 25

### Config vs Toolbar Toggles

| Option | Description | Selected |
|--------|-------------|----------|
| Toolbar toggles (read/write config) | Toggle buttons in toolbar that read/write from config file. Quick access for frequently-toggled settings | ✓ |
| No toolbar toggles | All display settings config-file-only until settings page | |

**User's choice:** Toolbar toggles that read/write from config file
**Notes:** User emphasized all user configurables should be stored in the config file. Toolbar is just UI for quick access. Settings page comes later for themes, code themes, tab size, font size, font family, etc.

---

## Invisible Characters

### Rendering Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Inline substitution | Spaces as · (middle dot), tabs as → (arrow), muted color. Standard VS Code/Sublime approach | ✓ |
| Overlay markers | Faint symbols overlaid on original whitespace | |
| Spaces only, no tabs | Only show space dots | |

**User's choice:** Inline substitution

### Trailing Whitespace

| Option | Description | Selected |
|--------|-------------|----------|
| Highlight trailing spaces | Subtle warning background (faint red/amber) on trailing whitespace | ✓ |
| Same as other spaces | Just show as · in muted color, no special treatment | |
| You decide | Claude picks simplest | |

**User's choice:** Highlight trailing spaces

### Line Endings

| Option | Description | Selected |
|--------|-------------|----------|
| No line endings | Only spaces and tabs shown. No CR/LF markers | ✓ |
| Show line endings | Show ↵ or LF/CRLF at end of each line | |
| You decide | Claude picks | |

**User's choice:** No line endings

---

## Word Wrap

### Wrapped Line Display

| Option | Description | Selected |
|--------|-------------|----------|
| Wrap at container edge, no indent | Lines wrap at viewer width. Continuation at column 0 past gutter. CSS pre-wrap | ✓ |
| Wrap with hanging indent | Continuation lines indented to match code indentation level | |
| You decide | Claude picks simplest | |

**User's choice:** Wrap at container edge, no indent

### Wrap Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Global toggle | Single preference applies to all view modes equally | ✓ |
| Per-view-mode | Each view mode remembers its own wrap setting | |

**User's choice:** Global toggle

---

## Claude's Discretion

- CSS implementation technique for invisible character rendering
- Exact muted color and warning background CSS custom properties
- Lucide icon choices for toolbar toggle buttons
- Whether invisibles rendering is frontend-only or involves Rust backend

## Deferred Ideas

- Settings/preferences page for all configurables (themes, fonts, tab size, context lines, etc.)
- Per-hunk context expand buttons (ADVD-02)
