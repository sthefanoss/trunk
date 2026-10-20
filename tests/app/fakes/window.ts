import { type TauriFake, UnknownFakeCommand } from "./index.js";

/** The window state a real `Window` reports, with the OS taken out of it. */
export class FakeWindow implements TauriFake {
	readonly plugin = "window";
	private fullscreen = false;
	private focused = true;
	private windowTitle = "";

	enterFullscreen(): void {
		this.fullscreen = true;
	}

	leaveFullscreen(): void {
		this.fullscreen = false;
	}

	/** The title the app last set, which the real window would show. */
	get title(): string {
		return this.windowTitle;
	}

	focus(): void {
		this.focused = true;
	}

	blur(): void {
		this.focused = false;
	}

	reset(): void {
		this.fullscreen = false;
		this.focused = true;
		this.windowTitle = "";
	}

	answer(command: string, args: Record<string, unknown> = {}): unknown {
		switch (command) {
			case "is_fullscreen":
				return this.fullscreen;
			case "is_maximized":
				return false;
			case "is_focused":
				return this.focused;
			case "set_title":
				this.windowTitle = String(args.value);
				return null;
			default:
				throw new UnknownFakeCommand(this.plugin, command);
		}
	}
}
