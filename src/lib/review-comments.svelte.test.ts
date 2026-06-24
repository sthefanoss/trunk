import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createReviewComments } from "./review-comments.svelte.js";
import type { Comment } from "./types";

// safeInvoke is a thin wrapper around @tauri-apps/api/core::invoke (src/lib/invoke.ts).
// Mock the underlying invoke (not safeInvoke) so the TrunkError-parsing path stays
// live, matching review-session.svelte.test.ts. listen is stubbed to a no-op unlisten
// so the rune installs no real session-changed subscription under test.
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
	listen: vi.fn().mockResolvedValue(() => {}),
}));

const mockInvoke = vi.mocked(invoke);

const comment: Comment = {
	id: "c1",
	text: "looks good",
	anchor: {
		commit_oid: "abc",
		file_path: "src/foo.ts",
		source: "Diff",
		side: "New",
		start_line: 10,
		end_line: 10,
	},
	cached_excerpt: null,
	commit_oid: "abc",
};

// Active session: one comment, a working-tree snapshot.
function activeSession() {
	mockInvoke.mockImplementation((cmd: string) => {
		switch (cmd) {
			case "get_review_session_status":
				return Promise.resolve({
					state: "active",
					file_exists: true,
					canonical_path: "/repo",
				});
			case "get_review_snapshots":
				return Promise.resolve({
					working_tree_snapshot: "wt1",
					index_snapshot: null,
				});
			case "list_session_comments":
				return Promise.resolve([comment]);
			default:
				return Promise.reject(new Error(`unexpected ${cmd}`));
		}
	});
}

// After End Review the backend removes the in-memory session, so
// list_session_comments rejects with no_session and status reports "none".
function endedSession() {
	mockInvoke.mockImplementation((cmd: string) => {
		switch (cmd) {
			case "get_review_session_status":
				return Promise.resolve({
					state: "none",
					file_exists: false,
					canonical_path: "/repo",
				});
			case "get_review_snapshots":
				return Promise.resolve({
					working_tree_snapshot: null,
					index_snapshot: null,
				});
			case "list_session_comments":
				return Promise.reject(
					'{"code":"no_session","message":"No active review session for this repository"}',
				);
			default:
				return Promise.reject(new Error(`unexpected ${cmd}`));
		}
	});
}

beforeEach(() => {
	mockInvoke.mockReset();
});

describe("createReviewComments — refresh", () => {
	it("populates comments, snapshots and active from a successful refresh", async () => {
		activeSession();
		const m = createReviewComments("/repo");

		await m.refresh();

		expect(m.comments).toEqual([comment]);
		expect(m.snapshots).toEqual({
			working_tree_snapshot: "wt1",
			index_snapshot: null,
		});
		expect(m.active).toBe(true);

		m.destroy();
	});

	it("clears stale comments and marks inactive when the session ends", async () => {
		activeSession();
		const m = createReviewComments("/repo");
		await m.refresh();
		expect(m.comments).toHaveLength(1);
		expect(m.active).toBe(true);

		// Regression: a naive Promise.all would let the no_session rejection abort
		// the whole update, leaving the stale comment (and active=true) on screen,
		// so inline comments would not vanish on End Review.
		endedSession();
		await m.refresh();

		expect(m.comments).toEqual([]);
		expect(m.active).toBe(false);
		expect(m.snapshots).toEqual({
			working_tree_snapshot: null,
			index_snapshot: null,
		});

		m.destroy();
	});
});
