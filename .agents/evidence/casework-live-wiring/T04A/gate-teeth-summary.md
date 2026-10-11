# T04 UNIT A — gate and teeth evidence

Extraction of case mutation + approval resolution into
`sea-forge-case-runner::case_ops`; CLI converted to thin callers;
`approval.decide` routed through the library in-process; `run_cli` deleted
(it had exactly one caller — the approval leg).

## Gates (all run 2026-09-23, logs beside this file)

| Gate | Command | Exit |
|---|---|---|
| CLI suite | `cargo test -p sea-forge-cli` | 0 (86 passed, 18 binaries; gate-test-cli.log) |
| case-runner suite | `cargo test -p sea-forge-case-runner` | 0 (9 passed incl. new tests/case_ops.rs 5; gate-test-case-runner.log) |
| server suite | `cargo test -p sea-forge-server` | 0 (all suites green; gate-test-server.log) |
| format | `cargo fmt --all -- --check` | 0 (zero diffs; gate-fmt.log) |
| kernel sync | `just no-async-kernel` | 0 (`ok: no async runtime or HTTP client in 19 kernel crates`; gate-no-async-kernel.log) |

## Teeth attack (verbatim)

Attack: temporarily replaced the verified-actor binding in the server's
`decide()` with a FIXED actor:

```rust
// crates/sea-forge-server/src/lib.rs (line 2669 at attack time)
- let actor = actor_id.unwrap_or("operator_local").to_string();
+ let actor = "operator_a".to_string(); // TEETH-ATTACK: fixed actor, verified actor ignored
```

`operator_a` is the fixture's submitter, i.e. the most dangerous substitution:
the gate-verified actor (`operator_b`) is ignored and the resolution is
attributed to the requester.

New test under attack
(`cargo test -p sea-forge-server --test conformance_identity approval_decide_resolves_as_the_verified_actor_and_refuses_unbound_ones`,
exit 101, transcript in teeth-fixed-actor.log):

```
---- approval_decide_resolves_as_the_verified_actor_and_refuses_unbound_ones stdout ----

thread 'approval_decide_resolves_as_the_verified_actor_and_refuses_unbound_ones' (890714) panicked at crates/sea-forge-server/tests/conformance_identity.rs:464:5:
assertion `left == right` failed: {"error":"approval resolver must differ from requester"}
  left: Null
 right: true
...
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out
```

The failure is the library's own SoD check firing on the substituted actor —
the in-process wiring is real, not a bypass: the actor the server passes is the
actor the library judges.

Pre-existing test under the same attack
(`a_submitter_cannot_approve_their_own_work_but_another_actor_can`, exit 0,
teeth-fixed-actor-preexisting-test.log): still passes — its SoD refusal comes
from the connection-level gate, which the attack does not touch. This is why
the new test was required (per the task's "if no such test exists, write one"
clause).

Attack reverted; identity suite re-run green (15 passed).

## Library tests added (crates/sea-forge-case-runner/tests/case_ops.rs)

1. `reopen_reactivates_a_closed_case_and_records_the_event` — reopen happy
   path: case.json Active, CaseReopened in case-events.jsonl and the case
   ledger.
2. `reopen_refuses_a_case_that_is_not_closed` — reopen guard.
3. `propose_item_refuses_a_cycle_and_leaves_the_plan_untouched` —
   `plan_cycle_error`, plan.json byte-identical, no PlanMutated.
4. `the_proposer_cannot_resolve_its_own_items_approval` — SoD refusal
   (`sod_violation`), no approval_resolution record, no approvals.jsonl write.
5. `an_uninvolved_actor_resolves_the_approval_and_mints_the_authority_decision`
   — grant mint: Allow authority_decision committed for the resolution action,
   approvals.jsonl line with resolved_by/note, re-resolution refused.

## Server test added (crates/sea-forge-server/tests/conformance_identity.rs)

`approval_decide_resolves_as_the_verified_actor_and_refuses_unbound_ones` —
unbound actor refused at the gate with an unchanged approvals journal; bound
second actor resolves in-process and the journal names the verified actor.
