# Phase 54: Frontend Unit Tests - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-26
**Phase:** 54-frontend-unit-tests
**Areas discussed:** Component testing strategy, Test coverage scope, Shared test utilities, Existing test gaps

---

## Component Testing Strategy

### How to test Svelte 5 components

| Option | Description | Selected |
|--------|-------------|----------|
| @testing-library/svelte | Industry standard. Renders in jsdom, queries by roles/text. Svelte 5 support. ~2 deps. | ✓ |
| Logic-only testing | Skip DOM. Test extracted logic in .svelte.ts stores/utilities. No new deps. | |
| You decide | Claude picks best approach. | |

**User's choice:** @testing-library/svelte
**Notes:** None

### Which components should get tests

| Option | Description | Selected |
|--------|-------------|----------|
| Logic-heavy only | Focus on ~8-10 complex components (CommitGraph, RebaseEditor, MergeEditor, etc.) | |
| All components | Every component gets at least render + basic interaction test. | ✓ |
| You decide | Claude picks based on complexity and risk. | |

**User's choice:** All components
**Notes:** None

### Testing depth for simpler components

| Option | Description | Selected |
|--------|-------------|----------|
| Render + props | Verify rendering and correct prop display. Minimal interaction testing. | |
| Render + props + events | Also verify click handlers, event emissions, conditional rendering. | ✓ |
| You decide | Claude picks appropriate depth per component. | |

**User's choice:** Render + props + events
**Notes:** None

---

## Test Coverage Scope

### invoke.ts handling

| Option | Description | Selected |
|--------|-------------|----------|
| Mock at module level | vi.mock('$lib/invoke') globally. Test mock setup once, components just work. | ✓ |
| Skip testing invoke.ts | Thin wrapper around @tauri-apps/api. Mock for components but don't test itself. | |
| You decide | Claude picks. | |

**User's choice:** Mock at module level
**Notes:** None

### types.ts and tab-types.ts

| Option | Description | Selected |
|--------|-------------|----------|
| Skip types.ts, dedicated test for tab-types.ts | types.ts has no runtime logic. Give tab-types.ts own test file. | |
| Skip both | types.ts is just types, tab-types.ts already covered by store.test.ts. | ✓ |
| You decide | Claude picks based on testable logic. | |

**User's choice:** Skip both
**Notes:** None

---

## Shared Test Utilities

### Factory function location

| Option | Description | Selected |
|--------|-------------|----------|
| src/lib/test-utils/ | Under $lib with code being tested. Importable as $lib/test-utils/factories. | |
| src/__tests__/helpers/ | Dedicated test directory at src root. Separate from production code. | ✓ |
| Keep collocated | Each test file defines own factories. Some duplication fine. | |

**User's choice:** src/__tests__/helpers/
**Notes:** User initially asked about src-tauri/tests/ but that's the Rust test harness. Clarified that frontend test utilities must be in src/ for vitest/TypeScript to resolve.

### Tauri invoke mock location

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, shared mock setup | src/__tests__/helpers/tauri-mock.ts with vi.mock for invoke. | ✓ |
| Vitest setup file | Auto-mock via vitest setupFiles. No explicit import needed. | |
| You decide | Claude picks. | |

**User's choice:** Shared mock setup in src/__tests__/helpers/tauri-mock.ts
**Notes:** None

---

## Existing Test Gaps

### Existing utility test review

| Option | Description | Selected |
|--------|-------------|----------|
| Focus on components | Existing utility tests are solid. Spend effort on untested components. | |
| Review and expand all | Audit existing tests for missing edge cases, then add component tests. | ✓ |
| You decide | Claude assesses quality and fills gaps. | |

**User's choice:** Review and expand all
**Notes:** None

### Store test rendering

| Option | Description | Selected |
|--------|-------------|----------|
| Keep as logic tests | Stores tested at state level. Component tests exercise rendering. No duplication. | ✓ |
| Add rendering tests too | Test store changes update DOM in components. Double coverage. | |
| You decide | Claude picks based on rendering bug likelihood. | |

**User's choice:** Keep as logic tests
**Notes:** None

---

## Claude's Discretion

- Exact jsdom/happy-dom environment choice for vitest
- Which edge cases to add when reviewing existing utility tests
- Component test file naming convention
- @testing-library/svelte render patterns

## Deferred Ideas

None — discussion stayed within phase scope.
