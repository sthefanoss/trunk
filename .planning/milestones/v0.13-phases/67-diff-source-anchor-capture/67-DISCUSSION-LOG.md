# Phase 67: Diff-Source Anchor Capture - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-25
**Phase:** 67-diff-source-anchor-capture
**Areas discussed:** Attach flow & composer, Draft collision, Selection → range rule, Blocked-anchor feedback

---

## Attach Flow & Composer

| Option | Description | Selected |
|--------|-------------|----------|
| Inline composer in diff | Floating "Comment" button on the active selection opens an expanding textarea under the lines (GitHub PR style); reuses the on-selection action pattern; independent of the Phase 69 panel. | ✓ |
| Compose in review panel | Selection sets a pending anchor; type in the panel composer. Centralizes UI but splits attention and the panel is a throwaway stub. | |
| Context-menu → inline | Right-click selection → "Add comment" opens inline composer. Discoverable but adds a click. | |

**User's choice:** Inline composer in diff
**Notes:** Recommended option; keeps the comment tied to the code and avoids depending on the Phase 69 panel shape.

---

## Draft Collision

| Option | Description | Selected |
|--------|-------------|----------|
| Confirm before discard | Prompt "Discard your unsaved comment?" when selecting a new range over a non-empty draft; empty drafts switch silently. Mirrors existing discard-confirmation pattern. | ✓ |
| Block until resolved | New selections inert until current draft submitted/cancelled. Safest but sticky. | |
| Overwrite silently | New selection replaces draft. Simplest, but a persisted half-typed draft is lost. | |

**User's choice:** Confirm before discard
**Notes:** Single `draft_comment` slot + persist-on-keystroke makes a half-typed draft durable, so silent loss is unacceptable; blocking is annoying.

---

## Selection → Range Rule

| Option | Description | Selected |
|--------|-------------|----------|
| Collapse min–max + preview | Any selection becomes start..end on the chosen side; composer shows "Comments on lines N–M". Transparent, no rejection friction. | ✓ |
| Collapse silently | Take min..max, no preview. Simplest, but gaps silently included may surprise. | |
| Require contiguous | Reject non-contiguous selections with a hint. Most literal, adds friction. | |

**User's choice:** Collapse min–max + preview
**Notes:** Anchor stores one range; v0.7 selection can be a non-contiguous Set within a hunk. Preview keeps the collapse transparent.

---

## Blocked-Anchor Feedback

| Option | Description | Selected |
|--------|-------------|----------|
| Disabled + tooltip | Comment affordance disabled on merge-commit diffs with a tooltip "Diff comments aren't available on merge commits." Teaches the rule at point of action. | ✓ |
| Hide affordance | No comment button on merge diffs. Clean but silently confusing. | |
| Allow then toast | Let the user try, explain via toast. After-the-fact and annoying. | |

**User's choice:** Disabled + tooltip
**Notes:** Applies to merge commits (diff-source disabled, Phase 66 D-08). File-status side constraints (Added/Deleted/Renamed) auto-force the side rather than blocking, so they are not part of this "blocked" feedback.

---

## Claude's Discretion

- Attach-success feedback: clear composer + selection on submit, rely on existing `session-changed` reload; optional success toast (planner's call, consistent with silent-success convention).
- Empty-text submit disabled.
- Command surface & naming: `add_comment` (+ any draft-persist command) in `commands/review.rs` (recommended); names per established serde conventions.
- Composer rendering across v0.12 hunk/split layout modes (both diff-source; full-file content mode is Phase 68).

## Deferred Ideas

- In-diff comment browser (gutter markers/badges/click-to-edit) → Phase 69 (CMT-04).
- Commit-level comments with no anchor → Phase 69 (ANCH-03).
- Render-time excerpt re-resolution + "unresolvable" section → Phase 70 (DOC-04).
- Reviewed-not-folded todo: `2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md` (merge/revert commit-message editing — unrelated; declined in Phase 66).
