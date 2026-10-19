import { afterEach, describe, expect, it, vi } from "vitest";

async function platformOn(platform: string, userAgent = "") {
	vi.resetModules();
	vi.stubGlobal("navigator", { platform, userAgent });
	return import("./platform.js");
}

afterEach(() => {
	vi.unstubAllGlobals();
});

describe("platform", () => {
	it("reserves room for the traffic lights on macOS", async () => {
		const { isMac, TRAFFIC_LIGHT_INSET } = await platformOn("MacIntel");
		expect(isMac).toBe(true);
		expect(TRAFFIC_LIGHT_INSET).toBe(78);
	});

	it("reserves none on Windows, which draws a native title bar", async () => {
		const { isMac, TRAFFIC_LIGHT_INSET } = await platformOn("Win32");
		expect(isMac).toBe(false);
		expect(TRAFFIC_LIGHT_INSET).toBe(0);
	});

	// jsdom's user agent names the host OS, so a test run on Windows would take
	// itself for the Windows app if the user agent counted.
	it("ignores the user agent, which jsdom fills with the host OS", async () => {
		const { isMac, isWindows } = await platformOn("", "Mozilla/5.0 (win32)");
		expect(isMac).toBe(false);
		expect(isWindows).toBe(false);
	});

	it("takes Win32 for Windows", async () => {
		expect((await platformOn("Win32")).isWindows).toBe(true);
	});
});
