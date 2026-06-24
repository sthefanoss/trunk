import { safeInvoke } from "./invoke.js";

// Thin wrappers over the review IPC commands so every inline host (diff views,
// commit-detail) shares one source of the exact command names + arg shapes. The
// shapes mirror ReviewPanel's existing callers verbatim:
//   edit_comment        { path, id, text }
//   delete_comment      { path, id }
//   add_commit_comment  { path, commitOid, text }
// All three emit `session-changed`, so callers do NOT refetch manually — the
// comments rune's listener round-trips the update.

export function editComment(
	repoPath: string,
	commentId: string,
	text: string,
): Promise<void> {
	return safeInvoke("edit_comment", { path: repoPath, id: commentId, text });
}

export function deleteComment(
	repoPath: string,
	commentId: string,
): Promise<void> {
	return safeInvoke("delete_comment", { path: repoPath, id: commentId });
}

export function addCommitComment(
	repoPath: string,
	commitOid: string,
	text: string,
): Promise<void> {
	return safeInvoke("add_commit_comment", {
		path: repoPath,
		commitOid,
		text,
	});
}
