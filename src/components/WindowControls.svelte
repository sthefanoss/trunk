<script lang="ts">
import Copy from "@lucide/svelte/icons/copy";
import Minus from "@lucide/svelte/icons/minus";
import Square from "@lucide/svelte/icons/square";
import X from "@lucide/svelte/icons/x";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMount } from "svelte";
import Button from "../lib/ui/Button.svelte";

// Windows draws no native frame (see setup_app), so the top bar carries the
// minimize, maximize and close buttons the frame would have.
const appWindow = getCurrentWindow();
let maximized = $state(false);

async function syncMaximized() {
	maximized = await appWindow.isMaximized();
}

onMount(() => {
	void syncMaximized();
	let unlisten: (() => void) | undefined;
	void appWindow.onResized(syncMaximized).then((fn) => {
		unlisten = fn;
	});
	return () => unlisten?.();
});
</script>

<div class="flex items-center flex-shrink-0 gap-1 px-2">
	<Button
		icon
		variant="ghost"
		aria-label="Minimize"
		onclick={() => appWindow.minimize()}
	>
		<Minus size={14} />
	</Button>
	<Button
		icon
		variant="ghost"
		aria-label={maximized ? "Restore" : "Maximize"}
		onclick={() => appWindow.toggleMaximize()}
	>
		{#if maximized}
			<Copy size={12} />
		{:else}
			<Square size={12} />
		{/if}
	</Button>
	<Button
		icon
		variant="ghost"
		aria-label="Close"
		onclick={() => appWindow.close()}
	>
		<X size={14} />
	</Button>
</div>
