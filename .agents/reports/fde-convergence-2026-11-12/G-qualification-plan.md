# G. End-to-end evidence and qualification plan (draft 2)

The acceptance programme for `Mission.md` and SOP v0.4.1 (`S§n.m`). Implements `H-egm-review-record.md`
(`RD-n`), identifiers from `_revision-ids.md`, work packages (WP) from F draft 2. Tests here are the proof column
of deliverable C (`R-n`). Draft 1 is in `draft1/`. Pre-registration: the S1 subset is frozen and anchored by WP-09,
the full set by WP-50. A later criterion change makes a new version and a re-run; it never edits a frozen file.

Commands: every test runs as `scripts/fde-proof.sh --only <ID>`; a slice as `scripts/fde-proof.sh --slice S<n>`;
replay as `--replay <run-id>`; mutation as `--mutate <NT-id>` (applies `mut/<NT-id>.patch`, expects red with the
named code); anchor check as `scripts/fde-anchor.sh verify <kind>/<id>`. `scripts/fde-anchor.sh` and the `scripts/fde-proof.sh` skeleton are created by WP-09 (RD-24); WP-19 completes
`fde-proof.sh` (S1 form) and WP-53 extends it. Mutation patches live in `.agents/preregistrations/fde/mut/`, listed with sha256 in
`mut/SHA256SUMS` inside the anchored pre-registration.

## 1. Proof scenario (RD-02)

- **Declared intent:** `docs/sop/GS-FDE-SOP-001-v0.4.1.md`, hash-pinned. **Declared model:** `.sea/interaction/fde-sop/`,
  world `godspeed-fde-sop`, validated by the real DomainForge CLI, digest anchored (WP-13).
- **Field Case FC-001 "Close recorded SEA-Forge and DomainForge debt items":** small, real, verifiable by a
  deterministic command, and not on this plan's critical path (no episode builds a package the proof needs).
  Supersedes `Mission.md`'s Refund Desk preference by the owner's ruling; Refund Desk fixtures may be test fixtures only.
- **Candidate episodes.** E1 derive `ReadinessView.overall = stale` (SEA-Forge debt); E2 `events.get_range` tail
  read; E3 inquiry with frozen hypothesis "DomainForge KG round trip preserves Policy and EntityInstance triples"
  (debt D-doc-egm-3 suggests it will be falsified); E4 the subsequent episode after a promoted revision (AT-13).
  Horizon gap A (request ids for route/reassign/withdraw) is not a candidate: WP-32 builds it (RD-20).
- **S1 episodes (RD-20).** S1 runs three `program` episodes in one world: a conforming control (E1 or E2); a
  falsifiable inquiry (E3) whose frozen hypothesis may fail when run (outcome unknown at freeze, deterministic on
  replay); and, after one revision, a subsequent episode (E4). The S1 revision candidate is the executor harness's
  own fresh-context verification step, which really happens and is not declared in model v1.
- **Selection rule.** Final selection is pre-registered after confirming each item still exists on `fde/integration`:
  WP-09 for the S1 subset, WP-50 for the full case. A candidate that no longer exists is dropped and the drop is
  recorded; none is invented to fill a kind.
- **Executor provenance** `program | live | injected_fault` is recorded on every occurrence and copied into the claim
  ledger. `program`: deterministic executor applying a hash-pinned patch and running the declared command through the
  real SEA-Forge / Cognate / RealityTrace path (S§3.1 permits program executors). `live`: an agent or person working
  undirected. `injected_fault`: negative suite only; never supports an execution claim (NT-26).

## 2. Behaviour kinds (RD-03): arise, are predicted, are reported

No episode is instructed to misbehave. Pre-registration records a *prediction* of the class per episode; a wrong
prediction is reported, never edited. A kind that does not occur through `program` or `live` execution is NOT TESTED.

| Kind | How it arises through real execution | Provenance allowed | Predicted (frozen, reported if wrong) | If it does not occur |
|---|---|---|---|---|
| Conforming | a `program` episode following the procedure (E1 or E2) | `program`, `live` | `confirmed` on freeze, authorize, execute, settle, dispose | NOT TESTED (no conforming control, S1 gate red) |
| Contradicted expectation | falsifiable inquiry E3 settles **F** on its frozen hypothesis; structurally, a procedure obligation whose trigger occurred and whose required record contradicts it | `program`, `live` | E3 result F; class `contradicted` on the hypothesis | NOT TESTED |
| Legitimate undeclared | fresh-context review before claim: the programme's standing practice, not declared in the SOP as an operation. In S1 it is the executor harness's own fresh-context verification step in the `program` episodes (it arises by construction of the harness, labelled `program`); on `live` execution it is the programme's practice | `live`; `program` only where the hash-pinned program contains the step | `observed_undeclared` | NOT TESTED; AT-12 and AT-13 then NOT TESTED |
| Missing observation | a real instrumentation gap named in the frozen reference as an evidence source (e.g. token cost of `program` executors). The episode result depends on what the frozen reference says that evidence was for | `program`, `live` | `insufficient`; episode result `U` only if the reference names that evidence as required for settlement | NOT TESTED |
| Unmeasured payment | a currency with no meter, no evidence named in the reference | any | payment `unknown`; episode result unaffected (S§5.3) | not a divergence class; reported only |
| Declared-unobserved | obligation whose trigger was observed **inside the frozen coverage window** and whose required record is absent (e.g. residue capture S§6.3 after a settlement with no record) | `program`, `live` | `declared_unobserved` only with manifest coverage and a closed window | NOT TESTED |
| Declared, untriggered | any declared element whose trigger never occurred | any | `not exercised`; not a divergence class; never `declared_unobserved` (NT-29) | n/a |
| Fault | `injected_fault` records in the negative suite | `injected_fault` | the named refusal code | n/a |

Distinction held by the frozen reference: an *unmeasured payment* is `unknown` (S§5.3); a *missing observation named
as an evidence source* in the frozen reference blocks settlement and yields `insufficient`/`U` (S§7.5).

**S1 claim (RD-20).** The S1 claim is exactly: "the Mission §1–7 path exists and runs once on program execution." It
is not evidence about organisational behaviour, and the claim ledger says so. In S1, AT-07 requires `confirmed` plus
one of `contradicted` / `observed_undeclared` to arise; the other classes may be NOT TESTED without failing the gate.
The five behaviour kinds arising on `live` execution are an S5 requirement.

**Contextual meaning (R-03, R-91):** "settlement" in the engineering-practice context (independent result on a frozen
expectation) and in the client-outcome context (accepted business result) are two concepts in two semantic packs; AT-01
asserts they do not unify and that the translation map keeps both.

## 3. Principals (RD-06, H §2)

| Role (SOP §3.1) | Principal in FC-001 | Identity |
|---|---|---|
| Sponsor, disposition authority, world-promotion approver | the owner | human operator; A3 on the paired phone |
| FDE / case lead | lead agent session | agent principal under the owner's delegation |
| Criterion author | lead agent, before execution; frozen and externally anchored | same, recorded at freeze |
| Executors | `program` executors; Sonnet implementer agents | distinct execution identities |
| Independent verifier | fresh-context agent on a different model CLI, making its own observation | separate agent principal, no shared context |
| Domain and policy custodian | the owner | human operator |

Before S3 ships, principals are SEA-Forge actor identities under its existing identity gate, and claims are scoped
accordingly. **Staged enforcement (RD-21):** the reserved-operation A3 check arrives with WP-31 and is off before it;
in S1 and S2 approvals use SEA-Forge's existing approval path with distinct actor identities, recorded with assurance
`session`. Rows that depend on A3 on a separate surface or on any human-reserved decision (R-67, R-110, R-111, human
settlement and disposition, world promotion approval) stay **PARTIAL until S3 (WP-31 and WP-32 exist)** and a real
authenticator is provided (OD-5). Emitter signatures arrive with WP-33; before that, authenticity is source-record
resolution plus the independent-emitter rule (NT-20).
**Virtual-authenticator rule:** the Chromium virtual authenticator is valid for automated regression only. A
human-reserved decision counts as PROVEN only with evidence from a real authenticator on the paired phone; the ledger
builder refuses otherwise (NT-50).

## 4. External anchoring of "before" (RD-04)

- **Anchored:** model digest at registration (`fde-anchor/model/<world_ref>`); each episode freeze digest
  (`fde-anchor/freeze/<episode_ref>`); each pre-registration digest (`fde-anchor/prereg/<id>`).
- **By whom:** the case-keeper service identity publishes a signed annotated tag and pushes it to the GitHub remote of
  the SEA-Forge repository. No executor credential has push rights to `fde-anchor/*`.
- **Verifier check** (`scripts/fde-anchor.sh verify <kind>/<id>`): recompute the digest from the artifact; tag object
  contains that digest; tag signature valid for the case-keeper key; tagger is not an executor identity; the
  **server-side push time** (GitHub API, not tag metadata) predates the first run record of the episode or world; the
  verifier's **own receipt**, issued at freeze and stored outside executor reach, matches. A hash chain alone never
  satisfies "before" (NT-16, NT-17, NT-18, NT-48, NT-60).
- **Limits (RD-26).** On a single-operator host, executors and the case keeper share an OS user, so the anchor proves
  "not rewritten after the server-recorded push time", not "could not have been written by an executor". A dedicated
  anchor credential with a protected `fde-anchor/*` tag namespace (no delete, no force) is requested from the owner
  (OD-6); without it, anchoring rows are PARTIAL with this limit stated. Verification needs network access; offline it
  verifies the recorded verifier receipt only and reports PARTIAL.

## 5. Acceptance tests (positive)

WP column: this table's owner column is authoritative for test ownership (F cites it).

| ID | Proves | Exact method | Command | Pass condition | WP | Slice |
|---|---|---|---|---|---|---|
| AT-01 | model valid, identified, contexts kept distinct | `domainforge validate` and `envelope --emit cep` on the model; check the two settlement packs do not unify | `--only AT-01` | exit 0; source hash, `world_ref`, compiler version recorded; rerun gives identical `world_ref`; packs distinct | WP-13 | S1 |
| AT-02 | model coverage and provenance | script over the translation map | `--only AT-02` | every numbered SOP clause has a row with status and model element, or `unknown` plus reason; no row without a source; every assertion tagged declared/observed/inferred/proposed/unknown | WP-13 | S2 |
| AT-03 | Field Case open, revise, strategy, activities | conformance suite (programmatic); Horizon J1 in S4 | `--only AT-03` | case from template; S§4.2 fields carry status and source; activities repeat via the `repetition` marker, omit, any order; stated vs adopted problem recorded | WP-21 | S2 |
| AT-04 | freeze precedes execution, externally | order check on case ledger and RealityTrace; `fde-anchor.sh verify freeze/<ref>`; variants: within-reference correction does not start a new version (R-97); incomplete freeze cannot be delegated against | `--only AT-04` | contract/plan/question/threshold and `episode.frozen` exist and precede the first run record; digest equals the anchored tag; anchor push time earlier than first run; verifier receipt matches | WP-14 | S1 |
| AT-05 | semantic exhaust complete and authentic | validate every consequential event against the extension schema; resolve `source.record_id` and `authority.decision_id` at owners; check signature; duplicate replay variant | `--only AT-05` | 100 % carry all required fields; every `source.record_id` resolves; correlation is a structured field; replayed duplicates do not change counts | WP-15, WP-14 | S1 |
| AT-06 | observed representation computed | `observed derive --episode --kind` twice from the same ledger | `--only AT-06` | byte-identical output and hash; every aggregate lists its occurrence ids (verified against the ledger); coverage section and profile present | WP-16 | S1 |
| AT-07 | five classes | `observed compare` over the frozen episode set with the comparison profile | `--only AT-07` | each class that arises (§2) carries rule, occurrences, elements, context, basis, world stamp; in S1 `confirmed` plus one of `contradicted` / `observed_undeclared` must arise and the other classes are reported NOT TESTED without failing the gate; all five on `live` execution is an S5 requirement; zero silent omissions; episode and structural residuals in separate lists | WP-17 | S1 |
| AT-08 | input sensitivity | derive and compare on a ledger variant produced by a real re-run with one input changed (`--until` excludes later records) | `--only AT-08` | the affected residual changes class or disappears; representation hash differs; no file edited | WP-16 | S1 |
| AT-09 | independent settlement with own observation | verifier principal runs the declared command itself in a clean cell, then `case.record.append episode.settled` | `--only AT-09` | settlement cites the verifier's own observation record (not executor output); result, reliability and independence stored separately; verifier ≠ executor by delegation chain (I-2); human settlement needs A3 (PARTIAL pre-S3); S1 form uses actor ids | WP-21 (S1 form WP-14) | S2 |
| AT-10 | status axes, dispositions, candidate comparison | conformance, then J3 and J5 in S4 | `--only AT-10` | four axes queryable independently; each case-level disposition yields its mapped lifecycle; episode-level outcomes never close a case; run without settlement shows result absent; blocked move names dimension and authorizer; residue and four field tests recorded | WP-21 | S2 |
| AT-11 | gates | conformance, then J8 | `--only AT-11` | all four gate states reachable; evidence change marks reassess; nothing advances automatically | WP-21 | S2 |
| AT-12 | governed revision | diagnose, propose with held-out rule, candidate validate, semantic diff, regression transition matrix, authorize, promote, re-pin rehearsal | `--only AT-12` | held-out set excluded by the frozen rule (no overlap with diagnosis input); the regression matrix of E §7 (total, RD-25) is applied and every transition result matches it; proposer ≠ approver; both worlds in registry; failed and inconclusive attempts retained; non-promotion variant retains divergence and rationale | WP-18 | S1 |
| AT-13 | capability delta | same subsequent-episode records classified under the old and the new world, plus a control variant without the newly declared step; falsifier and prediction frozen beforehand | `--only AT-13` | result is `GAIN` or `NO_GAIN`, reported either way. `GAIN` requires: old world `observed_undeclared` or undetectable, new world `confirmed`, control variant shows no gain, falsifier not triggered. R-58 is PROVEN only on `GAIN` | WP-18 | S1 |
| AT-14 | Horizon shows the source result | Playwright reads the rendered DOM text for case, residuals, settlement, world; compare with the ledger result computed independently from raw records | `--only AT-14` | rendered text equals ledger values after canonicalization; ids and digests present; absent source renders "not connected"; runs on a build containing only the read adapters (R-84) | WP-19 (full WP-40) | S1 (full S4) |
| AT-15 | resumption by a second operator | fresh agent or person given only Horizon Today and the handoff export | `--only AT-15` | states next move, authorizer, settlement condition correctly (real or blocked affordance); executes it; programmatic part in S2, operator part in S4 | WP-21 | S2 |
| AT-16 | transfer (PARTIAL, fixture-based) | tax pre-filing case: model, template and scenarios only; plus a same-gap variant (different declared model with the same structural gap, promoted pattern applied through the existing process) | `--only AT-16` | proof command passes with only model, template, scenario files changed; evidence-mechanism code diff empty and no SOP constants in it (grep check); same-gap variant shows the learned distinction applied; labelled non-actual and PARTIAL in the ledger | WP-56 | S5 |
| AT-17 | assignment lifecycle | delegate, interrupt, resume, complete; budgets; context bundle; verification records | `--only AT-17` | episode reference unchanged across resume; assignment id `assign:<caseId>:<stepId>:<n>:<k>` (`k` = delegation instance within episode `n`); spend per currency or `unknown`; verification bound to `episode_ref`; retrieved context labelled untrusted reference | WP-23, WP-43, WP-44 | S2 (full S4) |
| AT-18 | reproducibility and independent "before" | fresh verifier on a clean cell: verify anchors; verify and replay the immutable raw records to recompute representation and comparison; **re-execute the `program` episodes** and compare class-level results | `--only AT-18` | anchors verify; replay hashes equal; re-executed classes equal the recorded classes at class level (differences reported, not hidden); verifier report agrees with claim ledger; bundle complete (§7) | WP-19 (S1), WP-53 | S1 (full S5) |
| AT-19 | journeys | Playwright J1–J10 against live services; agent-browser pass; fresh-operator walk-through of J1, J3, J5, J6, J7 | `--only AT-19` | all pass; stalls recorded as defects; seam rows pass | WP-41, WP-42 | S4 |
| AT-20 | installation and release | install from release artifacts on a clean cell, launch, run AT-18; `local_operator` with FDE gates | `--only AT-20` | GO | WP-57 | S5 |

## 6. Negative tests

Each runs as `scripts/fde-proof.sh --only <NT-id>` and again under `--mutate <NT-id>`, where the hash-pinned patch
must turn it red with the **named code**. Modes: `tamper` = chain or digest verification must fail; `input-variation`
= the result must change; `refusal` = the operation is refused. One expected code per test; no alternatives. The
protected record is asserted unchanged after the attack. Codes marked ‡ are "(code to be defined in WP-11/WP-20)".

| ID | Protected property (source) | Attack | Expected code | Mode | Mutation patch changes | WP |
|---|---|---|---|---|---|---|
| NT-01 | frozen reference cannot be rewritten (M§7) | second freeze for the same `(case, item, n)` with different criteria | `FREEZE_KEY_EXISTS` ‡ | refusal | remove I-3 key check | WP-14 |
| NT-02 | no verified conclusion without identity or evidence (M§7) | (a) event without `world_ref`/`declaration_key`; (b) event with valid `world_ref` but no evidence record | `INGEST_REJECTED_IDENTITY` ‡ for (a); `INGEST_REJECTED_EVIDENCE` ‡ for (b); two rows of the test, one code each | refusal | accept missing key; accept missing evidence | WP-16 |
| NT-03 | changed record changes the result (M§7) | real re-run with one occurrence's content changed (a violating value) | class changes to `contradicted`; hash differs | input-variation | cache result by case id | WP-16 |
| NT-03b | missing or tampered record is detected (M§7) | delete one occurrence from the append-only ledger copy | `CHAIN_VERIFY_FAILED` ‡ | tamper | skip chain verify in derive | WP-16 |
| NT-04 | undeclared occurrence ≠ authorized change (M§7) | undeclared behaviour before promotion; then the same behaviour after an authorized promotion declaring it | before: `observed_undeclared`, world unchanged; after: `confirmed` only via the recorded transition | input-variation | auto-add undeclared to the model | WP-17 |
| NT-05 | denied or unknown authority cannot promote (M§7) | promotion with denied / unavailable / expired / digest-mismatched decision (four cases, one code each) | `PROMOTION_DENIED`, `AUTHORITY_UNAVAILABLE`, `DECISION_EXPIRED`, `DECISION_DIGEST_MISMATCH` ‡ | refusal | treat unknown as approved | WP-18 |
| NT-06 | missing observation stays unresolved (M§7) | episode whose frozen reference names an evidence source that emitted nothing | class `insufficient`, basis `evidence_source_silent`; result `U` | input-variation | default missing evidence to success | WP-17 |
| NT-07 | settled evidence stays with its world (M§7) | read evidence after promotion and after re-pin | still bound to the old `world_ref` | refusal | retarget refs on promotion | WP-18, WP-22 |
| NT-08 | failing regression blocks success (M§7) | candidate that breaks a prior confirmed element not in intended changes | `REGRESSION_FAILED` ‡ | refusal | skip regression | WP-18 |
| NT-09 | Horizon equals programmatic result (M§7) | alter the adapter's classification mapping | rendered DOM text differs from ledger value: `PARITY_MISMATCH` ‡ | input-variation | change adapter mapping (the attack is the patch) | WP-19 |
| NT-10 | no self-settlement (S§3.2, I-2) | executor principal, and an agent in the executor's delegation chain, call `episode.settled` | `SELF_SETTLEMENT_REFUSED` ‡ | refusal | remove I-2 chain check | WP-21 |
| NT-11 | hard gate not waivable (S§8.1) | set Allocate Authority `not_applicable` without approval | `HARD_GATE_NOT_WAIVABLE` ‡ | refusal | allow rationale-only | WP-21 |
| NT-12 | agent cannot make a human-reserved decision (R-67) | agent session, and the paired-browser session without the agent header, submit a disposition and a promotion approval | `RESERVED_DECISION_REFUSED` ‡ | refusal | accept session cookie | WP-31, WP-32 |
| NT-13 | exploratory never becomes a test (S§7.2) | relabel an exploratory episode as evaluated after the run | `KIND_IMMUTABLE` ‡ | refusal | allow kind change | WP-14 |
| NT-14 | skipped gate is NO-GO (R-81b) | readiness with a prerequisite missing | NO-GO, `GATE_SKIPPED` ‡ | refusal | count skip as pass | WP-57 |
| NT-15 | unverified world and undeclared identity refused at a boundary (R-32) | (a) valid-syntax unregistered `world_ref`; (b) registered world, `namespace::name` that does not exist in the pinned semantic snapshot, or an `actor`, `operation` or `resource` not declared for that element (RD-22) | `WORLD_UNREGISTERED` ‡ for (a); `IDENTITY_UNDECLARED` ‡ for (b); one code each | refusal | syntax-only check; accept the executor's claimed key and fields | WP-16 |
| NT-16 | "before" is externally anchored (M§3, F1) | regenerate the whole chain with the freeze inserted before the first run record; anchor tag absent or later | `ANCHOR_PREDATES_FAILED` ‡ | tamper | verifier trusts chain only | WP-14 |
| NT-17 | reference not reconstructed from the result (M§3) | freeze whose digest covers result content | `REFERENCE_DERIVED_FROM_RESULT` ‡ | tamper | skip result-field exclusion check | WP-14 |
| NT-18 | model registered before executions (M§2) | register the model after the first run event | `MODEL_ANCHOR_LATE` ‡ | tamper | no model-anchor check | WP-14 (anchor from WP-13) |
| NT-19 | unsigned or forged emitter event cannot count (RD-05; arrives with WP-33, S3) | event with no signature; event signed by another workload identity | `EMITTER_SIGNATURE_INVALID` ‡ | refusal | accept unsigned | WP-33 |
| NT-20 | executor-only evidence cannot yield `confirmed` (RD-05) | episode whose every occurrence comes from the executor's emitter | class `insufficient`, basis `no_independent_emitter` | input-variation | drop independent-emitter rule | WP-17 |
| NT-21 | regression matrix (RD-11) | (a) `confirmed→insufficient` candidate; (b) unlisted weakening of a policy | `LOST_CAPABILITY` ‡; `UNLISTED_WEAKENING` ‡ | refusal | omit those matrix rows | WP-18 |
| NT-22 | suspended case refuses freeze and delegation (RD-10) | freeze and delegate on a case whose latest case-level disposition is `suspend` | `CASE_SUSPENDED` ‡ | refusal | ignore suspension | WP-21 |
| NT-23 | no close without close disposition (RD-10) | advance a case to Completed with no close disposition | `CLOSE_DISPOSITION_REQUIRED` ‡ | refusal | derive closed from state alone | WP-21 |
| NT-24 | reserved decision cannot be confirmed on the desk (RD-06) | confirm a reserved decision on the desk page, with a valid passkey assertion | `A3_SURFACE_REQUIRED` ‡ | refusal | accept desk-surface assertion | WP-31, WP-32 |
| NT-25 | later observation cannot rewrite a frozen expectation (M§7) | ingest an observation carrying a different expected outcome for a frozen reference | `EXPECTATION_IMMUTABLE` ‡ | refusal | overwrite expectation on ingest | WP-16 |
| NT-26 | synthetic evidence is not execution evidence (M§1) | `injected_fault` or fixture record offered as support for an execution claim | `PROVENANCE_NOT_EXECUTION` ‡ | refusal | ignore provenance flag | WP-19 |
| NT-27 | model validated by the real compiler (M§2) | register a model the compiler rejects | `MODEL_INVALID` ‡ | refusal | skip compile step | WP-13 |
| NT-28 | no `confirmed` on vacuous or single-occurrence support (RD-08) | empty fact set; one occurrence where profile minimum is 3 | `SUPPORT_BELOW_PROFILE` ‡ (class `insufficient`) | input-variation | treat vacuous truth as confirmed | WP-17 |
| NT-29 | declared-unobserved needs a triggered obligation (RD-03) | untriggered declared element; then add the trigger event, record absent | untriggered: `not exercised`; triggered: `declared_unobserved` | input-variation | report every unexercised element | WP-17 |
| NT-30 | coverage manifest frozen with the episode (RD-12) | edit the manifest after freeze to claim coverage | `MANIFEST_DIGEST_MISMATCH` ‡ | tamper | skip manifest digest | WP-17 |
| NT-31 | clean control is non-vacuous and clean (M§7) | (a) control run on an empty ledger; (b) clean conforming control whose result contains a `contradicted` residual | `CONTROL_VACUOUS` ‡ for (a); `CONTROL_NOT_CLEAN` ‡ for (b); passes only with zero `contradicted` and at least the profile minimum of `confirmed` | refusal | pass control with zero occurrences; ignore `contradicted` in the control | WP-17 |
| NT-32 | freeze before move (I-1) | delegate for an evaluated episode with no freeze at all | `FREEZE_MISSING` ‡ | refusal | trust caller-supplied tag | WP-14 |
| NT-33 | freeze is complete before move (RD-12) | (a) RealityTrace records exist, case record absent; (b) the reverse; (c) both exist, anchor missing: delegation refused when the anchor is missing | `FREEZE_INCOMPLETE` ‡ for each of (a), (b), (c) | refusal | require one side only; ignore the anchor | WP-14 |
| NT-34 | authorization at the action boundary (S§7.3) | consequential action with a denied decision | `AUTHORITY_DENIED` ‡ | refusal | skip boundary check | WP-14 |
| NT-35 | occurrence resolves at the owner (RD-05, M§3) | event whose `source.record_id` does not exist in the owning system | `SOURCE_UNRESOLVED` ‡ | refusal | count unresolved events | WP-16 |
| NT-36 | expected outcome links to a frozen reference (M§3) | event citing an expectation ref that matches no freeze | `EXPECTATION_REF_UNKNOWN` ‡ | refusal | accept any ref | WP-16 |
| NT-37 | observed model is computed, not authored (M§4) | compare a hand-edited representation | `REPRESENTATION_NOT_DERIVED` ‡ | tamper | skip derive-hash check | WP-17 |
| NT-38 | new representation must show gain (M§5, AT-13) | AT-13 where the control variant (step absent) gives the same result | `NO_CAPABILITY_GAIN` ‡ | input-variation | skip control variant | WP-18 |
| NT-39 | continuation derives from source records (S§4.3) | delete the source record, keep journal notes | `CONTINUATION_SOURCE_MISSING` ‡ | input-variation | build from journal | WP-21 |
| NT-40 | disposition by the authorized actor (S§7.5) | disposition by an executor or unauthorized principal | `DISPOSITION_UNAUTHORIZED` ‡ | refusal | skip authority check | WP-21 |
| NT-41 | tool completion is not settlement (S§7.5) | dispose `close-satisfied` on an episode with no settlement | `SETTLEMENT_REQUIRED` ‡ | refusal | treat completed as settled | WP-21 |
| NT-42 | result and reliability are separate (S§7.5) | settle with result outside S/F/U/C, or reliability merged into the result | `RESULT_INVALID` ‡ | refusal | accept free text | WP-14 |
| NT-43 | authority decision is verified (RD-05) | event citing an `authority.decision_id` SEA-Forge does not know | `AUTHORITY_DECISION_UNVERIFIED` ‡ | refusal | trust the id | WP-16 |
| NT-44 | source records preserved (M§3) | alter one raw record in CAS; replay | `RAW_RECORD_DIGEST_MISMATCH` ‡ | tamper | skip digest on replay | WP-16 |
| NT-45 | a difference names evidence and basis (M§4) | write a residual without evidence refs or basis | `RESIDUAL_BASIS_MISSING` ‡ | refusal | make fields optional | WP-17 |
| NT-46 | separation of claim kinds (M§6) | adapter shows a hypothetical value with the "recorded" source mark | rendered mark differs from source: `SOURCE_MARK_MISMATCH` ‡ | input-variation | collapse marks in adapter | WP-19 |
| NT-47 | aggregates traceable to occurrences (M§4) | aggregate listing a non-existent or omitting a real occurrence id | `AGGREGATE_UNTRACEABLE` ‡ | tamper | skip id listing | WP-17 |
| NT-48 | anchor published by case keeper only (RD-04) | tag pushed with an executor credential | `ANCHOR_PUBLISHER_INVALID` ‡ | refusal | accept any tagger | WP-14 |
| NT-49 | order from owner's sequence, not arrival (RD-05) | events arriving in reversed order, with a sequence number the owner never issued | `SEQUENCE_UNVERIFIED` ‡ | refusal | use arrival order | WP-16 |
| NT-50 | virtual authenticator cannot make a reserved row PROVEN (RD-06) | ledger build with a virtual-authenticator assertion for a reserved decision | `AUTHENTICATOR_NOT_REAL` ‡ | refusal | ignore authenticator type | WP-53 |
| NT-51 | SEA-Forge validates A3 evidence (RD-06) | Horizon-minted approval evidence presented to SEA-Forge | `A3_EVIDENCE_INVALID` ‡ | refusal | trust Horizon | WP-31 |
| NT-52 | no second policy evaluator (RD-08) | `domainforge evaluate` stubbed unavailable | class `insufficient`, basis `EVALUATOR_UNAVAILABLE` ‡ (no local evaluation) | input-variation | add local evaluation | WP-17 |
| NT-53 | exploratory occurrences never confirm (RD-12) | exploratory occurrence for an element the evaluated episode did not exercise | `EXPLORATORY_EXCLUDED` ‡ (class unchanged) | input-variation | drop kind selector | WP-16 |
| NT-54 | renames need an explicit key mapping (RD-11) | compare across a renamed concept with no mapping | `KEY_MAPPING_REQUIRED` ‡ | refusal | match by name | WP-18 |
| NT-55 | held-out set excluded from diagnosis (M§5.6) | diagnosis input includes a held-out occurrence | `HELDOUT_CONTAMINATED` ‡ | refusal | ignore the rule | WP-18 |
| NT-56 | no stale classification after re-pin (RD-11) | read a classification stamped with the withdrawn world as current | `CLASSIFICATION_SUPERSEDED` ‡ | refusal | drop world stamp | WP-22 |
| NT-57 | independent verifier is independent (RD-17) | verifier on the same model CLI or sharing executor context | verdict PARTIAL, `VERIFIER_NOT_INDEPENDENT` ‡ | refusal | skip independence check | WP-54 |
| NT-58 | verifier makes its own observation (RD-17) | settlement whose only evidence is the executor's output | `VERIFIER_OBSERVATION_MISSING` ‡ | refusal | accept executor output | WP-21 |
| NT-59 | mutation patches are pinned (RD-17) | altered patch file | `MUTATION_PATCH_UNPINNED` ‡ | tamper | skip sha256 check | WP-52 |
| NT-60 | pre-registration is frozen (M execution rules) | edit an AT/NT definition after the anchor | `PREREG_DIGEST_MISMATCH` ‡ | tamper | skip prereg verify | WP-09 |
| NT-61 | a failed or skipped test cannot support PROVEN (M execution rules) | ledger build with one failed and one skipped test | `CLAIM_NOT_SUPPORTED` ‡ | refusal | count skip as pass | WP-19 |

NT-02, NT-05, NT-15, NT-31 and NT-33 are tables of cases, each case one expected code; each case is a separate sub-test (`NT-02a`, `NT-02b`, and so on).
NT-03b is the tamper variant of NT-03 and runs with it. A clean conforming control (the conforming episode alone)
runs first and must produce zero `contradicted` residuals and at least the profile's minimum of `confirmed` support (NT-31).

## 7. Evidence bundle

Single root `.agents/evidence/fde-field-case/<run-id>/`.

1. `manifest.json`: repo commits and local changes (baseline by Git remote), toolchain and runtime versions,
   environment, commands, test run ids and outcomes, approvals obtained, per-occurrence **provenance flags**.
2. `semantic/`: SOP source and hash, initial model and `world_ref`, semantic-pack relationships, translation map and
   assumptions, candidate model, semantic diff, approval status, `comparison-profile.json`, `coverage-manifest.json`.
3. `operational/`: pre-registration (`prereg/`) and its anchor tag, frozen references with establishment times and
   anchor tags, verifier receipts, raw event records, authority decisions, RealityTrace ledger export with ids and
   envelopes, derived representations, residuals (episode and structural separate), regression matrix, subsequent episode.
4. `horizon/`: rendered captures (PNG and DOM text) for each inspected item, parity output (ledger value vs DOM text),
   journey results.
5. `verification/`: machine-readable AT/NT results, commands and exit codes, mutation results with patch sha256,
   independent verifier report (own observations), skipped or inconclusive checks and their effect on claims.
6. `claim-ledger.md`: one row per claim: exact capability, location, command, evidence reference, result
   `PROVEN | PARTIAL | PLANNED | NOT TESTED`, executor provenance, scope, limitation, evidence level (bounded
   interaction, repetition, variation, recovery, transfer). Areas: initial semantic modelling, provenance, semantic
   exhaust, observed-model derivation, divergence classification, governed revision, subsequent operational
   capability, Horizon inspection; plus transfer, installation.

## 8. Independent confirmation (RD-17)

A fresh-context verifier on a **different model CLI** (the repository's existing independent-review route); if
unavailable the confirmation is PARTIAL (NT-57). The verifier holds the frozen criteria and the artifacts, not the
implementer's narration. It: runs the declared command itself for each settled outcome (own observation); runs
`fde-anchor.sh verify` on every anchor; replays raw records and re-executes `program` episodes (AT-18); **re-applies
every mutation patch from a clean checkout** and must see red with the named code; samples five claims end to end from
ledger to raw record; **chooses two further negative tests of its own** and records their results; writes a verdict
per claim. Disagreement is a defect; an unresolved verdict means not PROVEN.

## 9. Slice gates (F §1)

Each gate runs `scripts/fde-proof.sh --slice S<n>` on a clean cell.

| Slice | Must be green | Supports claim level |
|---|---|---|
| S1 | AT-01, 04, 05, 06, 07, 08, 12, 13, 14 (read adapters), 18 (S1 form); NT-01 to NT-09, NT-16. Claim, exactly: "the Mission §1–7 path exists and runs once on program execution." It is not evidence about organisational behaviour. Three `program` episodes (conforming control; falsifiable inquiry; subsequent episode after one revision); AT-07 requires `confirmed` plus one of `contradicted` / `observed_undeclared` | bounded interaction, once, on `program` execution; pre-S3 identity limits stated; rows depending on human-reserved decisions PARTIAL; anchoring rows PARTIAL without OD-6; other classes NOT TESTED |
| S2 | AT-02, 03, 09, 10, 11, 15 (programmatic), 17 (lifecycle); NT-07 (re-pin variant), NT-10, 11, 13, 15, 18, 20, 21, 22, 23, 25 to 38, 39, 40, 41, 42 to 49, 52 to 55, 56, 58, 60, 61; saga crash-injection. Tests owned by S1 packages and not in the S1 gate run here | SOP §4, §7–§10 record model |
| S3 | NT-12, 19, 24, 51; identity and authority package tests; A3 live on the paired phone | human-reserved and emitter-authenticity rows (PROVEN only with a real authenticator, OD-5) |
| S4 | AT-14 (full), 15 (operator), 17 (full), 19 | Mission §6; operator capabilities |
| S5 | all AT and NT; NT-14, 50, 57, 59; AT-16, 18 (full), 20; mutation re-application by the verifier; the five behaviour kinds on `live` execution | completion on the bounded case; repetition of `program` episodes; transfer PARTIAL |

## 10. What one passing case does and does not establish

Establishes: the bounded mechanism works once, end to end, on real work (evidence level: **bounded interaction**).
Separate levels, each with its own matrix row: **repetition** (the `program` episodes re-executed on a clean cell,
AT-18: establishes determinism of that path only); **variation** (meaningfully different inputs or contexts: not
established by FC-001; PLANNED); **recovery** (behaviour under fault beyond the injected set: only the saga
crash-injection and negative suite, PARTIAL); **transfer** (AT-16, fixture-based, PARTIAL). Does not establish
reliability, recovery in operation, or transferable organizational capability; those stay PARTIAL or PLANNED until
repeated cases and professional acceptance justify them. `program` executors make the proof reproducible but
exercise less of the agent runtime than `live` work; both are run and reported separately.

## Decisions made while revising

- Command surface: `--only`, `--slice`, `--replay` (from F) plus `--mutate` and `scripts/fde-anchor.sh`; mutation patches under `.agents/preregistrations/fde/mut/` with `SHA256SUMS` in the anchored pre-registration.
- Anchor tags are pushed to the GitHub remote of the SEA-Forge repository (H says "the GitHub remote").
- AT-13 exits 0 with result `GAIN` or `NO_GAIN`; only `GAIN` makes R-58 PROVEN (stricter than a pass/fail test).
- In S1 the executor harness's own fresh-context verification step is the revision candidate; it arises by construction of the harness and is labelled `program` (RD-20). The S1 gate test list is fixed by RD-20.
- NT-02 and NT-05 are sets of sub-tests with one code each, to avoid disjunctive pass conditions; NT-03 split into NT-03 (input-variation) and NT-03b (tamper).
- Ledger builder (claim-ledger rules NT-26, NT-61) ships in S1 form with WP-19, not WP-53.
- G's owner column is authoritative for test ownership; F cites it. NT-20 has one owner (WP-17); NT-19 belongs to WP-33 (S3).
- A human settling a disposition on a real authenticator is the only route to PROVEN for reserved rows; pre-S3 principal identity yields PARTIAL.
