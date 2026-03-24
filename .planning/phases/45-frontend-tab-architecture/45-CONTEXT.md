# Phase 45: Frontend Tab Architecture - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can open multiple repositories as independent tabs within a single window. Each tab has fully isolated state (graph, staging, diffs, selection, rebase/merge). Covers TAB-01 through TAB-07: multi-tab open, Cmd+T new tab, Cmd+W close tab, Cmd+1-9/Ctrl+Tab switching, per-tab state isolation, tab persistence across relaunch, and dirty indicator.

</domain>

<decisions>
## Implementation Decisions

### Shared vs per-tab settings
- **D-01:** Zoom level is global — Cmd+/- changes zoom for the entire app, not per-tab.
- **D-02:** Pane widths (sidebar, right panel) are global — resizing applies across all tabs.
- **D-03:** Pane collapsed state is global — collapsing the sidebar collapses it for all tabs.

### Dirty indicator
- **D-04:** "Dirty" means staged + unstaged — any modified/added/deleted file (staged or unstaged) shows the dot badge on the tab.
- **D-05:** Detection via watcher events — the fs watcher already emits `repo-changed` events per repo path. Listen to those and update a dirty flag per tab. No polling needed.

### Tab overflow
- **D-06:** Scroll only — tabs keep their natural width, and the tab bar becomes horizontally scrollable when tabs don't fit. No shrinking.
- **D-07:** No tab limit — users can open as many tabs as they want.

### Tab switching strategy
- **D-08:** Keep-alive (hidden) — all mounted tabs stay in the DOM with `display: none` when inactive. Preserves scroll position, open diffs, and all transient UI state across tab switches.
- **D-09:** Restore selected commit — when switching back to a tab, the previously selected commit remains selected (natural consequence of keep-alive).

### From Phase 44 (carried forward)
- **D-10:** Normal tab close (X button) = graceful — let running remote ops finish naturally (Phase 44 D-02).
- **D-11:** Force close (Shift+click X) = cancel running op via SIGTERM before cleanup (Phase 44 D-03).
- **D-12:** Remote-progress events already carry repo path in payload (Phase 44 D-04) — frontend filters by active tab's repo path.

### Claude's Discretion
- How to structure per-tab state (component extraction pattern, context API vs props)
- How to isolate `remoteState` and `undoRedoState` per tab (currently global singletons)
- Tab bar component implementation details (scrolling mechanism, active tab indicator styling)
- Keyboard shortcut handler architecture for Cmd+T/W/1-9/Ctrl+Tab

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Frontend architecture
- `src/App.svelte` — Current monolithic app component; all repo state lives here as top-level `$state()`. This is the primary extraction target.
- `src/components/TabBar.svelte` — Current single-tab stub (repo name + close button). Needs full multi-tab rewrite.
- `src/lib/store.ts` — Persistence layer using `@tauri-apps/plugin-store`. Currently stores single `open_repo`; needs `open_tabs[]` + `active_tab`.

### Per-tab state (currently global singletons)
- `src/lib/remote-state.svelte.ts` — Global `$state({ isRunning, progressLine, error })`. Needs per-tab scoping.
- `src/lib/undo-redo.svelte.ts` — Global `$state({ redoStack })`. Needs per-tab scoping.

### Backend integration
- `src-tauri/src/commands/repo.rs` — `open_repo`, `close_repo`, `force_close_repo` (added in Phase 44)
- `src-tauri/src/commands/remote.rs` — Remote ops with per-repo RunningOp (Phase 44)
- `src-tauri/src/state.rs` — Per-repo state HashMap pattern

### Splash/project picker
- `src/components/WelcomeScreen.svelte` — Existing splash screen; Cmd+T should show this for new tabs

### Requirements
- `.planning/REQUIREMENTS.md` — TAB-01 through TAB-07 acceptance criteria

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **WelcomeScreen.svelte**: Already serves as project picker. Can be reused for new tab (Cmd+T) content.
- **store.ts (LazyStore)**: Persistence infrastructure ready. Just needs schema change from single repo to tab array.
- **`repo-changed` event listener**: Already debounced (200ms) and path-filtered in App.svelte. Pattern can drive dirty detection for background tabs.
- **`get_dirty_counts` command**: Returns `{ staged, unstaged, conflicted }` — already exists for the current single-repo view.

### Established Patterns
- **Svelte 5 runes**: All state uses `$state()`, `$derived()`, `$effect()`. Per-tab state should follow the same pattern.
- **`safeInvoke` for Tauri commands**: All backend calls go through `src/lib/invoke.ts`. Tab-aware code should continue using this.
- **CSS custom properties for theming**: All colors via `var(--color-*)`. Tab bar must follow this.
- **Lucide icons**: All icons use `@lucide/svelte`. Tab close button already uses `X` icon.

### Integration Points
- **App.svelte → RepoView extraction**: ~30 `$state` variables and ~15 handler functions need to move into a new RepoView component.
- **Keyboard shortcut handler**: Currently in App.svelte `$effect`. Needs tab-level shortcuts (Cmd+T/W/1-9) added at app level.
- **Tauri title bar**: The `data-tauri-drag-region` div contains TabBar + Toolbar. Multi-tab bar needs to fit in the same 32px height.

</code_context>

<specifics>
## Specific Ideas

- Keep-alive chosen specifically to preserve scroll position and open diffs when switching between repos — user wants zero context loss on tab switch.
- Scroll-only overflow (no shrinking) — tabs keep readable width, horizontal scroll handles excess.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 45-frontend-tab-architecture*
*Context gathered: 2026-03-23*
