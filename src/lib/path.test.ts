import { afterEach, describe, expect, it, vi } from "vitest";
import { joinPath, pathBasename, pathParent } from "./path.js";

vi.mock("@tauri-apps/api/path", () => ({ homeDir: vi.fn() }));

// displayPath caches the home directory for the life of the module, so each
// test loads a fresh copy against the home directory it wants.
async function displayPathWithHome(home: string, path: string) {
	vi.resetModules();
	const { homeDir } = await import("@tauri-apps/api/path");
	vi.mocked(homeDir).mockResolvedValue(home);
	const { displayPath } = await import("./path.js");
	return displayPath(path);
}

afterEach(() => {
	vi.resetModules();
});

describe("pathBasename", () => {
	it("takes the last segment of a POSIX path", () => {
		expect(pathBasename("/Users/a/trunk")).toBe("trunk");
	});

	it("takes the last segment of a Windows path", () => {
		expect(pathBasename("C:\\Users\\a\\trunk")).toBe("trunk");
	});

	it("ignores a trailing separator", () => {
		expect(pathBasename("/Users/a/trunk/")).toBe("trunk");
		expect(pathBasename("C:\\Users\\a\\trunk\\")).toBe("trunk");
	});

	it("falls back to the whole path when no segment is left", () => {
		expect(pathBasename("/")).toBe("/");
	});
});

describe("pathParent", () => {
	it("keeps the separator that ends the parent", () => {
		expect(pathParent("/Users/a/trunk")).toBe("/Users/a/");
		expect(pathParent("C:\\Users\\a\\trunk")).toBe("C:\\Users\\a\\");
	});

	it("is empty for a bare name", () => {
		expect(pathParent("trunk")).toBe("");
	});
});

describe("joinPath", () => {
	it("joins under a POSIX base with /", () => {
		expect(joinPath("/Users/a/trunk", "src/App.svelte")).toBe(
			"/Users/a/trunk/src/App.svelte",
		);
	});

	it("joins under a Windows base with \\", () => {
		expect(joinPath("C:\\Users\\a\\trunk", "src/lib/path.ts")).toBe(
			"C:\\Users\\a\\trunk\\src\\lib\\path.ts",
		);
	});

	it("does not double a trailing separator", () => {
		expect(joinPath("/a/", "b")).toBe("/a/b");
		expect(joinPath("C:\\a\\", "b")).toBe("C:\\a\\b");
	});
});

describe("displayPath", () => {
	it("abbreviates a POSIX home directory", async () => {
		expect(
			await displayPathWithHome("/Users/a", "/Users/a/Developer/trunk"),
		).toBe("~/Developer/trunk");
	});

	it("abbreviates a Windows home directory and shows forward slashes", async () => {
		expect(
			await displayPathWithHome(
				"C:\\Users\\a",
				"C:\\Users\\a\\Developer\\trunk",
			),
		).toBe("~/Developer/trunk");
	});

	it("leaves a path outside the home directory alone", async () => {
		expect(await displayPathWithHome("C:\\Users\\a", "D:\\work\\trunk")).toBe(
			"D:\\work\\trunk",
		);
	});
});
