/**
 * Pure capture-time adapter for the full-file-at-commit view: translates a line
 * selection over the flat full-file line list into a stable source-coordinate
 * `Anchor` plus a plain-content `cachedExcerpt`.
 *
 * Sibling of `buildDiffAnchor`, deliberately divergent (CONTEXT D-02/D-03/D-04):
 * - Operates on the flat list `file.hunks.flatMap(h => h.lines)`, not a single
 *   hunk — the full-file view is one continuous document with no hunk boundaries.
 * - `side` is the constant "New" and `source` is the constant "FullFile": the
 *   full-file frame is "the file AS IT EXISTS at the commit", so coordinates are
 *   absolute 1-based blob line numbers on the new side only (L-01).
 * - The range covers only new-side lines; lines with no new-side number (the
 *   removed lines) are excluded from both the range and the excerpt (D-02).
 * - `cachedExcerpt` is plain code content (each line's `content` verbatim), with
 *   NO diff prefix characters — Phase 70 renders it language-fenced, not
 *   diff-fenced (D-04).
 * - A selection straddling a dropped region keeps a correct monotonic range and
 *   inserts a "… N lines unchanged …" marker at the gap (D-03).
 *
 * All functions are pure: no IPC, no mutation of inputs, no Svelte.
 */

import type { Anchor, DiffLine, FileDiff } from "./types.js";

export interface FullFileAnchorResult {
	anchor: Anchor;
	cachedExcerpt: string;
}

const GAP_MARKER = (count: number): string => `… ${count} lines unchanged …`;

/**
 * Build the source-coordinate anchor and plain-content excerpt for a selection
 * of line indices into the file's flat line list.
 */
export function buildFullFileAnchor(
	commitOid: string,
	file: FileDiff,
	selectedIndices: Set<number>,
): FullFileAnchorResult {
	const allLines: DiffLine[] = file.hunks.flatMap((h) => h.lines);
	const orderedIndices = Array.from(selectedIndices).sort((a, b) => a - b);

	// Keep only new-side lines (the removed lines carry no new-side number and are
	// excluded from both the range and the excerpt — D-02).
	const survivors = orderedIndices
		.map((i) => allLines[i])
		.filter((l): l is DiffLine => l.new_lineno !== null);

	const lineNumbers = survivors.map((l) => l.new_lineno as number);
	const start_line = Math.min(...lineNumbers);
	const end_line = Math.max(...lineNumbers);

	const cachedExcerpt = buildExcerpt(survivors);

	const anchor: Anchor = {
		commit_oid: commitOid,
		file_path: file.path,
		source: "FullFile",
		side: "New",
		start_line,
		end_line,
	};

	return { anchor, cachedExcerpt };
}

/**
 * Join the surviving new-side lines' content by newline, inserting a
 * "… N lines unchanged …" marker wherever consecutive survivors skip new-side
 * line numbers (a dropped region). N = the skipped count = (next - prev - 1).
 */
function buildExcerpt(survivors: DiffLine[]): string {
	const parts: string[] = [];

	for (let i = 0; i < survivors.length; i++) {
		const line = survivors[i];
		if (i > 0) {
			const prev = survivors[i - 1].new_lineno as number;
			const curr = line.new_lineno as number;
			const skipped = curr - prev - 1;
			if (skipped > 0) {
				parts.push(GAP_MARKER(skipped));
			}
		}
		parts.push(line.content);
	}

	return parts.join("\n");
}
