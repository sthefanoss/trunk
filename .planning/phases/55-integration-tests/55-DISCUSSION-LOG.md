# Phase 55: Integration Tests - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-26
**Phase:** 55-integration-tests
**Areas discussed:** IPC test approach, Git operation sequences, Watcher test strategy, Test organization

---

## IPC Test Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Serialization round-trip tests | Verify Rust command return types serialize/deserialize correctly at serde boundary. No Tauri runtime needed. | ✓ |
| Full Tauri runtime via tauri-test | Spin up real Tauri app instance, call invoke(), check responses. Heavier, platform-dependent. | |
| You decide | Claude picks the approach. | |

**User's choice:** Serialization round-trip tests
**Notes:** None

### Follow-up: Command Scope

| Option | Description | Selected |
|--------|-------------|----------|
| All commands | Every registered Tauri command gets a round-trip test (~30 commands). | ✓ |
| Key commands only | Focus on commands with complex return types. | |
| You decide | Claude selects based on type complexity and risk. | |

**User's choice:** All commands
**Notes:** None

---

## Git Operation Sequences

| Option | Description | Selected |
|--------|-------------|----------|
| Multi-step workflows | Test realistic user flows: init→commit→branch→merge, etc. | |
| State transition chains | Focus on state transitions between operation modes. | |
| Both workflows + state chains | Full coverage of realistic workflows AND state transition correctness. | ✓ |
| You decide | Claude selects based on Phase 53 gaps. | |

**User's choice:** Both workflows + state chains
**Notes:** None

---

## Watcher Test Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Real fs events with generous timeouts | Write files to tempdir, wait with generous timeout (2s). Tests actual notify crate. | ✓ |
| Mock the notify crate | Inject mock watcher with synthetic events. Deterministic but less realistic. | |
| Skip watcher integration tests | Unit-test start/stop logic only, defer to E2E tests. | |
| You decide | Claude picks approach balancing coverage vs CI reliability. | |

**User's choice:** Real fs events with generous timeouts
**Notes:** None

### Follow-up: Tauri Dependency

| Option | Description | Selected |
|--------|-------------|----------|
| Extract watcher logic behind a trait | Refactor to accept callback/trait. Test with channel receiver. | |
| Use tauri::test module | Use tauri::test::mock_builder() for test AppHandle instances. | ✓ |
| You decide | Claude picks based on practicality. | |

**User's choice:** Use tauri::test module
**Notes:** None

---

## Test Organization

| Option | Description | Selected |
|--------|-------------|----------|
| Same directory, tagged files | Integration tests in src-tauri/tests/ alongside unit tests, named test_integ_*.rs. | |
| Separate integration directory | New src-tauri/tests/integration/ with clear separation. | ✓ |
| You decide | Claude picks approach. | |

**User's choice:** Separate integration directory
**Notes:** None

### Follow-up: Shared Infrastructure

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse existing common/ | Import TestContext, builders, drivers from shared common/ module. | ✓ |
| Own integration helpers | Integration tests get own helper module. | |

**User's choice:** Reuse existing common/
**Notes:** None

---

## Claude's Discretion

- Exact Cargo configuration for separate integration test directory
- Specific multi-step workflow scenarios beyond discussed examples
- Serialization round-trip test implementation details
- Watcher test timeout values and retry strategy
- tauri::test mock builder setup structure

## Deferred Ideas

None — discussion stayed within phase scope.
