---
status: partial
phase: 73-review-lifecycle-end-review-cold-boot-resume
source: [73-VERIFICATION.md]
started: 2026-05-27T18:00:00Z
updated: 2026-05-27T18:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Cold-boot resume across actual app restart
expected: After `pnpm tauri dev` on a repo that already has an on-disk session, opening the ReviewPanel for the first time shows the existing comments without any manual mutation (no toolbar toggle, no Add Note). Closure proof for Bug 3 from 72-VERIFICATION.md.
result: [pending]

### 2. Multi-tab End → cold state in other window
expected: Open two real Tauri windows on the same repo. Click End review (twice to confirm) in window A. Window B re-renders to the cold empty state ("No active review") without manual reload.
result: [pending]

### 3. Visual treatment of two-step End button danger color
expected: Click End review once — label flips to "Click again to confirm" and the button background tints with the danger color (`--color-danger-bg`/`--color-danger-border`). Wait ~3s — button reverts to idle "End review". Hover while confirming — heightened danger contrast.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
