---
phase: 72
slug: review-pane-ux-integration
status: draft
shadcn_initialized: false
preset: none
created: 2026-05-27
---

# Phase 72 — UI Design Contract

> Visual and interaction contract for the Review-Pane UX Integration phase. Most fields are pre-populated from CONTEXT.md (design-locked 2026-05-26) and the existing project theme (`src/app.css`). This phase is a surface-area refactor — no new design system, no new tokens, no new packages.

---

## Design System

| Property | Value | Source |
|----------|-------|--------|
| Tool | none (no shadcn — Svelte 5 + Tailwind 4 + project-native CSS tokens) | RESEARCH.md §Standard Stack |
| Preset | not applicable | — |
| Component library | none (project ships hand-rolled Svelte components on a CSS-custom-property theme) | `src/app.css`, `src/components/Toolbar.svelte` |
| Icon library | `@lucide/svelte` 0.577.0 (already installed) | RESEARCH.md §Standard Stack |
| Font (sans) | `Inter, system-ui, -apple-system, sans-serif` (`var(--font-sans)`) | `src/app.css:101` |
| Font (mono) | `JetBrains Mono, Fira Code, Cascadia Code, monospace` (`var(--font-mono)`) | `src/app.css:100` |
| Theme | dark-only, forced (no `prefers-color-scheme`, no `data-theme`) | `src/app.css:4` |

> **Light-mode validation is OUT OF SCOPE** for this phase. RESEARCH Finding 3 verified the app is dark-only; any UAT step referencing light-mode toggling is obsolete and should be omitted from validation matrices.

---

## Spacing Scale

The project does not declare a named spacing scale token set. Existing components (Toolbar, ReviewPanel header, ReviewDocPreview header) consistently use 2/4/6/8/10/12 px values — multiples of 2 with 4 px as the dominant rhythm. This phase **inherits** the precedent verbatim. No new spacing tokens introduced.

Declared values for Phase 72 surfaces (all px, all multiples of 2; the project standard):

| Token | Value | Usage in this phase | Source / Precedent |
|-------|-------|---------------------|-------------------|
| `gap-xs` | 2px | Group-internal button gap inside `.toolbar-group` | `Toolbar.svelte:206` |
| `gap-sm` | 4px | Icon-to-label gap inside `.toolbar-btn`; spacing inside `ReviewPanel` header `.preview-spacer` row | `Toolbar.svelte:220`, `ReviewPanel.svelte:350` |
| `gap-md` | 8px | Inter-group gap inside `ReviewPanel` header row | `ReviewPanel.svelte:350` |
| `gap-lg` | 12px | Inter-group horizontal gap on `.toolbar` between toolbar-groups; outer padding on `.toolbar` | `Toolbar.svelte:199-200` |
| `pad-btn-y` | 4px | `.toolbar-btn` vertical padding | `Toolbar.svelte:215` |
| `pad-btn-x` | 10px | `.toolbar-btn` horizontal padding | `Toolbar.svelte:215` |
| `pad-header-y` | 6px | ReviewPanel header vertical padding | `ReviewPanel.svelte:351` |
| `pad-header-x` | 12px | ReviewPanel header horizontal padding | `ReviewPanel.svelte:351` |
| `btn-height` | 26px | Fixed `.toolbar-btn` height for visual alignment within the toolbar | `Toolbar.svelte:221` |
| `radius-sm` | 4px | All button/group corner radii in this phase | `Toolbar.svelte:212`, `ReviewDocPreview.svelte:91` |
| `border` | 1px | All borders (button, header underline) | project standard |

Exceptions: none. The new Toolbar Review button MUST match the existing `.toolbar-btn` shape (26 px tall, 4/10 padding, 4 px radius, 4 px icon-label gap) so it sits flush in the toolbar row. The renamed Copy button in `ReviewPanel.svelte` keeps its current ReviewPanel-header dimensions (`padding: 2px 8px`, 4 px radius) — these are the existing values for inline header buttons in that header and should not change.

> **Note on the 8-point scale guideline:** The project standard is multiples of 2 with 4 px as the dominant step, not multiples of 4 only. The Toolbar's existing `gap: 2px` between buttons inside a group, and the ReviewPanel header's `padding: 6px 12px` and `gap: 8px`, are documented precedents. This phase preserves them — introducing a new scale would create visual inconsistency for a refactor whose explicit goal is *integration* with surrounding chrome.

---

## Typography

Source: `src/app.css:99-101` declares font families only — no named size/weight tokens. The project uses literal `font-size` values per component. This phase inherits the Toolbar and ReviewPanel header precedents.

| Role | Size | Weight | Line Height | Family | Usage in this phase |
|------|------|--------|-------------|--------|---------------------|
| Toolbar button label | 12px | 400 (browser default for `<button>`) | normal | `var(--font-sans)` | New Toolbar Review button label "Review" — must match existing Undo / Push / Branch buttons (Toolbar.svelte:214) |
| Panel header button label | 12px | 400 | normal | `var(--font-sans)` (`font-family: inherit`) | Renamed Copy button label in ReviewPanel header (mirrors ReviewDocPreview.svelte:129) |
| Body / panel content | 12px | 400 | 1.5 | `var(--font-sans)` | ReviewPanel list body (ReviewPanel.svelte:379-380) — unchanged |
| Affordance glyph "✓" | 14px (default span inheriting button font + Lucide `size={14}` for icons) | 400 | normal | inherit | "✓ Copied" check mark inside the Copy button — mirrors ReviewDocPreview.svelte:57 |

Weight palette for this phase: **one weight (400)**. No bold / semibold / display weights are added. The toggle-button "active" emphasis is conveyed by color, not weight.

Icon sizing: **`size={14}`** for all Lucide glyphs added in this phase (toolbar Review icon, ReviewPanel Copy icon). Matches every existing `.toolbar-btn` icon at `size={14}` (Toolbar.svelte) and the ReviewDocPreview Copy button (line 60).

---

## Color

The project uses CSS custom properties exclusively — no hex literals in components (CLAUDE.md: "Never inline colors — always use CSS custom properties from the theme"). All tokens below already exist in `src/app.css` and are inherited.

| Role | Token | Resolved Value | Usage in this phase |
|------|-------|----------------|---------------------|
| Dominant (60%) — app background | `var(--color-bg)` | `#0d1117` | Drag-region toolbar surface (existing), inherited via parent |
| Secondary (30%) — chrome surfaces | `var(--color-surface)` | `#161b22` | Toolbar background (existing parent), ReviewPanel header (existing) |
| Primary text | `var(--color-text)` | `#c9d1d9` | All Toolbar button labels, Copy button label (default state) |
| Muted text | `var(--color-text-muted)` | `#8b949e` | Copy button label (default state, mirrors ReviewDocPreview.svelte:123) |
| Border | `var(--color-border)` | `#30363d` | Toolbar button hover background, Copy button border, header underline |
| Hover surface | `var(--color-hover)` | `rgba(255,255,255,0.06)` | Copy button hover (mirrors ReviewDocPreview.svelte:133) |
| Accent (10%) — toggle ACTIVE state | `var(--color-accent)` | `#388bfd` | Toolbar Review button background **only when `reviewActive === true`** |
| On-accent text | `var(--color-on-accent)` | `#fff` | Toolbar Review button label color **only when `reviewActive === true`** |
| Destructive | `var(--color-danger)` | `#f87171` | Not used in this phase (no destructive action introduced) |

**Accent (`var(--color-accent)`) is reserved in this phase for exactly one element:**

- **The Toolbar Review button's `background` and the `color` of its label/icon when `reviewActive === true`.** No other surface in Phase 72 uses accent. The defunct blue button at `RepoView.svelte:815-827` (which currently misuses accent as a no-op breadcrumb) is being deleted as part of this phase — its removal is the *reason* accent can be safely reclaimed for the new toggle indicator without ambiguity.

**Active-state contract for the Toolbar Review button:**

| State | `background` | `color` (label + icon) | `border` |
|-------|--------------|------------------------|----------|
| Default (review off) | `transparent` (inherits `var(--color-surface)`) | `var(--color-text)` | none (matches sibling `.toolbar-btn`) |
| Hover (review off) | `var(--color-border)` | `var(--color-text)` | none |
| Active (`reviewActive === true`) | `var(--color-accent)` | `var(--color-on-accent)` | none |
| Hover while active | `var(--color-accent)` (unchanged — already filled) | `var(--color-on-accent)` | none |

Contrast: `var(--color-on-accent)` (#fff) on `var(--color-accent)` (#388bfd) — APCA Lc ≈ 71 (AAA for body text equivalent). Verified by tokens already in use elsewhere in the project (CommitRow review markers, Phase 66).

**Wiring discipline (CLAUDE.md rule):**

- The active-state styling MUST be a class toggle (`class:toolbar-btn-active={reviewActive}`), not inline `style="background: ..."` ternaries. The deleted blue button at `RepoView.svelte:815-827` is the explicit anti-pattern (eight lines of inline-style ternary) we are removing — recreating that pattern on the new button defeats the refactor.
- No new color tokens introduced.

---

## Copywriting Contract

Every user-facing string this phase touches. Pre-populated from CONTEXT.md decisions and Phase 71 carry-forward.

| Element | Copy | Notes |
|---------|------|-------|
| Toolbar Review button label | `Review` | One word, sentence case, matches `Branch` / `Stash` / `Pop` precedent in the same toolbar group. Singular noun, not verb. |
| Toolbar Review button `aria-label` | (not set — visible label suffices) | `aria-pressed={reviewActive}` IS set (RESEARCH Pitfall 5) — screen readers announce "Review toggle button, pressed/not pressed". |
| Toolbar Review button `title` (tooltip) | not required for v1 — keyboard shortcut is discoverable via the View menu | Future polish; not in scope. |
| ReviewPanel Copy button label (default) | `Copy` | Single word verb, replacing the previous "Generate". The Generate→Copy collapse is the whole point of the phase. |
| ReviewPanel Copy button label (success affordance) | `✓ Copied` | Exact carry-forward from Phase 71 (`ReviewDocPreview.svelte:55-58`). Glyph is a literal `✓` (U+2713) with `aria-hidden="true"`, then the word `Copied`. Affordance window: 1500 ms. |
| ReviewPanel Copy button `title` (when disabled) | `Add at least one comment to generate` | Inherited verbatim from the current Generate button (`ReviewPanel.svelte:364`). The user's intent — "generate then copy" — is collapsed into one button but the gating reason is unchanged: zero comments → nothing to copy. |
| Error toast on copy failure | `Failed to copy: ${msg}` | Exact carry-forward from Phase 71 (`ReviewDocPreview.svelte:42`). `msg` is `e.message` when `e instanceof Error`, else `String(e)`. |
| Empty-state heading (no commits) | `No commits in this review yet.` | Unchanged from existing ReviewPanel (`ReviewPanel.svelte:385`). |
| Empty-state body (no commits) | `Add commits from the graph to start reviewing.` | Unchanged (`ReviewPanel.svelte:387`). |
| Empty-state heading (no comments) | `No comments yet.` | Unchanged (`ReviewPanel.svelte:392`). |
| Empty-state body (no comments) | `Select lines in a diff to comment, or add a note to a commit above.` | Unchanged (`ReviewPanel.svelte:394`). |
| Destructive confirmations | not applicable | Phase 72 introduces no destructive action. The existing "Delete this comment? This cannot be undone." dialog (`ReviewPanel.svelte:291`) is untouched. |

**Copy that is NOT changing but is removed by component deletion:**

- `← Back to comments` (button label on `ReviewDocPreview.svelte:53`) — disappears with the file. The Copy action now lives on the ReviewPanel; there is no preview face to back out of.

**Menu / accelerator copy (Tauri-side, Rust):**

- Menu item label: `Start/End Code Review` (unchanged from `src-tauri/src/lib.rs:28`).
- Keyboard shortcut: `CmdOrCtrl+Shift+R` (new; added via `.accelerator(...)`). macOS displays this as `⌘⇧R` natively in the View menu — no frontend rendering needed.

---

## Interaction Contract (Phase 72-specific)

This phase is interaction-heavy and refactor-light. Each interaction is defined to the level the executor can implement without ambiguity.

### Interaction 1: Toolbar Review button click

| Property | Value |
|----------|-------|
| Trigger | `onclick` on the new `.toolbar-btn` |
| Effect | `void emit("review-toggle")` via `@tauri-apps/api/event` |
| Routing | The Rust menu handler at `src-tauri/src/lib.rs:67-69` already emits the same event when the menu item fires. `App.svelte:557` is the single listener that flips `reviewPanelOpen`. The Toolbar becomes a second emitter onto the existing bus. |
| Disabled state | Never disabled. The button is always present and always actionable when the Toolbar is mounted (the Toolbar is only mounted when `activeTab?.repoPath` is truthy — see `App.svelte:584` — so the button's existence already implies a repo context). |
| Visual feedback latency | Immediate (next frame): `reviewActive` prop flips → class toggle re-renders → background swaps to accent. No spinner, no delay. |
| Error handling | `void` the emit promise. Emit on the local event bus never user-fails (RESEARCH Assumption A5); wrapping in try/catch would be defensive over our own code (CLAUDE.md). |

### Interaction 2: Cmd+Shift+R keyboard shortcut

| Property | Value |
|----------|-------|
| Trigger | OS-level keyboard accelerator on the existing menu item `review-toggle` |
| Implementation | `.accelerator("CmdOrCtrl+Shift+R")` chained onto `MenuItemBuilder::with_id("review-toggle", "Start/End Code Review")` at `src-tauri/src/lib.rs:28` |
| Display | macOS native: `⌘⇧R` appears in the View menu next to the item, automatically. No frontend rendering. |
| Conflict surface | Browsers' Ctrl+Shift+R is reload — irrelevant inside a Tauri webview where browser shortcuts are not bound. Verified zero conflict with existing app shortcuts (only `Cmd+F` at lib.rs:22). |
| Scope | Global to the app while the focused window has the menu. Works regardless of which pane is focused. |

### Interaction 3: ReviewPanel Copy button click

| Property | Value |
|----------|-------|
| Trigger | `onclick` on the renamed `.copy-button` in the ReviewPanel header (replaces the existing Generate button at `ReviewPanel.svelte:359-368`) |
| Effect (happy path) | `await session.generate(repoPath)` → `await writeText(markdown)` → `copied = true` for 1500 ms → label flips back to `Copy` |
| Effect (failure) | `showToast(\`Failed to copy: ${msg}\`, "error")`; button label stays `Copy` (does NOT flip to `Copied`) |
| Disabled state | Disabled when `!hasAnyComment` (zero comments in session). Tooltip: `Add at least one comment to generate` (carry-forward). |
| Re-click during 1500 ms affordance window | Extends the window. `clearTimeout(copyTimer)` BEFORE `setTimeout(...)` (Pitfall 2, carry-forward from Phase 71). |
| Error narrowing | `instanceof Error` → `e.message`; else `String(e)`. Never `as any`, never `as Error` (CLAUDE.md). |
| Cached markdown | None. Each click re-invokes `generate_review_doc`. IPC is local, comment-count-bounded, cheap (CONTEXT.md design rationale; RESEARCH Anti-Pattern). |

### Interaction 4: Returning from a jumped-to diff back to the comments view

| Property | Value |
|----------|-------|
| Trigger | User clicks DiffPanel's existing close affordance after jumping to a diff from a comment |
| Effect | `handleDiffClose()` + `reviewSession.showPanel()` (already wired at `RepoView.svelte:839`) |
| Replaces | The deleted blue Review button at `RepoView.svelte:815-827`, which was the only "back" affordance and was visually broken (active-looking but no-op when on the panel). |
| New surface | None. The existing DiffPanel close button is the sole back path. The Toolbar Review button is for entering/exiting review mode entirely, not for swapping panes within it. |

### Layout placement

- **Toolbar Review button:** new `.toolbar-group` appended as the rightmost group in `Toolbar.svelte` (after the existing Branch / Stash / Pop group at `Toolbar.svelte:264-274`). Single button, no sibling buttons in the group. Lays out via the existing `.toolbar` flexbox with `gap: 12px` between groups — no positioning hacks (CLAUDE.md).
- **ReviewPanel Copy button:** stays in the same docked position the Generate button occupies today — right-edge of the panel header row, pushed by `<span class="preview-spacer" style="flex: 1;"></span>` (`ReviewPanel.svelte:358`). Only the label, icon, and click handler change.
- **Deleted header strip in `RepoView.svelte`:** lines 813-828 collapse out. Body inside `{#if reviewSession.state.reviewActive}` becomes a direct render of `ReviewPanel` or `DiffPanel` without the surrounding wrapper. Children flow naturally via the parent's existing flex column (`RepoView.svelte:809`).

---

## Iconography (Phase 72-specific)

| Element | Icon | Lucide export | Size | Source |
|---------|------|---------------|------|--------|
| Toolbar Review button | `MessagesSquare` | `MessagesSquare` from `@lucide/svelte` | `size={14}` | CONTEXT.md design choice 1; verified at `node_modules/@lucide/svelte/dist/icons/index.js:1042` |
| ReviewPanel Copy button (default state) | clipboard glyph | `Clipboard` from `@lucide/svelte` | `size={14}` | Carry-forward from `ReviewDocPreview.svelte:13`; verified at `node_modules/@lucide/svelte/dist/icons/index.js:453` |
| ReviewPanel Copy button (✓ Copied state) | literal `✓` (U+2713) in a `<span aria-hidden="true">` | not a Lucide icon | inherits font-size (12px) | Carry-forward from `ReviewDocPreview.svelte:57` |

**Icon-choice rationale (Toolbar Review button):** `MessagesSquare` (stacked multiple speech bubbles) signals multi-comment / review semantics and is visually distinct from `MessageSquareText` (used elsewhere in the codebase as the per-comment add-note glyph) and `Clipboard` (used for the new Copy action). The four candidates from CONTEXT.md (`MessagesSquare` / `MessageSquareText` / `BookOpenCheck` / `ClipboardList`) are all verified present in Lucide 0.577.0; the design lock chooses `MessagesSquare` for distinctness within the same screen.

---

## Accessibility Contract

| Element | Property | Value | Why |
|---------|----------|-------|-----|
| Toolbar Review button | `aria-pressed` | `{reviewActive}` (boolean) | Toggle-button ARIA pattern; screen readers announce pressed / not-pressed state (RESEARCH Pitfall 5). Free a11y win, no design change. |
| Toolbar Review button | Visible label | `Review` (text inside the button) | No `aria-label` needed when visible text is sufficient. |
| Toolbar Review button | Keyboard focus | inherits from `<button>` native behavior | Tab order falls naturally after the Branch / Stash / Pop group. |
| ReviewPanel Copy button | `aria-disabled` / `disabled` | `disabled={!hasAnyComment}` | Native `disabled` prevents click + announces unavailable. |
| ReviewPanel Copy button | `title` (when disabled) | `Add at least one comment to generate` | Surfaces the gating reason to mouse users (carry-forward). |
| `✓` glyph inside Copy button | `aria-hidden` | `"true"` | The adjacent text `Copied` carries the meaning; the check is decorative (carry-forward from `ReviewDocPreview.svelte:57`). |
| Error toast | inherited from `showToast` | existing toast a11y (project-level) | No change. |

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| none | none — no shadcn, no third-party component blocks, no new packages | not applicable |

Phase 72 introduces zero new dependencies (CONTEXT.md Net effect table: "Files added: 0"; threat model T-72-SC: "zero new packages"; RESEARCH §Package Legitimacy Audit: skipped). Every package referenced (`@lucide/svelte`, `@tauri-apps/api/event`, `@tauri-apps/plugin-clipboard-manager`, `vitest`, `@testing-library/svelte`) is pre-existing in `package.json` and was vetted in prior phases (65, 70, 71).

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS — every user-facing string declared; carry-forward strings cite their source line; no destructive copy introduced
- [ ] Dimension 2 Visuals: PASS — every new surface either matches a documented Toolbar/ReviewPanel precedent or is explicitly justified (the active-state styling is the one new visual contract, and it reuses `var(--color-accent)` reclaimed from the deleted blue button)
- [ ] Dimension 3 Color: PASS — zero hex literals introduced; all colors via existing `var(--color-*)` tokens; accent reserved for exactly one element (the Toolbar Review button's active state)
- [ ] Dimension 4 Typography: PASS — single weight (400), single size for new UI (12px), one icon size (14px); inherits `var(--font-sans)`
- [ ] Dimension 5 Spacing: PASS — every value matches a documented Toolbar or ReviewPanel-header precedent; no new spacing magic numbers
- [ ] Dimension 6 Registry Safety: PASS — N/A (no registry, no new packages)

**Approval:** pending

---

*Spec compiled: 2026-05-27 from CONTEXT.md (design-locked 2026-05-26), RESEARCH.md (researched 2026-05-27), and direct reads of `src/app.css`, `src/components/Toolbar.svelte`, `src/components/ReviewPanel.svelte`, `src/components/ReviewDocPreview.svelte`, `src/components/RepoView.svelte`.*
