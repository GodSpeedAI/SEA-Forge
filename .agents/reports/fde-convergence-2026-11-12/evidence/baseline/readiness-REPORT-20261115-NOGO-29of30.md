# GodSpeed readiness report

Generated 2026-11-15T04:36:36.340Z from 406 run(s). Only evidence at the currently pinned revisions counts.

## Local single operator (Linux / WSL2, loopback only, internal use)

**NO-GO** — 29/30 gates passed at pinned clean revisions. License: permitted (SEA-Forge Sustainable Use License, internal business use)

| Gate | Invariant | Outcome | Evidence |
|---|---|---|---|
| L01 | 7 authority is external; 9 external consequences need independent evidence | PASS | 2026-11-15T04-18-24-754Z |
| S01 | loopback surfaces cannot be driven by a foreign web origin | PASS | 2026-11-15T04-25-36-577Z |
| S02 | 9 an executing agent cannot file its own observations (contained agent cannot reach RealityTrace or SEA-Forge) | PASS | 2026-11-15T04-25-39-952Z |
| S03 | proof verifies artifact content, not existence | PASS | 2026-11-15T04-25-42-835Z |
| R01 | a crash mid-effect never silently repeats an external effect | PASS | 2026-11-15T04-24-08-658Z |
| C01 | one DomainForge producer version across the stack (world identity agreement) | PASS | 2026-11-15T04-15-01-760Z |
| R02 | missing/late observation is never settlement | PASS | 2026-11-15T04-25-27-068Z |
| O01 | clean install from pinned revisions on a fresh home, no author environment (GitHub-hosted runner evidence) | PASS | 2026-11-15T04-20-48-941Z |
| O02 | RealityTrace settlement ledger survives backup, loss and restore | PASS | 2026-11-15T04-20-52-508Z |
| O03 | Workbench state (Postgres schema + home) survives backup, loss and restore | PASS | 2026-11-15T04-20-57-814Z |
| O04 | SEA-Forge cell survives its shipped backup/restore scripts | PASS | 2026-11-15T04-21-06-549Z |
| J02 | Horizon navigation commits are decided by SEA-Forge, committed once after approval, observed by RealityTrace and settled by SEA-Forge (IS3-IS6; IS2 and IS7 open in the frozen source) | PASS | 2026-11-15T04-15-37-293Z |
| SW01 | 9 governed software work is settled by SEA-Forge on independent RealityTrace evidence: Workbench -> SEA-Forge authority -> RealityTrace (contained proof) -> SEA-Forge settlement | FAIL | 2026-11-15T04-25-43-815Z — exit 1 (expected 0); fail marker: \b[1-9]\d* fail\b |
| J02R2 | 4/9 a bound Horizon operation is paid within capacity, preregistered with RealityTrace, verified by a pinned confined verifier and settled by SEA-Forge from RealityTrace's ledger (HORIZON-001 1.1.0 IS2, IS7) | PASS | 2026-11-15T04-17-04-534Z |
| U01 | the governed Workbench UI stays usable: Continue advances, an idle page does not grow SEA-Forge's ledger | PASS | 2026-11-15T04-26-10-692Z |
| SW02 | the declared proof comes from a SWE_SEED work-and-proof contract (frozen by digest), not a hand-typed command | PASS | 2026-11-15T04-25-51-772Z |
| G01 | a Gauntlet run killed at a durable boundary (real binary, SIGKILL) resumes to one run id and dispatches the irreversible unit exactly once; a poison child terminates bounded | PASS | 2026-11-15T04-15-14-758Z |
| CK01 | SWE_SEED gets a cited context packet from a live Context Kernel binary, and reports Context Kernel absence explicitly, never by fabrication | PASS | 2026-11-15T04-15-05-694Z |
| CK02 | a tampered, falsely complete, or partial context bundle is refused at the SWE_SEED boundary (contract, fixtures; no live Context Kernel) | PASS | 2026-11-15T04-15-13-713Z |
| U02a | upgrade keeps durable data: Horizon 1.1.0 suspends v1 runs; the rollback guard refuses a 1.0.1 start against v2 state (contract; the script's exit codes are U02e) | PASS | 2026-11-15T04-26-51-997Z |
| U02e | upgrade keeps durable data: Horizon's rollback guard script exits 0 for a v1 store, 2 for any v2 state, 1 for a missing store (real process) | PASS | 2026-11-15T04-27-50-502Z |
| U02b | upgrade keeps durable data: Cognate SQLite-to-PostgreSQL migration preserves every event, verifies, and refuses a second import (live PostgreSQL) | PASS | 2026-11-15T04-26-52-670Z |
| U02c | upgrade keeps durable data: RealityTrace's ledger refuses a store with a newer user_version (unit; not an end-to-end rollback) | PASS | 2026-11-15T04-26-55-529Z |
| U02d | upgrade keeps durable data: SEA-Forge admits a DomainForge compiler upgrade only when no meaning-bearing field changes (contract) | PASS | 2026-11-15T04-26-56-353Z |
| U02f | upgrade keeps durable data: Cognate refuses to open a SQLite or PostgreSQL store written by a newer schema version, and writes nothing (live PostgreSQL) | PASS | 2026-11-15T04-27-57-294Z |
| U02g | upgrade keeps durable data: Workbench refuses a store written by a newer schema (Cognate 0.7.1, live PostgreSQL), and its Experts state refuses unknown or future schemas without writing | PASS | 2026-11-15T04-28-01-188Z |
| BM-AUTH | under load at and beyond its admission limit, SEA-Forge answers every authority request once (decision or explicit busy), busy leaves no ledger trace, the chain verifies; latency within regression budgets (baseline, not a requirement) | PASS | 2026-11-15T04-28-08-566Z — informational (no budget): auth.c1.busy_rate, auth.c1.p50_ms, auth.c1.p99_ms, auth.c1.throughput_rps, auth.c12.busy_rate, auth.c12.p50_ms, auth.c12.p99_ms, auth.c12.throughput_rps, auth.c24.busy_rate, auth.c24.p50_ms, auth.c24.p95_ms, auth.c24.p99_ms, auth.c24.throughput_rps, auth.c4.busy_rate, auth.c4.p50_ms, auth.c4.p99_ms, auth.c4.throughput_rps |
| BM-RTO | after kill -9 under load SEA-Forge recovers on restart, no answered decision is lost or duplicated, in-flight requests never get a silent success; recovery within the inherited 20 s reference bound | PASS | 2026-11-15T04-28-15-105Z — informational (no budget): requests.answered, requests.total, restart.ready_ms.max, restart.ready_ms.p50, rto.cycles, rto.first_success_ms.p50 |
| BM-SETTLE | repeated settled Horizon operations: every one settles, RealityTrace verifies, stores do not grow while idle; cost per operation within regression budgets (baseline, not a requirement) | PASS | 2026-11-15T04-28-48-837Z — informational (no budget): idle_growth_bytes.horizon_store, idle_growth_bytes.realitytrace, idle_growth_bytes.sea_forge, op.wall.p50_ms, reps_settled, rss_kb.horizon_bench.after_n, rss_kb.horizon_bench.slope_per_op, rss_kb.horizon_bench.start, rss_kb.sea_forge_server.slope_per_op, rss_kb.sea_forge_server.start, stage.authority_decision.p50_ms, stage.authority_decision.p95_ms, stage.closure.p50_ms, stage.closure.p95_ms, stage.discrimination.p50_ms, stage.discrimination.p95_ms, stage.execution.p50_ms, stage.execution.p95_ms, stage.settlement.p50_ms, stage.settlement.p95_ms, stage.threshold.p50_ms, stage.threshold.p95_ms, stage.verification.p50_ms, stage.verification.p95_ms, verifications_per_op, verifier_runs_per_op |
| BM-STACK | the whole governed stack (pg0, SEA-Forge, RealityTrace, Workbench) starts ready on the declared 8 GB host, an idle open UI does not grow the ledger beyond U01's bound, U01's actions pass, and at least 1 GB stays available; footprint within regression budgets (baseline, not a requirement) | PASS | 2026-11-15T04-33-24-062Z — informational (no budget): ledger.idle_growth_entries, ledger.u01_idle_growth_entries, mem_available_mb.after_actions, mem_available_mb.after_idle, mem_available_mb.before, rss_kb.postgres.after_actions, rss_kb.postgres.after_idle, rss_kb.sea_forge.after_idle, rss_kb.total.after_idle, rss_kb.workbench.after_idle, u01.continue_ms |

## Self-hosted internal team (several humans, one organization)

**WITHHELD** — license: permitted for internal use only
- Horizon spec marks shared_remote as roadmap only; no identity provider bound at Cognate's authenticate boundary.
- SEA-Forge Rust kernel authenticates by Unix socket peer only; OIDC exists only in the Go casework gateway, with no cross-product proof.
- Cognate Workbench has a single LOCAL_OPERATOR tenant model at boot and binds loopback only.

## Hosted, managed or embedded for third parties

**NOT SUPPORTED** — license: NOT permitted without a SEA-Forge commercial license (LICENSE:48-66)

## Performance benchmarks

### BM-AUTH — PASS

The authority boundary is responsive at its admission limit and explicit beyond it. Run 2026-11-15T04-28-08-566Z. informational (no budget): auth.c1.busy_rate, auth.c1.p50_ms, auth.c1.p99_ms, auth.c1.throughput_rps, auth.c12.busy_rate, auth.c12.p50_ms, auth.c12.p99_ms, auth.c12.throughput_rps, auth.c24.busy_rate, auth.c24.p50_ms, auth.c24.p95_ms, auth.c24.p99_ms, auth.c24.throughput_rps, auth.c4.busy_rate, auth.c4.p50_ms, auth.c4.p99_ms, auth.c4.throughput_rps.

Hardware: {"cpu":"Intel(R) Core(TM) Ultra 5 226V","cores":6,"mem_total_mb":7944,"os":"Linux 6.18.58.1-microsoft-standard-WSL2","profile":"debug"}. Revisions: {"sea-forge":"7c083c86542d97c351f85e01640122a78830990a","godspeed-readiness":"1a881b9f197ad0a2ef452f7a6982f32be520d213"}.

Assertions: every-request-answered-once-within-deadline=ok, only-decisions-or-explicit-busy=ok, no-busy-within-capacity=ok, busy-leaves-no-trace=ok, ledger-chain-verifies=ok.

| Metric | Value | Budget | Provenance | Status |
|---|---|---|---|---|
| auth.c1.p95_ms | 17.392 | max 25.5 | baseline:2026-10-24T13-38-01-652Z p95 x 1.5 (17.0 ms) | ok |
| auth.c4.p95_ms | 79.511 | max 112 | baseline:2026-10-24T13-38-01-652Z p95 x 1.5 (74.7 ms) | ok |
| auth.c12.p95_ms | 332.214 | max 463 | baseline:2026-10-24T13-38-01-652Z p95 x 1.5 (308.9 ms) | ok |
| server.rss_peak_kb | 51084 | max 73134 | baseline:2026-10-24T13-38-01-652Z RSS peak x 1.5 (48756 KB) | ok |
| auth.c1.busy_rate | 0 | none | — | no budget |
| auth.c1.p50_ms | 13.176 | none | — | no budget |
| auth.c1.p99_ms | 159.263 | none | — | no budget |
| auth.c1.throughput_rps | 60.275 | none | — | no budget |
| auth.c12.busy_rate | 0 | none | — | no budget |
| auth.c12.p50_ms | 288.711 | none | — | no budget |
| auth.c12.p99_ms | 334.543 | none | — | no budget |
| auth.c12.throughput_rps | 39.727 | none | — | no budget |
| auth.c24.busy_rate | 0.76 | none | — | no budget |
| auth.c24.p50_ms | 2.201 | none | — | no budget |
| auth.c24.p95_ms | 305.753 | none | — | no budget |
| auth.c24.p99_ms | 367.231 | none | — | no budget |
| auth.c24.throughput_rps | 135.849 | none | — | no budget |
| auth.c4.busy_rate | 0 | none | — | no budget |
| auth.c4.p50_ms | 64.922 | none | — | no budget |
| auth.c4.p99_ms | 79.888 | none | — | no budget |
| auth.c4.throughput_rps | 61.003 | none | — | no budget |

### BM-RTO — PASS

After a kill -9 the authority boundary recovers on restart, and no answered decision is lost. Run 2026-11-15T04-28-15-105Z. informational (no budget): requests.answered, requests.total, restart.ready_ms.max, restart.ready_ms.p50, rto.cycles, rto.first_success_ms.p50.

Hardware: {"cpu":"Intel(R) Core(TM) Ultra 5 226V","cores":6,"mem_total_mb":7944,"os":"Linux 6.18.58.1-microsoft-standard-WSL2","profile":"debug"}. Revisions: {"sea-forge":"7c083c86542d97c351f85e01640122a78830990a","godspeed-readiness":"1a881b9f197ad0a2ef452f7a6982f32be520d213"}.

Assertions: every-answered-decision-in-ledger-exactly-once=ok, no-allow-lost=ok, in-flight-at-kill-errored-not-success=ok, ledger-chain-verifies-after-each-restart=ok, every-cycle-recovered=ok.

| Metric | Value | Budget | Provenance | Status |
|---|---|---|---|---|
| rto.first_success_ms.max | 1017.075 | max 20000 | inherited:sea-forge@0106ffc:apps/godspeed-casework-go/internal/loadtest/budgets_test.go#budgetRecoveryMs | ok |
| requests.answered | 325 | none | — | no budget |
| requests.total | 345 | none | — | no budget |
| restart.ready_ms.max | 920.378 | none | — | no budget |
| restart.ready_ms.p50 | 587.785 | none | — | no budget |
| rto.cycles | 5 | none | — | no budget |
| rto.first_success_ms.p50 | 713.822 | none | — | no budget |

### BM-SETTLE — PASS

What does one consequential, settled Horizon operation cost, and does the stack leak? (docs/benchmarks.md, BM-SETTLE). Run 2026-11-15T04-28-48-837Z. informational (no budget): idle_growth_bytes.horizon_store, idle_growth_bytes.realitytrace, idle_growth_bytes.sea_forge, op.wall.p50_ms, reps_settled, rss_kb.horizon_bench.after_n, rss_kb.horizon_bench.slope_per_op, rss_kb.horizon_bench.start, rss_kb.sea_forge_server.slope_per_op, rss_kb.sea_forge_server.start, stage.authority_decision.p50_ms, stage.authority_decision.p95_ms, stage.closure.p50_ms, stage.closure.p95_ms, stage.discrimination.p50_ms, stage.discrimination.p95_ms, stage.execution.p50_ms, stage.execution.p95_ms, stage.settlement.p50_ms, stage.settlement.p95_ms, stage.threshold.p50_ms, stage.threshold.p95_ms, stage.verification.p50_ms, stage.verification.p95_ms, verifications_per_op, verifier_runs_per_op.

Hardware: {"cpu":"Intel(R) Core(TM) Ultra 5 226V","cores":6,"mem_total_mb":7944,"os":"Linux 6.18.58.1-microsoft-standard-WSL2","profile":"debug"}. Revisions: {"horizon":"11f8ab0d792c48a7c0f520e7017f8a474da4d976","sea-forge":"7c083c86542d97c351f85e01640122a78830990a","realitytrace":"521d363a4ed398a0bc8e51a58f8e0462f9a73065"}.

Assertions: every-rep-settled-supported=ok, realitytrace-ledger-verifies=ok, idle-growth-zero-sea-forge=ok, idle-growth-zero-realitytrace=ok, idle-growth-zero-horizon-store=ok.

| Metric | Value | Budget | Provenance | Status |
|---|---|---|---|---|
| op.wall.p95_ms | 22093 | max 32590 | baseline:2026-10-24T15-45-00-091Z op.wall.p95 x 1.5 (21727 ms) | ok |
| bytes_per_op.sea_forge | 36473.6 | max 54146 | baseline:2026-10-24T15-45-00-091Z x 1.5 (36097.6 B/op) | ok |
| bytes_per_op.realitytrace | 51335 | max 76921 | baseline:2026-10-24T15-45-00-091Z x 1.5 (51280.6 B/op) | ok |
| bytes_per_op.horizon_store | 425028 | max 639396 | baseline:2026-10-24T15-45-00-091Z x 1.5 (426264 B/op) | ok |
| rss_kb.sea_forge_server.after_n | 44392 | max 73518 | baseline:2026-10-24T15-45-00-091Z RSS after n=10 x 1.5 (49012 KB) | ok |
| idle_growth_bytes.horizon_store | 0 | none | — | no budget |
| idle_growth_bytes.realitytrace | 0 | none | — | no budget |
| idle_growth_bytes.sea_forge | 0 | none | — | no budget |
| op.wall.p50_ms | 20733 | none | — | no budget |
| reps_settled | 10 | none | — | no budget |
| rss_kb.horizon_bench.after_n | 103920 | none | — | no budget |
| rss_kb.horizon_bench.slope_per_op | 1483.4666666666667 | none | — | no budget |
| rss_kb.horizon_bench.start | 252836 | none | — | no budget |
| rss_kb.sea_forge_server.slope_per_op | 415.7818181818182 | none | — | no budget |
| rss_kb.sea_forge_server.start | 24708 | none | — | no budget |
| stage.authority_decision.p50_ms | 756 | none | — | no budget |
| stage.authority_decision.p95_ms | 3339 | none | — | no budget |
| stage.closure.p50_ms | 138 | none | — | no budget |
| stage.closure.p95_ms | 167 | none | — | no budget |
| stage.discrimination.p50_ms | 51 | none | — | no budget |
| stage.discrimination.p95_ms | 100 | none | — | no budget |
| stage.execution.p50_ms | 807 | none | — | no budget |
| stage.execution.p95_ms | 1037 | none | — | no budget |
| stage.settlement.p50_ms | 112 | none | — | no budget |
| stage.settlement.p95_ms | 168 | none | — | no budget |
| stage.threshold.p50_ms | 240 | none | — | no budget |
| stage.threshold.p95_ms | 1456 | none | — | no budget |
| stage.verification.p50_ms | 50 | none | — | no budget |
| stage.verification.p95_ms | 51 | none | — | no budget |
| verifications_per_op | 1 | none | — | no budget |
| verifier_runs_per_op | 2 | none | — | no budget |

### BM-STACK — PASS

Does the full governed stack fit the declared host with headroom? (docs/benchmarks.md, BM-STACK). Run 2026-11-15T04-33-24-062Z. informational (no budget): ledger.idle_growth_entries, ledger.u01_idle_growth_entries, mem_available_mb.after_actions, mem_available_mb.after_idle, mem_available_mb.before, rss_kb.postgres.after_actions, rss_kb.postgres.after_idle, rss_kb.sea_forge.after_idle, rss_kb.total.after_idle, rss_kb.workbench.after_idle, u01.continue_ms.

Hardware: {"cpu":"Intel(R) Core(TM) Ultra 5 226V","cores":6,"mem_total_mb":7944,"os":"Linux 6.18.58.1-microsoft-standard-WSL2","profile":"debug"}. Revisions: {"sea-forge":"7c083c86542d97c351f85e01640122a78830990a","godspeed-readiness":"1a881b9f197ad0a2ef452f7a6982f32be520d213"}.

Assertions: stack-up-exit-0=ok, status-all-ready-after-up=ok, idle-ledger-growth-within-bound=ok, u01-actions-pass=ok, status-all-ready-after-actions=ok, host-mem-available-after-actions-floor=ok.

| Metric | Value | Budget | Provenance | Status |
|---|---|---|---|---|
| time_to_ready_ms | 5190 | max 10695 | baseline:2026-10-24T23-59-18-401Z x 1.5 (7130 ms) | ok |
| rss_kb.total.after_actions | 550100 | max 663000 | baseline:2026-10-24T23-59-18-401Z x 1.5 (442000 KB) | ok |
| rss_kb.sea_forge.after_actions | 43268 | max 56022 | baseline:2026-10-24T23-59-18-401Z x 1.5 (37348 KB) | ok |
| rss_kb.workbench.after_actions | 130956 | max 190104 | baseline:2026-10-24T23-59-18-401Z x 1.5 (126736 KB) | ok |
| ledger.idle_growth_entries | 0 | none | — | no budget |
| ledger.u01_idle_growth_entries | 0 | none | — | no budget |
| mem_available_mb.after_actions | 5042 | none | — | no budget |
| mem_available_mb.after_idle | 5055 | none | — | no budget |
| mem_available_mb.before | 5088 | none | — | no budget |
| rss_kb.postgres.after_actions | 375876 | none | — | no budget |
| rss_kb.postgres.after_idle | 371060 | none | — | no budget |
| rss_kb.sea_forge.after_idle | 43228 | none | — | no budget |
| rss_kb.total.after_idle | 542016 | none | — | no budget |
| rss_kb.workbench.after_idle | 127728 | none | — | no budget |
| u01.continue_ms | 910 | none | — | no budget |

## Pinned revisions

| Product | Checkout | Revision | Dirty | Ahead of upstream |
|---|---|---|---|---|
| domainforge |  | ac8c4218d76c | 0 | ? |
| sea-forge |  | 7c083c86542d | 0 | ? |
| context-kernel |  | 36bd8c8977ac | 0 | ? |
| gauntlet |  | 1543cbf0696e | 0 | ? |
| swe-seed |  | 232e02d577f3 | 0 | ? |
| realitytrace |  | 521d363a4ed3 | 0 | ? |
| cognate |  | ca74ab5f4225 | 0 | ? |
| cognate-workbench |  | bd4c0ea996a6 | 0 | ? |
| horizon |  | 11f8ab0d792c | 0 | ? |
| cep |  | 5111ec388ebf | 0 | ? |
| godspeed-agent | migration/cep-world-required-evidence | 4bfed12fb091 | 2 | 0 |
| project-topology-architect | main | eb812ac91b33 | 0 | 0 |

