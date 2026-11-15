# G. End-to-end evidence and qualification plan

The acceptance program for `Mission.md` and SOP v0.4 §11. Tests here are the proof column of deliverable C.
This file is pre-registered (WP-50): it is hash-pinned and committed before the first evaluated episode, and a
later change to a criterion creates a new version and a re-run.

## 1. Proof scenario: GodSpeed runs its own SOP

- **Declared intent:** `docs/sop/GS-FDE-SOP-001-v0.4.md` (hash-pinned).
- **Declared model:** `.sea/interaction/fde-sop/`, world `godspeed-fde-sop`, validated by the real DomainForge CLI.
- **Implemented behaviour:** Casework field-case verbs, Cognate runtime, Workbench assignments, RealityTrace.
- **Actual executions:** one Field Case, `FC-001 "Finish the Horizon enrolment authority chain"`, whose episodes
  are real engineering assignments from this plan (WP-22 parts: route-scope enforcement D3 and the saga D5). The
  work is real, the outcome matters, and it is bounded.
- This supersedes `Mission.md`'s Refund Desk preference. Refund Desk fixtures may be reused only as test fixtures.

## 2. Episodes and the four behaviour kinds

Every episode is a real assignment executed through SEA-Forge authorization, Cognate/Workbench and RealityTrace.
Inputs are fixed files under `scenarios/fde-sop/`; nothing is edited after the run.

| Episode | Assignment | Designed to exercise | Expected class |
|---|---|---|---|
| E1 | D3 route-scope table and enforcement, frozen reference, independent verifier | behaviour matching the declared procedure | `confirmed` on freeze, authorize, execute, settle, dispose |
| E2 | D5 saga crash-injection tests, where the scenario input directs the executor to begin work before the reference is frozen | a consequential contradiction; the system must refuse (I-1) and the attempt must be recorded | `contradicted` on "freeze before move", with the refusal as evidence |
| E3 | D5 reconciliation job, where the executor inserts a fresh-context review of its diff before claiming | legitimate behaviour the SOP does not declare as an operation | `observed_undeclared` |
| E4 | a documentation assignment run with the cost emitter disabled by scenario configuration | missing observation | `insufficient` for payment; episode result `U`, never `S` |
| E5 (after promotion) | a second review-bearing assignment under the revised model | subsequent episode | review step now `confirmed`; a missing review is now detectable as `contradicted` |

The legitimate-undeclared behaviour in E3 is the expected revision candidate (declare an independent-review
operation with its evidence obligation). If the evidence points elsewhere, the candidate follows the evidence and
this table is not edited; the difference is reported.

Contextual meaning (R-03): "settlement" in the engineering-practice context (independent result on a frozen
expectation) and in the client-outcome context (accepted business result) are modelled as distinct concepts in
two semantic packs; a test asserts they do not unify.

## 3. Acceptance tests (positive)

| ID | Proves | Method | Pass condition |
|---|---|---|---|
| AT-01 | model valid and identified | `domainforge validate` + `envelope --emit cep` on the model | exit 0; source hash, `world_ref`, compiler version recorded; rerun yields identical `world_ref` |
| AT-02 | model coverage and provenance | script over the translation map | every numbered SOP clause has a row with status and model element or `unknown` + reason; no row without a source |
| AT-03 | Field Case open, revise, strategy, activities | SFWP conformance + Horizon J1 | case created from template; §4.2 fields carry status and source; activities repeat and run in any order |
| AT-04 | freeze precedes execution | ledger and RealityTrace order check | `episode.frozen` and `question/threshold.declared` precede the first run event; digest matches |
| AT-05 | semantic exhaust complete | validate every event of E1 against `interaction_event/1` | 100 % of consequential events carry all required fields; source record resolvable |
| AT-06 | observed representation is computed | `observed derive` twice from the same ledger | byte-identical output and hash; every aggregate lists its occurrence ids; coverage section present |
| AT-07 | five classes | `observed compare` over E1–E4 | at least one residual in each class, each with rule, occurrences, elements, context |
| AT-08 | input sensitivity | derive and compare on the ledger with E3 excluded (`--until`) | the `observed_undeclared` residual disappears; hashes differ; no file edited |
| AT-09 | independent settlement | `case.episode.settle` by the verifier principal | result, reliability and independence recorded separately; settlement references RealityTrace evidence |
| AT-10 | status axes, dispositions, candidate comparison | conformance + J3, J5 | four axes independently queryable; each disposition produces the mapped lifecycle; a completed run with no settlement shows result absent, not success; blocked move shows reason and authorizer |
| AT-11 | gates | conformance + J8 | all four states reachable; evidence change marks reassess; nothing advances automatically |
| AT-12 | governed revision | diagnose → propose → candidate validate → semantic diff → regression → authorize → promote | each step recorded; proposer ≠ approver; registry shows both worlds; failed and inconclusive attempts retained |
| AT-13 | capability delta | E5 under the promoted world compared with E3 under the old one | a distinction is `confirmed` or detectable in E5 that was `observed_undeclared` or undetectable before; stated as "what the next observer can now do" |
| AT-14 | Horizon shows the source result | parity test: Horizon JSON for case, residuals, settlement, world versus CLI/SFWP for the same ids | equal after canonicalization; ids and digests embedded |
| AT-15 | resumption by a second operator | fresh agent or person given only Horizon Today and the handoff export | states next move, authorizer and settlement condition correctly; then executes it |
| AT-16 | transfer | tax pre-filing case (WP-60) | proof command passes with only model, template and scenario files changed; code diff of evidence mechanism is empty |
| AT-17 | assignment lifecycle | delegate, interrupt, resume, complete; budgets; Context Kernel bundle; Gauntlet/SWE_SEED records | episode reference unchanged across resume; spend per currency recorded or `unknown`; verification records bound to `episode_ref` |
| AT-18 | reproducibility | fresh verifier runs `scripts/fde-field-case-proof.sh` on a clean cell | regenerates comparison and ledger; verifier report agrees with claim ledger |
| AT-19 | journeys | Playwright J1–J10 against live services; agent-browser pass; fresh-operator walk-through of J1, J3, J5, J6 | all pass; stalls recorded as defects |
| AT-20 | installation | install from release artifacts on a clean cell, launch, run AT-18 | GO |

## 4. Negative tests (each must fail for the intended reason)

For every row, the mutation column names the change that must turn the test red; the mutation result is recorded.

| ID | Protected property (source) | Attack | Expected | Mutation |
|---|---|---|---|---|
| NT-01 | frozen expectation cannot be rewritten (M§7) | re-freeze same `episode_ref` with different criteria after observation; edit a RealityTrace threshold | refused; chain verify fails on tamper | remove I-3 check |
| NT-02 | no verified conclusion without model identity or evidence (M§7) | event without `world_ref` / `declaration_key` | `ingest.rejected` or unanchored; never `confirmed` | accept missing key |
| NT-03 | changed or missing record changes the result (M§7) | delete one occurrence | representation hash and comparison change | cache result by case id |
| NT-04 | undeclared occurrence ≠ authorized change (M§7) | E3 behaviour before promotion | `observed_undeclared`, world unchanged | auto-add to model |
| NT-05 | denied or unknown authority cannot promote (M§7) | promotion with denied, unavailable, expired, digest-mismatched decision | registry unchanged | treat unknown as approved |
| NT-06 | missing observation stays unresolved (M§7) | E4 | `insufficient`; result `U` | default missing cost to zero |
| NT-07 | settled evidence stays with its world (M§7) | read E1 evidence after promotion | still bound to old `world_ref` | retarget refs on promotion |
| NT-08 | failing regression blocks success (M§7) | candidate that breaks a prior confirmed element | promotion request refused | skip regression |
| NT-09 | Horizon equals programmatic result (M§7) | alter the adapter's classification mapping | parity test fails | — (the attack is the mutation) |
| NT-10 | no self-settlement (S§3.2) | executor principal calls `case.episode.settle` | refused | remove I-2 |
| NT-11 | hard gate not waivable (S§8.1) | set Allocate Authority `not_applicable` without approval | refused | allow rationale-only |
| NT-12 | agent cannot make a human-reserved decision (R-67) | agent session, and a request from the paired browser session without the agent header, submits a disposition and a promotion approval | refused without a valid A3-UV assertion bound to the operation digest | accept session cookie |
| NT-13 | exploratory cannot become a test (S§7.2) | relabel an exploratory episode as evaluated after the run | refused | allow kind change |
| NT-14 | skipped gate is NO-GO (R-81) | run readiness with a prerequisite missing | NO-GO | count skip as pass |
| NT-15 | unverified world refused at a boundary | event with a syntactically valid but unregistered `world_ref` | refused or escalated | syntax-only check |

A clean conforming control (E1 alone) runs first and must produce zero `contradicted` residuals.

## 5. Evidence bundle

`.agents/evidence/fde-field-case/<run-id>/`, following repository conventions:

1. `manifest.json`: repo commits and local changes, toolchain and runtime versions, environment, commands,
   test run ids and outcomes.
2. `semantic/`: SOP source and hash, initial model and `world_ref`, semantic-pack relationships, translation map
   and assumptions, candidate model, semantic diff, approval status.
3. `operational/`: frozen references with establishment timestamps, raw event records, authority decisions,
   RealityTrace ledger export with ids and envelopes, derived observed representations, residuals, the
   subsequent episode.
4. `verification/`: machine-readable AT/NT results, commands and exit codes, mutation results, independent
   verifier report, skipped or inconclusive checks with their effect on claims.
5. `claim-ledger.md`: one row per claim with implementation location, verification command, evidence reference,
   result (`PROVEN | PARTIAL | PLANNED | NOT TESTED`), scope and limitation. Eight required areas: initial
   semantic modelling, provenance, semantic exhaust, observed-model derivation, divergence classification,
   governed revision, subsequent operational capability, Horizon inspection. Plus: transfer, installation.

## 6. Independent confirmation

A fresh verifier with the frozen criteria (this file, deliverable C) and the artifacts, not the implementer's
narration: reruns AT-18, samples five claims end to end from ledger to raw record, attempts two negative tests
of its own choosing, and writes a verdict per claim. Independence is claimed only if the verifier ran in a fresh
context; using a different model CLI is preferred and recorded.

## 7. What one passing case does and does not establish

Establishes: the bounded mechanism works once, end to end, on real work. Does not establish: reliability under
variation, recovery under fault beyond the injected set, or transferable organizational capability. Those claims
stay `PARTIAL` until repeated cases, including the tax transfer with real professional acceptance, justify them.
