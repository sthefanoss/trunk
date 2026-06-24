import { listen } from "@tauri-apps/api/event";
import { safeInvoke } from "./invoke.js";
import type { Comment, ReviewSnapshots, SessionStatus } from "./types";

/**
 * The single reactive source of truth for review comments, lifted to RepoView
 * and consumed by every surface (ReviewPanel, DiffPanel/diff views, CommitDetail).
 *
 * Comments are a property of the code, not of which pane is open, so they live
 * in one place: one `session-changed` subscription, one re-fetch on change.
 * Visibility is gated on `active` (session status === "active"), independent of
 * the center-pane review toggle (see plan §3, §8).
 */
export interface ReviewCommentsManager {
	readonly comments: Comment[];
	readonly snapshots: ReviewSnapshots;
	readonly active: boolean;
	readonly totalCount: number;
	refresh(): Promise<void>;
	destroy(): void;
}

export function createReviewComments(repoPath: string): ReviewCommentsManager {
	const state = $state({
		comments: [] as Comment[],
		snapshots: {
			working_tree_snapshot: null,
			index_snapshot: null,
		} as ReviewSnapshots,
		active: false,
	});

	const totalCount = $derived(state.comments.length);

	// The canonical path the backend reports for this repo. The session-changed
	// payload is that canonical string, so the listener filters on it. Tracked
	// separately so the filter can fail-closed while it is still null (a missing
	// or inactive session is a normal state — mirrors CommitGraph:330).
	let canonicalPath: string | null = null;

	async function refresh(): Promise<void> {
		// allSettled, not all: list_session_comments rejects with "no_session" once
		// a review ends (review.rs removes the in-memory session), and get_review_*
		// can reject when the repo is closing. With Promise.all a single reject
		// aborts the whole update, leaving stale comments/active on screen — so
		// ending a review would NOT clear inline comments. Settling each lets a
		// rejection collapse to the correct empty/inactive state instead.
		const [statusR, snapshotsR, commentsR] = await Promise.allSettled([
			safeInvoke<SessionStatus>("get_review_session_status", {
				path: repoPath,
			}),
			safeInvoke<ReviewSnapshots>("get_review_snapshots", { path: repoPath }),
			safeInvoke<Comment[]>("list_session_comments", { path: repoPath }),
		]);

		if (statusR.status === "fulfilled" && statusR.value) {
			canonicalPath = statusR.value.canonical_path;
			state.active = statusR.value.state === "active";
		} else {
			state.active = false;
		}

		state.snapshots =
			snapshotsR.status === "fulfilled" && snapshotsR.value
				? snapshotsR.value
				: { working_tree_snapshot: null, index_snapshot: null };

		state.comments =
			commentsR.status === "fulfilled" && Array.isArray(commentsR.value)
				? commentsR.value
				: [];
	}

	// Live coordination: refresh when a session-changed event arrives for this
	// repo's canonical path. Fail-closed when canonicalPath is null so cross-repo
	// events during the cold-start window don't trigger a refresh. The `cancelled`
	// flag disposes a listener the promise delivers after destroy() (mirrors
	// CommitGraph:1448-1467 / ReviewPanel:444-458).
	let unlisten: (() => void) | undefined;
	let cancelled = false;
	listen<string>("session-changed", (event) => {
		if (!canonicalPath || event.payload !== canonicalPath) return;
		refresh().catch(() => {});
	}).then((fn) => {
		if (cancelled) fn();
		else unlisten = fn;
	});

	refresh().catch(() => {});

	return {
		get comments() {
			return state.comments;
		},
		get snapshots() {
			return state.snapshots;
		},
		get active() {
			return state.active;
		},
		get totalCount() {
			return totalCount;
		},
		refresh,
		destroy() {
			cancelled = true;
			unlisten?.();
		},
	};
}
