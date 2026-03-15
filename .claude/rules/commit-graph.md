---
paths:
  - "src-tauri/src/git/graph.rs"
  - "src/lib/active-lanes.ts"
  - "src/lib/overlay-paths.ts"
  - "src/lib/graph-constants.ts"
  - "src/components/CommitGraph.svelte"
  - "src/lib/graph-svg-data.ts"
---

# Commit Graph Rules

Before making changes to the graph pipeline, read these references:

- @.planning/COMMIT-GRAPH-ARCHITECTURE.md — full architecture of Trunk's 4-layer graph pipeline (Rust → active-lanes → overlay-paths → Svelte)
- @.planning/GITAMINE-ALGORITHM-STUDY.md — study of the "straight branches" algorithm from gitamine, with a detailed comparison against Trunk's algorithm

Key principles:

- Never post-process the output of one layer to fix something the prior layer should have done — the layers are interdependent
- Stashes should use the same algorithm as regular commits. The only differences are visual (dashed square, dashed line)
- Test commands: `cd src-tauri && cargo test --lib` (Rust), `npx vitest run` (TypeScript)
