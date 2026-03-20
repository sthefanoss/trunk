---
status: resolved
trigger: "Clicking a conflicted file in the Conflicted Files section of the staging panel shows no diff. The left panel stays completely empty."
created: 2026-03-20T00:00:00Z
updated: 2026-03-20T00:00:00Z
---

## Current Focus

hypothesis: diff_unstaged backend command returns empty array for conflicted files because git2's diff_index_to_workdir skips files with conflict index entries (stages 1/2/3 instead of stage 0)
test: traced full event chain from click to backend to rendering
expecting: confirmed -- empty FileDiff[] causes DiffPanel empty state
next_action: return diagnosis

## Symptoms

expected: Clicking a conflicted file should show a read-only diff (using diffKind='commit' to suppress hunk action buttons)
actual: Clicking a conflicted file shows no diff; the left panel stays completely empty (DiffPanel renders but shows "Select a file or commit to view its diff" placeholder)
errors: No error messages -- the invoke call succeeds but returns an empty array
reproduction: 1) Create a merge conflict (start merge/rebase with conflicting changes) 2) Open repo in Trunk 3) Click any file in the "Conflicted Files" section of the staging panel
started: Since initial implementation of conflict detection UI (37-02)

## Eliminated

(none -- root cause found on first hypothesis)

## Evidence

- timestamp: 2026-03-20T00:01:00Z
  checked: StagingPanel.svelte conflicted file click handler (line 287)
  found: Correctly calls onfileselect?.(f.path, 'conflicted') -- event dispatch is working
  implication: Frontend event chain starts correctly

- timestamp: 2026-03-20T00:02:00Z
  checked: App.svelte handleFileSelect (lines 120-138)
  found: When kind==='conflicted', sets selectedFile = { path, kind } then calls safeInvoke('diff_unstaged', { path: repoPath, filePath: path }). selectedFile is set before the async call, so showDiff becomes true and DiffPanel renders.
  implication: The invoke call is made correctly; DiffPanel does render (it's not a routing/visibility issue)

- timestamp: 2026-03-20T00:03:00Z
  checked: App.svelte DiffPanel props (line 413)
  found: diffKind is correctly computed as 'commit' when selectedFile?.kind === 'conflicted'. This part works as intended.
  implication: The read-only mode (suppressing hunk buttons) is wired correctly

- timestamp: 2026-03-20T00:04:00Z
  checked: Backend diff_unstaged_inner (diff.rs lines 93-105)
  found: Uses repo.diff_index_to_workdir(None, Some(&mut opts)) with pathspec filter. This is a standard git2 index-to-workdir diff.
  implication: This is where the chain breaks

- timestamp: 2026-03-20T00:05:00Z
  checked: git2 behavior for conflicted files in diff_index_to_workdir
  found: Conflicted files have higher-stage index entries (stages 1=base, 2=ours, 3=theirs) but NO stage-0 entry. diff_index_to_workdir only considers stage-0 entries. Therefore, conflicted files produce ZERO diff deltas -- the returned Vec<FileDiff> is empty.
  implication: ROOT CAUSE -- the diff command itself is structurally incapable of producing output for conflicted files

- timestamp: 2026-03-20T00:06:00Z
  checked: DiffPanel rendering with empty fileDiffs (DiffPanel.svelte line 357)
  found: When fileDiffs.length === 0 && commitDetail === null, renders empty state "Select a file or commit to view its diff"
  implication: Confirms the user-visible symptom -- panel shows but appears empty/shows placeholder

- timestamp: 2026-03-20T00:07:00Z
  checked: Backend get_status (staging.rs lines 38-89)
  found: Correctly detects conflicted files via Status::CONFLICTED flag and puts them in the conflicted array with a `continue` (skipping staged/unstaged classification). This is correct.
  implication: The status detection is fine; only the diff retrieval is broken

## Resolution

root_cause: The backend `diff_unstaged` command (diff.rs) uses git2's `repo.diff_index_to_workdir()` to compute diffs. For conflicted files, git stores conflict entries at index stages 1 (base), 2 (ours), and 3 (theirs) -- NOT at stage 0. Since `diff_index_to_workdir` only considers stage-0 entries, conflicted files produce zero diff deltas, returning an empty `Vec<FileDiff>`. The frontend correctly calls this command and renders the DiffPanel, but receives an empty array, causing the empty state placeholder to display instead of actual diff content.

fix: The backend needs a different diff strategy for conflicted files. Options:
  1. Use `repo.diff_tree_to_workdir()` (compare HEAD tree to workdir) -- this would show all changes from HEAD including conflict markers in the working copy
  2. Use `repo.index().conflicts()` to read the conflict entries and construct a custom diff showing the conflict markers
  3. Read the raw file content and display it as an "all additions" diff (since conflict markers are in the workdir file)
  Option 1 is the most straightforward: add a new backend command (e.g., `diff_conflicted`) or modify `diff_unstaged` to detect conflicted files and use `diff_tree_to_workdir` instead. The frontend's `handleFileSelect` (App.svelte line 128-130) would call this new/modified command.

verification: (not yet applied)
files_changed: []
