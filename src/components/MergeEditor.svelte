<script lang="ts">
  import { safeInvoke, type TrunkError } from '../lib/invoke.js';
  import { showToast } from '../lib/toast.svelte.js';
  import type { MergeSides } from '../lib/types.js';
  import {
    parseConflictRegions,
    computeOutput,
    takeAllCurrent,
    takeAllIncoming,
    toggleHunk,
    toggleLine,
    getConflictIndices,
    type ConflictRegion,
  } from '../lib/merge-parser.js';
  import { Check, CircleCheck, CircleX, ChevronUp, ChevronDown, X } from '@lucide/svelte';

  interface Props {
    repoPath: string;
    filePath: string;
    onclose: () => void;
    onresolved: () => void;
  }

  let { repoPath, filePath, onclose, onresolved }: Props = $props();

  // ---------- State ----------
  let regions = $state<ConflictRegion[]>([]);
  let takenLines = $state<Set<string>>(new Set());
  let manualEdit = $state(false);
  let manualText = $state('');
  let loading = $state(true);
  let error = $state<string | null>(null);
  let focusedConflictIdx = $state(0);
  let saving = $state(false);

  let panelRefs: HTMLElement[] = [];

  // ---------- Derived ----------
  let conflictIndices = $derived(getConflictIndices(regions));
  let outputText = $derived.by(() => {
    if (manualEdit) return manualText;
    return computeOutput(regions, takenLines);
  });
  let hasPrev = $derived(focusedConflictIdx > 0);
  let hasNext = $derived(focusedConflictIdx < conflictIndices.length - 1);
  let hasConflicts = $derived(conflictIndices.length > 0);

  // ---------- Data loading ----------
  $effect(() => {
    // Re-run when filePath changes
    const currentPath = filePath;
    loading = true;
    error = null;

    safeInvoke<MergeSides>('get_merge_sides', { path: repoPath, filePath: currentPath })
      .then((result) => {
        regions = parseConflictRegions(result.base, result.ours, result.theirs);
        takenLines = new Set();
        manualEdit = false;
        manualText = '';
        focusedConflictIdx = 0;
        loading = false;
      })
      .catch((e) => {
        const err = e as TrunkError;
        error = err.message ?? 'Failed to load merge sides';
        loading = false;
      });
  });

  // ---------- Synchronized scroll ----------
  let scrolling = false;

  function handleScroll(sourceIdx: number) {
    if (scrolling) return;
    scrolling = true;
    const source = panelRefs[sourceIdx];
    if (!source) { scrolling = false; return; }
    const scrollTop = source.scrollTop;
    panelRefs.forEach((el, i) => {
      if (el && i !== sourceIdx) el.scrollTop = scrollTop;
    });
    requestAnimationFrame(() => { scrolling = false; });
  }

  // ---------- Event handlers ----------
  function handleTakeAllCurrent() {
    takenLines = takeAllCurrent(regions);
    manualEdit = false;
  }

  function handleTakeAllIncoming() {
    takenLines = takeAllIncoming(regions);
    manualEdit = false;
  }

  function handleToggleHunk(side: 'ours' | 'theirs', regionIdx: number) {
    takenLines = toggleHunk(side, regionIdx, regions, takenLines);
    manualEdit = false;
  }

  function handleToggleLine(key: string) {
    takenLines = toggleLine(key, takenLines);
    manualEdit = false;
  }

  function handleOutputEdit(e: Event) {
    manualEdit = true;
    manualText = (e.target as HTMLTextAreaElement).value;
  }

  function handlePrevConflict() {
    if (!hasPrev) return;
    focusedConflictIdx--;
    scrollToConflict(focusedConflictIdx);
  }

  function handleNextConflict() {
    if (!hasNext) return;
    focusedConflictIdx++;
    scrollToConflict(focusedConflictIdx);
  }

  function scrollToConflict(idx: number) {
    const regionIndex = conflictIndices[idx];
    if (regionIndex == null) return;
    for (const panel of panelRefs) {
      if (!panel) continue;
      const el = panel.querySelector(`[data-region-idx="${regionIndex}"]`);
      el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }

  async function handleSaveAndResolve() {
    saving = true;
    try {
      await safeInvoke('save_merge_result', {
        path: repoPath,
        filePath,
        content: outputText,
      });
      showToast('Resolved ' + filePath, 'success');
      onresolved();
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Save failed', 'error');
    } finally {
      saving = false;
    }
  }

  // ---------- Helpers ----------
  /** Check if all lines from one side of a conflict region are taken */
  function isHunkAllTaken(side: 'ours' | 'theirs', regionIdx: number): boolean {
    const region = regions[regionIdx];
    if (!region || region.type !== 'conflict') return false;
    const lines = side === 'ours' ? region.oursLines : region.theirsLines;
    if (lines.length === 0) return false;
    return lines.every((_, j) => takenLines.has(`${side}-${regionIdx}-${j}`));
  }

  /** Track conflict number (1-indexed among conflict regions only) */
  function conflictNumber(regionIdx: number): number {
    let count = 0;
    for (let i = 0; i <= regionIdx; i++) {
      if (regions[i]?.type === 'conflict') count++;
    }
    return count;
  }

  /** Compute cumulative line number for a side at a given region/line index */
  function lineNumber(side: 'ours' | 'theirs', regionIdx: number, lineIdx: number): number {
    let num = 1;
    for (let i = 0; i < regionIdx; i++) {
      const r = regions[i];
      const lines = side === 'ours' ? r.oursLines : r.theirsLines;
      num += lines.length;
    }
    return num + lineIdx;
  }
</script>

<div style="
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-bg);
">
  {#if loading}
    <!-- Loading state -->
    <div style="
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--color-text-muted);
      font-size: 13px;
    ">
      Loading merge editor...
    </div>
  {:else if error}
    <!-- Error state -->
    <div style="
      flex: 1;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 8px;
      color: var(--color-text-muted);
      font-size: 13px;
    ">
      <span style="color: var(--color-diff-delete);">{error}</span>
      <button
        onclick={() => { loading = true; error = null; safeInvoke<MergeSides>('get_merge_sides', { path: repoPath, filePath }).then((result) => { regions = parseConflictRegions(result.base, result.ours, result.theirs); takenLines = new Set(); manualEdit = false; manualText = ''; focusedConflictIdx = 0; loading = false; }).catch((e) => { error = (e as TrunkError).message ?? 'Failed to load'; loading = false; }); }}
        style="
          background: var(--color-surface);
          border: 1px solid var(--color-border);
          border-radius: 3px;
          color: var(--color-text);
          font-size: 12px;
          padding: 4px 12px;
          cursor: pointer;
        "
      >Retry</button>
    </div>
  {:else}
    <!-- Top row: Current + Incoming side by side (50% height) -->
    <div style="flex: 1; display: flex; min-height: 0;">

      <!-- Current (Ours) Panel -->
      <div style="flex: 1; display: flex; flex-direction: column; min-width: 0; border-right: 1px solid var(--color-border);">
        <!-- Header -->
        <div style="
          height: 28px;
          background: var(--color-merge-current-header);
          border-bottom: 1px solid var(--color-merge-current-border);
          display: flex;
          align-items: center;
          padding: 0 8px;
          gap: 8px;
          flex-shrink: 0;
        ">
          <span style="font-size: 12px; color: var(--color-text);">Current (Ours)</span>
          <span style="flex: 1;"></span>
          <button
            onclick={handleTakeAllCurrent}
            style="
              background: var(--color-btn-continue-bg);
              border: 1px solid var(--color-btn-continue-border);
              border-radius: 3px;
              color: var(--color-btn-continue);
              font-size: 11px;
              font-family: var(--font-sans, sans-serif);
              padding: 2px 8px;
              cursor: pointer;
              white-space: nowrap;
              flex-shrink: 0;
            "
          >Take All Current</button>
        </div>

        <!-- Scrollable content -->
        <div
          bind:this={panelRefs[0]}
          onscroll={() => handleScroll(0)}
          style="
            flex: 1;
            overflow-y: auto;
            font-family: var(--font-mono);
            font-size: 12px;
            line-height: 18px;
          "
        >
          {#each regions as region, regionIdx}
            {#if region.type === 'conflict'}
              <!-- Hunk header row -->
              <div
                data-region-idx={regionIdx}
                onclick={() => handleToggleHunk('ours', regionIdx)}
                style="
                  width: 100%;
                  height: 24px;
                  background: var(--color-surface);
                  border-top: 1px solid var(--color-border);
                  border-bottom: 1px solid var(--color-border);
                  display: flex;
                  align-items: center;
                  padding: 0 8px;
                  gap: 4px;
                  cursor: pointer;
                  font-size: 11px;
                  color: var(--color-text-muted);
                "
              >
                {#if isHunkAllTaken('ours', regionIdx)}
                  <Check size={14} style="color: var(--color-merge-taken-check);" />
                {:else}
                  <span style="width: 14px; height: 14px; display: inline-block;"></span>
                {/if}
                Conflict {conflictNumber(regionIdx)}
              </div>

              <!-- Ours lines -->
              {#each region.oursLines as line, lineIdx}
                {@const key = `ours-${regionIdx}-${lineIdx}`}
                {@const taken = takenLines.has(key)}
                <div
                  onclick={() => handleToggleLine(key)}
                  class="merge-line"
                  style="
                    display: flex;
                    background: var(--color-diff-add-bg);
                    opacity: {taken ? 1 : 'var(--color-merge-dimmed)'};
                    cursor: pointer;
                  "
                >
                  <!-- Line number gutter -->
                  <span style="
                    width: 48px;
                    flex-shrink: 0;
                    text-align: right;
                    padding-right: 8px;
                    color: var(--color-text-muted);
                    -webkit-user-select: none;
                    user-select: none;
                  ">{lineNumber('ours', regionIdx, lineIdx)}</span>
                  <!-- Check icon gutter -->
                  <span style="
                    width: 20px;
                    flex-shrink: 0;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                  " class="icon-gutter">
                    {#if taken}
                      <span class="taken-icon"><Check size={14} style="color: var(--color-merge-taken-check);" /></span>
                      <span class="remove-icon"><CircleX size={14} style="color: var(--color-merge-remove-icon);" /></span>
                    {:else}
                      <span class="untaken-icon"><CircleCheck size={14} style="color: var(--color-text-muted); opacity: 0.4;" /></span>
                    {/if}
                  </span>
                  <!-- Line content -->
                  <span style="
                    padding-left: 4px;
                    white-space: pre;
                    overflow-x: auto;
                    flex: 1;
                    min-width: 0;
                  ">{line}</span>
                </div>
              {/each}
            {:else}
              <!-- Context lines -->
              {#each region.oursLines as line, lineIdx}
                <div style="
                  display: flex;
                  background: transparent;
                ">
                  <span style="
                    width: 48px;
                    flex-shrink: 0;
                    text-align: right;
                    padding-right: 8px;
                    color: var(--color-text-muted);
                    -webkit-user-select: none;
                    user-select: none;
                  ">{lineNumber('ours', regionIdx, lineIdx)}</span>
                  <span style="width: 20px; flex-shrink: 0;"></span>
                  <span style="
                    padding-left: 4px;
                    white-space: pre;
                    overflow-x: auto;
                    flex: 1;
                    min-width: 0;
                    color: var(--color-text);
                  ">{line}</span>
                </div>
              {/each}
            {/if}
          {/each}
        </div>
      </div>

      <!-- Incoming (Theirs) Panel -->
      <div style="flex: 1; display: flex; flex-direction: column; min-width: 0;">
        <!-- Header -->
        <div style="
          height: 28px;
          background: var(--color-merge-incoming-header);
          border-bottom: 1px solid var(--color-merge-incoming-border);
          display: flex;
          align-items: center;
          padding: 0 8px;
          gap: 8px;
          flex-shrink: 0;
        ">
          <span style="font-size: 12px; color: var(--color-text);">Incoming (Theirs)</span>
          <span style="flex: 1;"></span>
          <button
            onclick={handleTakeAllIncoming}
            style="
              background: var(--color-btn-continue-bg);
              border: 1px solid var(--color-btn-continue-border);
              border-radius: 3px;
              color: var(--color-btn-continue);
              font-size: 11px;
              font-family: var(--font-sans, sans-serif);
              padding: 2px 8px;
              cursor: pointer;
              white-space: nowrap;
              flex-shrink: 0;
            "
          >Take All Incoming</button>
        </div>

        <!-- Scrollable content -->
        <div
          bind:this={panelRefs[1]}
          onscroll={() => handleScroll(1)}
          style="
            flex: 1;
            overflow-y: auto;
            font-family: var(--font-mono);
            font-size: 12px;
            line-height: 18px;
          "
        >
          {#each regions as region, regionIdx}
            {#if region.type === 'conflict'}
              <!-- Hunk header row -->
              <div
                data-region-idx={regionIdx}
                onclick={() => handleToggleHunk('theirs', regionIdx)}
                style="
                  width: 100%;
                  height: 24px;
                  background: var(--color-surface);
                  border-top: 1px solid var(--color-border);
                  border-bottom: 1px solid var(--color-border);
                  display: flex;
                  align-items: center;
                  padding: 0 8px;
                  gap: 4px;
                  cursor: pointer;
                  font-size: 11px;
                  color: var(--color-text-muted);
                "
              >
                {#if isHunkAllTaken('theirs', regionIdx)}
                  <Check size={14} style="color: var(--color-merge-taken-check);" />
                {:else}
                  <span style="width: 14px; height: 14px; display: inline-block;"></span>
                {/if}
                Conflict {conflictNumber(regionIdx)}
              </div>

              <!-- Theirs lines -->
              {#each region.theirsLines as line, lineIdx}
                {@const key = `theirs-${regionIdx}-${lineIdx}`}
                {@const taken = takenLines.has(key)}
                <div
                  onclick={() => handleToggleLine(key)}
                  class="merge-line"
                  style="
                    display: flex;
                    background: var(--color-diff-delete-bg);
                    opacity: {taken ? 1 : 'var(--color-merge-dimmed)'};
                    cursor: pointer;
                  "
                >
                  <!-- Line number gutter -->
                  <span style="
                    width: 48px;
                    flex-shrink: 0;
                    text-align: right;
                    padding-right: 8px;
                    color: var(--color-text-muted);
                    -webkit-user-select: none;
                    user-select: none;
                  ">{lineNumber('theirs', regionIdx, lineIdx)}</span>
                  <!-- Check icon gutter -->
                  <span style="
                    width: 20px;
                    flex-shrink: 0;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                  " class="icon-gutter">
                    {#if taken}
                      <span class="taken-icon"><Check size={14} style="color: var(--color-merge-taken-check);" /></span>
                      <span class="remove-icon"><CircleX size={14} style="color: var(--color-merge-remove-icon);" /></span>
                    {:else}
                      <span class="untaken-icon"><CircleCheck size={14} style="color: var(--color-text-muted); opacity: 0.4;" /></span>
                    {/if}
                  </span>
                  <!-- Line content -->
                  <span style="
                    padding-left: 4px;
                    white-space: pre;
                    overflow-x: auto;
                    flex: 1;
                    min-width: 0;
                  ">{line}</span>
                </div>
              {/each}
            {:else}
              <!-- Context lines -->
              {#each region.theirsLines as line, lineIdx}
                <div style="
                  display: flex;
                  background: transparent;
                ">
                  <span style="
                    width: 48px;
                    flex-shrink: 0;
                    text-align: right;
                    padding-right: 8px;
                    color: var(--color-text-muted);
                    -webkit-user-select: none;
                    user-select: none;
                  ">{lineNumber('theirs', regionIdx, lineIdx)}</span>
                  <span style="width: 20px; flex-shrink: 0;"></span>
                  <span style="
                    padding-left: 4px;
                    white-space: pre;
                    overflow-x: auto;
                    flex: 1;
                    min-width: 0;
                    color: var(--color-text);
                  ">{line}</span>
                </div>
              {/each}
            {/if}
          {/each}
        </div>
      </div>
    </div>

    <!-- Bottom panel: Output (50% height) -->
    <div style="flex: 1; display: flex; flex-direction: column; min-height: 0; border-top: 1px solid var(--color-border);">
      <!-- Header -->
      <div style="
        height: 28px;
        background: var(--color-merge-output-header);
        border-bottom: 1px solid var(--color-merge-output-border);
        display: flex;
        align-items: center;
        padding: 0 8px;
        gap: 8px;
        flex-shrink: 0;
      ">
        <span style="font-size: 12px; color: var(--color-text);">Output</span>
        {#if manualEdit}
          <span style="font-size: 10px; color: var(--color-text-muted);">(manual edit)</span>
        {/if}
        <span style="flex: 1;"></span>

        {#if hasConflicts}
          <!-- Prev conflict -->
          <button
            onclick={handlePrevConflict}
            disabled={!hasPrev}
            aria-label="Previous conflict"
            style="
              background: none;
              border: none;
              cursor: {hasPrev ? 'pointer' : 'default'};
              color: {hasPrev ? 'var(--color-text)' : 'var(--color-text-muted)'};
              opacity: {hasPrev ? 1 : 0.4};
              padding: 2px;
              display: flex;
              align-items: center;
            "
          ><ChevronUp size={16} /></button>

          <!-- Conflict counter -->
          <span style="font-size: 11px; color: var(--color-text-muted); white-space: nowrap;">{focusedConflictIdx + 1}/{conflictIndices.length}</span>

          <!-- Next conflict -->
          <button
            onclick={handleNextConflict}
            disabled={!hasNext}
            aria-label="Next conflict"
            style="
              background: none;
              border: none;
              cursor: {hasNext ? 'pointer' : 'default'};
              color: {hasNext ? 'var(--color-text)' : 'var(--color-text-muted)'};
              opacity: {hasNext ? 1 : 0.4};
              padding: 2px;
              display: flex;
              align-items: center;
            "
          ><ChevronDown size={16} /></button>
        {/if}

        <!-- Save and Mark Resolved -->
        <button
          onclick={handleSaveAndResolve}
          disabled={saving}
          style="
            background: var(--color-btn-continue-bg);
            border: 1px solid var(--color-btn-continue-border);
            border-radius: 3px;
            color: var(--color-btn-continue);
            font-size: 11px;
            font-family: var(--font-sans, sans-serif);
            padding: 2px 8px;
            cursor: {saving ? 'not-allowed' : 'pointer'};
            opacity: {saving ? 0.4 : 1};
            white-space: nowrap;
            flex-shrink: 0;
          "
        >Save and Mark Resolved</button>

        <!-- Close button -->
        <button
          onclick={onclose}
          aria-label="Close merge editor"
          style="
            background: none;
            border: none;
            cursor: pointer;
            color: var(--color-text-muted);
            padding: 2px;
            display: flex;
            align-items: center;
          "
        ><X size={16} /></button>
      </div>

      <!-- Editable output textarea -->
      <textarea
        bind:this={panelRefs[2]}
        value={outputText}
        oninput={handleOutputEdit}
        onscroll={() => handleScroll(2)}
        style="
          flex: 1;
          width: 100%;
          resize: none;
          border: none;
          background: var(--color-bg);
          color: var(--color-text);
          font-family: var(--font-mono);
          font-size: 12px;
          line-height: 18px;
          padding: 4px 8px;
          outline: none;
          box-sizing: border-box;
        "
      ></textarea>
    </div>
  {/if}
</div>

<style>
  /* Icon hover: show remove icon on taken lines, show dimmed check on untaken */
  .icon-gutter .remove-icon {
    display: none;
  }
  .icon-gutter:hover .taken-icon {
    display: none;
  }
  .icon-gutter:hover .remove-icon {
    display: inline-flex;
  }
  .icon-gutter .untaken-icon {
    display: none;
  }
  .icon-gutter:hover .untaken-icon {
    display: inline-flex;
  }
</style>
