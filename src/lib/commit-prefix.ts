/** Parsed conventional-commit summary. `prefix` is null when the summary does
 *  not start with a `type(scope)?!?:` header, in which case `rest` is the whole
 *  summary unchanged. */
export interface ParsedSummary {
	prefix: string | null;
	scope: string | null;
	bang: string | null;
	rest: string;
}

const CONVENTIONAL = /^(\w+)(\([^)]+\))?(!?):\s*(.*)$/;

export function parseSummary(summary: string): ParsedSummary {
	const m = summary.match(CONVENTIONAL);
	if (!m) return { prefix: null, scope: null, bang: null, rest: summary };
	return { prefix: m[1], scope: m[2] ?? "", bang: m[3] ?? "", rest: m[4] };
}

const PREFIX_TONE: Record<string, string> = {
	feat: "var(--ok)",
	perf: "var(--ok)",
	fix: "var(--warn)",
	docs: "var(--info)",
	refactor: "var(--info)",
	revert: "var(--err)",
};

/** Theme token for a conventional-commit prefix; muted caption tone otherwise. */
export function prefixToneVar(prefix: string): string {
	return PREFIX_TONE[prefix] ?? "var(--fg-3)";
}
