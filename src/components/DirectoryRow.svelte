<script lang="ts">
  import { ChevronDown, ChevronRight } from '@lucide/svelte';
  import type { DirectoryNode } from '../lib/build-tree.js';

  interface Props {
    node: DirectoryNode;
    depth: number;
    expanded: boolean;
    focused: boolean;
    ontoggle: () => void;
  }

  let { node, depth, expanded, focused, ontoggle }: Props = $props();

  let hovered = $state(false);
</script>

<div
  role="treeitem"
  aria-expanded={expanded}
  aria-level={depth + 1}
  onmouseenter={() => (hovered = true)}
  onmouseleave={() => (hovered = false)}
  onclick={ontoggle}
  style="
    height: 26px;
    padding: 0 8px;
    padding-left: {8 + depth * 16}px;
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    background: {focused ? 'var(--color-tree-focus)' : hovered ? 'var(--color-surface)' : 'transparent'};
    color: var(--color-text);
    font-size: 12px;
  "
>
  <span style="display: inline-flex; align-items: center; color: var(--color-text-muted); width: 12px; min-width: 12px;">
    {#if expanded}
      <ChevronDown size={12} />
    {:else}
      <ChevronRight size={12} />
    {/if}
  </span>
  <span style="
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
  ">{node.name}</span>
</div>
