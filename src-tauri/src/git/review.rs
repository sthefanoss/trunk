//! Phase 70: pure markdown renderer for review sessions.
//!
//! Pure Rust logic: takes `&ReviewSession` + `&git2::Repository`, returns a
//! single `String`. No `tauri::*` imports (L-01), no calls into
//! `crate::git::syntax` (L-10), never panics (L-04).
//!
//! This module is `tauri`-free and exposes ONE public function: [`render`].
//! All resolution failures are routed INTO the returned markdown (per L-04 +
//! L-09); the renderer NEVER returns an error.

use crate::commands::review::OrphanReason;
use crate::git::types::ReviewSession;

/// Render-only failure kinds. Does NOT cross the IPC wire (the Phase 69
/// `OrphanReason` does — never extend it). All variants funnel into either the
/// resolved per-file section (via the `[binary file, no excerpt]` placeholder
/// for `Binary`) or the unresolvable trailing section (everything else).
#[allow(dead_code)] // wired up in task 2 / task 3
#[derive(Debug)]
pub(crate) enum ExcerptError {
    /// `blob.is_binary()` returned true; emit `[binary file, no excerpt]`
    /// INSIDE the resolved per-file section (L-05, not the unresolvable
    /// section).
    Binary,
    /// `classify_anchor` rejected the anchor — wraps the Phase 69 reason.
    Orphaned(OrphanReason),
    /// Generic re-resolution failure (git2 error during slicing).
    ResolutionFailed,
    /// Diff replay-slice produced an empty body (file unchanged from parent at
    /// the anchored commit; Pitfall 2).
    NoHunks,
}

/// L-03: fence length is `max(3, longest_contiguous_backtick_run + 1)`.
/// Linear byte-scan over the entire body — counter resets on any non-backtick
/// byte (including newlines), so two separate `` ``` `` runs split by a
/// newline do NOT compose into a longer run. CommonMark §4.5 requires the
/// opening fence be strictly longer than any inner backtick run.
#[allow(dead_code)] // wired up in task 3
pub(crate) fn fence_length(body: &str) -> usize {
    let mut longest = 0usize;
    let mut current = 0usize;
    for b in body.as_bytes() {
        if *b == b'`' {
            current += 1;
            if current > longest {
                longest = current;
            }
        } else {
            current = 0;
        }
    }
    std::cmp::max(3, longest + 1)
}

/// L-07: extension → markdown fence language tag for `Source::FullFile`
/// excerpts. Hand-rolled per L-10 (no syntect call). Distinct from
/// `syntax::fallback_extension` which targets syntect syntax IDs.
#[allow(dead_code)] // wired up in task 3
pub(crate) fn fence_language(file_path: &str) -> &'static str {
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs" => "rust",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" => "tsx",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "jsx",
        "svelte" => "svelte",
        "json" => "json",
        "md" | "markdown" => "markdown",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "css" => "css",
        "html" | "htm" => "html",
        "sh" | "bash" => "bash",
        "py" => "python",
        "go" => "go",
        _ => "text",
    }
}

/// Top-level pure renderer. Placeholder scaffold — task 3 implements the full
/// D-03..D-10 section assembly. Always returns a `String`, never panics.
#[allow(dead_code)] // task 3 fleshes this out
pub fn render(_session: &ReviewSession, _repo: &git2::Repository) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Task 1: fence_length unit tests (L-03) ────────────────────────────

    #[test]
    fn fence_length_floor_with_no_backticks() {
        assert_eq!(fence_length("hello world\n"), 3);
    }

    #[test]
    fn fence_length_floor_on_empty_body() {
        // Triangulation: empty body → still the max(3, …) floor.
        assert_eq!(fence_length(""), 3);
    }

    #[test]
    fn fence_length_avoids_backtick_collision() {
        // A 3-backtick run forces the opening fence to be at least 4
        // backticks so CommonMark §4.5 closes the outer fence correctly.
        assert_eq!(fence_length("foo ``` bar"), 4);
    }

    #[test]
    fn fence_length_handles_four_backtick_run() {
        assert_eq!(fence_length("foo ```` bar"), 5);
    }

    #[test]
    fn fence_length_resets_across_newlines() {
        // Two separate 3-runs split by a newline must NOT compose; longest
        // contiguous run is 3, so the fence is 3 + 1 = 4.
        assert_eq!(fence_length("```\n```"), 4);
    }

    #[test]
    fn fence_length_finds_longest_run_anywhere_in_body() {
        // The 5-run lives in the middle of a longer line; the scan must find
        // it regardless of line position. 5 + 1 = 6.
        assert_eq!(fence_length("a\nbbb`````ccc\nd"), 6);
    }
}
