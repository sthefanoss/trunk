---
created: 2026-03-20T00:25:55.437Z
title: Limit horizontal graph scroll to graph column only
area: ui
files:
  - src/components/CommitGraph.svelte:803-809
---

## Problem

The `onwheel` handler that controls horizontal graph scrolling is attached to the entire center pane wrapper (`div.flex-1.overflow-hidden` at line 803). This means any horizontal scroll gesture (trackpad swipe or shift+wheel) anywhere in the center pane — over the message column, author column, date column, etc. — triggers horizontal panning of the git graph. This is annoying because normal trackpad usage over non-graph columns unintentionally scrolls the graph.

## Solution

Restrict the horizontal scroll (`deltaX`) handling so it only triggers when the pointer is over the graph column itself. Options:
1. Move the `onwheel` handler (or the `deltaX` branch of it) to the graph column's own element instead of the outer wrapper
2. Use hit-testing in the wheel handler to check if the pointer X position falls within the graph column bounds before applying `graphScrollX`

Vertical scrolling should remain unchanged (whole pane scrolls vertically together).
