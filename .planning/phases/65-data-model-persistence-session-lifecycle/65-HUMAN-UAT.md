---
status: partial
phase: 65-data-model-persistence-session-lifecycle
source: [65-VERIFICATION.md]
started: 2026-05-25T11:50:00Z
updated: 2026-05-25T11:50:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. End-to-End Lifecycle via View Menu

expected: The three D-12 states cycle correctly: none → active → (force-quit + reopen) resume-available. Clicking Resume shows the active state again. Clicking End from any state returns to none.

steps:
1. Open Trunk on a Git repository.
2. Open the View menu and click "Start/End Code Review". The ReviewPanel should appear showing "No code review session" with a "Start Code Review" button.
3. Click Start. The panel should change to the active state ("Code review in progress" with an "End Review" button).
4. Force-quit Trunk (`kill -9` or Activity Monitor force-quit) — NOT a graceful close.
5. Reopen Trunk on the same repository. The ReviewPanel should show "A saved review session is available" with Resume and Discard buttons (resume-available state).
6. Click Resume — the active state should return.
7. Click End — the session is cleared; reopening the repo shows the none state.
8. (SC-3 spot, optional) Open the same repo via a symlinked path — it should resume the same session, not a fresh one.

why_human: Full end-to-end cycle requires a running Tauri app. The force-quit durability guarantee — that an atomic write survives a kill signal — cannot be proven by unit tests alone, only by actually performing the force-quit scenario.

result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
