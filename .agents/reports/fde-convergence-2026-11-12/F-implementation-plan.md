# F. Dependency-ordered implementation plan (draft 3)

Implements the decisions in `H-egm-review-record.md` (`RD-01`…`RD-27`, including review round 2). Identifiers follow `_revision-ids.md`.
Requirements (`R-`) are in C, tests (`AT-`, `NT-`) in G, architecture decisions (`AD-`, `I-`) in E, journeys
(`J-`) in B. Draft 1 is preserved in `draft1/`.

## 0. Operating rules for every work package

**Branch safety (RD-18).** No owner checkout, worktree or branch is modified, rebased or deleted.
1. Before a repo is touched: `git tag fde-backup/<branch>/<yyyymmdd> <head>` for every local branch.
2. Work happens in new worktrees under `~/projects/_wt/fde-<repo>-<wp>`.
3. `fde/integration` is created from fetched `origin/main` and receives unfinished work by `git merge` or
   `git cherry-pick` of copies. A branch whose commits are already on main by patch-id
   (`git cherry origin/main <branch>` shows only `-`) is recorded as landed and left alone.
4. Where two branches carry the same change, the one whose patch-ids are a superset wins; otherwise the owner
   decides and both are kept.
5. Work-package branches are `fde/<wp>-<slug>`, cut from `fde/integration`, merged back by pull request.

**Lanes (RD-24).** One serial lane per repository; parallelism only across repositories. Order within each lane:

| Repository | Lane order |
|---|---|
| SEA-Forge | 00c → 09 → 13 → 14 → 18 → 19 → 20 → 21 → 22 → 31 → 33 → 53 |
| Cognate | 00c → 15 → 30 → 23 → 33 |
| Horizon | 00c → 19 → 32 → 40 → 41 |
| Workbench | 00c → 23 → 34 → 42 → 44 |
| RealityTrace | 00c → 10 → 16 → 17 → 18 → 22 → 33 → 43 |
| DomainForge | 00c → 10 → 12 |
| CEP | 00c → 11 |

A package that spans repositories takes its turn in each lane. Shared interfaces are fixed in the contracts
packages (WP-11, WP-20) before the code that uses them.

**Agents.** Sonnet implementers on file-level briefs written when a package is dispatched (files, functions,
test names, acceptance command). Haiku for recon and mechanical edits. The lead model writes briefs, reviews
diffs and decides. No Opus or Fable subagents.

**Host limits.** One heavy build or test at a time; free memory checked first (2.5 GB, or 3.5 GB for
full-stack gates); Postgres through pg0, never Docker; `flock -o` around cargo; a clean cell is a fresh
directory tree with its own cell root and state directories on this host, built from release artifacts.

**Repository rules.** Each schema, interface, dependency or architecture change carries its spec and ADR update
in the same pull request. SEA-Forge status goes through `CURRENT_STATUS.yaml` and `just status-check`, run from
a clean worktree. Other repos follow their own `AGENTS.md`.

**Done means.** The acceptance command exits 0 on `fde/integration`; each negative test owned by the package
fails when its hash-pinned mutation patch is applied; the repo's own gates pass; the evidence path exists. A
skipped prerequisite is not green.

**Test ownership.** G §5 and §6 are authoritative for which work package owns each acceptance and negative
test (G lists 20 acceptance and 62 negative tests). The acceptance lines below name the principal ones; a
package is not done until every test G assigns to it passes, including those G marks † (assigned there, not
repeated here).

**Review.** Packages touching identity, authority, sandbox, IPC or settlement get the independent hardening
review (different model CLI) before merge. A change to an `AD` or `RD` gets an EGM review.

**Evidence root.** `sea-rs/.agents/evidence/fde-field-case/<run-id>/`. Baseline evidence:
`.agents/reports/fde-convergence-2026-11-12/evidence/baseline/`.

**Recovery, generic.** A failed package leaves its branch, its evidence and a `STATUS.md` entry; nothing that
depends on it starts. Contract changes are additive and versioned, so a consumer can stay on the prior version.
Re-running a package starts from its branch, never from a reset.

**Escalate to the owner.** Changes to frozen specifications or identifier grammars beyond those listed in H §4;
anything touching credentials; an irreversible external effect; ties between owner branches; the open
decisions OD-1…OD-5.

## 1. Slices and their gates

```
S0 baseline ─► S1 walking skeleton ─► S2 Field Case records ─► S4 operator experience ─► S5 proof, transfer,
                     │                         │                        ▲                  operate, release
                     └────────► S3 identity and authority ──────────────┘
```

| Slice | Ends when | Claim level it supports |
|---|---|---|
| S0 | every repo has a measured baseline, backup tags, and an `fde/integration` branch whose gates match or beat the baseline | none (precondition) |
| S1 | `scripts/fde-proof.sh --slice S1` is green on a clean cell: three `program` episodes in one world (conforming control, falsifiable inquiry, subsequent episode after one revision), with derive, compare and Horizon read parity. Gate tests: AT-01, 04, 05, 06, 07, 08, 12, 13, 14, 18; NT-01…NT-09, NT-16 (RD-20) | "the Mission §1–7 path exists and runs once on program execution"; not evidence about organisational behaviour; approvals at assurance `session` (RD-21) |
| S2 | Field Case records, gates, dispositions, lifecycle, continuation and the promotion saga pass their conformance and negative tests | SOP §4, §7–§10 record model |
| S3 | D1–D6 closed, A3 confirmation live on the paired phone, emitter signatures verified; production enrolment no longer refuses | human-reserved and emitter-authenticity rows |
| S4 | five proof-critical journeys pass against live services; seam works | Mission §6; operator capabilities |
| S5 | full FC-001 run, negative suite, bundle, independent verification, operate package, transfer, release and clean-cell install; `local_operator` GO with the FDE gates | completion |

S3 packages start when their repository lane is free (table in §0): Cognate's WP-30 after WP-15, SEA-Forge's
WP-31 after WP-21. S4 starts against the contract fixtures once WP-20 is reviewed. Before S3, the A3 check is
off, approvals use the existing approval path with distinct actor identities, and every row that depends on a
human-reserved decision is PARTIAL (RD-21).

## 2. Work packages

Columns: prerequisite · change · acceptance command · evidence · recovery.

### S0 Baseline

**WP-00a Measured baseline** (all repos). *Prereq:* none (fetch is done: `evidence/baseline/heads-after-fetch.md`).
*Change:* none. Run each repo's documented gate once on `origin/main` in a scratch worktree, serially, recording
command, exit code, pass/fail/skip counts, duration, toolchain versions; run `sea-rs/scripts/e2e-world-loop.sh`
and record its first-ever output. *Acceptance:* `evidence/baseline/<repo>.md` exists for each of the twelve
repos, plus `world-loop.log`. *Recovery:* a red baseline is recorded as a prerequisite defect for the owning
package; it is not fixed here.

**WP-00b Backup** (all repos). *Prereq:* 00a. *Change:* backup tags on every local branch head; push
`horizon-desk-gaps` and Workbench `feat/horizon-connect` to origin as backups after a secret scan of the
unpushed range (OD-4); commit the 37 Workbench PNG re-captures to `evidence/ui-recapture-20261112`.
*Acceptance:* `git tag -l 'fde-backup/*'` count equals the branch count per repo; remote branches exist.
*Recovery:* a secret-scan hit stops the push and is reported.

**WP-00c Integration branches** (each repo, one at a time). *Prereq:* 00b. *Change, per repo:*
- SEA-Forge: main + PR #91 (`feat/horizon-desk-support`, fast-forward) + `fix/settlement-orphan-guard` and
  `test/settlement-class-fail-closed` if not landed by patch-id. Already on main and left alone: `t1/*`, the
  settlement-claim `-main` branch, `fix/server-ambiguous-declare-keeps-lease`. Every other branch gets a ledger
  line: landed / superseded (with the superseding commit) / kept.
- Horizon: start from main; merge `horizon-desk-gaps`; then for each of the nine branches with patch-unique
  commits, diff content against the result and port what is missing (including the untracked enforcement-probe
  test from the `stage0b-containment-core` worktree, copied, not moved).
- Cognate: main + PR #76 + `horizon/shared-identity` merged (not rebased).
- Workbench: the `origin` desktop line + the seven patch-unique `feat/horizon-connect` commits, on Cognate 0.9.x.
- RealityTrace: main (already contains the packaging and ingest merges) + the eight local commits if not landed.
- CEP, Context Kernel: main. Gauntlet: PR #7 line. GodSpeed-Agent: `migration/cep-world-required-evidence`.
  DomainForge: main; add its head to `gsr-heads`.
*Acceptance:* each repo's gate on `fde/integration` is no worse than its 00a baseline;
`evidence/baseline/branch-ledger-<repo>.md` complete. *Recovery:* a branch that does not merge cleanly stays out,
is listed as kept, and the gap is named.

### S1 Walking skeleton

**WP-09 Pre-registration v1** (SEA-Forge). *Prereq:* 00c. *Change:* freeze G's S1 subset (tests, falsifiers,
`program`-executor inputs, predicted classes) under `.agents/preregistrations/fde-s1/`; hash it; create
`scripts/fde-anchor.sh` (publish and verify anchors) and the `scripts/fde-proof.sh` skeleton (`--slice`,
`--only`, `--replay`, `--mutate`); anchor the pre-registration (RD-04, with the limits in RD-26).
*Acceptance:* `scripts/fde-anchor.sh verify fde-s1` exits 0. *Recovery:* a later criterion change is a
new version and a re-run.

**WP-10 Spike** (DomainForge library, RealityTrace scratch crate; no merge). *Prereq:* 00c. *Change:* ten-element
model; generated fact set; call DomainForge's policy evaluation through the library. *Pass thresholds:* per-policy
`true/false/unknown` obtainable with bound instances for (a) empty fact set, (b) one occurrence, (c) an
obligation that is unknown, (d) an operation-bound policy. *Acceptance:* `evidence/spike/wp-10.md` states which
of a–d pass. *Recovery:* any failing case becomes a stated requirement of WP-12; if the library cannot be made
to evaluate operation-bound policies at all, AD-5's policy-evaluable part is reduced to non-operation policies
and the affected SOP clauses classify as `insufficient / not_evaluable`, recorded as a limitation (no second
evaluator is built).

**WP-11 Contracts package S1** (CEP as schema home; consumers vendor by digest). *Prereq:* 00c. *Change:* JSON
Schemas, typed field tables, valid and invalid fixtures, error-code tables for: the `godspeed.interaction_event`
envelope extension; the CEP-0008 hash rule; the frozen reference (as a DomainForge verification contract) and
the `episode_ref` digest input; the comparison profile and coverage manifest; the observed-representation
document; the residual payload; the anchor tag format; the `evaluate` facts-file schema (authored in WP-12); the `outcome.status` enum;
`case.record.append` request/response/errors with kinds `episode.frozen` and `episode.settled` (RD-23). Emitters and validators are generated from the schemas. *Acceptance:*
`cep validate all` green; contract gate fails when a vendored copy's digest differs. *Recovery:* additive.

**WP-12 DomainForge `evaluate`** (DomainForge). *Prereq:* 10. *Change:* `domainforge evaluate --model --facts
--format json` per RD-08, with golden tests for a–d. *Acceptance:* `cargo test -p domainforge-core evaluate`.
*Recovery:* `validate` is untouched; the new command is additive.

**WP-13 SOP and model** (SEA-Forge). *Prereq:* 00c. *Change:* promote SOP v0.4.1 to `docs/sop/`; author
`.sea/interaction/fde-sop/` with translation map (one row per numbered clause), assumptions-and-unknowns,
validation output; check against the existing `limitations.md`; two semantic packs for the contextual meanings
of "settlement"; anchor the model digest. *Acceptance:* AT-01, AT-02. *Recovery:* an inexpressible clause is
recorded `unknown` with the grammar limitation.

**WP-14 Case keeper, episode loop** (SEA-Forge). *Prereq:* 11, 13. *Change:* `case.record.append` and schema
registry; kinds `episode.frozen`, `episode.settled`; two-phase freeze (RD-12) including the RealityTrace
contract/plan/question/threshold writes and the external anchor; I-1 on execution identity; I-3 on
`(case_id, plan_item_id, n)`; signed interaction-event emission for ledger events and authority decisions
through a durable outbox; Go client and TypeScript contracts regenerated; golden frames. Identities are SEA-Forge actor and delegation identities at this stage (RD-21). *Acceptance:*
`cargo test -p sea-forge-server --test conformance_field_case`; AT-04; NT-01, NT-16 in the S1 gate, plus the
other tests G assigns to this package.
*Recovery:* emitter failure never blocks the case write; an incomplete freeze is reported by the reconciler and
cannot be delegated against.

**WP-15 Runtime events and program executor** (Cognate). *Prereq:* 11. *Change:* the extension on the existing
CEP outbox for run and invocation events (world, semantic reference, case context, authority decision,
provenance flag); a `program` executor capability that applies a hash-pinned patch and runs a declared command.
*Acceptance:* Cognate package tests; AT-05 (runtime part). *Recovery:* fields are optional for non-case runs.

**WP-16 RealityTrace ingest and derive** (RealityTrace). *Prereq:* 11. *Change:* envelope-extension ingest with
signature check hook, source-record resolution, `ingest.rejected` for unanchorable events; crate `sxr-observed`;
`observed derive --world --scope --episode --kind --until`. *Acceptance:* AT-06, AT-08; NT-02, NT-03 (both
modes). *Recovery:* read-only over the ledger; no existing record kind changes.

**WP-17 RealityTrace compare** (RealityTrace). *Prereq:* 12, 16. *Change:* `observed compare`; three-way
classification split; comparison profile; residual records with class, basis, rule, occurrences, elements,
context, world stamp, and an episode-versus-structural field. *Acceptance:* AT-07; NT-04, NT-06, NT-20.
*Recovery:* —; compare never writes outside its own record kinds.

**WP-18 Minimal governed revision** (RealityTrace, then SEA-Forge). *Prereq:* 17, 14. *Change:* proposal with
held-out rule; regression transition matrix command; durable world pin record with promote / re-pin over the
existing approval-gated `semantic_change` transition, proposer ≠ approver; subsequent episode under the new
world. *Acceptance:* AT-12, AT-13 (S1 form); NT-05, NT-07, NT-08, NT-21. *Recovery:* re-pin the previous world
through the same gated transition.

**WP-19 Horizon read path and S1 gate** (Horizon, then SEA-Forge). *Prereq:* 14, 17, 18. *Change:* read adapters
for case records and RealityTrace structure feeding the debugger sources; parity test against rendered text;
completion of `scripts/fde-proof.sh` (skeleton from WP-09) and the S1 claim-ledger builder; clean-cell run. *Acceptance:*
`scripts/fde-proof.sh --slice S1` on a clean cell; AT-14, AT-18 (S1 form); NT-09. *Recovery:* adapters are
read-only; an absent source renders "not connected".

### S2 Field Case records

**WP-20 Contracts package S2** (SEA-Forge as schema home for case contracts). *Prereq:* S1 gate. *Change:* the
field-case document schema (generated from the model); kinds `field_case.revised`, `strategy.recorded`,
`gate.assessed`, `disposition.recorded`; `case.field.get`, `case.continuation`; the reserved-operation interface
(operation → class → evidence, per RD-06) and the A3 evidence format; the assignment payload and id.
*Acceptance:* schema and frame fixtures validate; reviewed before WP-21, WP-31, WP-41 start.

**WP-21 Field Case records** (SEA-Forge). *Prereq:* 20. *Change:* document, strategy, gates with the hard-gate
rule, dispositions, lifecycle function with golden tests, suspended refusals, no close without disposition,
continuation read model, template `fde.field_case@1` with a test that checks it against the model, I-2 on the
SEA-Forge delegation chain (re-proven on the shared identity contract in S3; no dependency on WP-30). *Acceptance:* AT-03, AT-09, AT-10, AT-11, AT-15 (programmatic part); NT-10, NT-11, NT-22,
NT-23. *Recovery:* existing case verbs untouched; regression suite must stay green.

**WP-22 Promotion saga and world stamping** (SEA-Forge, RealityTrace). *Prereq:* 18, 21. *Change:* saga keyed
`(proposal_id, candidate_world_ref)`; reconciler for partial freeze, orphan records, partial promotion; key
mapping from the semantic diff; in-flight cases keep their pin. *Acceptance:* crash-injection tests after every
saga step; NT-07 (re-pin variant). *Recovery:* uncertain outcomes fail closed and surface in the reconciler.

**WP-23 Assignment binding** (Cognate, then Workbench). *Prereq:* 20. *Change:* sessions opened from an
assignment carry the assignment id `assign:<caseId>:<stepId>:<n>:<k>` (episode counter `n`, delegation
instance `k`), episode reference, brief, budget; spend per currency; interruption and resume
events. *Acceptance:* AT-17 (lifecycle part). *Recovery:* Workbench without a case behaves as today.

### S3 Identity and authority (each package starts when its lane is free)

**WP-30 Identity package** (Cognate). *Change:* `@godspeedai/cognate-identity`; D2 evidence-verified issuance
and `pending_activation`; session scope claims; operator recording for bootstrap and recovery. *Acceptance:*
package tests: refusal on no evidence, wrong digest, missing decision, replayed proof; pending credential cannot
sign in. *Recovery:* the package has no consumers yet.

**WP-31 Authority bridge** (SEA-Forge; after WP-21 in that lane, interface from WP-20). *Change:* D1 enrolment
decision API bound to operation digest and expiry; D4/D6 controller classes; reserved-operation check that
validates A3 evidence. *Acceptance:* conformance tests with a real signed decision; refusal on denied,
unavailable, expired, digest mismatch; NT-12, NT-24. *Recovery:* disabled by configuration until Horizon wires it.

**WP-32 Horizon authority** (Horizon). *Prereq:* 30, 31. *Change:* D3 route-scope table; D5 saga and
reconciliation; real P3 authority client in production wiring; first-controller bootstrap on the local console;
A3 confirmation on the paired phone for reserved decisions; request ids and Reassign control (gaps A, B);
containment and enrolment as two pull requests. *Acceptance:* Horizon suite; crash after any step leaves no
usable credential; NE-73 retired only with the real client; NT-12, NT-24 end to end. *Recovery:* production
keeps refusing enrolment until 30 and 31 pass.

**WP-33 Emitter authenticity** (Cognate, SEA-Forge, RealityTrace). *Prereq:* 30. *Change:* events signed with
workload identities; RealityTrace verifies. (The independent-emitter rule itself ships in WP-17.)
*Acceptance:* NT-19; NT-20 re-run with signed events. *Recovery:* unsigned events are rejected, never downgraded.

**WP-34 Consumers** (Workbench, Gauntlet). *Prereq:* 30. *Change:* Workbench uses the identity package and
deletes its duplicate WebAuthn/CBOR; Gauntlet binding record before launch. *Acceptance:* existing remote and
pairing tests on the shared package; Gauntlet launch refused without a binding.

### S4 Operator experience

**WP-40 Debugger sources and extensions** (Horizon). *Prereq:* 19, 20. *Change:* move-availability answer,
regression-consequence source for `counterfactual`, re-derivation for `replay`, semantic diff source for `diff`.
*Acceptance:* debugger tests and teeth suite; AT-14 full.

**WP-41 Journeys** (Horizon). *Prereq:* 40, 21, 32 for reserved steps. *Change:* J1, J3, J5, J6, J7 first, then
J2, J4, J8, J9, J10, on the v2 layout. *Acceptance:* AT-19: Playwright against live services plus agent-browser
pass per journey; fresh-operator walk-through of the five.

**WP-42 Workbench seam** (Workbench). *Prereq:* 23. *Acceptance:* AT-19 (seam rows).

**WP-43 Verification bridge** (Gauntlet, SWE_SEED, RealityTrace). *Prereq:* 16. *Change:* Gauntlet verification
records and SWE_SEED `ProofCompleted` as observations bound to the episode; Gauntlet reference digest equals
the frozen reference digest. *Acceptance:* AT-17 (verification part). *Recovery:* a missing verifier leaves the
episode unresolved.

**WP-44 Context bundle** (Context Kernel, Workbench). *Prereq:* 23. *Acceptance:* AT-17 (context part).

### S5 Proof, transfer, operate, release

**WP-50 Pre-registration v2.** Full FC-001 episodes, predictions, falsifiers; anchored.
**WP-51 FC-001 full run.** *Prereq:* S2–S4 gates. Episodes run one at a time; each ends in its own evidence
directory; a failed episode stops the run and is kept. Re-running an episode allocates `n+1`.
**WP-52 Negative suite.** Each test already exists with its owning package; this collects them into one run and
has the verifier re-apply every mutation patch from a clean checkout.
**WP-53 Bundle, ledger, command.** AT-18; owner repositories' proof ledgers updated.
**WP-54 Independent verification** (RD-17). Disagreements are defects; unresolved means not PROVEN.
**WP-55 Operate.** Launch topology and ports for the whole stack; configuration reference; migration of existing
cases and ledgers; integration health (outbox depth, rejection rate, projection lag) surfaced in SEA-Forge
readiness; retention and backup for the CAS representations, outboxes and world pin record, added to the
existing backup/restore and upgrade gates.
**WP-56 Tax transfer.** Model, template and scenarios only; AT-16; reported PARTIAL and fixture-based.
**WP-57 Release.** FDE gates added to `local_operator` (which must stay GO); releases per repo; install from
release artifacts on a clean cell; AT-20, NT-14; GO report with exact versions.

## 3. Critical path and first batch

Critical path: 00a → 00b → 00c → 11 → 13 → 14 → 16 → 17 → 18 → 19 (S1 gate) → 20 → 21 → 22 → 41 → 50 → 51 →
52 → 53 → 54 → 57. WP-10 → WP-12 feeds WP-17. S3 (30 → 31 → 32) feeds WP-41's reserved steps and WP-51.

Riskiest three: WP-12/WP-17 (classification depends on a DomainForge capability that does not exist yet);
WP-14/WP-21 (the largest change to a governed kernel); WP-31/WP-32 (cross-service authority with a human
authenticator in the loop).

First batch: WP-00a, then WP-00b; then in parallel across repositories WP-00c, and as soon as DomainForge and
RealityTrace integration branches exist, WP-10.

## 4. Risks carried into implementation

| Risk | Early check | If it materializes |
|---|---|---|
| DomainForge cannot evaluate operation-bound policies | WP-10 | affected clauses classify `insufficient / not_evaluable`; reported as a limitation; no second evaluator |
| SOP obligations exceed what `policy` can express | WP-13 translation map counts `unknown` | same |
| Integration branches do not reach baseline | WP-00c per repo | the offending branch stays out and is listed |
| Host memory makes full gates flaky | WP-00a durations | serialize; use the readiness harness's RAM waits |
| No real authenticator for human-reserved rows | S1 clean-cell run asks OD-5 early | rows stay PARTIAL; virtual authenticator covers regression only |
| A behaviour kind does not occur | WP-09, WP-50 predictions | row NOT TESTED; the claim is not made |
| Anchor independence on a single-operator host | WP-09 states the limit; OD-6 asks for a dedicated credential and protected tag namespace | anchoring rows PARTIAL (RD-26) |
| The integrated gate never goes green | each slice has its own gate and claim level | the last green slice is the delivered verified state, reported with the precise blocker |
