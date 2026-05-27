---
phase: 73
slug: review-lifecycle-end-review-cold-boot-resume
status: verified
threats_open: 0
asvs_level: 1
created: 2026-05-27
---

# Phase 73 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.
> SECURED — all 11 declared threats CLOSED (4 mitigate verified in code; 7 accept documented).

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Svelte UI → Tauri IPC | `repoPath` flows into `safeInvoke("resume_review_session" / "end_review_session" / "get_review_session_status", …)`. Backend validates via `canonical_repo_path` (review.rs:61-69). | Repo path string (validated, rejects unknown repos) |
| Tauri event bus → Svelte listener | `session-changed` payload (canonical path string) flows into the listener at `ReviewPanel.svelte:447-461`. Cross-repo isolation via `canonicalPath && event.payload !== canonicalPath` filter. | Canonical repo path string |
| User mouse → End button | Two-step inline confirmation (D-05) gates destructive End action. | UI intent |
| setTimeout callback → Svelte component | `endTimer` cleared on unmount via `$effect` teardown to prevent torn-down state mutation. | In-process timer handle |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation / Evidence | Status |
|-----------|----------|-----------|-------------|------------------------|--------|
| T-73-01 | Tampering | `reload()` recursion via `session-changed` re-fire after `resume_review_session` emits | mitigate | Resume branch gated on `sessionState === "resume-available"` at `ReviewPanel.svelte:255`; `sessionState` assigned from `status.state` at `:242` before gate. Recursion-safety test: `ReviewPanel.test.ts:875,893` asserts call count `== 1`. | closed |
| T-73-02 | Tampering | Race deleting session file mid-resume (another tab / external process) | accept | Backend `Mutex<HashMap>` + atomic tmp+rename (Phase 65 D-10). Phase 73 introduces no new IPC handler — only frontend files modified. | closed |
| T-73-03 | Information Disclosure | `Failed to resume review: {extracted}` toast | accept | `errorMessage()` at `:169-173` extracts `.message` only from native `Error` or `TrunkError` (both from our own backend, no PII). `showToast` renders plain text. | closed |
| T-73-04 | Repudiation / DoS | `resume_review_session` retry loops | accept | Single `safeInvoke` call at `:257` inside one try/catch — no loop, retry, or recursion. Persistent failures (RefusedNewer) are user-actionable. | closed |
| T-73-05 | Tampering | Accidental destructive misclick on End button | mitigate | Two-step inline confirmation at `ReviewPanel.svelte:381-408`; `startEndConfirm` arms 3000ms revert timer at `:370-379`; second click invokes IPC at `:396`. Tests 1-3 in `ReviewPanel.test.ts:1038` prove first-click-no-invoke, second-click-invokes-once, auto-revert at `:1126`. | closed |
| T-73-06 | Denial of Service | Timer leak on component unmount | mitigate | Dedicated `$effect` at `ReviewPanel.svelte:435-439` returning `() => { if (endTimer !== null) clearTimeout(endTimer); }`. Test 6 at `ReviewPanel.test.ts:~1212` advances 3000ms post-`unmount()` and asserts no `console.error`. | closed |
| T-73-07 | Tampering | Stale `canonicalPath` after End in same tab | accept | `onEndClick` does not mutate `canonicalPath`. Listener `$effect` at `:447-461` calls `reload()` which refreshes from `status.canonical_path` at `:241`. Post-End cold empty-state branch at `:538` renders. | closed |
| T-73-08 | Information Disclosure | `Failed to end review: {extracted}` toast | accept | Same composition as T-73-03: `errorMessage` at `:405`, `showToast` at `:406`, plain text only. | closed |
| T-73-09 | Spoofing | `session-changed` event for unrelated repo's canonical path | mitigate | Listener filter `if (canonicalPath && event.payload !== canonicalPath) return;` preserved byte-for-byte at `ReviewPanel.svelte:451` (D-09 invariant). Test 2 in `describe("multi-tab coordination")` at `ReviewPanel.test.ts:1477,1482` asserts `safeInvoke.mock.calls.length` unchanged after `fireSessionChanged("/different-repo")`. | closed |
| T-73-10 | Tampering | Race deleting session file while tab B is mid-reload | accept | No new IPC. Backend `Mutex<HashMap>` from Phase 65 serializes. Tab B's reload sees deterministic `state="none"` and renders cold empty state at `:538-544`. | closed |
| T-73-11 | Information Disclosure | Summary caption count metadata via DOM | accept | Caption at `:527-531` derives `{comments.length} comments · {commits.length} commits` from already-rendered state. No new IPC, no PII, no privilege boundary crossed. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-73-01 | T-73-02 | Phase 73 adds no new IPC handler; backend serialization + atomic file ops from Phase 65 D-10 cover the existing handler unchanged. No new race surface. | gsd-security-auditor | 2026-05-27 |
| AR-73-02 | T-73-03 | Error toasts render plain text from our own backend's `.message`; no PII path; no HTML interpolation in `showToast`. | gsd-security-auditor | 2026-05-27 |
| AR-73-03 | T-73-04 | Phase 73 does not add retry loops; one-shot `safeInvoke` with a single user-visible toast on failure. | gsd-security-auditor | 2026-05-27 |
| AR-73-04 | T-73-07 | Listener round-trip `reload()` refreshes `canonicalPath` from backend status; no manual cleanup required in End handler. | gsd-security-auditor | 2026-05-27 |
| AR-73-05 | T-73-08 | Same plain-text toast composition as AR-73-02. | gsd-security-auditor | 2026-05-27 |
| AR-73-06 | T-73-10 | No new IPC; backend Mutex serializes; tab B sees deterministic post-deletion state. | gsd-security-auditor | 2026-05-27 |
| AR-73-07 | T-73-11 | Counts derived from already-rendered DOM state; no new IPC, no privilege boundary crossed. | gsd-security-auditor | 2026-05-27 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-05-27 | 11 | 11 | 0 | gsd-security-auditor (opus) |

### Audit 2026-05-27 — Notes

- All four `mitigate` threats (T-73-01, T-73-05, T-73-06, T-73-09) verified by direct grep + read of cited file:line in `src/components/ReviewPanel.svelte` and `src/components/ReviewPanel.test.ts`.
- All seven `accept` threats verified by checking the implementation introduces no new attack surface that would invalidate the declared accept reasoning (no new IPC handlers, no retry loops, no HTML interpolation in error toasts, no manual `canonicalPath` mutation in End handler).
- D-09 listener invariant grep-verified: `if (canonicalPath && event.payload !== canonicalPath) return;` byte-for-byte preserved at `ReviewPanel.svelte:451`.
- No `## Threat Flags` section present in any SUMMARY file — executor declared no unmodelled attack surface during implementation.

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-05-27
