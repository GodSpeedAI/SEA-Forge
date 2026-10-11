# T04 confirmation — units A+B+C (2026-09-24 session 2)

## Verdict: CONFIRMED (orchestrator verification; independent critic ran, procedural REJECT reconciled)

T04 units A (1b73724), B (9c153b2), C (this commit) verified against plan T04
(proof P2). The independent critic (t04-critic, run_00001) executed its pass
and delivered a full report via team mailbox (msg_00002 + msg_00003,
2026-09-24): verdict REJECT on **procedural** grounds only — (1) unit C was
uncommitted at review time (this commit remedies that); (2) no converged
`T04/confirmation.md` existed (this file remedies that); (3) the critic's own
live `cargo test` re-runs timed out at 30s in its environment, so it claimed
no live passes (remedied below: orchestrator fresh 2026-09-24 re-runs are all
green with logs). The critic found **zero code defects** and returned an
explicit APPROVE-gate checklist, every item now satisfied. Its substantive
review agrees with the orchestrator's on all points (identity classification
unchanged, ADR-003 additivity intact, cannot-miss choke point shared, lock
order acyclic, fail-closed opt-in, sandbox-class selection, honest
Service-role attribution). The critic's procedural REJECT stands as received;
its grounds are closed here with evidence.

## Gates re-run fresh on the tree WITH unit C (2026-09-24)

- `cargo test -p sea-forge-cli -p sea-forge-case-runner -p sea-forge-server`
  → `T04C/gates/rerun-3crate-2026-09-24.log`: 57 `test result: ok` suites,
  0 FAILED, 494 individual `ok` lines. (No `EXIT=` trailer from the
  background redirect; completeness via suite count + zero failures +
  process exit. The known `conformance_m13 t13_3` flake did NOT reproduce.)
- `cargo test -p sea-forge-server --test sfwp_case_mutations`
  → `T04C/gates/rerun-sfwp-case-mutations-2026-09-24.log`: 10/10 PASS
  (cycle refusal, SoD refusal, subscriber kill/resume all green).
- `cargo test -p sea-forge-server --test sfwp_supervisor`
  → `T04C/teeth/rerun-sfwp-supervisor-2026-09-24.log`: 4/4 PASS.
- `cargo test -p sea-forge-server --lib config`
  → `T04C/teeth/rerun-config-units-2026-09-24.log`: 15 passed.
- `cargo fmt --all -- --check` → EXIT 0 (critic also ran live: EXIT 0).
- `just no-async-kernel` → EXIT 0
  (`ok: no async runtime or HTTP client in 19 kernel crates`).
- `workbench/`: clean → contracts gate N/A (no schema change; only the
  internal non-`JsonSchema` `AdvanceCaller` enum added).
## Teeth (plan T04 + step 5)

- Cycle `case.add_item` → refused, `plan.json` byte-identical, no
  `PlanMutated` (green in re-run).
- Proposer self-approval → SoD refusal, `approvals.jsonl` unchanged (green).
- Subscriber kill + `from_cursor` resume → no gaps/duplicates, monotonic
  cursors (green).
- Supervisor (b) default: ready SandboxedTask untouched across 6.5s
  (> 5s default poll), journal byte-identical, zero run dirs.
- Supervisor (c) enabled: exact order `case_created → item_enabled →
  item_activated → settlement_recorded → item_completed →
  milestone_achieved → case_closed`; run trace attributed to
  `cell_supervisor`, never `operator_local`; byte-identical once terminal.
- Supervisor (d) human-only: journal byte-identical across 3 polls,
  `approvals.jsonl` never created, no `human_task_completed`, stays active.
- Supervisor (e) verb↔supervisor race: coherent outcomes, journal parses
  line-clean, exactly one `item_activated` + one `settlement_recorded`.
- TDD honored: `T04C/teeth/sfwp_supervisor-tdd-baseline-expected-failure.log`
  (1 expected pre-impl failure) →
  `T04C/teeth/sfwp_supervisor-post-impl-pass.log` (4/4 post-impl).

## Diff review (orchestrator + critic agree)

- `identity.rs`: unchanged by unit C; all T04 verbs protected
  (`is_protected:499-548`). Supervisor is server-internal with
  `ActorRole::Service` (real variant, `types.rs:299`,
  `actor_type_for_role` maps `Service→Service`), never a client claim.
- `Request` enum: no new variants in unit C; ADR-003 additivity intact.
- Cannot-miss: verb + supervisor share `run_case_mutation`
  (`case_mutations.rs:409` lock → `:414` closure → `:444` prune).
- Lock order: slot (`supervisor.rs:126`) → run-pool (`pass:159`) → case
  lock (`advance:409`); verb: run-pool (`advance_response:2663`) → case.
  Acyclic; map pruned on release (`prune_case_lock:370`).
- Fail-closed: default `enabled=false`; absent section → default (unit
  test); `run()` spawns only when enabled (`lib.rs:1221-1224`); missing
  active policy skips the wave with one `warn` (`supervisor.rs:97`).
- Sandbox selection: double filter (`advance.rs:140-148`, `:181-187`) over
  the loop's kind check (`:276`); human-only cases read `idle`, zero writes.
- Blocking: enumeration via `spawn_blocking` (`supervisor.rs:110`); kernel
  stays sync (`no-async-kernel` EXIT 0).
- Nit (non-defect): module docs "the pass also takes the case's keyed lock"
  — the lock is taken inside shared `advance`, not `pass`. Wording only.

## Settlement

Unit C committed here; T04 gates + teeth green; independent critic ran and
its report is reconciled above (procedural REJECT grounds closed, zero code
findings). T04 settles: `settled_tasks` gains T04, `ready_tasks` → `[T02]`,
`in_flight_task` → `T02`. Next: T02 (prereg written; ADR-first).
