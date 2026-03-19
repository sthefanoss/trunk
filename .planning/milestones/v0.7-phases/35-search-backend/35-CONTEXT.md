# Phase 35: Search Backend - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement a backend `search_commits` command that queries CommitCache for commits matching SHA, message, branch/ref, or author — returning all matches regardless of frontend pagination. Requirements: SRCH-02, SRCH-03, SRCH-04, SRCH-05, SRCH-11. No frontend UI changes — Phase 36 handles search overlay.

</domain>

<decisions>
## Implementation Decisions

### SearchResult shape
- Return `Vec<SearchResult>` where each result has `oid: String` and `match_types: Vec<MatchType>`
- `MatchType` enum: `Sha`, `Message`, `Ref`, `Author`
- Collect ALL match types per commit — if "main" matches both a ref and the message, report both `[Message, Ref]`
- No matched text snippets — frontend already has full GraphCommit data in memory

### Unified search command
- Single `search_commits(path, query)` command that checks all fields simultaneously
- No per-field commands, no field filter parameter — unified search only
- Scoped prefixes (`author:`, `sha:`) are explicitly deferred to SRCH-E01

### Field matching strategies
- **SHA**: `oid.to_lowercase().starts_with(&query)` — case-insensitive, any prefix length
- **Message**: case-insensitive substring match on both `summary` AND `body` (skip body if None)
- **Ref**: case-insensitive substring match on `RefLabel.short_name` only (not full ref path)
- **Author**: case-insensitive substring match on `author_name`

### Query handling
- Accept any non-empty string (no minimum length)
- Empty query returns empty results `vec![]`
- Trim leading/trailing whitespace before matching
- All comparisons case-insensitive (lowercase both sides)

### Result ordering & limits
- Return matches in graph order (same order as CommitCache — topological + timestamp)
- No result cap — return all matches (lightweight: just OID + enum per result)
- Frontend needs full count for "3 of 17 matches" display (SRCH-07)

### Claude's Discretion
- Whether to lowercase the query once upfront or per-field
- Inner-fn parameter design (must follow established inner-fn pattern)
- Test fixture design (multi-field matches, no matches, edge cases)
- Whether `search_commits` lives in `history.rs` or a new `search.rs` file

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### CommitCache & graph data
- `src-tauri/src/state.rs` — `CommitCache` structure (Mutex<HashMap<String, GraphResult>>). Search iterates over cached `GraphResult.commits`.
- `src-tauri/src/git/types.rs` §52-69 — `GraphCommit` struct with all searchable fields: `oid`, `short_oid`, `summary`, `body`, `author_name`, `author_email`, `refs` (Vec<RefLabel>)
- `src-tauri/src/git/types.rs` §71-78 — `RefLabel` struct with `short_name` field used for ref matching

### Existing command patterns
- `src-tauri/src/commands/history.rs` — `get_commit_graph` and `refresh_commit_graph` commands that read from CommitCache. Search command follows same cache-read pattern.
- `src-tauri/src/commands/staging.rs` — Inner-fn pattern examples: sync `_inner` function with `&str`/`&HashMap` params, async `#[tauri::command]` wrapper with `spawn_blocking`

### Command registration
- `src-tauri/src/lib.rs` §21-65 — `invoke_handler` list where `search_commits` must be registered
- `src-tauri/src/commands/mod.rs` — Module declarations for command files

### Type definitions (frontend mirror)
- `src/lib/types.ts` §25-42 — TypeScript `GraphCommit` type. New `SearchResult` and `MatchType` types needed here.
- `src/lib/invoke.ts` — `safeInvoke<T>` IPC wrapper for calling `search_commits`

### Requirements
- `.planning/REQUIREMENTS.md` §24-34 — SRCH-02 through SRCH-05, SRCH-11 requirements
- `.planning/REQUIREMENTS.md` §48-50 — SRCH-E01 (scoped prefixes) explicitly deferred

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `CommitCache`: Already holds all commits — search is a pure cache read, no git2 access needed
- `open_repo_from_state()`: Not needed for search — cache is read directly
- `safeInvoke<T>`: Frontend IPC wrapper for calling the new command
- Inner-fn pattern: `_inner` functions with `&str`/`&HashMap` params for testability

### Established Patterns
- Cache-read commands (get_commit_graph) lock CommitCache, read data, return — search follows same pattern
- `Result<T, TrunkError>` in inner fns, `Result<T, String>` at command boundary
- Serde serialization for all IPC types — new SearchResult/MatchType need `#[derive(Serialize)]`
- Tests: `make_test_repo()` + `make_state_map()` helpers in existing test modules

### Integration Points
- New command registers in `lib.rs` invoke_handler
- Command reads from `CommitCache` (same state access as `get_commit_graph`)
- Frontend will call via `safeInvoke('search_commits', { path, query })` in Phase 36
- New types (SearchResult, MatchType) added to `types.rs` (Rust) and `types.ts` (TypeScript)

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

*Phase: 35-search-backend*
*Context gathered: 2026-03-18*
