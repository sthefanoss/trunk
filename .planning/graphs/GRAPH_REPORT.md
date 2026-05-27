# Graph Report - trunk  (2026-05-14)

## Corpus Check
- 186 files · ~125,713 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1613 nodes · 2569 edges · 101 communities (93 shown, 8 thin omitted)
- Extraction: 87% EXTRACTED · 13% INFERRED · 0% AMBIGUOUS · INFERRED: 346 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `787867bb`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 27|Community 27]]
- [[_COMMUNITY_Community 28|Community 28]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 32|Community 32]]
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]
- [[_COMMUNITY_Community 35|Community 35]]
- [[_COMMUNITY_Community 36|Community 36]]
- [[_COMMUNITY_Community 37|Community 37]]
- [[_COMMUNITY_Community 38|Community 38]]
- [[_COMMUNITY_Community 39|Community 39]]
- [[_COMMUNITY_Community 40|Community 40]]
- [[_COMMUNITY_Community 41|Community 41]]
- [[_COMMUNITY_Community 42|Community 42]]
- [[_COMMUNITY_Community 43|Community 43]]
- [[_COMMUNITY_Community 44|Community 44]]
- [[_COMMUNITY_Community 45|Community 45]]
- [[_COMMUNITY_Community 46|Community 46]]
- [[_COMMUNITY_Community 47|Community 47]]
- [[_COMMUNITY_Community 48|Community 48]]
- [[_COMMUNITY_Community 49|Community 49]]
- [[_COMMUNITY_Community 50|Community 50]]
- [[_COMMUNITY_Community 51|Community 51]]
- [[_COMMUNITY_Community 52|Community 52]]
- [[_COMMUNITY_Community 53|Community 53]]
- [[_COMMUNITY_Community 55|Community 55]]
- [[_COMMUNITY_Community 56|Community 56]]
- [[_COMMUNITY_Community 57|Community 57]]
- [[_COMMUNITY_Community 58|Community 58]]
- [[_COMMUNITY_Community 59|Community 59]]
- [[_COMMUNITY_Community 60|Community 60]]
- [[_COMMUNITY_Community 61|Community 61]]
- [[_COMMUNITY_Community 62|Community 62]]
- [[_COMMUNITY_Community 63|Community 63]]
- [[_COMMUNITY_Community 64|Community 64]]
- [[_COMMUNITY_Community 65|Community 65]]
- [[_COMMUNITY_Community 66|Community 66]]
- [[_COMMUNITY_Community 67|Community 67]]
- [[_COMMUNITY_Community 68|Community 68]]
- [[_COMMUNITY_Community 69|Community 69]]
- [[_COMMUNITY_Community 70|Community 70]]
- [[_COMMUNITY_Community 71|Community 71]]
- [[_COMMUNITY_Community 72|Community 72]]
- [[_COMMUNITY_Community 73|Community 73]]
- [[_COMMUNITY_Community 74|Community 74]]
- [[_COMMUNITY_Community 75|Community 75]]
- [[_COMMUNITY_Community 76|Community 76]]
- [[_COMMUNITY_Community 77|Community 77]]
- [[_COMMUNITY_Community 78|Community 78]]
- [[_COMMUNITY_Community 79|Community 79]]
- [[_COMMUNITY_Community 80|Community 80]]

## God Nodes (most connected - your core abstractions)
1. `./CommitGraph.svelte` - 73 edges
2. `walk_commits()` - 59 edges
3. `ReactiveListManager` - 44 edges
4. `safeInvoke()` - 19 edges
5. `./OperationBanner.svelte` - 18 edges
6. `open_repo_from_state()` - 17 edges
7. `Git Worktrees in Trunk — Design` - 17 edges
8. `build_search_ctx()` - 16 edges
9. `system_path()` - 16 edges
10. `TestContext` - 15 edges

## Surprising Connections (you probably didn't know these)
- `bench_get_status()` --calls--> `get_status_inner()`  [INFERRED]
  src-tauri/benches/bench_commands.rs → src-tauri/src/commands/staging.rs
- `refresh_commit_graph()` --calls--> `walk_commits()`  [INFERRED]
  src-tauri/src/commands/history.rs → src-tauri/src/git/graph.rs
- `list_refs_ahead_behind_tracking()` --calls--> `list_refs_inner()`  [INFERRED]
  src-tauri/tests/test_branches.rs → src-tauri/src/commands/branches.rs
- `delete_remote_branch_removes_ref()` --calls--> `list_refs_inner()`  [INFERRED]
  src-tauri/tests/test_branches.rs → src-tauri/src/commands/branches.rs
- `classify_auth_failure_password()` --calls--> `classify_git_error()`  [INFERRED]
  src-tauri/tests/test_remote.rs → src-tauri/src/commands/remote.rs

## Communities (101 total, 8 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.06
Nodes (43): build_partial_patch_text(), classify_index(), classify_workdir(), DirtyCounts, discard_all(), discard_all_inner(), discard_file(), discard_file_inner() (+35 more)

### Community 1 - "Community 1"
Cohesion: 0.05
Nodes (48): bench_diff_code_file(), bench_diff_unstaged(), bench_enrich_new(), bench_get_status(), bench_list_refs(), bench_stage_hunk(), BenchRepo, make_repo_for_stage_hunk() (+40 more)

### Community 2 - "Community 2"
Cohesion: 0.06
Nodes (4): benchmarkListManager(), createListManager(), ReactiveListManager, RecomputeScheduler

### Community 3 - "Community 3"
Cohesion: 0.04
Nodes (8): DEFAULT_REBASE_VISIBILITY, DEFAULT_REBASE_WIDTHS, DEFAULT_VISIBILITY, DEFAULT_WIDTHS, RebaseColumnVisibility, RebaseColumnWidths, RecentRepo, store

### Community 4 - "Community 4"
Cohesion: 0.07
Nodes (39): buildTree(), collectFilePaths(), compress(), convert(), countFiles(), DirectoryNode, FileNode, IntermediateDir (+31 more)

### Community 5 - "Community 5"
Cohesion: 0.05
Nodes (39): actualIndex, atBottom, contentHeight, dirtyItems, dirtyItemsCount, displayItems, elementIndex, handleScroll() (+31 more)

### Community 6 - "Community 6"
Cohesion: 0.09
Nodes (41): clean_file_produces_empty_unstaged_diff(), commit_detail_includes_committer_fields(), commit_detail_returns_metadata(), diff_commit_respects_context_lines(), diff_commit_root_commit_shows_added_files(), diff_commit_succeeds_for_head(), diff_unstaged_ignores_indentation_whitespace(), diff_unstaged_ignores_whitespace_when_enabled() (+33 more)

### Community 7 - "Community 7"
Cohesion: 0.08
Nodes (26): amend_commit(), amend_commit_inner(), build_message(), create_commit(), create_commit_inner(), get_head_commit_message(), get_head_commit_message_inner(), open_repo_from_state() (+18 more)

### Community 8 - "Community 8"
Cohesion: 0.05
Nodes (38): addLines, addSpans, addTexts, binaryDiff, closeBtn, combinedAddSpans, combinedSpans, { container } (+30 more)

### Community 9 - "Community 9"
Cohesion: 0.06
Nodes (32): activeButton, sortable, activeTab, activeTabs, closeBtns, { container }, defaultProps, dirtyDots (+24 more)

### Community 10 - "Community 10"
Cohesion: 0.06
Nodes (24): [], checkoutLocalBranch(), checkoutRemoteBranch(), diff, displayItems, graphData, handleRefCheckout(), handleRowContextMenu() (+16 more)

### Community 11 - "Community 11"
Cohesion: 0.07
Nodes (36): create_add_delete_hunk_file(), create_multi_hunk_file(), dirty_counts_includes_untracked(), discard_all_deletes_nested_untracked_file(), discard_all_reverts_all_changes(), discard_file_deletes_untracked_file(), discard_file_deletes_untracked_file_in_subdirectory(), discard_file_reverts_tracked_modification() (+28 more)

### Community 12 - "Community 12"
Cohesion: 0.1
Nodes (24): bench_ipc_get_graph(), bench_ipc_list_refs(), bench_startup_sequence(), BenchRepo, make_linear_repo(), checkout_branch(), checkout_branch_inner(), create_branch() (+16 more)

### Community 13 - "Community 13"
Cohesion: 0.06
Nodes (30): closeBtn, detail, detailWithBody, fileDiffs, onclose, BranchInfo, CommitDetail, ContentMode (+22 more)

### Community 14 - "Community 14"
Cohesion: 0.06
Nodes (30): BranchInfo, CommitDetail, DiffHunk, DiffLine, DiffOrigin, DiffRequestOptions, DiffStatus, EdgeType (+22 more)

### Community 15 - "Community 15"
Cohesion: 0.06
Nodes (32): 10. Filesystem Watching, 1. Open Repository, 2. Commit History & Graph, 3. Branch List (Sidebar), 4. Working Tree Status, 5. Stage / Unstage Files, 6. Create Commit, 7. File Diffs (+24 more)

### Community 16 - "Community 16"
Cohesion: 0.15
Nodes (29): find_free_column_near(), walk_commits(), branch_fork_topology(), color_index_deterministic(), color_index_head_zero(), consistent_max_columns(), freed_column_reuse(), is_merge_flag() (+21 more)

### Community 17 - "Community 17"
Cohesion: 0.09
Nodes (25): handleNextConflict(), handlePrevConflict(), handleReset(), handleSaveAndResolve(), scrollToConflict(), buildLineIndex(), computeOutput(), ConflictRegion (+17 more)

### Community 18 - "Community 18"
Cohesion: 0.11
Nodes (21): check_undo_available(), check_undo_available_inner(), checkout_commit(), checkout_commit_inner(), cherry_pick(), cherry_pick_inner(), create_tag(), create_tag_inner() (+13 more)

### Community 19 - "Community 19"
Cohesion: 0.11
Nodes (21): extract_merge_source(), find_branch_color(), get_operation_state(), get_operation_state_inner(), merge_abort(), merge_abort_inner(), merge_branch(), merge_branch_inner() (+13 more)

### Community 20 - "Community 20"
Cohesion: 0.08
Nodes (18): { container }, mockInvoke, MockLazyStore, store, TEST_COMMITS, pill, refs, makeCommit() (+10 more)

### Community 21 - "Community 21"
Cohesion: 0.08
Nodes (16): canStart, editingBody, editingSummary, handleEditorKeydown(), handleMessageCancel(), hasChanges, lastVisibleColumn, openMessageEditor() (+8 more)

### Community 22 - "Community 22"
Cohesion: 0.08
Nodes (17): { container }, leftPane, mockInstance, mockInvoke, MockLazyStore, MockSortable, store, RemoteState (+9 more)

### Community 23 - "Community 23"
Cohesion: 0.14
Nodes (20): classify_git_error(), delete_remote_branch(), git_fetch(), git_fetch_background(), git_pull(), git_push(), refresh_graph(), run_git_remote() (+12 more)

### Community 24 - "Community 24"
Cohesion: 0.08
Nodes (23): 10. Remove flow — dirty guard, 11. Atomic create-from-commit — rollback, 12. Main worktree synthesis, 13. Error handling, 14. Testing strategy, 15. Observability, 16. Open questions / future work, 1. Summary (+15 more)

### Community 25 - "Community 25"
Cohesion: 0.11
Nodes (16): if(), showHeaderContextMenu(), newWidth, options, @tauri-apps/api/event, @tauri-apps/api/menu, @tauri-apps/plugin-clipboard-manager, [] (+8 more)

### Community 26 - "Community 26"
Cohesion: 0.13
Nodes (18): cancel_remote_op(), close_repo(), force_close_repo(), open_repo(), run(), main(), CommitCache, kill_process() (+10 more)

### Community 27 - "Community 27"
Cohesion: 0.13
Nodes (12): DEFAULT_GRAPH_SETTINGS, buildOverlayPaths(), buildPath(), isHollowTip(), makePathContext(), PathContext, conn, conns (+4 more)

### Community 28 - "Community 28"
Cohesion: 0.12
Nodes (14): emptySnippet, oncreate, ontoggle, item, ontoggle, current, dirPaths, migrated (+6 more)

### Community 29 - "Community 29"
Cohesion: 0.13
Nodes (12): buildRefPillData(), estimateBadgeWidth(), isRemoteOnlyRef(), sortRefs(), commits, nodes, ref, refs (+4 more)

### Community 30 - "Community 30"
Cohesion: 0.13
Nodes (11): TestContext, amend_commit_includes_newly_staged_files(), amend_commit_updates_message(), create_commit_creates_new_commit(), create_commit_uses_configured_signature(), create_commit_works_on_unborn_head(), get_head_commit_message_returns_subject_and_body(), staged_file_on_unborn_head_produces_diff() (+3 more)

### Community 31 - "Community 31"
Cohesion: 0.11
Nodes (12): { container }, mockInvoke, refsWithRemote, remoteBranchRow, buttons, defaultProps, tabLabels, { container } (+4 more)

### Community 32 - "Community 32"
Cohesion: 0.13
Nodes (3): BuildStep, TestContextBuilder, main()

### Community 33 - "Community 33"
Cohesion: 0.12
Nodes (13): handleAbort(), handleContinue(), handleSkip(), isMerge, isRebase, label, loading, sourceBranch (+5 more)

### Community 34 - "Community 34"
Cohesion: 0.18
Nodes (11): button, renderDropdown(), name, mockInvoke, result, TrunkError, createRemoteState(), a (+3 more)

### Community 35 - "Community 35"
Cohesion: 0.17
Nodes (9): frameTimestamps, isPerfEnabled(), measureAsync(), measureSync(), metrics, perfMetrics, record(), recordDuration() (+1 more)

### Community 36 - "Community 36"
Cohesion: 0.12
Nodes (10): { container }, mockInstance, mockInvoke, MockLazyStore, MockSortable, onclose, pickOptions, store (+2 more)

### Community 37 - "Community 37"
Cohesion: 0.16
Nodes (11): getVisibleOverlayElements(), node, path, paths, pills, result, VisibleOverlayElements, OverlayNode (+3 more)

### Community 38 - "Community 38"
Cohesion: 0.24
Nodes (15): author_match(), author_match_case_insensitive(), build_search_ctx(), empty_query_returns_empty(), message_body_none_does_not_crash(), message_match_case_insensitive(), message_summary_match(), multi_field_match() (+7 more)

### Community 39 - "Community 39"
Cohesion: 0.13
Nodes (11): segments, gutterW, [], allLines, gw, maxLn, segments, sliced (+3 more)

### Community 40 - "Community 40"
Cohesion: 0.15
Nodes (10): _cache, MeasureFn, measureTextWidth(), resetCache(), big, first, result, second (+2 more)

### Community 41 - "Community 41"
Cohesion: 0.13
Nodes (14): checkout_clean_workdir_succeeds(), checkout_with_non_conflicting_changes_succeeds(), create_branch_dirty_workdir_returns_error(), create_branch_duplicate_fails(), create_branch_from_head(), create_branch_from_specific_oid(), delete_branch_removes_ref(), delete_head_branch_fails() (+6 more)

### Community 42 - "Community 42"
Cohesion: 0.13
Nodes (14): state_transition_cherry_pick_conflict(), state_transition_fast_forward_merge(), state_transition_merge_conflict_abort(), state_transition_merge_conflict_resolve_commit(), state_transition_rebase_conflict_abort(), state_transition_rebase_conflict_skip(), workflow_branch_commit_merge(), workflow_cherry_pick_from_branch() (+6 more)

### Community 43 - "Community 43"
Cohesion: 0.19
Nodes (14): handleCherryPick(), handleDeleteBranch(), handleDeleteRemoteBranch(), handleDeleteTag(), handleInteractiveRebaseBranch(), handleMergeBranch(), handleRebaseBranch(), handleReset() (+6 more)

### Community 44 - "Community 44"
Cohesion: 0.19
Nodes (9): status, statuses, _resetToasts(), found, ids, uniqueIds, Toast, ToastKind (+1 more)

### Community 45 - "Community 45"
Cohesion: 0.23
Nodes (10): get_fork_point(), get_fork_point_inner(), get_rebase_todo(), get_rebase_todo_inner(), open_repo(), RebaseTodoAction, start_interactive_rebase(), start_interactive_rebase_blocking() (+2 more)

### Community 46 - "Community 46"
Cohesion: 0.18
Nodes (8): updateHeight(), calculateAverageHeightDebounced(), buildBlockSums(), calculateAverageHeight(), calculateTransformY(), calculateVisibleRange(), getValidHeight(), updateHeightAndScroll()

### Community 47 - "Community 47"
Cohesion: 0.26
Nodes (12): build_two_commit_ctx(), checkout_commit_detaches_head(), checkout_commit_dirty_workdir_fails(), cherry_pick_applies_commit(), create_tag_annotated(), create_tag_duplicate_fails(), create_tag_empty_message_uses_name(), delete_tag_removes_ref() (+4 more)

### Community 48 - "Community 48"
Cohesion: 0.41
Nodes (7): openRepo(), waitForBranchSidebar(), waitForCommitGraph(), cleanupRepo(), createBranchRepo(), createDirtyRepo(), createLinearRepo()

### Community 49 - "Community 49"
Cohesion: 0.17
Nodes (9): arrows, { container }, onclick, buttons, { container }, defaultProps, oncheckout, ondblclick (+1 more)

### Community 50 - "Community 50"
Cohesion: 0.17
Nodes (11): code:bash (just              # List all recipes), code:block2 (new-project → [per phase: discuss → plan → execute → verify]), Commands, Get Shit Done (GSD), Key commands, Navigation, Rules, Stack (+3 more)

### Community 51 - "Community 51"
Cohesion: 0.18
Nodes (9): buttons, defaultProps, disabledButtons, input, onclose, onnext, onprev, onquerychange (+1 more)

### Community 52 - "Community 52"
Cohesion: 0.18
Nodes (10): fileRow, files, fileText, items, list, onfileaction, onfileclick, { rerender } (+2 more)

### Community 53 - "Community 53"
Cohesion: 0.18
Nodes (9): diff, allVisible, commit, { container }, defaultWidths, italicEl, onselect, ColumnVisibility (+1 more)

### Community 55 - "Community 55"
Cohesion: 0.25
Nodes (5): GraphResponse, refresh_commit_graph(), search_commits(), search_commits_inner(), TestContext

### Community 56 - "Community 56"
Cohesion: 0.28
Nodes (3): from_path_helper(), parse_path_helper_output(), resolve()

### Community 57 - "Community 57"
Cohesion: 0.36
Nodes (6): get_merge_sides(), get_merge_sides_inner(), open_repo_from_state(), save_merge_result(), save_merge_result_inner(), TestContext

### Community 58 - "Community 58"
Cohesion: 0.22
Nodes (8): currentValues, string, confirmBtn, defaultProps, input, oncancel, onsubmit, ./InputDialog.svelte

### Community 59 - "Community 59"
Cohesion: 0.53
Nodes (8): scroll(), alignToEdge(), alignVisibleToNearestEdge(), calculateBottomToTopScrollTarget(), calculateScrollTarget(), calculateTopToBottomScrollTarget(), clampValue(), getScrollOffsetForIndex()

### Community 60 - "Community 60"
Cohesion: 0.22
Nodes (8): clean_repo_returns_none_operation_type(), merge_abort_clears_merge_state(), merge_branch_fast_forward_when_linear(), merge_branch_non_conflicting_creates_merge_commit(), merge_branch_with_conflict_returns_error(), merge_in_progress_reports_merge_state(), rebase_abort_clears_rebase_state(), rebase_branch_with_no_conflicts_completes()

### Community 61 - "Community 61"
Cohesion: 0.25
Nodes (7): 1. Repository Opening, 2. Commit History Browsing (E2E-02), 3. Staging and Committing (E2E-03), 4. Branch Operations (E2E-04), 5. General, macOS Pre-Release E2E Validation Checklist, Prerequisites

### Community 62 - "Community 62"
Cohesion: 0.25
Nodes (4): { container }, mockInvoke, MockLazyStore, store

### Community 63 - "Community 63"
Cohesion: 0.25
Nodes (7): list_stashes_returns_parent_oid(), stash_apply_keeps_entry(), stash_drop_removes_entry_without_restoring(), stash_pop_removes_entry_and_restores_changes(), stash_save_creates_entry(), stash_save_on_clean_workdir_returns_error(), stash_save_with_empty_message_uses_default()

### Community 64 - "Community 64"
Cohesion: 0.29
Nodes (4): pullBtn, pushBtn, redoBtn, remoteState

### Community 65 - "Community 65"
Cohesion: 0.29
Nodes (6): builder_creates_binary_file(), builder_creates_branch_and_merge(), builder_creates_conflict_state(), builder_creates_repo_with_initial_commit(), builder_creates_tag(), status_clean_after_commit()

### Community 66 - "Community 66"
Cohesion: 0.52
Nodes (6): get_fork_point_returns_merge_base(), get_rebase_todo_inclusive_includes_base_commit(), get_rebase_todo_item_has_correct_fields(), get_rebase_todo_returns_commits_oldest_first(), get_rebase_todo_returns_empty_when_base_equals_head(), make_three_commit_ctx()

### Community 67 - "Community 67"
Cohesion: 0.33
Nodes (3): binaryPath, config, __dirname

### Community 68 - "Community 68"
Cohesion: 0.33
Nodes (6): onMouseUp(), setLeftPaneCollapsed(), setLeftPaneWidth(), setRightPaneCollapsed(), setZoomLevel(), handleKeydown()

### Community 69 - "Community 69"
Cohesion: 0.4
Nodes (3): build_ref_map(), ref_map_head(), ref_map_stash()

### Community 70 - "Community 70"
Cohesion: 0.4
Nodes (4): { container }, MERGE_SIDES, mockInvoke, onclose

### Community 71 - "Community 71"
Cohesion: 0.67
Nodes (3): bench_walk_commits(), BenchRepo, make_linear_repo()

### Community 74 - "Community 74"
Cohesion: 0.5
Nodes (3): get_merge_sides_no_ancestor_returns_empty_base(), get_merge_sides_returns_conflict_content(), save_merge_result_writes_and_stages()

## Knowledge Gaps
- **453 isolated node(s):** `BuildStep`, `TestContext`, `BenchRepo`, `BenchRepo`, `BenchRepo` (+448 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **8 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `walk_commits()` connect `Community 16` to `Community 69`, `Community 7`, `Community 71`, `Community 12`, `Community 45`, `Community 18`, `Community 19`, `Community 23`, `Community 55`, `Community 57`, `Community 26`?**
  _High betweenness centrality (0.478) - this node is a cross-community bridge._
- **Why does `run()` connect `Community 26` to `Community 25`?**
  _High betweenness centrality (0.415) - this node is a cross-community bridge._
- **Why does `open_repo()` connect `Community 26` to `Community 16`, `Community 7`?**
  _High betweenness centrality (0.399) - this node is a cross-community bridge._
- **Are the 57 inferred relationships involving `walk_commits()` (e.g. with `linear_topology()` and `merge_commit_edges()`) actually correct?**
  _`walk_commits()` has 57 INFERRED edges - model-reasoned connections that need verification._
- **What connects `BuildStep`, `TestContext`, `BenchRepo` to the rest of the system?**
  _453 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.06 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.05 - nodes in this community are weakly interconnected._