# GodSpeed readiness report

Generated 2026-11-13T07:20:21.709Z from 328 run(s). Only evidence at the currently pinned revisions counts.

## Local single operator (Linux / WSL2, loopback only, internal use)

**NO-GO** — 19/30 gates passed at pinned clean revisions. License: permitted (SEA-Forge Sustainable Use License, internal business use)

| Gate | Invariant | Outcome | Evidence |
|---|---|---|---|
| L01 | 7 authority is external; 9 external consequences need independent evidence | BLOCKED | 2026-11-13T07-04-27-105Z — probe failed: ${domainforge}/target/release/domainforge --version | grep -q ' 0.21.1$' (DomainForge must be 0.21.1) |
| S01 | loopback surfaces cannot be driven by a foreign web origin | PASS | 2026-11-13T07-10-09-392Z |
| S02 | 9 an executing agent cannot file its own observations (contained agent cannot reach RealityTrace or SEA-Forge) | PASS | 2026-11-13T07-10-13-185Z |
| S03 | proof verifies artifact content, not existence | PASS | 2026-11-13T07-10-16-360Z |
| R01 | a crash mid-effect never silently repeats an external effect | PASS | 2026-11-13T07-08-43-628Z |
| C01 | one DomainForge producer version across the stack (world identity agreement) | FAIL | 2026-11-13T07-02-31-907Z — exit 1 (expected 0) |
| R02 | missing/late observation is never settlement | PASS | 2026-11-13T07-10-02-094Z |
| O01 | clean install from pinned revisions on a fresh home, no author environment (GitHub-hosted runner evidence) | PASS | 2026-11-13T07-04-27-669Z |
| O02 | RealityTrace settlement ledger survives backup, loss and restore | PASS | 2026-11-13T07-04-31-941Z |
| O03 | Workbench state (Postgres schema + home) survives backup, loss and restore | PASS | 2026-11-13T07-06-54-966Z |
| O04 | SEA-Forge cell survives its shipped backup/restore scripts | PASS | 2026-11-13T07-07-02-519Z |
| J02 | Horizon navigation commits are decided by SEA-Forge, committed once after approval, observed by RealityTrace and settled by SEA-Forge (IS3-IS6; IS2 and IS7 open in the frozen source) | FAIL | 2026-11-13T07-03-01-812Z — exit 1 (expected 0); fail marker: \b[1-9]\d* fail\b |
| SW01 | 9 governed software work is settled by SEA-Forge on independent RealityTrace evidence: Workbench -> SEA-Forge authority -> RealityTrace (contained proof) -> SEA-Forge settlement | FAIL | 2026-11-13T07-10-17-333Z — exit 1 (expected 0); fail marker: \b[1-9]\d* fail\b |
| J02R2 | 4/9 a bound Horizon operation is paid within capacity, preregistered with RealityTrace, verified by a pinned confined verifier and settled by SEA-Forge from RealityTrace's ledger (HORIZON-001 1.1.0 IS2, IS7) | FAIL | 2026-11-13T07-04-13-766Z — exit 1 (expected 0); fail marker: \b[1-9]\d* fail\b |
| U01 | the governed Workbench UI stays usable: Continue advances, an idle page does not grow SEA-Forge's ledger | FAIL | 2026-11-13T07-10-22-668Z — exit 1 (expected 0) |
| SW02 | the declared proof comes from a SWE_SEED work-and-proof contract (frozen by digest), not a hand-typed command | FAIL | 2026-11-13T07-10-20-785Z — exit 1 (expected 0); fail marker: \b[1-9]\d* fail\b |
| G01 | a Gauntlet run killed at a durable boundary (real binary, SIGKILL) resumes to one run id and dispatches the irreversible unit exactly once; a poison child terminates bounded | PASS | 2026-11-13T07-02-39-493Z |
| CK01 | SWE_SEED gets a cited context packet from a live Context Kernel binary, and reports Context Kernel absence explicitly, never by fabrication | BLOCKED | 2026-11-13T07-02-35-880Z — setup step 0 failed (exit 101); see CK01.setup.log |
| CK02 | a tampered, falsely complete, or partial context bundle is refused at the SWE_SEED boundary (contract, fixtures; no live Context Kernel) | PASS | 2026-11-13T07-02-37-732Z |
| U02a | upgrade keeps durable data: Horizon 1.1.0 suspends v1 runs; the rollback guard refuses a 1.0.1 start against v2 state (contract; the script's exit codes are U02e) | PASS | 2026-11-13T07-11-55-273Z |
| U02e | upgrade keeps durable data: Horizon's rollback guard script exits 0 for a v1 store, 2 for any v2 state, 1 for a missing store (real process) | PASS | 2026-11-13T07-13-06-767Z |
| U02b | upgrade keeps durable data: Cognate SQLite-to-PostgreSQL migration preserves every event, verifies, and refuses a second import (live PostgreSQL) | PASS | 2026-11-13T07-11-56-417Z |
| U02c | upgrade keeps durable data: RealityTrace's ledger refuses a store with a newer user_version (unit; not an end-to-end rollback) | PASS | 2026-11-13T07-12-00-355Z |
| U02d | upgrade keeps durable data: SEA-Forge admits a DomainForge compiler upgrade only when no meaning-bearing field changes (contract) | PASS | 2026-11-13T07-12-06-549Z |
| U02f | upgrade keeps durable data: Cognate refuses to open a SQLite or PostgreSQL store written by a newer schema version, and writes nothing (live PostgreSQL) | PASS | 2026-11-13T07-13-10-418Z |
| U02g | upgrade keeps durable data: Workbench refuses a store written by a newer schema (Cognate 0.7.1, live PostgreSQL), and its Experts state refuses unknown or future schemas without writing | PASS | 2026-11-13T07-13-15-666Z |
| BM-AUTH | under load at and beyond its admission limit, SEA-Forge answers every authority request once (decision or explicit busy), busy leaves no ledger trace, the chain verifies; latency within regression budgets (baseline, not a requirement) | FAIL | 2026-11-13T07-13-24-334Z — auth.c4.p95_ms = 168.431 exceeds max 112 |
| BM-RTO | after kill -9 under load SEA-Forge recovers on restart, no answered decision is lost or duplicated, in-flight requests never get a silent success; recovery within the inherited 20 s reference bound | PASS | 2026-11-13T07-13-32-021Z — informational (no budget): requests.answered, requests.total, restart.ready_ms.max, restart.ready_ms.p50, rto.cycles, rto.first_success_ms.p50 |
| BM-SETTLE | repeated settled Horizon operations: every one settles, RealityTrace verifies, stores do not grow while idle; cost per operation within regression budgets (baseline, not a requirement) | FAIL | 2026-11-13T07-14-08-247Z — exit 1 (expected 0); fail marker: hard assertion failed |
| BM-STACK | the whole governed stack (pg0, SEA-Forge, RealityTrace, Workbench) starts ready on the declared 8 GB host, an idle open UI does not grow the ledger beyond U01's bound, U01's actions pass, and at least 1 GB stays available; footprint within regression budgets (baseline, not a requirement) | FAIL | 2026-11-13T07-18-42-056Z — assertion failed: stack-up-exit-0 (gsr up exit 1); budgeted metric missing: time_to_ready_ms; budgeted metric missing: rss_kb.total.after_actions; budgeted metric missing: rss_kb.sea_forge.after_actions; budgeted metric missing: rss_kb.workbench.after_actions |

## Self-hosted internal team (several humans, one organization)

**WITHHELD** — license: permitted for internal use only
- Horizon spec marks shared_remote as roadmap only; no identity provider bound at Cognate's authenticate boundary.
- SEA-Forge Rust kernel authenticates by Unix socket peer only; OIDC exists only in the Go casework gateway, with no cross-product proof.
- Cognate Workbench has a single LOCAL_OPERATOR tenant model at boot and binds loopback only.

## Hosted, managed or embedded for third parties

**NOT SUPPORTED** — license: NOT permitted without a SEA-Forge commercial license (LICENSE:48-66)

## Performance benchmarks

### BM-AUTH — FAIL

The authority boundary is responsive at its admission limit and explicit beyond it. Run 2026-11-13T07-13-24-334Z. auth.c4.p95_ms = 168.431 exceeds max 112.

Hardware: {"cpu":"Intel(R) Core(TM) Ultra 5 226V","cores":6,"mem_total_mb":7944,"os":"Linux 6.18.58.1-microsoft-standard-WSL2","profile":"debug"}. Revisions: {"sea-forge":"595b5270552e622b883e8df3fddbb3fe6acc273d","godspeed-readiness":"ae9060781e04ed2a2fbe70e7e118eae6bb922b4a"}.

Assertions: every-request-answered-once-within-deadline=ok, only-decisions-or-explicit-busy=ok, no-busy-within-capacity=ok, busy-leaves-no-trace=ok, ledger-chain-verifies=ok.

| Metric | Value | Budget | Provenance | Status |
|---|---|---|---|---|
| auth.c1.p95_ms | 22.391 | max 25.5 | baseline:2026-10-24T13-38-01-652Z p95 x 1.5 (17.0 ms) | ok |
| auth.c4.p95_ms | 168.431 | max 112 | baseline:2026-10-24T13-38-01-652Z p95 x 1.5 (74.7 ms) | over |
| auth.c12.p95_ms | 378.412 | max 463 | baseline:2026-10-24T13-38-01-652Z p95 x 1.5 (308.9 ms) | ok |
| server.rss_peak_kb | 52648 | max 73134 | baseline:2026-10-24T13-38-01-652Z RSS peak x 1.5 (48756 KB) | ok |
| auth.c1.busy_rate | 0 | none | — | no budget |
| auth.c1.p50_ms | 13.599 | none | — | no budget |
| auth.c1.p99_ms | 174.361 | none | — | no budget |
| auth.c1.throughput_rps | 54.585 | none | — | no budget |
| auth.c12.busy_rate | 0 | none | — | no budget |
| auth.c12.p50_ms | 317.841 | none | — | no budget |
| auth.c12.p99_ms | 380.256 | none | — | no budget |
| auth.c12.throughput_rps | 35.298 | none | — | no budget |
| auth.c24.busy_rate | 0.76 | none | — | no budget |
| auth.c24.p50_ms | 2.086 | none | — | no budget |
| auth.c24.p95_ms | 365.102 | none | — | no budget |
| auth.c24.p99_ms | 429.493 | none | — | no budget |
| auth.c24.throughput_rps | 116.216 | none | — | no budget |
| auth.c4.busy_rate | 0 | none | — | no budget |
| auth.c4.p50_ms | 95.359 | none | — | no budget |
| auth.c4.p99_ms | 180.429 | none | — | no budget |
| auth.c4.throughput_rps | 39.659 | none | — | no budget |

### BM-RTO — PASS

After a kill -9 the authority boundary recovers on restart, and no answered decision is lost. Run 2026-11-13T07-13-32-021Z. informational (no budget): requests.answered, requests.total, restart.ready_ms.max, restart.ready_ms.p50, rto.cycles, rto.first_success_ms.p50.

Hardware: {"cpu":"Intel(R) Core(TM) Ultra 5 226V","cores":6,"mem_total_mb":7944,"os":"Linux 6.18.58.1-microsoft-standard-WSL2","profile":"debug"}. Revisions: {"sea-forge":"595b5270552e622b883e8df3fddbb3fe6acc273d","godspeed-readiness":"ae9060781e04ed2a2fbe70e7e118eae6bb922b4a"}.

Assertions: every-answered-decision-in-ledger-exactly-once=ok, no-allow-lost=ok, in-flight-at-kill-errored-not-success=ok, ledger-chain-verifies-after-each-restart=ok, every-cycle-recovered=ok.

| Metric | Value | Budget | Provenance | Status |
|---|---|---|---|---|
| rto.first_success_ms.max | 1950.913 | max 20000 | inherited:sea-forge@0106ffc:apps/godspeed-casework-go/internal/loadtest/budgets_test.go#budgetRecoveryMs | ok |
| requests.answered | 234 | none | — | no budget |
| requests.total | 254 | none | — | no budget |
| restart.ready_ms.max | 1152.034 | none | — | no budget |
| restart.ready_ms.p50 | 601.204 | none | — | no budget |
| rto.cycles | 5 | none | — | no budget |
| rto.first_success_ms.p50 | 1300.942 | none | — | no budget |

### BM-STACK — FAIL

Does the full governed stack fit the declared host with headroom? (docs/benchmarks.md, BM-STACK). Run 2026-11-13T07-18-42-056Z. assertion failed: stack-up-exit-0 (gsr up exit 1); budgeted metric missing: time_to_ready_ms; budgeted metric missing: rss_kb.total.after_actions; budgeted metric missing: rss_kb.sea_forge.after_actions; budgeted metric missing: rss_kb.workbench.after_actions.

Hardware: {"cpu":"Intel(R) Core(TM) Ultra 5 226V","cores":6,"mem_total_mb":7944,"os":"Linux 6.18.58.1-microsoft-standard-WSL2","profile":"debug"}. Revisions: {"sea-forge":"595b5270552e622b883e8df3fddbb3fe6acc273d","godspeed-readiness":"ae9060781e04ed2a2fbe70e7e118eae6bb922b4a"}.

Assertions: stack-up-exit-0=FAILED.

| Metric | Value | Budget | Provenance | Status |
|---|---|---|---|---|
| time_to_ready_ms | — | max 10695 | baseline:2026-10-24T23-59-18-401Z x 1.5 (7130 ms) | missing |
| rss_kb.total.after_actions | — | max 663000 | baseline:2026-10-24T23-59-18-401Z x 1.5 (442000 KB) | missing |
| rss_kb.sea_forge.after_actions | — | max 56022 | baseline:2026-10-24T23-59-18-401Z x 1.5 (37348 KB) | missing |
| rss_kb.workbench.after_actions | — | max 190104 | baseline:2026-10-24T23-59-18-401Z x 1.5 (126736 KB) | missing |
| mem_available_mb.before | 4128 | none | — | no budget |

## Pinned revisions

| Product | Checkout | Revision | Dirty | Ahead of upstream |
|---|---|---|---|---|
| domainforge |  | f90905a91987 | 0 | ? |
| sea-forge |  | 595b5270552e | 0 | ? |
| context-kernel |  | 36bd8c8977ac | 0 | ? |
| gauntlet |  | 1543cbf0696e | 0 | ? |
| swe-seed |  | 232e02d577f3 | 0 | ? |
| realitytrace |  | 4d154b12da03 | 0 | ? |
| cognate |  | bcef6637449a | 0 | ? |
| cognate-workbench |  | 74601f3a7ca4 | 0 | ? |
| horizon |  | 822e349e2fe1 | 0 | ? |
| cep |  | 5111ec388ebf | 0 | ? |
| godspeed-agent | migration/cep-world-required-evidence | 4bfed12fb091 | 2 | 0 |
| project-topology-architect | main | eb812ac91b33 | 0 | 0 |

