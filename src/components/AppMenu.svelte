<script lang="ts">
import MenuIcon from "@lucide/svelte/icons/menu";
import { LogicalPosition } from "@tauri-apps/api/dpi";
import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { reportErrorToast } from "../lib/error-report.js";
import Button from "../lib/ui/Button.svelte";

// Windows draws no native menu bar (see setup_app), so this button opens the
// same menus the bar held, as a native popup under it. The items mirror
// build_menu in lib.rs: the two custom ones emit the events that menu emitted.
const appWindow = getCurrentWindow();

async function openMenu(event: MouseEvent) {
	// currentTarget is null once the event has finished dispatching, so the
	// button is measured before the first await.
	const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
	try {
		await showMenu(box.left, box.bottom);
	} catch (e) {
		reportErrorToast(e, "Could not open the menu");
	}
}

async function showMenu(x: number, y: number) {
	const { Menu, MenuItem, PredefinedMenuItem, Submenu } = await import(
		"@tauri-apps/api/menu"
	);
	const separator = () => PredefinedMenuItem.new({ item: "Separator" });

	const app = await Submenu.new({
		text: "Trunk",
		items: [
			await MenuItem.new({
				text: "Quit",
				action: () => appWindow.close(),
			}),
		],
	});
	const edit = await Submenu.new({
		text: "Edit",
		items: [
			await PredefinedMenuItem.new({ item: "Undo" }),
			await PredefinedMenuItem.new({ item: "Redo" }),
			await separator(),
			await PredefinedMenuItem.new({ item: "Cut" }),
			await PredefinedMenuItem.new({ item: "Copy" }),
			await PredefinedMenuItem.new({ item: "Paste" }),
			await PredefinedMenuItem.new({ item: "SelectAll" }),
			await separator(),
			await MenuItem.new({
				text: "Find",
				accelerator: "CmdOrCtrl+F",
				action: () => emit("search-toggle"),
			}),
		],
	});
	const view = await Submenu.new({
		text: "View",
		items: [
			await MenuItem.new({
				text: "Toggle Review Panel",
				action: () => emit("review-toggle"),
			}),
			await MenuItem.new({
				text: "Toggle Full Screen",
				action: async () =>
					appWindow.setFullscreen(!(await appWindow.isFullscreen())),
			}),
		],
	});
	const window = await Submenu.new({
		text: "Window",
		items: [
			await MenuItem.new({
				text: "Minimize",
				action: () => appWindow.minimize(),
			}),
		],
	});

	const menu = await Menu.new({ items: [app, edit, view, window] });
	await menu.popup(new LogicalPosition(x, y));
}
</script>

<div class="flex items-center flex-shrink-0 px-2">
	<Button
		icon
		variant="ghost"
		aria-label="Application menu"
		tooltip="Menu"
		onclick={openMenu}
	>
		<MenuIcon size={14} />
	</Button>
</div>
