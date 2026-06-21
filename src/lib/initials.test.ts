import { describe, expect, it } from "vitest";
import { initials } from "./initials.js";

describe("initials", () => {
	it.each([
		{ name: "João Fernandes", expected: "JF" },
		{ name: "Marco Pio", expected: "MP" },
		{ name: "alice", expected: "A" },
		{ name: "a b c", expected: "AC" },
		{ name: "dependabot[bot]", expected: "D" },
	])("derives $expected from $name", ({ name, expected }) => {
		expect(initials(name)).toBe(expected);
	});

	it("returns an empty string for blank input", () => {
		expect(initials("   ")).toBe("");
	});
});
