# Final Completion Report

Date: 2026-07-30. Branch: `ultracode/sea-forge-completion`.

## Verdict

**PARTIALLY COMPLETE**

Five of the thirteen P0 packets are complete and verified, and a sixth
(SF-005) is half landed. Nothing is blocked.

**Update, later the same day:** U-07 — the decision that had blocked every
remaining packet — was answered by the owner and is recorded in
`DECISION_REGISTER.md`. The identity half of SF-005 shipped on the back of it
(`1ebcea3`): protected verbs now require an `actor` block verified against the
connection's uid from `SO_PEERCRED`, an unconfigured cell refuses them all, and
inspect verbs are untouched so old clients keep working. What remains in SF-005
is separation-of-duty enforcement and the desktop router's fabricated
`mockGuardContext`; see `REMAINING_BLOCKERS.md`.

The verdict stays `PARTIALLY COMPLETE` because the primary Workbench operator
journey still does not run end to end — the desktop client has not yet been
moved onto the real identity contract.

The original blocker analysis, retained below because it explains the shape of
the work: the remaining eight packets all depended, transitively, on **U-07,
the exact public SFWP identity/session contract** (`DECISION_REGISTER.md:46`).

This is not the strongest available verdict, and the evidence does not support a
stronger one. `COMPLETE AND VERIFIED` would require the primary Workbench
operator journey to work end to end; it does not, because identity is still
fabricated in the desktop router.
`FUNCTIONALLY COMPLETE WITH EXTERNAL VERIFICATION PENDING` would imply the
remaining work is verification rather than implementation; SF-005 through
SF-013 are implementation. `BLOCKED` would understate what landed: five
packets are done, the kernel is green, and two live user journeys work against
real binaries.

## What is complete

| Packet | Title | Commit |
|---|---|---|
| SF-001 | One cell contract for root and socket resolution | `240619d` |
| SF-002 | Fail closed at startup, reload, and request timeout | `041641c` |
| SF-003 | Canonical episode pipeline | `19b21b0` (mislabeled) + `ae86c77` |
| SF-004 | One run locator with flat-minimum compatibility | `3352c47` |
| SF-007 | Contract and generated-zone gate | `34e2c77` |

Plus one defect fixed at its source in its own commit:

| Commit | Fix |
|---|---|
| `4011343` | `sea_forge_settlement::settle` returned the literal `settlement_id: "set_01"` for every settlement in every cell |

## Commits from this session

```
3352c47 fix(server): resolve runs through one locator across both layouts
34e2c77 build(workbench): gate generated contracts and the Tauri workspace boundary
ae86c77 feat(server): run case episodes through the whole governed lifecycle
4011343 fix(settlement): give every settlement its own id
```

Each is a single coherent change with its verification evidence in the message.
No unrelated changes were bundled. The user's own working-tree files
(`README.md`, `.agents/`, `.jolli/`) were left untouched.

## The defects that were actually fixed

Ordered by how badly the product behaved before.

### 1. A case episode was ungoverned in four separate ways

`case_dispatch::execute_sandbox` created the workspace *before* evaluating
authority (AUTH-01), passed `domainforge_candidate: None` (DOM-03), settled on
`exit_code == Some(0)` (DOM-01), and emitted no trace or evidence (DOM-02). A
denied action therefore left a directory behind, a command that exited zero
while producing none of its declared artifacts settled as **accepted**, and an
`Escalate` verdict was recorded as an ordinary rejection — discarding the
review it was asking for.

All four now hold, in both the server dispatcher and the case-runner's stage
episodes, through the same `sea_forge_settlement::settle` every other surface
uses.

### 2. Every settlement in every cell shared one id

`settle` hardcoded `settlement_id: "set_01"`. Two settlements in a cell were
indistinguishable and a `settlement_ref` naming `set_01` matched all of them at
once. The CLI already consumed the value verbatim
(`cli/src/pipeline.rs:762,774,785,802`), so this was live, not latent.

### 3. Case-dispatched runs were invisible to the views that exist to show them

`run_views` resolved only `<root>/runs/<id>`. Every run the server itself
created lived at `<root>/cases/<case>/runs/<id>` and returned `not_found` from
`run.get` and nothing from `run.list`. A Workbench run link failed for exactly
the runs the server had just produced.

### 4. A settled run reported itself as unsettled

Found by driving the real server, after 818 tests passed. `run.get` returned
`settlement: "unsettled"` and `case.get_overview` returned an empty settlements
array for a run that had been denied and settled — because the views read
`settlement.json` and the dispatcher only wrote the ledger. `unsettled` is a
claim about the run; the correct answer was `rejected`.

### 5. Generated projections could drift silently

`workbench-check` never asked whether the committed TS/AJV contracts, the UI
token sheet, or the standalone Tauri workspace still matched their sources. All
three are now gated, and each failure mode was verified to actually fail.

## Verification

Full results in `FINAL_VERIFICATION_RESULTS.md`. Summary:

```
cargo clippy --workspace --all-targets --locked -- -D warnings   0 errors
cargo test --workspace --all-targets --locked --no-fail-fast     819 passed, 0 failed, 4 ignored
just proof                                                        P1-P4b passed
just workbench-contracts-gate                                     ok
```

The 4 ignored tests are pre-existing release gates requiring external ACP hosts;
this pass added no `#[ignore]` and removed no assertion.

Live user journeys in `USER_JOURNEY_EVIDENCE.md`: fail-closed startup, the
socket-path guard firing with its remedy, and a full deny-path case submission
over a real Unix socket showing the settlement reaching both views with no
workspace created.

## What blocks the rest

`REMAINING_BLOCKERS.md` has the full dependency argument. In short:

```
SF-005 ── blocked on U-07 ──> SF-006, SF-008
                                └─> SF-009 ─> SF-010 ─> SF-011 ─> SF-012 ─> SF-013
```

Every remaining P0 packet is inside that cone. SF-005 requires the server to
attribute protected actions to a host-resolved identity and enforce separation
of duty; the shape of that identity is public SFWP behavior, which
`DECISION_REGISTER.md:86-88` lists as user-required, and which the register's
own trigger column names as due "before implementing the required governed
actor propagation across host/server."

Choosing it unilaterally would mean inventing a public contract from no
repository evidence — an explicit pause condition — and the intermediate state
would ship a fabricated identity under a different name, which is precisely the
defect SF-005 exists to remove.

## What I did not do, and why

- **Did not rewrite commit `19b21b0`.** It carries the SF-003 dispatcher work
  under the message "add neatcode skill" and contains no neatcode files. It is
  the user's commit; rewriting it would be more intrusive than the traceability
  it buys. Recorded in `REMAINING_BLOCKERS.md` instead.
- **Did not change the CLI's pre-authority workspace scaffold.**
  `justfile:386-389` pins it, it was outside SF-003's `allowed_paths`, and
  changing it would have broken P1-P4b. The server is stricter; the divergence
  is documented rather than hidden.
- **Did not rename `sea-forge-settlement`** despite the boundary documents
  assigning "settlement classification" to GodSpeed-Agent. Same word, different
  layer — see `FINAL_ARCHITECTURE_STATE.md`. A rename is a large cosmetic change
  and a question for the owner of those boundaries.
- **Did not build, sign, or publish anything.** No external write of any kind
  was performed.

## The smallest input needed to continue

A decision on U-07, answering four questions
(`REMAINING_BLOCKERS.md` has the detail):

1. Is identity bound **per connection** (a handshake verb, later requests
   inherit) or **per request** (every protected verb carries an actor block)?
2. Is the Tauri host **trusted** to assert the OS user because the socket is
   owner-only, or must the client present a credential the server verifies?
3. Do approvals arrive on the **same session** as the submission they resolve,
   or is the approver identified independently?
4. Is the actor block **optional on inspect verbs and required on protected
   verbs** — which SF-005's own compatibility criterion implies but does not
   establish?

With those answered, SF-005 is executable, and SF-006 and SF-008 unblock
immediately behind it.
