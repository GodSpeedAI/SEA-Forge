# Remaining Blockers

Status date: 2026-07-30. Branch: `ultracode/sea-forge-completion`.

## The single blocker: decision U-07

**Every remaining P0 packet depends, transitively, on one decision the
repository itself designates as user-required.**

`docs/execution/DECISION_REGISTER.md:46`

> | U-07 | Exact public SFWP identity/session contract. | Before implementing
> the required governed actor propagation across host/server. |

and `DECISION_REGISTER.md:86-88`

> **User-required:** U-01 through U-07 when triggered, plus repository ask-first
> changes to persisted schemas/layouts, **public SFWP behavior**, dependencies,
> CI, or deployment configuration.

SF-005 ("Protected SFWP commands and approvals carry host-resolved actor
context") is precisely the trigger named in U-07's condition column.

### Why it blocks everything downstream

```
SF-005  ── U-07 ──┐
                  ├─> SF-006 (idempotency)
                  ├─> SF-008 (readiness/identity/affordance slice)
                  └─> SF-009 (case-to-settled-run slice)
                        └─> SF-010 (approval/intervention/recovery)
                              └─> SF-011 (evidence/settlement/reuse)
                                    └─> SF-012 (packaged Linux stack)
                                          └─> SF-013 (release gate)
```

Declared dependencies, from each packet's own front matter:

| Packet | `dependencies` | Reaches SF-005 |
|---|---|---|
| SF-006 | `[SF-005]` | directly |
| SF-008 | `[SF-002, SF-005, SF-007]` | directly |
| SF-009 | `[SF-003, SF-004, SF-005, SF-006, SF-008]` | directly |
| SF-010 | `[SF-006, SF-009]` | via SF-006 |
| SF-011 | `[SF-009, SF-010]` | via SF-009 |
| SF-012 | `[SF-001, SF-002, SF-007, SF-011]` | via SF-011 |
| SF-013 | `[SF-012]` | via SF-012 |

There is no remaining P0 packet outside this cone. SF-001, SF-002, SF-003,
SF-004, and SF-007 — the five that are not behind U-07 — are complete.

### What the decision has to settle

SF-005 requires the server to attribute every protected action to a resolved
operator identity and to enforce separation of duty server-side. Today:

- `crates/sea-forge-server/src/case_dispatch.rs` builds
  `Actor { actor_id: entity, role: ActorRole::Operator }` from the request's
  `entity` string, with the role hardcoded.
- `workbench/apps/desktop/src/router.tsx:38-51` supplies a fabricated
  `mockGuardContext` (`actor_op_01`, `sha256:policy_v1`, `verified`).

Both have to be replaced by something the *protocol* carries. That protocol
shape is public SFWP behavior, and it is what U-07 names. Choosing it
unilaterally would mean inventing a public contract from no repository
evidence — one of the mission's explicit pause conditions, and the exact case
the decision register was written to catch.

The concrete open questions:

1. **Where identity is bound.** Per connection (a handshake verb establishes a
   session; later requests inherit it) or per request (every protected verb
   carries an actor block, and the server is stateless about identity)?
2. **What the host is trusted to assert.** Does the Tauri host resolve the OS
   user and the server trust it because the socket is owner-only (0600), or
   must the client present a credential the server verifies independently?
3. **How the approver is distinguished from the submitter.** SoD needs two
   identities that the server can tell apart across a restart — which requires
   knowing whether approvals arrive on the same session as the submission.
4. **How old clients behave.** SF-005's own acceptance criterion says "Old SFWP
   clients without actor context still work for inspect verbs," which implies
   the actor block is optional on inspect and required on protected verbs. That
   split needs confirming as the contract, not assuming.

### What is *not* blocking

- **U-06** (Tauri-supervised sidecar versus separately installed local service)
  gates SF-012's distribution/service lifecycle. It is a second decision that
  will be needed, but it is not on the critical path yet — SF-012 is already
  blocked behind SF-011.
- Nothing is blocked on credentials, network access, or an irreversible
  external action. No packaging, publication, or external write has been
  attempted or is pending.

## Work that can continue without the decision

Nothing of substance in the P0 set. Specifically ruled out as busywork rather
than progress:

- Building SF-005 against a guessed contract and reworking it later. The
  packet's blast radius is the SFWP schema, the generated TS/AJV projections,
  the desktop router, and every protected verb's handler — a guessed shape
  would be rewritten wholesale, and the intermediate state would ship a
  fabricated identity under a different name.
- Starting SF-009's vertical slice. Its first acceptance step is a submission
  attributed to a resolved actor.

## Deferred items recorded during completed work

These are recorded so they are not lost; none blocks anything.

| Item | Where | Why deferred |
|---|---|---|
| Dispatcher-level timeout test | SF-003 criterion 5 | `settle` maps `ExecutionStatus::TimedOut` to `Rejected` with basis `timed_out` (`settlement/src/lib.rs:37-40`), so the mapping is covered; no test drives a real 600s stall through `case_dispatch`. |
| DomainForge drift test at the dispatcher layer | SF-003 criterion 6 | The candidate is now built and passed; no server fixture policy declares a `domainforge` engine, so the drift branch is exercised only by the authority crate's tests. |
| Run-level `authority.json` / `settlement.json` materialization in the server | SF-003 | CLI run-view artifacts; the server's case views read the ledger. Adding them duplicates a projection. |
| Pre-action assurance / integrity-ledger witnessing in the server | SF-003 | Gated on `integrity_ledger.required_for_side_effects`, which no server fixture sets. |
| Shared synchronous episode function | SF-003 | `execute_sandbox` and `run_stage_episode` now agree on every governance property; unifying them is a refactor with no behavioural requirement behind it. |
| Other flat-only run readers | SF-004 | `delegation.rs`, `agent_probe.rs`, `server/lib.rs`, `transcript_seal.rs`, `swe_seed_reconciliation.rs` read runs they themselves wrote flat. Outside SF-004's `allowed_paths` and not a reachable defect. |
| CI wiring for `workbench-contracts-gate` | SF-007 | Explicitly SF-013's scope; CI changes are ask-first per the decision register. |

## Repository hygiene noted, not acted on

- Commit `19b21b0`, titled "add neatcode skill", contains no neatcode files. It
  contains the SF-003 dispatcher work plus two tooling artifacts
  (`.jolli/jollimemory/debug.log`,
  `workbench/apps/desktop/src-tauri/.omc/state/subagent-tracking.json`).
  History was left alone rather than rewritten; this note is the audit trail.
- `workbench/apps/desktop/src-tauri/Cargo.toml` uses CRLF line endings while
  the rest of the workspace uses LF. Pre-existing; it only matters because
  line-anchored edits to that file need `\r`.
