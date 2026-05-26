# Phase 70: Excerpt Resolution + Markdown Render - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-26
**Phase:** 70-excerpt-resolution-markdown-render
**Areas discussed:** Trigger surface, AI-framed doc anatomy, Commit refs detail, Unresolvable + empty cases

---

## Trigger surface

### Q1: Where does the user invoke "Generate doc" from?

| Option | Description | Selected |
|--------|-------------|----------|
| Button in ReviewPanel header (Recommended) | Lives with the artifact it acts on; Phase 71 adds Copy/Save to same area | ✓ |
| View menu (like Start/End Review) | Mirrors Phase 65 D-12; discoverable but far from the panel | |
| Floating action in panel footer | Sticky full-width primary action | |
| Hotkey only (no visible affordance now) | Defer all visible trigger to Phase 71; feature invisible until then | |

**User's choice:** Button in ReviewPanel header
**Notes:** Locks the affordance near the artifact and gives Phase 71 a docking site for Copy/Save.

### Q2: After clicking Generate, what does the user see (before Phase 71 ships Copy/Save)?

| Option | Description | Selected |
|--------|-------------|----------|
| Markdown preview in the panel (Recommended) | Panel swaps from comment-list view to a 'Generated doc' view; Back affordance returns | ✓ |
| Modal/sheet over the panel | Easier to close but reintroduces a modal pattern the app avoids | |
| Doc held in state, no preview yet | Computed and stored on the rune; user can't verify this phase | |
| Open in a new tab/window | Heavier UI work; static-snapshot doc shouldn't read as a tab | |

**User's choice:** Markdown preview in the panel
**Notes:** Mirrors Phase 69 D-07's right-pane swap pattern but kept INSIDE the panel.

### Q3: Generate button enablement — when disabled vs always-on?

| Option | Description | Selected |
|--------|-------------|----------|
| Disabled until session has ≥1 comment (Recommended) | Zero-comment doc is useless to an AI reviewer; tooltip explains | ✓ |
| Always enabled, render whatever exists | Even empty session renders — just commit refs | |
| Disabled only when zero commits | Requires commits but allows zero comments | |

**User's choice:** Disabled until session has ≥1 comment
**Notes:** Resolvable AND unresolvable both count toward enablement (unresolvable still renders).

---

## AI-framed doc anatomy

### Q1: What goes at the very top of the doc — the "AI framing"?

| Option | Description | Selected |
|--------|-------------|----------|
| Short framing + commit refs (Recommended) | H1 title + 1-2 sentences explaining the doc + commit refs list before any comments | ✓ |
| Just commit refs, no prose framing | Structure speaks for itself; risk of misread intent | |
| Detailed system-prompt-style preamble | Heavier; bakes prompt-engineering into a snapshot | |
| No top section — jump straight into per-file comments | No commit-ref overview; AI loses scope | |

**User's choice:** Short framing + commit refs
**Notes:** Plain declarative framing, no response-format instructions.

### Q2: Same file appearing in multiple commits — how do per-file groups look?

| Option | Description | Selected |
|--------|-------------|----------|
| One section per (file, commit) pair (Recommended) | Same file in different commits is meaningfully different code | ✓ |
| One section per file, anchors ordered by commit then line | Nice "all feedback on this file" view but mixes commits | |
| Flat list, no per-file headings | Simpler renderer; loses file-level scaffolding | |

**User's choice:** One section per (file, commit) pair
**Notes:** Matches the per-comment `path:Lstart-Lend (sha)` heading naturally.

### Q3: Within one anchor block — excerpt vs comment text order?

| Option | Description | Selected |
|--------|-------------|----------|
| Excerpt first, comment after (Recommended) | Quote-then-react; AI sees the lines before any planning | ✓ |
| Comment first, excerpt after | Risks AI planning a fix before seeing the actual lines | |
| Comment only, no excerpt | Defeats the cached/re-resolved excerpt mechanic | |

**User's choice:** Excerpt first, comment after

### Q4: Section order — where does the unresolvable section sit?

| Option | Description | Selected |
|--------|-------------|----------|
| Resolved files → commit-level → unresolvable (Recommended) | Strongest signal first; failures at the end is conventional | ✓ |
| Resolved files → unresolvable → commit-level | Splits no-anchor (commit-level) and lost-anchor (unresolvable) when adjacent | |
| Per-file: resolved + unresolvable inline; commit-level trailing | Messier renderer; "commit gone" has no clear file home | |

**User's choice:** Resolved files → commit-level → unresolvable

---

## Commit refs detail

### Q1: What info per commit in the top refs list?

| Option | Description | Selected |
|--------|-------------|----------|
| Short SHA + subject line (Recommended) | Minimal; at-a-glance map | ✓ |
| Short SHA + subject + author + date | Closer to `git log --oneline`; noise for an AI | |
| Full SHA + subject (no short SHA) | Unambiguous but ugly in a list | |
| SHA only, no subject | Bare list; loses human context | |

**User's choice:** Short SHA + subject line

### Q2: SHA consistency — same format in refs list and per-anchor `(sha)` headings?

| Option | Description | Selected |
|--------|-------------|----------|
| Both short (7 chars) (Recommended) | Visually matchable; matches existing panel convention | ✓ |
| Refs full, anchor headings short | Unambiguous list, compact headings; minor cognitive cost | |
| Both full (40 chars) | Unambiguous everywhere; uglier headings repeated dozens of times | |

**User's choice:** Both short (7 chars)

---

## Unresolvable + empty cases

### Q1: Reason phrasing for unresolvable entries?

| Option | Description | Selected |
|--------|-------------|----------|
| Human-readable phrases (Recommended) | Map OrphanReason → plain English the AI can act on | ✓ |
| Raw enum tokens (CommitGone/FileGone/LineOutOfRange) | Matches code; reads as a code symbol to an AI | |
| Phrase + enum token | Both; slight redundancy | |

**User's choice:** Human-readable phrases
**Notes:** CommitGone → "commit no longer exists in the repository"; FileGone → "file no longer exists at this commit/side"; LineOutOfRange → "anchor line range is outside the current file bounds".

### Q2: Unresolvable excerpt fences — same DOC-02 rule as resolved excerpts?

| Option | Description | Selected |
|--------|-------------|----------|
| Fence by Source, mark as cached (Recommended) | Same DOC-02 fencing; consistent doc; AI sees uniform code fences | ✓ |
| Always plain code fence | Simpler renderer; loses diff-vs-file signal | |
| Omit excerpt, comment text only | Loses context; contradicts ROADMAP "use cached excerpt" note | |

**User's choice:** Fence by Source, mark as cached

### Q3: Empty-session generation — confirm no zero-comment renderer?

| Option | Description | Selected |
|--------|-------------|----------|
| Planner assumes ≥1 comment; no zero-comment renderer (Recommended) | Trigger gating is the contract; keeps renderer invariants tight | ✓ |
| Renderer still handles zero comments (defensive) | Robust to a future trigger that bypasses the gate (e.g. CLI command) | |

**User's choice:** Planner assumes ≥1 comment; no zero-comment renderer

---

## Claude's Discretion

Delegated to the planner — captured in CONTEXT.md `<decisions>` → "Claude's Discretion":
- Module placement for the pure render and its helpers (likely `src-tauri/src/git/review.rs`).
- Exact Tauri command name + signature; read-only, no `session-changed` emit.
- Whether to reuse / extend / mirror `OrphanReason` for render-side failure classification.
- File-group heading text and heading levels (H1/H2/H3 hierarchy).
- Exact prose of the preamble framing (plain, ≤2 sentences, no response-format instructions).
- Language detection table for full-file language fences (extension-based, `text` fallback).
- Line-counting reconciliation between capture (Phase 67/68) and render — fix in ONE place.
- Preview component shape (new file vs. inline) + "Back to comments" affordance + Phase 71 toolbar slot.
- Generate action + view-state on `review-session.svelte.ts` rune (reset semantics on `endSession` / `session-changed`).

## Deferred Ideas

- Clipboard / Save-to-file (Phase 71).
- Re-anchoring on history rewrite (REQUIREMENTS Out of Scope).
- Inline gutter badges / in-diff comment browser (still backlog from Phase 67/69).
- Syntax / word-span enrichment in excerpts (explicitly excluded by ROADMAP §70 + L-10).
- No scope-creep surfaced during discussion.
