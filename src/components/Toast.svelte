<script lang="ts">
import { fly } from "svelte/transition";
import { toasts } from "../lib/toast.svelte.js";
</script>

<div class="fixed bottom-4 right-4 flex flex-col gap-2 z-50 pointer-events-none">
  {#each toasts.items as toast (toast.id)}
    <div
      role="status"
      transition:fly={{ y: 8, duration: 150 }}
      class="toast px-4 py-2 rounded-lg text-sm font-medium shadow-lg pointer-events-auto"
      class:error={toast.kind === 'error'}
    >
      {toast.message}
    </div>
  {/each}
</div>

<style>
  .toast {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-text);
  }
  .toast.error {
    background: var(--color-danger-bg);
    border-color: var(--color-danger-border);
    color: var(--color-danger);
  }
</style>
