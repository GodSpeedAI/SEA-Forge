# WP-21 part A: server enforcement of the S2 episode-loop contracts

Branch `fde/integration` of SEA-Forge (worktree `~/projects/_wt/fde-sea-rs-integration`). Progress file (resume): `~/projects/_wt/tmp/wp21a-notes.md`. Mutation log: `~/projects/_wt/tmp/mutations.md`.

## Harness

`crates/sea-forge-server/tests/conformance_fde_scenarios.rs` loads every scenario of the four manifests (156: 53 + 82 + 7 + 14), drives each over an authenticated socket against a fresh cell and compares outcome, code, invariant (and the v1 fallback message prefix). `PASSES_TODAY` lists the scenarios that pass; everything else is pending with a reason (`part-B`, `wp-31`, `stepN`). A passing scenario that is still listed pending fails the suite.

## Step 2: append v2 for episode.frozen / episode.settled

Where: `crates/sea-forge-server/src/sfwp/case_records.rs` (kernel), `src/lib.rs` (`contract` field, transport oversize answer), `src/cep_authority.rs` (`pinned_world`). `AppendPolicy` carries what the cell knows (pinned world, case keeper, anchor credential identity, principal is human); anchoring configuration is `<root>/fde-config.json` (`case_keeper_identity`, `anchor_publisher_identity`).

Locks: every write of the verb runs under the case lock (`acquire_case_lock`), including the world-pin read, so the freeze check and the write see one pin.

Contract ambiguities met:
- `anchor-publisher-invalid` has a freeze anchor but no model anchor and no run event, yet expects `ANCHOR_PUBLISHER_INVALID` (precedence lists `MODEL_ANCHOR_LATE` first). Read as: a model anchor can only be late once a run event exists.
- `model-anchor-late` lists a run record without `episode.frozen` (a settlement then expects past `FREEZE_MISSING`); the harness seeds the freeze the run implies.
- EPISODE_REF_DIGEST_MISMATCH precedes idempotency in the contract's precedence list; the S1 test `replay_returns_the_original_response...` requires `IDEMPOTENCY_CONFLICT` for a changed body under a used key. S1 order kept (digest check stays inside the freeze path, after idempotency).

## Steps 3, 4, 6 (commit 90f93acb)

Gate: `crates/sea-forge-server/src/sfwp/episode_gate.rs`; run records and links: `src/sfwp/run_records.rs`; wiring in `src/lib.rs` (`admit_episode`, `transition_only`). The case template comes from `cases/<case_id>/plan.json` and the freeze, lifecycle, settlement and latest `n` from the case store (`case_records::episode_facts`), never from a caller tag. `delegate` has a required `case_id` (Go: `NewDelegate`, `NewCaseAdvanceEpisode`, `NewItemExecuteEpisode`; 7 golden frames captured from the real server by `golden_capture_episode_gate_test.go`; no TypeScript contract is generated for these verbs).

Lock: the gate check, the assignment record, the link (written first) and the run record are one critical section under the case's keyed lock (`acquire_case_lock`, the same lock as every `case.record.append`). The run starts after the lock is released: a record written in between (a suspension) is not seen by that run (stated limit).

Reading of the contract used: R13 refuses a closed case except the action that ends the plan (the scenarios give both "closed" and "latest close disposition, terminal reached accepted"); a delegate's agent principal is `agent:<endpoint>`, a delegation to an endpoint listed in `fde-config.json` `verifier_endpoints` is a `verifier` run (the request has no role field); workspace of a delegate is `ws:agent:<endpoint>`.

Contract problems found:
1. `delegation-link` scenario `sibling-self-asserted-fresh-is-refused` expects `VERIFIER_CONTEXT_MISMATCH` with invariant I-2; `case-record-append.v2` (the verb's schema) forbids an invariant on that code and its own two sibling scenarios have none. Left pending, reason `contract`.
2. `anchor-publisher-invalid` (append v2) has no model anchor and no run yet precedence lists `MODEL_ANCHOR_LATE` first; `model-anchor-late` lists a run with no freeze. Read as stated above.
3. R13 versus R11 in the scenarios (see above).
4. EPISODE_REF_DIGEST_MISMATCH precedes idempotency in the precedence list; S1's test requires `IDEMPOTENCY_CONFLICT`; S1 order kept.

## Step 5: program-execution decision (owner decision D39, option a): DONE

Where: `sfwp/episode_gate.rs` (`execution_object`, `find_decision`, R7 before R10), `sfwp/run_records.rs` (`decision_id`), `sfwp/interaction_events.rs` (`run.started` event), `case_records::enqueue_run_event`.
- The gate builds the nine-member `BoundExecution` itself (frozen episode_ref and world_ref, the plan item's command or, for a delegation, `[endpoint]` with profile `agent`; cwd; sha256 of the empty patch; env digest of `{}`; workspace; sandbox profile; time bound), digests it (RFC 8785) and looks for an `authority_decision` entry in the `cognate-authority` ledger (the existing CEP path) whose `operation.resource_id` is `execution:sha256:<digest>` with outcome `allow`, decided within `execution_decision_ttl_secs` (fde-config.json, default 86400).
- `runs_used` is the count of lines per decision entry in `cases/<id>/field-exec-decisions.jsonl`; `max_runs` is `execution_max_runs` (default 1). Both are read and written under the case lock, in the same critical section as the freeze check, the assignment, the link and the run record.
- Missing, denied, expired, another execution (any bound member differs) or exhausted: `AUTHORITY_DENIED`, no side effect. No new code, no CEP profile change.
- The run record carries `decision_id`; the interaction event `run.started` carries it as `authority.decision_id` (test `run_event_carries_the_program_execution_decision_id`).
- Assumptions: the event maps to the Episode clause 7.2d like the freeze (no translation-map row for an execution exists); a delegation has no command so its argv is the endpoint and its time bound `max_turns`.

## Mutation checks (steps 2-4)

Log: `~/projects/_wt/tmp/mutations.md`. Each guard removed, the named scenario failed, guard restored. Step 2: WORLD_NOT_PINNED, KIND_IMMUTABLE, SETTLEMENT_EXISTS, REFERENCE_DERIVED, MODEL_ANCHOR_LATE, ANCHOR_PUBLISHER (freeze and settlement), CASE_CLOSED, CASE_SUSPENDED, human AUTHORITY_DENIED, STORE_INTEGRITY, PAYLOAD_TOO_LARGE (transport): all KILLED. Step 3: R13, R6, R11 individually; R14, R1, R2, R3, R4, R5, R12 (both), R10 (both), R9 in one combined build: all KILLED. Step 4: delegated run without link, executor set (on_behalf_of and delegator closure), link delegate in the set: KILLED. `declared != derived` verifier-context check: SURVIVED, masked by the sibling rule (same code) in the only fixture that reaches it; recorded as a gap, no fixture separates the two.

## Gates at 90f93acb

fmt clean; clippy -D warnings clean for `-p sea-forge-server --all-targets` (the workspace-wide clippy was not run: host memory); sea-forge-server lib 190, conformance_field_case 31, interaction_events 10, world_pin 4, sfwp 23, identity 15, cep_authority 13, case_episode 11, m13 and the whole `--lib --tests` run of the crate 0 failed (run before the run-record change; re-run of the named suites after it: green); conformance_fde_scenarios 9; sea-forge-authority lib 60, world_pin 21, cep_transition 17; fde_contracts 34; Go sfwp tests ok; `scripts/fde-proof.sh --slice s1`: AT-04 PARTIAL, NT-01 PASS, NT-05 PASS, NT-07 PASS, NT-08 PASS, NT-16 PARTIAL (unchanged); `--slice s2`: NT-13, 17, 18, 32, 33, 48 PASS, NT-34 NOT_RUN.

## Update after the owner decisions D39-D44

Done: D39 (above); D41, D42, D44 are implemented as already described; D43 is the implemented order. Mutation checks of the step-5 guards (combined build, all KILLED): deny outcome ignored, execution not bound, exhausted ignored, absent decision accepted, decision id not carried on the event.
NOT done, blocked by the permission system (the contract fixtures and README are contracts): D40 (remove the invariant from delegation-link/sibling-self-asserted-fresh-is-refused), D43 (precedence text), D44 and D39 rows in the README decision table, the new fixture separating `declared != derived` from the sibling rule. The scenario stays pending with reason `contract`; `FINAL_REASONS` is therefore {part-B, wp-31, contract}, not {part-B, wp-31}.
Full run after step 5: `cargo test -p sea-forge-server --lib --tests`: 49 test binaries, 640 passed, 0 failed.
