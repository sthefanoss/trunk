---
status: complete
phase: 71-output-clipboard-save-to-file
source: [71-01-SUMMARY.md, 71-VERIFICATION.md]
started: 2026-05-26T23:30:00Z
updated: 2026-05-26T23:55:00Z
disposition: validated-with-deferred-design-rework
---

## Disposition

User closed UAT explicitly: phase 71 considered done and validated. The
clipboard-write mechanism itself (awaited writeText, ✓ Copied affordance,
error-toast failure path, 1500ms timer, theme tokens) is accepted as-is.

Two design-level gaps surfaced and are deferred to follow-up work as concrete
TODOs (not blockers on Phase 71's completion):

  - G-71-A → `.planning/todos/pending/2026-05-26-relocate-copy-action-off-preview-pane.md`
  - G-71-B → `.planning/todos/pending/2026-05-26-review-pane-navigation-and-dead-review-button.md`

Both will likely be addressed together in a follow-up phase covering review-pane
UX integration (see the TODO files for decision lists and the carry-forward
checklist from this implementation).

## Current Test

[testing complete — closed with deferred design rework]

## Tests

### 1. Real OS clipboard receives the markdown
expected: After clicking Copy in `just dev`, paste into TextEdit or VS Code and confirm the full review markdown arrives verbatim.
result: issue
reported: "We don't need to preview the markdown, we can just copy it directly from the comments view."
severity: major

### 2. Visual styling parity with Back to comments button
expected: Copy button border, padding, font-size, and hover color visually match the "← Back to comments" button. Copy button docked to right of header via flexbox (no overlap).
result: [pending]
note: blocked on G-71-A — moot if button relocates off the preview pane header

### 3. Copied affordance duration feels right
expected: Button stays in "✓ Copied" state for approximately 1.5 seconds — long enough to register, not sluggish — before reverting to "Copy".
result: [pending]
note: still meaningful after relocation — duration is independent of placement

### 4. Failure path surfaces a readable toast
expected: Temporarily revoke `clipboard-manager:allow-write-text` in `src-tauri/capabilities/default.json`, restart `just dev`, click Copy — toast appears with intelligible text like "Failed to copy: <reason>" (not "[object Object]", not empty). Restore capability before commit.
result: [pending]
note: still meaningful after relocation — error handling is independent of placement

### 5. Theme custom-property correctness across light and dark modes
expected: In `just dev`, toggle OS theme (System Preferences → Appearance → Light/Dark). Copy button border and hover background track the active theme correctly in both modes.
result: [pending]
note: blocked on G-71-A — re-test against new placement's host element

## Summary

total: 5
passed: 0
issues: 1
pending: 4
skipped: 0
blocked: 0

## Gaps

- truth: "User can copy the generated markdown to the clipboard without first navigating to a dedicated preview pane"
  status: failed
  reason: "User reported: We don't need to preview the markdown, we can just copy it directly from the comments view."
  severity: major
  test: 1
  id: G-71-A
  scope: design-rework
  notes: |
    Phase 71 placed Copy on the ReviewDocPreview header (per CONTEXT/PLAN), which assumed
    the markdown preview pane was the natural artifact-export surface. User feedback:
    the preview pane is overhead for the copy action. The copy affordance should sit on
    the comments view itself.

    Design questions for replan:
      1. Where on the comments view does Copy live? (Toolbar? Per-comment? Header action?)
      2. Does the preview pane still have a purpose, or does it become a "view raw markdown"
         option behind a toggle rather than a default-render destination?
      3. What does Copy emit — the full review document, or a per-comment excerpt, or both
         (two affordances)?
  artifacts:
    - path: src/components/ReviewDocPreview.svelte
      issue: Hosts the Copy button at lines 55-63; will move or be removed when relocated
    - path: src/components/ReviewPanel.svelte
      issue: Parent of the preview pane; likely the comments view that should host Copy
  missing:
    - "Decision on Copy button host (comments view location)"
    - "Decision on preview pane's continued role (keep / fold-into-toggle / remove)"

- truth: "Entering and leaving the review pane is a smooth, discoverable interaction with no dead UI"
  status: failed
  reason: "User reported: We should have a better way of going in and out of the review pane. That menu option is too clunky, and for some reason we have a blue review button in the review pane that doesn't do anything. We should think about how to fit this new review pane in the UI, and a good user experience way of going in and out of it."
  severity: major
  test: 1
  id: G-71-B
  scope: design-rework + bug
  notes: |
    Two sub-issues bundled in this feedback:

    Sub-issue B1 (UX) — Navigation in/out of the review pane via the current menu route is
    clunky. Need a better-fitting entry/exit pattern (likely a dedicated toolbar action,
    keyboard shortcut, or contextual button on the comments view).

    Sub-issue B2 (bug) — A blue "Review" button is visible inside the review pane and
    does nothing on click. Per ownership.md, this is now ours regardless of which phase
    introduced it (probably Phase 70 — the pane host phase). Needs source location +
    decision (wire it up, remove it, or repurpose it).
  artifacts:
    - path: src/components/ReviewPanel.svelte
      issue: Likely host of the dead "Review" button — needs grep to confirm
    - path: src/App.svelte
      issue: Hosts the menu route into the review pane that the user finds clunky
  missing:
    - "Source-locate the dead blue Review button"
    - "Decision on review pane entry/exit pattern (toolbar action vs. shortcut vs. contextual)"
    - "Wire/remove the dead button"
