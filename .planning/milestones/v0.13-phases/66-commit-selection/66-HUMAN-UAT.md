---
status: partial
phase: 66-commit-selection
source: [66-VERIFICATION.md]
started: 2026-05-25T15:25:00Z
updated: 2026-05-25T15:25:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Two-right-click range gesture seeding
expected: Right-click commit A → "Set as review base" highlights row A with the pending-base marker (bottom inset, `--color-review-pending-base`). Right-click a descendant commit B → "Add range to review" seeds the inclusive `[A..B]` range into the session and clears the pending-base highlight.
result: [pending]

### 2. Add to review from context menu
expected: Right-click a commit NOT in the session → menu item reads "Add to review"; clicking it adds the row in-session marker (left inset, `--color-review-row`) and the commit appears in the ReviewPanel list.
result: [pending]

### 3. Remove from review from context menu
expected: Right-click a commit ALREADY in the session → menu item reads "Remove from review"; clicking it removes the in-session marker and the commit disappears from the ReviewPanel list.
result: [pending]

### 4. Merge commit is selectable (D-08)
expected: Right-click a MERGE commit → "Add to review" is ENABLED (not greyed out); clicking adds the merge to the session and it appears in the panel. The merge commit can also be used as a range base or range tip.
result: [pending]

### 5. Invalid range shows toast and leaves session unchanged
expected: Set a review base on commit A, then right-click an UNRELATED or SIBLING commit B (not a descendant of A) → "Add range to review" shows a toast error (e.g. "Base is not an ancestor of tip") and the session set is unchanged; the pending-base highlight clears either way.
result: [pending]

### 6. Clear review base cancels range gesture
expected: With a pending base set, right-click any commit → "Clear review base" item present; clicking it clears the pending-base highlight without seeding any range.
result: [pending]

### 7. ReviewPanel per-row remove button
expected: In the panel's commit list, each row has a × button; clicking it removes that commit from the session (row disappears from panel, in-session marker disappears from the CommitGraph row).
result: [pending]

### 8. Range result in panel is graph-ordered with no duplicates
expected: After seeding a range `[A..B]`, the ReviewPanel shows commits in graph order (newest first, matching CommitGraph order) with no duplicate entries.
result: [pending]

### 9. Pending-base highlight clears when session becomes inactive
expected: If a review session is closed while a pending base is set, the pending-base highlight on the CommitGraph row clears (no stale highlight with no active session).
result: [pending]

### 10. session-changed sync across multiple windows (optional)
expected: Open the same repo in two windows; add/remove a commit in one window; the other window reflects the change (in-session markers and panel list update) via the session-changed event.
result: [pending]

## Summary

total: 10
passed: 0
issues: 0
pending: 10
skipped: 0
blocked: 0

## Gaps
