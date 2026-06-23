import type { CommitNav, GraphCommit } from "./types.js";

// Sentinel oid of the synthetic working-tree row prepended to the graph list.
// Mirrors the value CommitGraph uses; excluded from every nav computation.
const WIP_OID = "__wip__";

/** Derive the selected commit's pager position and topology neighbors from the
 * loaded graph list (newest-first). Returns null when nothing real is selected
 * (no selection, the WIP row, or an oid not present in the loaded list).
 *
 * Index 0 is the newest commit (top); higher index is older. `newerOid` steps
 * toward HEAD (up), `olderOid` toward the root (down) — matching the graph's
 * keyboard navigation. */
export function computeCommitNav(
	items: GraphCommit[],
	selectedOid: string | null,
	hasMore: boolean,
): CommitNav | null {
	if (selectedOid === null || selectedOid === WIP_OID) return null;

	const real = items.filter((c) => c.oid !== WIP_OID);
	const index = real.findIndex((c) => c.oid === selectedOid);
	if (index === -1) return null;

	const childOids = real
		.filter((c) => c.parent_oids.includes(selectedOid))
		.map((c) => c.oid);

	return {
		index: index + 1,
		total: real.length,
		hasMore,
		newerOid: index > 0 ? real[index - 1].oid : null,
		olderOid: index < real.length - 1 ? real[index + 1].oid : null,
		childOids,
	};
}
