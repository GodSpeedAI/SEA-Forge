# WP-19 evidence: S1 gate (SEA-Forge half)

Run on SEA-Forge `fde/integration` `fef0d900` (clean tree), episode cell `20261114T234200Z` (scratch, `~/projects/_wt/tmp/fde-s1/`, base commit of the episode workspaces `4af5652a`; later commits only changed the verifier, the claim-ledger builder and two proof wrappers). Bundle: `.agents/evidence/fde-field-case/s1-20261114T234200Z/` (`claim-ledger.md|json`, `verification/results.json` and per-test logs, `operational/` kept raw records, `raw-manifest.sha256` over all 198 kept raw files; 3 files (RealityTrace ledger.db and others over 1 MB) are referenced by digest only, listed in `raw-not-copied.txt`, and stay in the cell).

## What exists
- `scripts/fde-proof.sh --slice s1`: records per test command, exit code, status, duration, reason, repo commits and tool versions (`results.json`). Wrappers in `scripts/fde-proof.d/` for all 21 frozen tests. Absent or off-commit siblings give NOT_RUN (exit 12), never PASS. `--mutate` and `--replay` remain NOT_RUN stubs.
- `scripts/fde-s1-run.sh` (`scripts/fde-s1/run.py`, `cognate_run.ts`, `crates/sea-forge-server/examples/fde_s1_driver.rs`): the whole episode run; every step in `steps.jsonl`, gaps in `gaps.jsonl`.
- `scripts/fde-s1-verify.sh` (AT-18, S1 form), `scripts/fde-claim-ledger.py` (mechanical; fails when a frozen test is missing), `scripts/fde-s1/bundle.py`.
- Hash-pinned patches `scripts/fde-s1/patches/` (ep-1 events tail read, ep-2 KG round-trip test, ep-3 readiness invalidation) and `scripts/fde-s1/model/sop-v2-fresh-context-verification.patch` (the revision candidate, declares the harness's fresh-context verification step).

## Episodes (real runs)
ep-1 (E2): freeze via `case_record_append` (two-phase, anchor script, tag in a scratch repo), Cognate `program.run` applying the pinned patch and running `cargo test --locked -p sea-forge-server --test conformance_lifecycle` (exit 0), a second Cognate run by `fresh-verifier` re-ran it, RealityTrace ingest/derive x2/compare, settlement by `independent-settler` (not an executor). Result U (predicted S): `authorize_consequential_action` is `insufficient/policy_unknown` because no SEA-Forge authority decision exists for program execution.
ep-2 (E3, DomainForge origin/main ac8c4218): test fails (Policy triples 0 vs 1), settled F (predicted F). No `contradicted` residual arises: no model element or mapping form expresses the hypothesis outcome.
Revision: diagnosis ep-1, held-out ep-2 (rule digest pinned), `observed propose`, candidate world v2 (`0d55378c...`), regression matrix pass (8 rows), escalation by `cognate-service`, self-approval refused, early promote refused (retained), approval by `op-alice` (session assurance), `authority_promote` ok.
ep-3 (E1) frozen under v2, control variant (`pi-e1-stale-control`) run without the verification step, `observed capability-delta`: **NO_GAIN** (predicted GAIN): the step is emitted only by Cognate, the executor's own emitter, so RealityTrace keeps it `insufficient/executor_only_evidence` under v2. R-58 not supported. ep-3 settled U.

## Results
Counts: {'FAIL': 0, 'NOT_RUN': 0, 'NOT_TESTED': 0, 'PARTIAL': 11, 'PASS': 10}; pre-registered {'FAIL': 0, 'NOT_TESTED': 0, 'PARTIAL': 9, 'PASS': 12}. Mismatches: AT-14, NT-03, NT-06, NT-07.

| Test | Predicted | Observed | Match | Reason |
|---|---|---|---|---|
| AT-01 | PASS | PASS | match |  |
| AT-04 | PARTIAL | PARTIAL | match | order check and digest equality pass in the producer tests; anchor time is the local clock only (AnchorTimeLocalOnly, OD-6), execution-side I-1 gate n |
| AT-05 | PARTIAL | PARTIAL | match | SEA-Forge half and Cognate half pass; emitter signatures arrive with WP-33 so the signature check cannot run; authenticity rests on source-record reso |
| AT-06 | PASS | PASS | match |  |
| AT-07 | PARTIAL | PARTIAL | match | classes arose on the real episode set (see cell results); declared_unobserved and any unarising class NOT TESTED; postcondition expectations insuffici |
| AT-08 | PASS | PASS | match |  |
| AT-12 | PARTIAL | PARTIAL | match |  |
| AT-13 | PARTIAL | PARTIAL | match |  |
| AT-14 | PASS | PARTIAL | MISMATCH | parity passes on the producer's fixtures (real RealityTrace CLI output and real case_records::append output); the cell's own raw records were not rend |
| AT-18 | PARTIAL | PARTIAL | match | anchors verify (prereg by fde-anchor.sh; freeze tags by digest, local clock only), replay hashes and classes equal the recorded ones; re-execution of  |
| NT-01 | PASS | PASS | match |  |
| NT-02 | PASS | PASS | match |  |
| NT-03 | PASS | PARTIAL | MISMATCH | producer tests pass; a real re-run with a changed outcome was not run |
| NT-03b | PASS | PASS | match |  |
| NT-04 | PARTIAL | PARTIAL | match | both halves pass; the authorizing approval is a session-assurance approval (A3 and a human-reserved decision need S3) |
| NT-05 | PASS | PASS | match |  |
| NT-06 | PASS | PARTIAL | MISMATCH | producer test passes; a silent source on the real cell was not run |
| NT-07 | PARTIAL | PASS | MISMATCH |  |
| NT-08 | PASS | PASS | match |  |
| NT-09 | PASS | PASS | match |  |
| NT-16 | PARTIAL | PARTIAL | match | the tamper/order check refuses as specified; the server push time is not read from the remote (AnchorTimeLocalOnly) and the anchor is a shared-OS-user |

S1 claim status (from the ledger): SUPPORTED at bounded-interaction level, once, on program execution; PARTIAL rows: AT-04, AT-05, AT-07, AT-12, AT-13, AT-14, AT-18, NT-03, NT-04, NT-06, NT-16; all other behaviour classes NOT TESTED.

## Deviations (read these)
- **DomainForge version.** The stack ran on the DomainForge 3960326 build (`domainforge 0.21.2` line, has `evaluate`, sha256 `67559ad8b5def99b1cdb0dd6d65521abeb635b937cbeca69a6513522d4719d52`), not the released 0.22.0. Reason: SEA-Forge at tip embeds domainforge-core =0.21.2, so its world registry and pin produce `world:godspeed-fde-sop@sha256:78eddd4e...` (the world frozen in episodes.yaml); 0.22.0 gives `e3c3d396...` for the same sources and cannot be registered by SEA-Forge until its lockstep PR (#99) lands. 0.22.0 (`2d0822810644746417cfb82065472baac11296e2a6ab2ba94d95196e428d5d14`) was run for `validate` and `envelope` on both models; its world_refs are recorded in `state.json`. The 3960326 binary was rebuilt locally from the detached worktree (my own 0.22.0 test build had overwritten it).
- Execution is not gated by SEA-Forge (I-1 unwired): Cognate ran under a local allow policy.
- Interaction events travel as spool files; evidence rows are seeded (rt-seed); only `run.invocation` events are emitted (host filter) because run-lifecycle events carry per-run resource ids that make undeclared residuals incomparable across episodes.
- Anchors: scratch repository, local clock (OD-6); the programme origin received no freeze tags. The results digest was not anchored (no new tag kind was added to `fde-anchor.sh`; the digests are in `raw-manifest.sha256`).
- AT-14, NT-03, NT-06 are PARTIAL (predicted PASS) because the cell's own records were not rendered by Horizon, a real re-run with a changed value was not run, and no silent-source episode was run; the producer tests pass. AT-04 and NT-16 are PARTIAL (predicted PARTIAL) with the producer tests passing. NT-07 is PASS (predicted PARTIAL): both promotion and re-pin variants run in producer tests.
- Patches are not regenerated into SFWP schemas/golden frames; ep-2's patch is a test appended to an existing file (`program.run` cannot create files).
- AT-18: the verifier re-ran every derive/compare/propose/regression/delta over a copy of the kept ledger and got identical bytes and classes; it did NOT re-execute the program episodes.
- Resources: other agent's load kept free memory near 3 GB during the run; no OOM occurred.
