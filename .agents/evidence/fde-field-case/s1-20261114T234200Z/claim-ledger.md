# S1 claim ledger

Built mechanically by `scripts/fde-claim-ledger.py` from `results.json` and the frozen predictions; no hand-edited verdicts.

**Claim (G section 2, RD-20, verbatim):** "the Mission §1–7 path exists and runs once on program execution." It is not evidence about organisational behaviour.

**Claim status:** SUPPORTED at bounded-interaction level, once, on program execution; PARTIAL rows: AT-04, AT-05, AT-07, AT-12, AT-13, AT-14, AT-18, NT-03, NT-04, NT-06, NT-16; all other behaviour classes NOT TESTED

**Evidence level:** bounded interaction (once, program executors). Repetition, variation, recovery and transfer are not established by this run.

**Counts:** observed {'PASS': 10, 'FAIL': 0, 'PARTIAL': 11, 'NOT_TESTED': 0, 'NOT_RUN': 0}; pre-registered {'PASS': 12, 'PARTIAL': 9, 'FAIL': 0, 'NOT_TESTED': 0}. Prediction mismatches: AT-14, NT-03, NT-06, NT-07.

## Per test: prediction, observed, match

| Test | Prediction | Observed | Match | Exit | Reason / limitation |
|---|---|---|---|---|---|
| AT-01 | PASS | PASS | match | 0 |  |
| AT-04 | PARTIAL | PARTIAL | match | 10 | order check and digest equality pass in the producer tests; anchor time is the local clock only (AnchorTimeLocalOnly, OD-6), execution-side I-1 gate not wired (WP-20), so freeze-before-execution is verified after the fact, not enforced |
| AT-05 | PARTIAL | PARTIAL | match | 10 | SEA-Forge half and Cognate half pass; emitter signatures arrive with WP-33 so the signature check cannot run; authenticity rests on source-record resolution and the independent-emitter rule |
| AT-06 | PASS | PASS | match | 0 |  |
| AT-07 | PARTIAL | PARTIAL | match | 10 | classes arose on the real episode set (see cell results); declared_unobserved and any unarising class NOT TESTED; postcondition expectations insufficient (DomainForge accepts only at precondition) |
| AT-08 | PASS | PASS | match | 0 |  |
| AT-12 | PARTIAL | PARTIAL | match | 10 |  |
| AT-13 | PARTIAL | PARTIAL | match | 10 |  |
| AT-14 | PASS | PARTIAL | MISMATCH | 10 | parity passes on the producer's fixtures (real RealityTrace CLI output and real case_records::append output); the cell's own raw records were not rendered |
| AT-18 | PARTIAL | PARTIAL | match | 10 | anchors verify (prereg by fde-anchor.sh; freeze tags by digest, local clock only), replay hashes and classes equal the recorded ones; re-execution of the program episodes was not repeated by the verifier |
| NT-01 | PASS | PASS | match | 0 |  |
| NT-02 | PASS | PASS | match | 0 |  |
| NT-03 | PASS | PARTIAL | MISMATCH | 10 | producer tests pass; a real re-run with a changed outcome was not run |
| NT-03b | PASS | PASS | match | 0 |  |
| NT-04 | PARTIAL | PARTIAL | match | 10 | both halves pass; the authorizing approval is a session-assurance approval (A3 and a human-reserved decision need S3) |
| NT-05 | PASS | PASS | match | 0 |  |
| NT-06 | PASS | PARTIAL | MISMATCH | 10 | producer test passes; a silent source on the real cell was not run |
| NT-07 | PARTIAL | PASS | MISMATCH | 0 |  |
| NT-08 | PASS | PASS | match | 0 |  |
| NT-09 | PASS | PASS | match | 0 |  |
| NT-16 | PARTIAL | PARTIAL | match | 10 | the tamper/order check refuses as specified; the server push time is not read from the remote (AnchorTimeLocalOnly) and the anchor is a shared-OS-user tag without a protected namespace (RD-26, OD-6) |

Evidence pointers: per test the log is in `evidence`; command is `scripts/fde-proof.sh --slice s1 --only <ID>`.

- AT-01: `scripts/fde-proof.sh --slice s1 --only AT-01`, log `.tmp/fde-proof/s1/AT-01.log`, duration 4.26 s
- AT-04: `scripts/fde-proof.sh --slice s1 --only AT-04`, log `.tmp/fde-proof/s1/AT-04.log`, duration 0.91 s
- AT-05: `scripts/fde-proof.sh --slice s1 --only AT-05`, log `.tmp/fde-proof/s1/AT-05.log`, duration 8.69 s
- AT-06: `scripts/fde-proof.sh --slice s1 --only AT-06`, log `.tmp/fde-proof/s1/AT-06.log`, duration 2.15 s
- AT-07: `scripts/fde-proof.sh --slice s1 --only AT-07`, log `.tmp/fde-proof/s1/AT-07.log`, duration 3.11 s
- AT-08: `scripts/fde-proof.sh --slice s1 --only AT-08`, log `.tmp/fde-proof/s1/AT-08.log`, duration 1.77 s
- AT-12: `scripts/fde-proof.sh --slice s1 --only AT-12`, log `.tmp/fde-proof/s1/AT-12.log`, duration 6.51 s
- AT-13: `scripts/fde-proof.sh --slice s1 --only AT-13`, log `.tmp/fde-proof/s1/AT-13.log`, duration 5.41 s
- AT-14: `scripts/fde-proof.sh --slice s1 --only AT-14`, log `.tmp/fde-proof/s1/AT-14.log`, duration 19.38 s
- AT-18: `scripts/fde-proof.sh --slice s1 --only AT-18`, log `.tmp/fde-proof/s1/AT-18.log`, duration 8.53 s
- NT-01: `scripts/fde-proof.sh --slice s1 --only NT-01`, log `.tmp/fde-proof/s1/NT-01.log`, duration 0.42 s
- NT-02: `scripts/fde-proof.sh --slice s1 --only NT-02`, log `.tmp/fde-proof/s1/NT-02.log`, duration 1.38 s
- NT-03: `scripts/fde-proof.sh --slice s1 --only NT-03`, log `.tmp/fde-proof/s1/NT-03.log`, duration 1.36 s
- NT-03b: `scripts/fde-proof.sh --slice s1 --only NT-03b`, log `.tmp/fde-proof/s1/NT-03b.log`, duration 1.54 s
- NT-04: `scripts/fde-proof.sh --slice s1 --only NT-04`, log `.tmp/fde-proof/s1/NT-04.log`, duration 4.25 s
- NT-05: `scripts/fde-proof.sh --slice s1 --only NT-05`, log `.tmp/fde-proof/s1/NT-05.log`, duration 0.76 s
- NT-06: `scripts/fde-proof.sh --slice s1 --only NT-06`, log `.tmp/fde-proof/s1/NT-06.log`, duration 1.23 s
- NT-07: `scripts/fde-proof.sh --slice s1 --only NT-07`, log `.tmp/fde-proof/s1/NT-07.log`, duration 2.09 s
- NT-08: `scripts/fde-proof.sh --slice s1 --only NT-08`, log `.tmp/fde-proof/s1/NT-08.log`, duration 1.65 s
- NT-09: `scripts/fde-proof.sh --slice s1 --only NT-09`, log `.tmp/fde-proof/s1/NT-09.log`, duration 2.93 s
- NT-16: `scripts/fde-proof.sh --slice s1 --only NT-16`, log `.tmp/fde-proof/s1/NT-16.log`, duration 0.41 s

## Per claim area

| Area | Status | Tests | Limitation |
|---|---|---|---|
| initial semantic modelling | PROVEN | AT-01 | none recorded |
| provenance (external 'before') | PARTIAL | AT-04, NT-01, NT-16 | AT-04 PARTIAL: order check and digest equality pass in the producer tests; anchor time is the local clock only (AnchorTimeLocalOnly, OD-6), execution-side I-1 gate not wired (WP-20), so freeze-before-execution is verified after the fact, not enforced; NT-16 PARTIAL: the tamper/order check refuses as specified; the server push time is not read from the remote (AnchorTimeLocalOnly) and the anchor is a shared-OS-user tag without a protected namespace (RD-26, OD-6) |
| semantic exhaust | PARTIAL | AT-05, NT-02 | AT-05 PARTIAL: SEA-Forge half and Cognate half pass; emitter signatures arrive with WP-33 so the signature check cannot run; authenticity rests on source-record resolution and the independent-emitter rule |
| observed-model derivation | PARTIAL | AT-06, AT-08, NT-03, NT-03b | NT-03 PARTIAL: producer tests pass; a real re-run with a changed outcome was not run |
| divergence classification | PARTIAL | AT-07, NT-04, NT-06 | AT-07 PARTIAL: classes arose on the real episode set (see cell results); declared_unobserved and any unarising class NOT TESTED; postcondition expectations insufficient (DomainForge accepts only at precondition); NT-04 PARTIAL: both halves pass; the authorizing approval is a session-assurance approval (A3 and a human-reserved decision need S3); NT-06 PARTIAL: producer test passes; a silent source on the real cell was not run |
| governed revision | PARTIAL | AT-12, NT-05, NT-07, NT-08 | AT-12 PARTIAL: see log |
| subsequent operational capability | PARTIAL | AT-13 | AT-13 PARTIAL: see log |
| Horizon inspection (read adapters) | PARTIAL | AT-14, NT-09 | AT-14 PARTIAL: parity passes on the producer's fixtures (real RealityTrace CLI output and real case_records::append output); the cell's own raw records were not rendered |
| reproducibility and independent 'before' | PARTIAL | AT-18 | AT-18 PARTIAL: anchors verify (prereg by fde-anchor.sh; freeze tags by digest, local clock only), replay hashes and classes equal the recorded ones; re-execution of the program episodes was not repeated by the verifier |
| transfer | PLANNED | none | AT-16 is an S5 fixture-based test; not in the S1 gate |
| installation | PLANNED | none | clean-cell install from release artifacts is WP-57; not in the S1 gate |

## Open limits (STATUS.md 2026-11-14 ~20:30Z, verbatim)

- F1(b): execution-side I-1 gate NOT wired — item_execute/case_advance/agent_run.delegate carry no `n`/`episode_ref`; needs WP-20 contract. I-1 in S1 = settlement requires complete freeze + post-hoc order verification (AT-04/NT-16), NOT prevention of execution before freeze.
- F8: freeze path does not refuse a world_ref ≠ pinned world (needs code WORLD_NOT_PINNED in case-record-append; closed contract) → pin not consumed on the write path.
- F7 residual: no record links an agent run's actor to its delegator → delegate-then-settle by the delegator not caught; gateway hop only when a gateway is configured.
- F5: remote push time unobtainable with plain git → every real receipt is AnchorTimeLocalOnly (never ExternallyVerified) — ties to OD-6.
- F3 residual: SEA-Forge cannot recompute matrix rows; approver-bound matrix_id only.
- F2 residual: single-process case lock only.
- Anchor required for settlement only when SEA_FORGE_FDE_ANCHOR_SCRIPT is configured.

Further S1 limits stated by the gate: approval is at `session` assurance, not A3 (pre-S3 identity); pre-S3 identity (actors are bound by a trusted-issuer test driver, assurance `session`); anchoring is a scratch-repository tag with a local clock (OD-6); `--mutate` and `--replay` of the runner are NOT_RUN; human-reserved rows are PARTIAL.

## Episodes (predicted vs observed)

- **ep-1-control**: predicted result S, settled U by `independent-settler` (executed by ['program-executor', 'fresh-verifier']).
  - observed classes: [('confirmed', 'policy_true', 'freeze_before_move'), ('insufficient', 'policy_unknown', 'authorize_consequential_action'), ('observed_undeclared', 'structural_undeclared', 'undeclared:undeclared:1f728504dc8cbfa577071e39')]
  - predicted classes: [('Freeze recorded before the first run rec', 'confirmed', None), ('Authority decision exists before execute', 'confirmed', None), ("Declared command passes (executor's own ", 'confirmed', None), ('Fresh-context verification step performe', 'observed_undeclared', None), ('Settlement postcondition (S7.5e: indepen', 'insufficient', None), ('Residue capture after settlement (S6.3)', 'not_exercised', None)]
- **ep-2-inquiry**: predicted result F, settled F by `independent-settler` (executed by ['program-executor', 'fresh-verifier']).
  - observed classes: [('confirmed', 'policy_true', 'freeze_before_move'), ('insufficient', 'policy_unknown', 'authorize_consequential_action'), ('observed_undeclared', 'structural_undeclared', 'undeclared:undeclared:1f728504dc8cbfa577071e39')]
  - predicted classes: [('Hypothesis: round trip preserves Policy ', 'contradicted', None), ('Patch changes no product source (invaria', 'confirmed', None)]
- **ep-3-subsequent**: predicted result S, settled U by `independent-settler` (executed by ['program-executor', 'fresh-verifier']).
  - observed classes: [('confirmed', 'policy_true', 'freeze_before_move'), ('insufficient', 'policy_unknown', 'authorize_consequential_action'), ('insufficient', 'executor_only_evidence', 'verification_by_independent_role')]
  - predicted classes: [('Fresh-context verification step performe', 'observed_undeclared', 'confirmed'), ('Declared command passes (executor and in', 'confirmed', 'confirmed')]

**AT-13 capability delta:** predicted GAIN, result NO_GAIN, classes {'control_new': 'insufficient', 'control_old': 'absent', 'new': 'insufficient', 'old': 'absent'}, falsifier triggered ['new_world_not_confirmed'], R-58 supported: False.

## Gaps recorded by the run

- [AT-12,NT-07,AT-13] SEA-Forge freeze does not check the pinned world (WORLD_NOT_PINNED missing in case-record-append.v1): ep-3 froze under v2 by the caller's own declaration; the pin is not consumed on the write path
- [AT-04,NT-16] execution-side I-1 gate is not wired: item_execute/case_advance/agent_run.delegate carry no n or episode_ref; Cognate program.run ran under its own local policy with no SEA-Forge authority decision (authority.decision_id is unknown/not_required)
- [AT-05,AT-06] no in-process RealityTrace writer/delivery from SEA-Forge: interaction events travel as spool files (interaction-events/<case>.jsonl, Cognate NDJSON) into `observed ingest`; evidence.recorded rows are seeded by rt-seed because no producer writes them
- [AT-05] Cognate's ungoverned run events ride Observation.interactionEvent, not a CEP envelope: wrapped in the minimal execution_trace envelope by cognate_run.ts; only run.invocation is emitted (host outboxFor filter)
- [AT-04,AT-18] anchor: scratch git repository with a local bare remote (not the programme origin); time is local-only (OD-6); freeze anchors are not verifiable with fde-anchor.sh verify (refused there by design)
- [AT-12] the non-promotion variant (decline with retained divergence and rationale) was not run; approval is at session assurance, not A3
- [AT-13] the harness's verification step is emitted only by Cognate (the executor's own emitter): RealityTrace refuses to confirm executor-only evidence, so the declared step stays insufficient under v2
- [AT-07,NT-03,NT-06] no model element or mapping form expresses 'declared command passes' / the ep-2 hypothesis or a postcondition: no contradicted residual arises from ep-2; NT-03 real re-run with a changed value and NT-06 silent source were verified by producer tests only
- [AT-01,AT-12] SEA-Forge embeds DomainForge 0.21.2; the stack ran on the 3960326 build (0.21.2 line, world_ref 78ed... equals the frozen episodes.yaml world); DomainForge 0.22.0 gives world_ref e3c3... for the same sources
