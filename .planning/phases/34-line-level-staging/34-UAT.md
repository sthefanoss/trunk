---
status: complete
phase: 34-line-level-staging
source: 34-01-SUMMARY.md, 34-02-SUMMARY.md
started: 2026-03-18T07:00:00Z
updated: 2026-03-18T07:10:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Click to Select a Diff Line
expected: In a file with unstaged changes, click on an added or deleted line in the diff view. The clicked line highlights with a brighter background color. Clicking the same line again deselects it.
result: pass

### 2. Shift+Click Range Selection
expected: Click a diff line to select it, then shift+click another line in the same hunk. All add/delete lines between the two clicks become selected (highlighted). The selection count updates accordingly.
result: issue
reported: "it works, but when I do it, it selected the text as well. We should fix this."
severity: minor

### 3. Toolbar Mode Switching
expected: With no lines selected, toolbar shows "Stage Hunk" / "Discard Hunk" buttons. After selecting one or more lines, toolbar switches to "Stage Lines (N)" / "Discard Lines (N)" where N is the count of selected lines.
result: pass

### 4. Stage Selected Lines
expected: Select a subset of added/deleted lines in an unstaged hunk. Click "Stage Lines (N)". Only the selected lines are staged — the remaining changes in that hunk stay unstaged. The diff view updates to reflect the partial staging.
result: pass

### 5. Discard Selected Lines
expected: Select a subset of added/deleted lines in an unstaged hunk. Click "Discard Lines (N)". A confirmation dialog appears. After confirming, only the selected lines are discarded — the remaining changes in that hunk are preserved.
result: pass

### 6. Escape Clears Selection
expected: Select one or more diff lines. Press Escape. All selections are cleared and the toolbar reverts to hunk-mode buttons.
result: pass

### 7. Cross-Hunk Click Clears Previous Selection
expected: Select lines in one hunk, then click a line in a different hunk. The previous hunk's selections are cleared and the new hunk's line is selected.
result: pass

## Summary

total: 7
passed: 6
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "Shift+click range selection works without triggering browser text selection"
  status: failed
  reason: "User reported: it works, but when I do it, it selected the text as well. We should fix this."
  severity: minor
  test: 2
  artifacts: []
  missing: []
