---
status: diagnosed
trigger: "squash commit has no dialog for editing combined squash message"
created: 2026-03-23T12:00:00Z
updated: 2026-03-23T12:00:00Z
---

## Current Focus

hypothesis: The signal/wait IPC mechanism (which showed a dialog when git paused for squash message editing) was deliberately removed in commit de9ea19. The replacement approach (numbered message files consumed by GIT_EDITOR) only works when the user has pre-written a message in the rebase editor BEFORE starting. For squash, the frontend explicitly blocks opening the message editor, so no pre-written message is ever provided. Git's GIT_EDITOR receives the combined squash message but auto-accepts it (exit 0 with no file to copy), silently using git's default combined message.
test: confirmed via code reading
expecting: confirmed
next_action: return diagnosis

## Symptoms

expected: When git pauses for squash message editing (GIT_EDITOR invoked), a dialog should appear showing the combined commit messages, allowing the user to edit before submitting.
actual: No dialog appears. The squash completes silently using git's default concatenated message. User must manually reword the resulting commit afterward.
errors: none (silent failure — feature gap, not crash)
reproduction: Set a commit to "squash" in the rebase editor, click Start Rebase. No squash message dialog appears.
started: Since commit de9ea19 removed the signal/wait mechanism.

## Eliminated

(none — root cause found on first hypothesis)

## Evidence

- timestamp: 2026-03-23T12:00:00Z
  checked: src-tauri/src/commands/interactive_rebase.rs (current state)
  found: GIT_EDITOR script consumes numbered files from msg-queue/ directory. If no file exists, it does nothing and exits 0 (accepting git's default message). There is no signal/response IPC, no poll loop, no event emission.
  implication: Squash messages cannot be interactively edited — they are auto-accepted.

- timestamp: 2026-03-23T12:00:00Z
  checked: src/components/RebaseEditor.svelte openMessageEditor()
  found: Line 285 — `if (item.action === 'drop' || item.action === 'squash') return;` — the inline message editor is explicitly blocked for squash actions. The ondblclick handler on line 435 also only opens for pick/reword.
  implication: Users cannot provide a pre-written squash message in the editor, so msg-queue/ will never have a file for the squash editor invocation.

- timestamp: 2026-03-23T12:00:00Z
  checked: src-tauri/src/commands/interactive_rebase.rs msg-queue writing (lines 139-151)
  found: Message files are only written when `item.new_message` is Some. For squash items where the editor is blocked, newMessage is always null.
  implication: Even though msg_index is incremented for squash items (line 143-149), no file is written, confirming the GIT_EDITOR will fall through to exit 0.

- timestamp: 2026-03-23T12:00:00Z
  checked: git history — commit de9ea19
  found: The original signal/wait mechanism (commit cfba929) used spawn + poll loop. GIT_EDITOR would touch a signal file, Rust would detect it and emit rebase-message-needed event, frontend would show InputDialog. This was all removed in de9ea19 because "the counter-based shell script with signal/wait fallback was causing the rebase to hang."
  implication: The feature EXISTED but was removed to fix a hang bug. The replacement approach (numbered files) only works for pre-written messages, creating a regression for squash (which needs interactive editing).

- timestamp: 2026-03-23T12:00:00Z
  checked: src/App.svelte (current state)
  found: No listener for rebase-message-needed event. No showMessageDialog state. No handleMessageDialogSubmit. No InputDialog for rebase messages. All removed in de9ea19.
  implication: Frontend has no mechanism to receive or display squash message editing prompts.

## Resolution

root_cause: The interactive squash message editing feature was removed in commit de9ea19 and never replaced. The original implementation used a file-based IPC mechanism: GIT_EDITOR shell script would touch a signal file, a Rust poll loop would detect it and emit a `rebase-message-needed` Tauri event to the frontend, which would show an InputDialog for the user to edit the combined squash message. This was replaced with a simpler approach (numbered pre-written message files) because the signal/wait mechanism was causing hangs. However, the replacement only works when messages are pre-written before the rebase starts. Since the RebaseEditor explicitly blocks the message editor for squash actions (line 285: `if (item.action === 'drop' || item.action === 'squash') return`), no pre-written message is ever provided for squash commits. The GIT_EDITOR script finds no queued file and exits 0, silently accepting git's default concatenated message without user interaction.

fix:
verification:
files_changed: []
