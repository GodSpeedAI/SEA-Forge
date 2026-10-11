# GodSpeed Judgment Plane — Specification Correction Note (v0.2.0 to v0.3.0)

**Status:** Spec correction complete. No implementation. No PROOF-1 execution. No production code touched. No baseline file overwritten.
**Date:** 2026-09-16
**Scope:** Corrects eleven places where the v0.2.0 proposal asserted more than the repository-grounded evidence supports.

## Provenance — what this supersedes and what it does not

| Artifact | Status after this task |
|---|---|
| `.agents/specs/GODSPEED_JUDGMENT_PLANE_SPEC.yaml` (v0.1.0) | **UNCHANGED and still authoritative.** Nothing may be implemented from the proposal until it is accepted. |
| `.agents/specs/AGENT_SPEC_TEMPLATE.yaml` | **UNCHANGED.** |
| `2026-09-16-godspeed-judgment-plane-adversarial-review.md` | **RETAINED unchanged.** It is the evidence base. |
| `2026-09-16-godspeed-spec-delta.yaml` (v1) | **RETAINED unchanged.** Superseded *as a proposal* by the v2 delta. |
| `2026-09-16-godspeed-judgment-plane-spec-revised.yaml` (v0.2.0) | **RETAINED unchanged.** Superseded *as a proposal* by the v2 spec. |
| `2026-09-16-godspeed-current-architecture.mmd`, `...-capability-gap.yaml` | **RETAINED unchanged.** No correction required. |
| `2026-09-16-godspeed-target-architecture.mmd` (v1 diagram) | **RETAINED unchanged.** Superseded *as a proposal* by the v2 diagram. |
| `2026-09-16-godspeed-judgment-plane-spec-revised-v2.yaml` (v0.3.0) | **NEW** — corrected proposal. |
| `2026-09-16-godspeed-spec-delta-v2.yaml` | **NEW** — corrected delta. |
| `2026-09-16-godspeed-target-architecture-v2.mmd` | **NEW** — corrected target diagram. |
| `2026-09-16-godspeed-spec-correction-note.md` | **NEW** — this file. |

---

## 1. Each correction made

### C1 — `VerificationRecord` is a proving carrier, not the universal judgment type

| | |
|---|---|
| v0.2.0 said | A generic judgment record was rejected because `VerificationRecord` was treated as the **universal** replacement for any bounded probabilistic judgment. |
| v0.3.0 now says | **Slice 1 MUST reuse `VerificationRecord` unless evidence demonstrates it cannot preserve the required distinction.** Reuse is proven; **universality is not.** No new generic probabilistic-result type until a concrete **non-verification** use case demonstrates semantic insufficiency. The reverse error is also prohibited: the carrier MUST NOT be declared eternal. |
| Lands in | entity `JudgmentObservation.universality_clause`; `rejected_or_merged_ledger` (`permanence` + `scope_of_this_decision` per item); **NONGOAL-008**; roadmap gate `judgment_carrier_generalization`; spec `final_self_check` |
| Named mismatch families | incident classification; severity estimation; capability classification; represented versus unrepresented discrimination; routing classification; state interpretation; proposition estimation before a verification claim exists |

### C2 — PROOF-1 may falsify a surface, not the capability class

| | |
|---|---|
| v0.2.0 said | No information gain in PROOF-1 leads to a global `Hypothesis FALSIFIED` node, rejection of the uncertainty requirement generally, and collapse of the plane. |
| v0.3.0 now says | A negative result falsifies **a surface**. It is reported at `surface`, `task_family`, or `capability_class` scope. A `capability_class` conclusion requires evidence across **at least two materially different judgment shapes**. |
| New concept | **`ScopeOfFalsification`** with exactly three values: `surface`, `task_family`, `capability_class`. |
| Lands in | **REQ-JUDG-023**, **REQ-OUT-007**, amended **REQ-OUT-006**, entity `ScopeOfFalsification`, `proof_1_definition.admissible_outcomes` (A-E with forbidden inferences), new conformance group `claim_scope_integrity`, v2 diagram nodes `SOF`/`SHAPES` |
| Shapes to distinguish | 1 binary or proposition; 2 multiclass choice or classification; 3 ordered score or rubric. **Slice 1 need not cover all three, but MUST NOT generalize from one to another.** |

### C3 — Stop calling the 17-settlement output "calibration"

| | |
|---|---|
| v0.2.0 said | Required a **calibration report** from 17 settled judgments with incomplete model attribution. |
| v0.3.0 now says | The first output is a **bounded-judgment PILOT MEASUREMENT report**. |
| Reserved terms | *calibrated probabilities*, *reliable calibration curve*, *production thresholds*, *population-level reliability*, *model generalization* — all reserved for a later evidence level with a sufficient corpus, representative variation, attribution, and held-out evaluation. |
| May still compute | Brier score, log loss where applicable, accuracy, abstention rate, coverage, provider disagreement, confusion matrix, deterministic-baseline comparison, descriptive reliability bins **explicitly marked exploratory**. |
| Lands in | **REQ-VERIFY-012**; amended **REQ-VERIFY-004** and **REQ-SETTLE-003**; `pilot_measurement_metrics`; `CLAIM-003.explicitly_not_claiming`; roadmap gate `judgment_calibration`; amended **VAR-008**; v2 diagram node `NOTCAL` |

### C4 — Sharpen DecisionSurface authority

| | |
|---|---|
| v0.2.0 said | A DecisionSurface is derived from DomainForge `ApplicationContract` plus Gauntlet `Reference`/`ReferenceDefinition`, without stating which of the two owns what. |
| v0.3.0 now says | **DomainForge owns semantic possibility and identity** — canonical concepts, identities, admissible semantic distinctions, domain relationships, and the meaning of referenced values. **Gauntlet/CEP evaluation context owns the question asked of that world** — which semantic distinction is relevant for this evaluation, evaluation criteria, settlement criteria, the claim or question under examination, task-local constraints. **Gauntlet MUST NOT become a second semantic authority merely because it contributes evaluation context.** |
| Structural effect | A derived surface **SHOULD** carry `semantic_basis` and `evaluation_basis` distinguishably. The exact implementation shape remains implementation-defined; the distinguishable attribution does not. |
| Lands in | **REQ-AUTH-006**, **NONGOAL-010**, entity `DecisionSurface.semantic_basis`/`evaluation_basis`, `system_overview.components[DecisionSurface].dual_basis_requirement`, adversarial validation row, v2 diagram nodes `MEANING`/`EC`/`DS` |
| Not required | no `.sea` grammar change; no new compiler; no new store |

### C5 — Keep "Judgment Plane" as an architectural concern, not a runtime noun

| | |
|---|---|
| v0.2.0 said | Rejected the runtime component but left the phrase readable as a deployable thing. |
| v0.3.0 now says | **`Judgment Plane` names the architectural responsibility** that preserves the distinction between represented state, bounded learned inference, deterministic composition, and authority. It MUST NOT imply a daemon, microservice, database, deployment unit, third provider port, or independent persistence subsystem. |
| Normative use rule | A requirement MAY refer to the Judgment Plane as a responsibility boundary. A requirement MUST NOT require a component, service, port, store, or deployment artifact to exist under that name. |
| Resolves | The apparent contradiction: "GodSpeed needs bounded judgment" is a **capability** requirement; "GodSpeed does not need a new Judgment Plane component" is a **realization** constraint. Both are true. |
| Lands in | `architectural_concern` section, **NONGOAL-009**, v2 diagram outer subgraph labelling, `final_self_check` |

### C6 — Keep the loop correlation identity as first-class

| | |
|---|---|
| v0.2.0 said | Described the correlation identity as "one field echoed at four boundaries", implying a forced identical field in every subsystem. |
| v0.3.0 now says | The normative requirement is **joinability**, not a field name: artifacts participating in one operative loop MUST be deterministically joinable through a stable correlation identity **without heuristic string matching**. Whether this is implemented by direct echo, envelope projection, a mapping table, or another lossless mechanism is **implementation-defined** unless an existing repository contract forces one representation. |
| Proof obligation | The declared mechanism MUST be demonstrated lossless by a **joinability test**, and joinability MUST NOT be inferred from string similarity. |
| Lands in | amended **REQ-PROV-021** (still **blocking**, still first-class), new **REQ-PROV-022**, entity `LoopCorrelationIdentity`, new validation row |
| Independence | This requirement is independent of any external provider. It is not weakened by, and does not depend on, the Jev question. |

### C7 — Do not overstate "Jev-comparable"

| Level | Claim | Status |
|---|---|---|
| **A** | **Interface comparability.** GodSpeed can represent a bounded question and a typed or distributional answer. | **SUPPORTED NOW** — the required shape already exists in the external primitive (a choice question with criteria keyed by label, answered with a label, a confidence and a probability map). |
| **B** | **Operational comparability.** GodSpeed can execute that contract reliably with suitable latency, retries, structured-output reliability, and provider substitution. | **UNPROVEN** — requires the proving slice plus target-machine evidence. |
| **C** | **Model-quality comparability.** GodSpeed's providers achieve judgment quality or calibration comparable to an external service. | **UNPROVEN** — may require direct access to the external service and a materially larger evaluation set. |
| Lands in | **CLAIM-006** (with a rule that no statement may collapse the levels), roadmap `TypeSafe Jev provider` reclassified to level A, `implementation_definition_of_done`, conformance group `claim_scope_integrity` |

### C8 — Preserve the strong reductions from the review

No change requested. All of the following are preserved explicitly in `correction_ledger` C8 and drawn in the v2 diagram as node `R1`:

DomainForge unchanged; no `.sea` grammar change; DecisionSurface initially **derived**, not a new compiled language construct; Gauntlet `AgentRunner` remains the invocation boundary; **no third provider port**; HTTP provider coverage is the missing transport path; SEA-Forge remains the authority owner; provider and model output is **evidence, never permission**; deterministic composition remains outside the model; composition rules become versioned and inspectable; **no new database or message broker** on the judgment path; **no training pipeline in slice 1**; **no claim of learning**; **no destructive rewrite of existing ledgers**; memory and evidence ownership must be **explicitly scoped** rather than collapsed blindly.

### C9 — Handle CEP additivity conservatively

| | |
|---|---|
| v0.2.0 said | `AnswerDomain` is "realized in CEP-0005" and `Question` was "relocated into CEP", as though compatibility were established. |
| v0.3.0 now says | The **distinction** is normative: every bounded question MUST have an explicit, versioned answer domain. The **carrier** is NOT prematurely fixed. |
| Acceptable slice-1 carriers | additive CEP field **IF** compatibility is proven; the existing result-contract artifact; another existing hash-addressed evaluation artifact. |
| CEP's status | Preferred semantic home **only if** compatibility and ownership are proven. |
| Lands in | amended entity `Question` (`compatibility.carrier_status = OPEN`, three named carriers), new **REQ-JUDG-024**, new config field `answer_domain_carrier`, **U1 remains genuinely OPEN**, v2 diagram nodes `AD`/`CARRIER` |
| Why | The RealityTrace runtime pins the CEP envelope schema by hash. My own review flagged this as an open question; v0.2.0 wrote it as settled, contradicting its own evidence base. |

### C10 — Make the first experiment answer the right question

**Rewritten governing question:**

> Can the existing Gauntlet corpus reconstruct a consequence-grounded bounded-decision experiment end-to-end, and on the first selected surface, what incremental information — if any — is provided by uncertainty-preserving outputs from materially independent providers relative to the existing deterministic outcome?

It is now explicitly **two parts**, with a rule that part-2 conclusions MUST NOT be drawn when part 1 fails.

| Outcome | Condition | Conclusion | Forbidden inference | Scope of falsification |
|---|---|---|---|---|
| **A** | History cannot be reconstructed | provenance/correlation is the immediate blocker | nothing about probabilistic judgment quality, any answer domain, or the capability class | n/a |
| **B** | History reconstructs, but provider output cannot reliably satisfy the bounded contract | provider / structured-output / DecisionSurface mechanics need work | uncertainty-preserving judgment MUST NOT be concluded useless | n/a |
| **C** | History reconstructs, distribution adds little on **this** surface | do not promote probabilistic judgment **for this surface** | system-level rejection is NOT permitted from this outcome | `surface` |
| **D** | History reconstructs, distribution adds useful information | evidence supports proceeding with the minimal bounded-judgment implementation | level A MAY be claimed; level B needs target evidence; level C remains unproven | `surface` or `task_family` |
| **E** | Providers disagree strongly | inspect whether the surface is ambiguous, context is insufficient, providers are poorly matched, or the task is genuinely uncertain | the surface MUST NOT automatically be concluded wrong — **disagreement is itself evidence** | n/a |

Before any wider conclusion from outcome **C**, at least one **materially different** bounded-decision surface must be evaluated. A system-level falsification is justified only after evidence spans materially different judgment shapes.

Lands in: `proof_and_observability.proof_1_definition` (governing question, two-part structure, `admissible_outcomes`, `corpus_reality`), new `measurement failure` class in `failure_model_and_recovery`, `implementation_definition_of_done`.

### C11 — Preserve exploratory metrics without false precision

| Set | Contents |
|---|---|
| **MAY compute** | Brier score; log loss where applicable; accuracy; abstention rate; coverage; provider disagreement rate; confusion matrix; deterministic-baseline comparison; descriptive reliability bins explicitly marked exploratory |
| **MUST report with every metric** | N; number of independent runs; class counts; effective diversity / concentration; missing attribution; reconstructibility rate |
| **MUST NOT claim** | calibrated probabilities; a reliable calibration curve; production thresholds; population-level reliability; model generalization |
| **Instability rule** | Where sample size makes a metric unstable, the report MUST say so explicitly rather than presenting a precise-looking number. Inferential-looking precision MUST NOT be used to imply a population conclusion. |
| **Metric naming** | allowed: exploratory reliability bins, descriptive agreement, observed score. disallowed: calibration curve, calibrated probability, unqualified reliability diagram. |

Lands in: `pilot_measurement_metrics`, **REQ-VERIFY-012**, `validation_matrix` row, `implementation_definition_of_done`, v2 diagram nodes `DISC`/`NOTCAL`.

---

## 2. Why the previous wording exceeded the evidence

Every over-reach shares one root cause: **v0.2.0 used a single piece of evidence to licence a general statement.**

| Correction | The generalization error |
|---|---|
| **C1** | One excellent carrier for the **verification** family was promoted to a claim about **every** judgment family. The evidence establishes reuse for a slice; it says nothing about non-verification shapes, and the record is literally named and documented as a verification decision binding one claim to one evidence ref and one instrument outcome. |
| **C2** | One surface, one answer domain, one corpus and one task family were given authority over a **capability class**. With 17 settled judgments of a single shape, a null result is consistent with several competing explanations — wrong surface, weak providers, insufficient context, genuinely hard task — and the experiment as framed could not distinguish them. |
| **C3** | A corpus of 17 with incomplete model attribution and lower effective diversity than its run count was described as sufficient for **calibration**. My own review had already established the opposite, so this was an **internal contradiction** inside the v0.2.0 artifact set, not merely an overstatement. |
| **C4** | Two sources contributed to one artifact without stating their authorities. DomainForge carries canonical meaning; the harness's `QualityDimension` is an **evaluation criterion**. Letting both flow into one derived object with no attribution invites the evaluation context to be read as defining meaning. The evidence supports the distinction; the wording failed to state it. |
| **C5** | Rejecting a component while keeping a component-shaped name is how a rejected component returns. The contradiction between "needs bounded judgment" and "does not need a Judgment Plane component" was never resolved in the text, leaving it available to be re-inflated by the next reader. |
| **C6** | Describing the need as a **field** invited the assumption that every subsystem must adopt an identical column — an implementation decision the repositories do not force. The evidence establishes only that no reliable shared identity exists today. |
| **C7** | Interface shape was used to imply operational and quality parity. The external primitive's shape genuinely matches; nothing measured latency, retries, structured-output reliability, or judgment quality. |
| **C9** | A preferred home was described as an established one. The RealityTrace runtime **pins the CEP envelope schema by hash**, and my own review recorded that as open question U1. Writing the spec as though it were settled contradicted the evidence base it was derived from. |
| **C10** | A single governing question with one failure branch forced every possible result into a pass/fail frame — which is exactly what made an over-general conclusion available in the first place. |
| **C11** | Listing metrics without a mandatory disclosure block lets a precise-looking number from a tiny sample read as a population result. |

---

## 3. Did any requirement ID change?

**No identifier was renamed, renumbered, or reused.** All **57** baseline `REQ-*` identifiers are present, verified mechanically against `.agents/specs/GODSPEED_JUDGMENT_PLANE_SPEC.yaml` (script check: `MISSING baseline ids: NONE`). The baseline `VAR-001..008` and `RECOV-001..004` spellings, which carry no `REQ-` prefix, are also present and preserved.

**Amended — text narrowed, ID preserved (4):**

| ID | Was | Now |
|---|---|---|
| `REQ-OUT-006` | "a falsification of the bounded-judgment hypothesis" | a negative result at an **explicitly declared scope**, never generalized beyond it |
| `REQ-JUDG-020` | answer domain "realized in CEP-0005" | the answer domain is **required**; the carrier is declared, recorded and versioned (`REQ-JUDG-024`) |
| `REQ-PROV-021` | "one field echoed at four boundaries" | **joinability** without heuristic string matching; mechanism implementation-defined |
| `REQ-VERIFY-004` | historical replay supports the capability claim | historical replay establishes **measurability only**, with corpus limits stated in the same artifact |

**Added in v0.3.0 (10) — only where the corrected distinction did not previously exist:**

| ID | Purpose | Correction |
|---|---|---|
| `REQ-OUT-007` | A scoped non-promotion outcome is expressible | C2 |
| `REQ-JUDG-023` | A negative result MUST declare its scope of falsification; a capability-class conclusion needs two materially different shapes | C2 |
| `REQ-JUDG-024` | The answer-domain requirement is carrier-independent, declared, recorded and versioned | C9 |
| `REQ-AUTH-006` | Contributing evaluation context MUST NOT confer semantic authority | C4 |
| `REQ-PROV-022` | The correlation mechanism is implementation-defined and must be proven lossless | C6 |
| `REQ-VERIFY-012` | Pilot reporting discipline: disclosure block, reserved terminology, instability statements | C3, C11 |
| `CLAIM-006` | Comparability must be claimed at an explicit level (A/B/C) | C7 |
| `NONGOAL-008` | Neither universalizing the carrier nor forbidding a future carrier without evidence | C1 |
| `NONGOAL-009` | Judgment Plane MUST NOT be realized as a deployable unit | C5 |
| `NONGOAL-010` | Evaluation context MUST NOT become semantic authority | C4 |

**No baseline requirement was rejected or weakened.** Two were strengthened: `REQ-PROV-021` remains **blocking**, and `REQ-SETTLE-001` remains an upgraded MUST.

---

## 4. Was any previous architectural conclusion reversed?

**None.** Every correction narrows a **claim** or a **scope**. Eleven for eleven, the changes move toward less claim, less scope, more disclosure. Zero corrections widen the architecture, add a component, or soften a requirement.

| Conclusion from the adversarial review | Status after correction |
|---|---|
| No Judgment Plane service, daemon, or runtime component | **Preserved**, and made explicit by C5 |
| No third provider port | **Preserved** (C8) |
| No new database or message broker | **Preserved** (C8) |
| No `.sea` grammar change | **Preserved** (C8) |
| `VerificationRecord` reused for slice 1 | **Preserved and scoped** by C1 — reuse kept, universality withdrawn |
| `CaseAssessment`, `JudgmentComposer`, `JudgmentProvider` rejected | **Preserved, standing** (not slice-scoped) |
| `AgentRunner` remains the invocation boundary | **Preserved** (C8) |
| SEA-Forge remains the authority owner | **Preserved and extended** to semantic authority by C4 |
| DomainForge remains sole semantic authority | **Preserved and made enforceable** by C4 |
| Composition stays outside the model; rules versioned | **Preserved** (C8) |
| Loop correlation identity is first-class | **Preserved**; wording became joinability-based (C6) |
| No training pipeline, no learning claim | **Preserved** (C8, `CLAIM-005`) |
| No destructive ledger rewrite | **Preserved** (C8) |
| Evidence ownership scoped, not collapsed | **Preserved** (C8) |

**The one narrowed proposition.** PROP-03 in the delta was `JudgmentObservation deserves a new type → FALSE`. It is now `FALSE FOR SLICE 1; NOT UNIVERSALLY ESTABLISHED`. This narrows the **basis** of the conclusion, not the conclusion, and is required by C1. No other proposition changed verdict; six had their basis narrowed (because PROOF-1 previously carried too much epistemic authority) and two had the comparability reading split into levels.

---

## 5. Which unresolved questions remain

Nine remain open. None was closed by wishful writing, and three changed status because of the corrections.

| ID | Question | Class | Status after correction |
|---|---|---|---|
| **U1** | Can an additive answer-domain field coexist with the RealityTrace-pinned CEP envelope schema hash without a CEP amendment? | ASSUMPTION | **OPEN — deliberately not settled.** The carrier is decoupled from the requirement (C9). |
| **U2** | Do any of the 16 run databases contain records of kind `observation`, `comparison`, `residual`, or `promotion_decision`? | FACT_PARTIAL | open |
| **U3** | Is the authority-gated model probe's `Accepted` settlement an operational settlement or a probe-local result? | INFERENCE | open |
| **U4** | Can the Gauntlet crate graph depend on the SEA-Forge agent crate, or must the HTTP transport be reimplemented? | ASSUMPTION | open |
| **U5** | What is the real bounded-question latency and model-swap cost on the target appliance? | ASSUMPTION | open — **also required for claim level B** (C7) |
| **U6** | Which noun will carry probabilistic judgment, and how will the existing deterministic judgment usages be named? | DECISION_REQUIRED | open, **blocking** |
| **U7** | Does uncertainty carry incremental information on the first surface? | EVIDENCE_REQUIRED | open — **scope is `surface`, not the capability class** (C2) |
| **U8** | Which materially different shape should be examined if the first surface shows no gain? | DECISION_REQUIRED | **new** (C2) — must be a different shape from the first |
| **U9** | Are the named loop components intended to be connected, or intentionally out of the runtime path? | DECISION_REQUIRED | open |

---

## 6. Is PROOF-1 now safe to execute without prematurely constraining future architecture?

**Yes — PROOF-1 is now safe to execute LATER, on four conditions. It is still NOT authorised by this task.**

**Why it is safe.** Every property that could have made it architecture-binding has been removed or made conditional.

| Risk that existed in v0.2.0 | Why it is now contained |
|---|---|
| A null result would have been read as falsifying probabilistic judgment generally | Outcome C is scoped to `surface` and requires a materially different shape before any wider conclusion (C2, `REQ-JUDG-023`) |
| A null result would have collapsed the spec | Outcome C has its own bounded conclusion and next step; outcomes A, B and E carry **no** judgment-quality inference at all (C10) |
| The experiment would have silently chosen the universal carrier | The carrier decision is explicitly **slice-scoped**, and a future carrier requires only a demonstrated insufficiency rather than a redesign (C1, `NONGOAL-008`) |
| The experiment would have committed the answer domain to CEP physically | The requirement is decoupled from the carrier; three carriers are acceptable for slice 1 (C9, `REQ-JUDG-024`) |
| Small-sample numbers would have been published as calibration | The report is a pilot measurement with a mandatory disclosure block and reserved terminology (C3, C11, `REQ-VERIFY-012`) |
| The naming collision would have been resolved by accident inside the experiment | U6 remains open and blocking, independent of PROOF-1 (C1 table, `Judgment` entity) |
| The experiment would have implied a correlation mechanism commitment | The mechanism is implementation-defined; only a lossless joinability test is required (C6, `REQ-PROV-022`) |

**Four conditions that must hold before PROOF-1 is executed.**

1. **U6 is decided.** The probabilistic-judgment noun must be chosen before any type is written. Executing PROOF-1 first would create a de facto naming decision inside a measurement harness, which is exactly the accidental commitment this correction set exists to prevent.
2. **The first surface is selected and recorded** together with its judgment shape (binary/proposition, multiclass choice, or ordered score), so that outcome C's scope is determinable at the time of the result rather than argued later.
3. **The second surface is nominated in advance**, from a *different* shape, so that a null result on the first cannot be used to close the question by exhaustion.
4. **PROOF-1 remains read-only.** No production code path, no schema change, no persisted-data mutation. As of this correction it is still `DEFINED BUT NOT AUTHORISED`.

**What PROOF-1 still cannot decide, and must not appear to decide:** whether uncertainty-preserving judgment is valuable as a system capability; whether any external service is better at judgment; whether the answer domain belongs in CEP; whether the slice-1 carrier will serve non-verification families; whether the stack is production ready.

**Useful even on failure.** If PROOF-1 returns A (history not reconstructible), the deliverable is a provenance and correlation finding — which is currently **blocker B3** in the review's production assessment and would be valuable on its own. If it returns B, the deliverable is a structured-output and contract-mechanics finding. If it returns C, the deliverable is a correctly scoped negative result plus a nominated next shape. If it returns E, the deliverable is a provider-disagreement diagnostic. **In all five outcomes the task produces a decision**, which is what makes it a legitimate next experiment rather than a bet.

---

## 7. Final check — independent verification

Each required statement was verified against the corrected artifacts. Verification method: reading the v0.3.0 spec and v2 delta, plus mechanical ID and YAML checks.

| # | Required statement | Holds | Where it is now true |
|---|---|---|---|
| 1 | `VerificationRecord` is the first carrier, not asserted as the eternal universal judgment abstraction | ✅ | `JudgmentObservation.universality_clause` (status `NOT_ESTABLISHED`); `rejected_or_merged_ledger` (`scope_of_this_decision`, `permanence`); `NONGOAL-008`; roadmap gate `judgment_carrier_generalization` |
| 2 | One failed probabilistic surface cannot falsify the whole capability class | ✅ | `ScopeOfFalsification` entity; `REQ-JUDG-023`; `REQ-OUT-006`/`REQ-OUT-007`; `proof_1_definition.admissible_outcomes[C]` |
| 3 | The pilot does not claim calibration | ✅ | `pilot_measurement_metrics.must_not_claim`; `REQ-VERIFY-012`; `REQ-VERIFY-004` (measurability only); `CLAIM-003.explicitly_not_claiming`; roadmap gate `judgment_calibration` |
| 4 | DomainForge remains sole semantic authority | ✅ | `REQ-AUTH-006`; `dual_basis_requirement.semantic_basis.owner`; `NONGOAL-010` |
| 5 | Evaluation context cannot silently become semantic authority | ✅ | `REQ-AUTH-006`; `NONGOAL-010`; `REQ-SEC-004.reinforced_by`; `dual_basis_requirement.break_rule` |
| 6 | `.sea` grammar remains unchanged | ✅ | roadmap `DomainForge-compiled DecisionSurfaces.current_requirement`; `correction_ledger` C8 |
| 7 | No new judgment service, database, or provider port is required | ✅ | `architectural_concern.what_it_is_not`; `NONGOAL-007`; `NONGOAL-009`; `rejected_or_merged_ledger[JudgmentProvider]` |
| 8 | Loop-wide correlation remains first-class | ✅ | `REQ-PROV-021` (`blocking: true`); `REQ-PROV-022`; `LoopCorrelationIdentity` entity |
| 9 | Jev compatibility is separated into interface, operational, and model-quality claims | ✅ | `CLAIM-006.levels` A/B/C; roadmap `TypeSafe Jev provider.comparability_levels` |
| 10 | CEP physical placement of `AnswerDomain` remains contingent on compatibility evidence | ✅ | `REQ-JUDG-024`; `Question.compatibility.carrier_status = OPEN`; U1 status `OPEN - deliberately not settled by this specification` |
| 11 | PROOF-1 can produce a useful result even if the original judgment hypothesis fails | ✅ | outcomes A/B/C/E each carry a bounded conclusion and a next step; `REQ-OUT-007`; section 6 above |
| 12 | The specification still reflects the repository-grounded reduction discovered by the adversarial review | ✅ | `correction_ledger.net_effect_on_the_reduction`; `architectural_conclusions_reversed: none`; 13-row preservation table in section 4 |

**Mechanical checks run:**

```text
v0.3.0 spec      : YAML parses; 34 top-level keys; 10 stable entities
baseline REQ ids : 57 total, MISSING from v2: NONE
added REQ ids    : 15 relative to the baseline
                   (8 carried from v0.2.0: JUDG-020/021/022, PROV-020/021, COMP-020,
                    VERIFY-010/011; plus JUDG-023/024, AUTH-006, PROV-022,
                    VERIFY-012, OUT-007 new here; OUT-006 amended)
ID collisions    : NONE (JUDG-010..014 and CONFIG-010..012 are occupied by the baseline)
declared vs actual additions: consistent (10 items in the correction ledger)
VAR/RECOV/NONGOAL/CLAIM families: all present and enumerated; 10 non-goals; 6 claims
self_check statements in spec: 12/12 hold = true
structural assertions: 70 run, 0 failures (script checked the parsed structure, not prose)
dangling references: none (entity, roadmap, requirement, NONGOAL and CLAIM references resolve)
v2 delta         : YAML parses; 11 corrections; 11 entities; 14 requirement groups; 15 propositions
v2 diagram       : 1 subgraph balanced by 1 end; no duplicate node definitions; all edges resolve
PROOF-1          : DEFINED BUT NOT AUTHORISED
implementation   : not authorised
baseline         : unchanged (mtime 2026-09-16 21:21:46, predating this task) and still authoritative
template         : unchanged (mtime 2026-08-23)
production code  : not modified
```

**Conclusion.** All twelve required statements hold. No architectural conclusion from the adversarial review was reversed. No baseline requirement was weakened or renumbered. Every correction narrows a claim or a scope. The proposal remains a **proposal**: `.agents/specs/GODSPEED_JUDGMENT_PLANE_SPEC.yaml` v0.1.0 stays authoritative until the v0.3.0 proposal is accepted, and neither PROOF-1 nor any implementation work is authorised by these artifacts.
