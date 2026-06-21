import { describe, expect, it } from "vitest";
import { parseSummary, prefixToneVar } from "./commit-prefix.js";

describe("parseSummary", () => {
	it("splits a plain conventional prefix from the rest", () => {
		expect(parseSummary("fix: bug")).toEqual({
			prefix: "fix",
			scope: "",
			bang: "",
			rest: "bug",
		});
	});

	it("captures the scope", () => {
		expect(parseSummary("feat(api): add sort")).toEqual({
			prefix: "feat",
			scope: "(api)",
			bang: "",
			rest: "add sort",
		});
	});

	it("captures a breaking-change bang", () => {
		expect(parseSummary("refactor(core)!: drop legacy path")).toEqual({
			prefix: "refactor",
			scope: "(core)",
			bang: "!",
			rest: "drop legacy path",
		});
	});

	it.each([
		"Merge branch 'hotfix/typo' into main",
		"Initial commit",
		"WIP",
	])("leaves a non-conventional summary untouched: %s", (summary) => {
		expect(parseSummary(summary)).toEqual({
			prefix: null,
			scope: null,
			bang: null,
			rest: summary,
		});
	});
});

describe("prefixToneVar", () => {
	it.each([
		{ prefix: "feat", tone: "var(--ok)" },
		{ prefix: "perf", tone: "var(--ok)" },
		{ prefix: "fix", tone: "var(--warn)" },
		{ prefix: "refactor", tone: "var(--info)" },
		{ prefix: "docs", tone: "var(--info)" },
		{ prefix: "revert", tone: "var(--err)" },
	])("maps $prefix to $tone", ({ prefix, tone }) => {
		expect(prefixToneVar(prefix)).toBe(tone);
	});

	it("falls back to the muted caption tone for unknown prefixes", () => {
		expect(prefixToneVar("chore")).toBe("var(--fg-3)");
		expect(prefixToneVar("wip")).toBe("var(--fg-3)");
	});
});
