---
status: resolved
trigger: "Window zoom (Cmd+/Cmd-) is broken — content overflows, scrollbars appear, commit graph disappears"
created: 2026-04-02T00:00:00Z
updated: 2026-04-02T00:00:00Z
---

## Current Focus

hypothesis: CONFIRMED. VirtualList uses getBoundingClientRect() (returns zoomed px) for height measurements but uses scrollTop (returns unzoomed CSS-px) for scroll position. At zoom>1, heightCache stores inflated values, causing transformY to push items too far down relative to the SVG overlay (which uses fixed ROW_HEIGHT=26 in CSS-px). The coordinate system mismatch causes SVG lines to not align with commit rows.
test: Fix VirtualList to divide getBoundingClientRect() by currentCSSZoom to keep all measurements in CSS-px
expecting: SVG overlay aligns with commit rows at all zoom levels
next_action: Implement zoom compensation in VirtualList getBoundingClientRect calls

## Symptoms

expected: Content should scale like a browser — text, UI elements, and layout all zoom in/out proportionally
actual: Multiple issues: (1) Content overflows the window, (2) Scrollbars appear/disappear unexpectedly, (3) Commit graph disappears when zooming
errors: None reported
reproduction: Use Cmd+ to zoom in on the app window
started: Ongoing issue

## Eliminated

## Evidence

- timestamp: 2026-04-02T00:01:00Z
  checked: How zoom is applied
  found: App.svelte line 365 sets `document.documentElement.style.zoom = String(zoomLevel)`. This is a CSS zoom on the root element.
  implication: CSS zoom scales all content but viewport units (vh/vw) still reference the physical viewport, not the zoomed layout viewport.

- timestamp: 2026-04-02T00:02:00Z
  checked: Root layout container
  found: App.svelte line 479 uses `class="flex flex-col h-screen"` which maps to `height: 100vh`. WelcomeScreen also uses h-screen at line 71.
  implication: When zoom > 1, 100vh = physical viewport height, but zoomed content is larger than physical viewport. The zoomed content overflows because the container is sized to the unzoomed viewport.

- timestamp: 2026-04-02T00:03:00Z
  checked: VirtualList height measurement
  found: VirtualList uses getBoundingClientRect().height at lines 319, 327, 475, 516. Under CSS zoom, getBoundingClientRect returns values in CSS pixels (zoomed), not physical pixels.
  implication: VirtualList measurements should be correct since getBoundingClientRect already accounts for zoom. The SVG disappearing is likely caused by the root container overflow — when h-screen doesn't match the zoomed size, the flex layout breaks and the commit graph area gets zero or near-zero height.

- timestamp: 2026-04-02T00:04:00Z
  checked: Tailwind h-screen definition
  found: h-screen = height: 100vh. Under CSS zoom, 100vh = physical viewport, not the zoomed viewport.
  implication: The root fix is to stop using viewport units. Use height: 100% on html+body+root instead, which flows from the actual document dimensions.

- timestamp: 2026-04-02T00:05:00Z
  checked: Title bar padding-left uses zoom compensation
  found: App.svelte line 481 already does `padding-left: {isFullscreen ? 0 : 78 / zoomLevel}px` — dividing by zoomLevel to compensate for CSS zoom. This proves the team is aware zoom affects pixel values.
  implication: The pattern of dividing by zoomLevel is already used; the h-screen problem needs a different fix (avoid viewport units entirely).

- timestamp: 2026-04-02T01:00:00Z
  checked: How VirtualList measures item heights
  found: VirtualList uses getBoundingClientRect().height in 9 call sites (VirtualList.svelte lines 319, 327, 475, 516; virtualList.js lines 210, 295, 339; ReactiveListManager lines 650, 653). getBoundingClientRect returns ZOOMED values under CSS zoom. scrollTop returns UNZOOMED values.
  implication: heightCache stores zoomed values (26*zoom per row), causing transformY to be inflated. SVG overlay uses fixed ROW_HEIGHT=26 in CSS-px. When scrolled, items are positioned at zoomed offsets but SVG dots/lines at unzoomed offsets, creating growing misalignment.

- timestamp: 2026-04-02T01:01:00Z
  checked: MDN spec for CSS zoom coordinate systems
  found: Per MDN currentCSSZoom docs: "getBoundingClientRect returns the zoomed size, but other APIs like client*, offset*, scroll* return the un-zoomed size, so to convert coordinates between them you need currentCSSZoom". Fix is to divide getBoundingClientRect by currentCSSZoom.
  implication: All getBoundingClientRect calls in VirtualList need zoom compensation to return CSS-px values consistent with scrollTop, translateY, and SVG coordinates.

- timestamp: 2026-04-02T01:02:00Z
  checked: Visible range calculation under zoom
  found: calculateVisibleRange uses scrollTop (unzoomed) / averageHeight (zoomed) for start index. At zoom 1.5, start = floor(scrollTop / (26*1.5)) which gives fewer items than actual. contentHeight = totalHeight (zoomed) which inflates the scroll range. Items rendered via translateY(sum of zoomed heights) but flow at 26px CSS each, causing progressive misalignment with SVG overlay.
  implication: The bug manifests when scrolling at zoom>1. At scroll position 0 everything aligns, but drift increases with scroll depth.

## Resolution

root_cause: CSS zoom (`document.documentElement.style.zoom`) creates a split in DOM API coordinate systems — getBoundingClientRect returns zoomed values while scrollTop/offsetHeight/translateY return unzoomed values. This caused overflow (100vh refers to physical viewport), VirtualList height mismatches, and progressive graph drift.
fix: Replaced CSS zoom with Tauri's native webview zoom (`getCurrentWebview().setZoom()`), which operates at the rendering pipeline level and keeps all DOM coordinate systems internally consistent. Added `core:webview:allow-set-webview-zoom` permission. This eliminated the entire class of bugs with a 2-file, +5/-3 line change instead of patching 7 files with coordinate workarounds.
verification: All 6 checks pass. Awaiting human visual verification with Cmd+/Cmd- zoom.
files_changed: [src/App.svelte, src-tauri/capabilities/default.json]
