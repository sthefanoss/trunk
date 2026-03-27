# Phase 54: Frontend Unit Tests - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Unit tests for all TypeScript utility modules and Svelte components in the frontend codebase. Tests verify behavior, state transitions, user interactions, and event handling. Existing utility tests are reviewed and expanded. All tests run deterministically via `bun run test` and pass in CI.

</domain>

<decisions>
## Implementation Decisions

### Component Testing Strategy
- **D-01:** Use `@testing-library/svelte` for component tests. Renders components in jsdom, queries by accessible roles/text. Svelte 5 runes support required.
- **D-02:** All 20+ Svelte components get tests — not just logic-heavy ones.
- **D-03:** Even simple components (RefPill, CommitRow, BranchRow, FileRow, OperationBanner) get render + props + events testing depth — verify rendering, prop display, click handlers, event emissions, and conditional rendering.

### Test Coverage Scope
- **D-04:** Mock `invoke` at module level via `vi.mock('$lib/invoke')` so all component tests get a working Tauri mock. Test the mock setup once, components just work.
- **D-05:** Skip testing `types.ts` (pure type definitions, no runtime logic) and `tab-types.ts` (already covered by `store.test.ts`).
- **D-06:** Review and expand all existing utility tests (10 modules, ~2,357 lines) for missing edge cases. Not just adding new tests — also auditing existing coverage.

### Shared Test Utilities
- **D-07:** Extract factory functions (`makeCommit`, `makeEdge`, `makeFile`, etc.) to `src/__tests__/helpers/`. Reduces duplication across test files and makes factories available to component tests.
- **D-08:** Shared Tauri invoke mock lives in `src/__tests__/helpers/tauri-mock.ts`. Component tests import it for consistent mocking.

### Existing Test Gaps
- **D-09:** Existing store tests (toast, undo-redo, remote-state) remain as pure logic tests. Component tests will exercise the rendering side. No duplicate rendering tests for stores.

### Claude's Discretion
- Exact jsdom/happy-dom environment choice for vitest
- Which edge cases to add when reviewing existing utility tests
- Component test file naming convention (collocated vs `__tests__/` directory)
- How to structure @testing-library/svelte imports and render patterns

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — UNIT-02 (TypeScript utility unit tests) and UNIT-03 (Svelte component unit tests)

### Prior Phase Context
- `.planning/phases/53-rust-unit-tests-test-harness/53-CONTEXT.md` — Test harness patterns (D-08: descriptive naming, D-09: assertion helpers) to carry forward

### Existing Test Files
- `src/lib/active-lanes.test.ts` — Largest existing test (~550 lines), established patterns for graph data testing
- `src/lib/toast.svelte.test.ts` — Svelte store test pattern with timer mocking
- `src/lib/build-tree.test.ts` — Factory function pattern (`makeFile()`)
- `src/lib/merge-parser.test.ts` — String parsing test patterns

### Configuration
- `vite.config.ts` — Current vitest configuration (test section, environment, patterns)
- `package.json` — Test scripts and current dependencies

### Components to Test
- `src/components/` — All Svelte component files (20+)
- `src/lib/` — All utility modules and Svelte stores

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `makeCommit()` factory in `active-lanes.test.ts` — creates GraphCommit with sensible defaults. Will be extracted to shared helpers.
- `makeEdge()` factory in `active-lanes.test.ts` — creates GraphEdge. Will be extracted.
- `makeFile()` factory in `build-tree.test.ts` — creates FileStatus. Will be extracted.
- `_resetToasts()` in `toast.svelte.ts` — test reset function for store state. Pattern to follow for other stores.
- `resetCache()` in `text-measure.ts` — cache reset for deterministic testing.

### Established Patterns
- `describe`/`it` blocks organized by behavior, not implementation
- `vi.useFakeTimers()` + `vi.advanceTimersByTime()` for timer-based logic
- Direct function import and invocation for pure utilities
- `.svelte.test.ts` extension for Svelte store tests
- `beforeEach` hooks for state reset

### Integration Points
- `bun run test` (vitest run) — CI gate already exists from Phase 50
- `vite.config.ts` test section — needs environment update (jsdom) for component tests
- `package.json` — needs `@testing-library/svelte` and `jsdom` (or `happy-dom`) as devDependencies

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 54-frontend-unit-tests*
*Context gathered: 2026-03-26*
