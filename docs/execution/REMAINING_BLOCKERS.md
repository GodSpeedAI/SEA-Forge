# Remaining Blockers

Status date: 2026-07-31. Branch: `ultracode/sea-forge-completion`.

## Status update — U-06 is resolved; the product is packaged and gated

**U-06 was decided on 2026-07-31** under the standing instruction not to
escalate ordinary engineering choices: **supervised sidecar, with adoption**.
Recorded in `DECISION_REGISTER.md`. It was the last user-decision blocker.

SF-012 and the CI half of SF-013 have landed:

| Was missing | Now |
|---|---|
| No package existed | `just workbench-package` builds `.deb` and `.rpm` |
| The Workbench needed a hand-started server | it starts its own, or adopts one already running |
| `"csp": null` — no content-security policy at all | a real policy, verified with the renderer running under it |
| `bundle.targets` advertised untested macOS | Linux only; macOS is not built and not claimed |
| CI never compiled `src-tauri` or the renderer | `workbench` job runs `just workbench-check` |
| CI never built or inspected a package | `package` job builds and inventories the bundle |
| A fresh cell had nothing to show | `just cell-seed` / `just cell-reset` |

**No blocker remains.** SF-006 and SF-008 → SF-011 are ordinary implementation
work; the release half of SF-013 (checksummed artifacts, release notes) is
ordinary release work.

### What packaging found that 843 tests did not

Both fixed, both pinned, both described in `USER_JOURNEY_EVIDENCE.md`:

- **A pending approval was unreachable.** The approvals journal folded on
  `approval_id` alone, but ids are per-case ordinals, so resolving one case's
  `apr_0001` superseded another case's still-pending `apr_0001`. It stayed
  committed in the ledger and vanished from every inbox, stranding the work with
  no lawful path to the identifiers `approval.decide` requires. Needed two cases
  in one cell to reproduce; every test in that module used one.
- **Signalling the window orphaned its kernel.** `RunEvent::Exit` and `Drop`
  both miss a signalled process, so `kill <app-pid>` left a reparented server
  still serving the cell the operator had closed.

### Still true, and deliberate

- **macOS is not built and not advertised.** Its Seatbelt journey has not run.
- **`AppImage` is not built and no longer advertised.** Its tooling `dlopen`s
  `libfuse.so.2`; this host has FUSE 3 only. It had been listed as a target and
  had never once succeeded.
- **`SIGKILL` still orphans the kernel.** Uncatchable by anyone; adoption makes
  the next launch attach to the survivor rather than start a rival, so it is
  recoverable rather than corrupting.
- **No accessibility audit has been run.** SF-012 asks for keyboard, 200% zoom,
  reduced motion, and axe-core evidence. None of that has been produced, so
  none of it is claimed.
- **README.md is mid-rewrite by the owner** and was deliberately left alone, so
  SF-013's documentation refresh is not done.

## Status update — U-07 is resolved; nothing is blocked

**The owner answered U-07 on 2026-07-30.** The four answers are recorded in
`DECISION_REGISTER.md` under "U-07 — Resolved: the public SFWP identity
contract", and the identity half of SF-005 has landed (`1ebcea3`):

- Protected verbs require an `actor` block; inspect verbs do not.
- The server verifies the claim against the connection's uid from
  `SO_PEERCRED`.
- An unconfigured cell refuses every protected verb — no fallback.
- `identity::is_protected` is one exhaustive match, so a new verb cannot be
  added without being classified.

**No blocker remains.** The rest of SF-005 and everything downstream is
ordinary implementation work, listed under "Remaining work" below.

### SF-005 is complete

| Acceptance criterion | State |
|---|---|
| Actor required and verified on protected verbs | done (`1ebcea3`) |
| Two-actor test: operator submits, approver resolves; both recorded | done (`7a95800`, live in `drive_identity.py`) |
| SoD: same actor submits and approves → denied with no effect | done (`7a95800`) |
| Missing identity on a protected verb → typed denial, no side effect | done (`1ebcea3`) |
| UI displays resolved actor, role, cell from server state | done (`0d15202`) |
| Old SFWP clients without actor context still work for inspect verbs | done (`1ebcea3`) |
| `identity.get` in the SFWP schema and generated TS | done (`7a95800`) |

Two things were found only by driving a real server, and both are fixed
(`52565b5`):

- Every approval this dispatcher opened was **unresolvable** — no criteria
  binding and no active-policy snapshot. The kernel suite was green throughout,
  because the existing test asserted an escalation *opens* an approval and
  never that anyone can resolve one.
- An approval resolved by `operator_b` was recorded `resolved_by=operator_local`,
  because the server shells out to the CLI and never passed the verified actor.

Then SF-006, SF-008 → SF-013 in dependency order, unchanged.

### A verification gap that hid a live regression

`workbench/apps/desktop/src-tauri` is a separate Cargo workspace (ADR-004,
K-06), so `cargo test --workspace` never compiled it and **no gate ran the
desktop host's Rust tests at all**. The SF-005 identity gate broke the host's
correlation-recovery tests the day it landed and the whole kernel suite stayed
green. Closed by `just workbench-tauri-test`, now a dependency of
`workbench-check` (`52565b5`). Wiring it into CI belongs to SF-013.

---

## Historical: why this was a blocker

Retained because it explains the shape of the work and the decision record.

**Every remaining P0 packet depended, transitively, on one decision the
repository itself designated as user-required.**

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
| CI wiring for `workbench-tauri-test` | SF-005 | Same: the recipe exists and passes locally; adding it to `.github/workflows` is SF-013's scope. |
| Guards G3, G4, G6, G7, G8, G10 report `indeterminate` | SF-005 | No SFWP verb backs sponsorship, policy digest, resource lookup, disclosure scope, the compatibility matrix, or pre-decision standing. They are now honestly undetermined rather than fabricated; grounding them needs the kernel verbs SF-008/SF-010/SF-011 introduce. |
| `act_as` actor picker in the renderer | SF-005 | The host accepts and validates `act_as`, so a cell binding several actors is served correctly by the transport. No UI control selects among them yet, so such a cell currently refuses protected work with "choose one explicitly" rather than offering a choice. Single-actor cells — the common case — are unaffected. |
| An approval's criteria snapshot is per-escalation | SF-005 | `commit_item_criteria` reuses an already-committed record when the plan carries a resolving `settlement_criteria_ref`, and otherwise mints one from the item's inline criteria. A wire-submitted plan has no ref, so re-escalating the same item twice records two criteria ids with identical content. |

## Repository hygiene noted, not acted on

- Commit `19b21b0`, titled "add neatcode skill", contains no neatcode files. It
  contains the SF-003 dispatcher work plus two tooling artifacts
  (`.jolli/jollimemory/debug.log`,
  `workbench/apps/desktop/src-tauri/.omc/state/subagent-tracking.json`).
  History was left alone rather than rewritten; this note is the audit trail.
- `workbench/apps/desktop/src-tauri/Cargo.toml` uses CRLF line endings while
  the rest of the workspace uses LF. Pre-existing; it only matters because
  line-anchored edits to that file need `\r`.
