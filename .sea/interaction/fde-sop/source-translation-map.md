# Source translation map: GS-FDE-SOP-001 v0.4.1 to the declared model

Source: `docs/sop/GS-FDE-SOP-001-v0.4.1.md` (draft, pending owner ratification
OD-2; sha256 in `docs/sop/README.md`). Model: `fde-sop.sea`, `evidence.sea`,
`outcome.sea` in this directory, world `godspeed-fde-sop`.

One row per numbered clause. `scripts/check-fde-sop-map.sh` (AT-02) fails if a
numbered clause has no row, a row's excerpt is not a verbatim substring of the
SOP, a named element does not exist in the model, or an `unknown` row lacks a
reason or a recorded limitation (`validation/limitations.md`).

Columns: **Clause** is the SOP number (`§n` is a section heading; a letter
suffix splits one clause; `6/Name` and `8.1/Name` are table rows inside that
clause). **Excerpt** is verbatim SOP text. **Status**: `declared` means the SOP
states it and the model expresses it; `inferred` means the model expresses it by
interpretation or only in part (the note says what); `unknown` means it is not
expressed and the limitation says why. **Limits** names recorded limitations the
row depends on. A row marked S2 is scope-checked but not evaluated by
`validate`; its verdict is `insufficient` (`not_evaluable`) until the DomainForge
evaluator (WP-12) runs it, and must never be reported `confirmed` from this map.
Section rows (`§n`) are headings: the status describes the heading only, the
content is in the `n.m` rows.

| Clause | Excerpt | Model element(s) | Status | Limits | Reason or note |
|---|---|---|---|---|---|
| §1 | Purpose and governing rule | namespace:godspeed-fde-sop | inferred | - | Heading; content is in 1.x rows. |
| 1.1 | The unit of work is a Field Case: a bounded, versioned, purposeful case | entity:FieldCase, role_binding:FDE | declared | - | `FieldCase` is the typed case record with lifecycle. |
| 1.2 | pick the next move that is useful, presently affordable, authorized, executable, recoverable and independently settleable | - | unknown | S5 | A selection rule for people; the seven comparison dimensions are declared under 7.2a, the rule itself has no predicate. |
| 1.3 | An observed difference is information, not automatically a defect | - | unknown | S5 | Interpretive guidance; no checkable predicate. |
| 1.4a | a defensible case disposition | entity:Disposition, operation:record_disposition | declared | - | Disposition is a recorded, authorized decision. |
| 1.4b | inspectable evidence | operation:nav_claim, entity:EvidenceSettlement | inferred | - | Every operation carries `evidence operation_trace`; "inspectable" is not itself checkable. |
| 1.4c | an explicit account of what the experience changes, or does not change, for later work | - | unknown | S7 | The SOP fixes no structure for this account; not declared in this version. |
| 1.5 | Four field tests apply to any proposed loop | - | unknown | S5 | Heuristic tests, no decidable predicate. |
| §2 | Four layers | entity:Layer, policy:four_layers | declared | - | The layer table is four `Layer` instances. |
| 2.1a | A Field Case is an operational projection of case navigation, not a new ontology primitive | - | unknown | S3 | A negative claim about ontology status; not expressible. |
| 2.1b | An episode is a bounded inquiry or intervention bound to a predeclared evaluation reference | entity:Episode, entity:Assignment | inferred | - | Episode carries identity, kind and `world_ref`; Assignment carries authority and budget; scope, evidence and handoff of an assignment are not declared. |
| 2.2a | Vocabulary fixed by this version | entity:Framing, entity:Projection, entity:CaseHorizon, entity:Residual, entity:Residue, entity:Checkpoint | inferred | - | Terms are declared as schemaless entities; their definitions stay in the SOP. |
| 2.2b | Settle is the epistemic result of an episode | pattern:EpistemicResult, enum:CaseLifecycle, entity:Disposition, operation:nav_claim, operation:record_disposition | declared | - | Settle, dispose and close map to nav_claim, record_disposition and CaseLifecycle. |
| 2.2c | A case has n episodes; an episode has n runs and n assignments; an assignment has n sessions | entity:Episode, entity:Assignment, entity:Run, entity:Session, policy:episode_key_unique | inferred | S2, S4 | Case, episode and assignment links are string fields. The key (case, plan item, n) comes from design constraint I-3 (E 3), not the SOP; uniqueness is a caller-supplied count. |
| 2.2d | Verifier (independently observes and settles), approver (authorizes an action or promotion) and disposition authority (chooses the §7.5 disposition) are separate roles | role:IndependentVerifier, role:Approver, role:DispositionAuthority, relation:VerifierIndependence, relation:ApprovalAuthority, relation:DispositionChoice | declared | - | Three roles and three relations. |
| §3 | Roles, separation and component responsibilities | namespace:godspeed-fde-sop | inferred | - | Heading. |
| 3.1 | criterion author (what counts, before an evaluated episode) | role:FDE, role:Sponsor, role:CriterionAuthor, role:ExecutionActor, role:IndependentVerifier, role:DomainPolicyCustodian, role_binding:FDE, role_binding:Sponsor, role_binding:CriterionAuthor, role_binding:ExecutionActor, role_binding:IndependentVerifier | declared | L3 | Six roles bound to the entity they act on. DomainPolicyCustodian is bound to no entity (the SOP names none); no responsibility kind is expressible. |
| 3.2a | the evaluated executor never rewrites frozen criteria | policy:executor_never_rewrites_criteria, operation:nav_aim | declared | S2 | Precondition: the principal freezing a reference is not the executor. |
| 3.2b | never attests its own independent acceptance | policy:no_self_settlement, operation:nav_claim | declared | S2, L2 | I-2. Precondition: settler is not the executor. Delegation-chain containment is a list and is not expressible. |
| 3.2c | independent settlement is performed by a fresh verifier | policy:fresh_verifier_in_single_person_case | inferred | S2, S5 | Only the recorded attestation is checkable; whether the context is genuinely fresh is not. |
| 3.2d | one person may hold several roles where stakes allow | - | unknown | S5 | "Where stakes allow" is a judgement; no role-exclusivity predicate is declared. |
| 3.3a | The FDE owns (B:350-364): problem discovery, problem-class identification, vocabulary capture | - | unknown | L3 | An ownership list needs a responsibility kind on the binding. |
| 3.3b | The FDE does not own implementation detail or delivery mechanics | - | unknown | L3 | A negative responsibility is not expressible. |
| 3.4 | Components (A:38-48, authoritative over B:791-837) | entity:Component, policy:ten_components | declared | - | Ten `Component` instances. |
| 3.5 | Model provider does not determine authority or evidence | - | unknown | S3 | A negative claim; freshness of reviewer context is a fact about review sessions, outside the model. |
| 3.6 | Verify the current authoritative case interface before implementation | - | unknown | S5 | An instruction to implementers, not a domain rule. |
| 3.7 | DE and ADE specialists may own integrations, execution substrates, agents and evaluations | role:DEADESpecialist, role_binding:DEADESpecialist | inferred | L3 | Role and binding declared; ownership and the non-transfer of FDE accountability are not expressible. |
| §4 | Opening or resuming a Field Case | namespace:godspeed-fde-sop | inferred | - | Heading. |
| 4.1 | The FDE opens or retrieves the case in Horizon and confirms it against the authoritative case record | operation:case_open_or_resume, role:FDE | declared | - | A read operation over `FieldCase`. |
| 4.2 | Minimum case content | - | unknown | S7, L2 | About 60 fields in six groups, several list-valued; not declared in this version. |
| 4.3 | A reopened case must be continuable without the previous worker's memory | - | unknown | S3, S5 | A completeness and negative property of records; not checkable. |
| 4.4 | make the smallest useful strategy decision | - | unknown | S3 | "Before substantive work" is an ordering rule; no strategy record is declared. |
| 4.5 | Intake heuristics (B:506-520, 1939-2007) | - | unknown | S5 | Heuristics with no decidable predicate. |
| §5 | Payment | entity:PaymentCurrency | declared | - | Currency vocabulary only; see 5.1 to 5.3. |
| 5.1 | Every candidate move is priced in separate currencies, never one blended score: time, attention, money, risk, recovery | entity:PaymentCurrency, policy:five_payment_currencies, metric:payment_currency_count | declared | S3 | Five currencies declared; "never one blended score" is a negative and is not enforced. |
| 5.2 | Price the non-obvious items explicitly | - | unknown | S5 | Prescriptive guidance with no decidable predicate. |
| 5.3 | an unmeasured currency is shown as unknown, never zero | - | unknown | S6, S3 | Needs recorded measurements; "never zero" is a negative. |
| §6 | Delivery activities (not chronological) | entity:DeliveryActivityDef, enum:DeliveryActivity, policy:ten_delivery_activities, metric:delivery_activity_count | declared | - | Ten activities; no precedence between operations is declared, matching "not chronological". |
| 6/Embed | establish sponsor decision, setting, users, incentives, constraints, permission | operation:act_embed, instance:activity_embed, policy:activity_is_embed | declared | S2 | The actor is FDE by assumption (A-5). |
| 6/Observe | examine work-as-done: exceptions, workarounds, interruptions, handoffs | operation:act_observe, instance:activity_observe, policy:activity_is_observe | declared | S2 | As Embed. |
| 6/Map | reconstruct interactions, transitions, decision points, costs, failure paths | operation:act_map, instance:activity_map, policy:activity_is_map | declared | S2 | As Embed. |
| 6/Artifactize | capture minimum reviewable meaning in a validated IDM/.sea with source mappings and unknowns | operation:act_artifactize, instance:activity_artifactize, policy:activity_is_artifactize | declared | S2 | As Embed. This model is itself an Artifactize output. |
| 6/Prototype | exercise the smallest admitted path that distinguishes hypotheses or produces value | operation:act_prototype, instance:activity_prototype, policy:activity_is_prototype | declared | S2 | As Embed. |
| 6/Validate | compare against predeclared expectations | operation:act_validate, instance:activity_validate, policy:activity_is_validate, policy:validate_against_predeclared_expectation | declared | S2 | Precondition: the expectation was predeclared. |
| 6/Deploy | bind to real systems, actors, permissions, capacity, monitoring, reversibility | operation:act_deploy, instance:activity_deploy, policy:activity_is_deploy | declared | S2 | As Embed. |
| 6/Instrument | semantic observation throughout | operation:act_instrument, instance:activity_instrument, policy:activity_is_instrument | declared | S2 | As Embed. |
| 6/Learn | diagnose residuals: observation, reference, inference, implementation, or structure | operation:act_learn, instance:activity_learn, policy:activity_is_learn | declared | S2 | As Embed. |
| 6/Compound | preserve what later work can reuse and test its portability | operation:act_compound, instance:activity_compound, policy:activity_is_compound | declared | S2 | As Embed. |
| 6.1 | Instrumentation starts as soon as an episode needs evidence | - | unknown | S3 | A start-time trigger; temporal. |
| 6.2 | Episode sizing (B:2087-2097): prefer one team, one recurring problem, one loop | - | unknown | S5 | Prescriptive guidance with no decidable predicate. |
| 6.3 | Residue to capture (B:765): traces, examples, corrected outputs, evaluation cases, domain rules, decisions, policies, lessons | entity:Residue | inferred | L2 | The concept is declared; the enumerated kinds are a list and are not typed. |
| §7 | The bounded decision and episode procedure | operation:nav_listen, operation:nav_aim, operation:nav_move, operation:nav_claim | inferred | - | Heading; the four navigation operations are rows 7.1 to 7.5. |
| 7.1a | Read the case, world_ref, checkpoint, decision history and reliable observations | operation:nav_listen, entity:NavListen, entity:Checkpoint | declared | - | A read operation; decision history and observations are not returned fields. |
| 7.1b | Separate direct observation, interpretation, disputed claim and missing evidence | - | unknown | S5 | Prescriptive guidance with no decidable predicate. |
| 7.2a | Compare candidate paths on seven dimensions: visibility, reach, payment, executable capability, authority, recovery, settlement access | entity:AimDimension, policy:seven_aim_dimensions | declared | - | Seven `AimDimension` instances. |
| 7.2b | Show blocked or unknown possibilities as such | - | unknown | S5 | Presentation rule. |
| 7.2c | versioned intention, expected change, invariants, predicted irrelevancies, failure and success regions, falsifiers, evidence source, authorized decision-maker | - | unknown | L2, S7 | Eight-part list-valued frozen content; Episode does not carry it. |
| 7.2d | Freeze the episode reference and its hash | operation:nav_aim, entity:Episode | inferred | S4 | The freeze is the nav_aim operation; the digest is part of the episode_ref (E 2), not a separate SOP field. |
| 7.2e | fix before execution: versioned intention | policy:freeze_before_move, operation:nav_move | declared | S2 | I-1. For an evaluated episode, a move requires a recorded freeze. |
| 7.2f | An exploratory observation is labelled exploratory at the time, never relabelled a passing test | enum:EpisodeKind, entity:Episode | inferred | S3 | The label is the `kind` field; "never relabelled" is immutability. |
| 7.3a | Bind the assignment to case, world, criterion, authority and budget | operation:nav_move, entity:Assignment | declared | - | All five bindings are required, non-empty input fields. |
| 7.3b | Obtain SEA-Forge authorization at the action boundary for consequential actions | policy:authorize_consequential_action | declared | S2 | Caller supplies `authorization_recorded`. |
| 7.3c | Route deep agent work through Cognate/Workbench and, where needed, Gauntlet and SWE_SEED | - | unknown | S5 | Architectural routing; no operation-level predicate. |
| 7.3d | Capture execution, cost, external effects, denials and interruptions | flow:ExecutionTrace, operation:nav_move | inferred | - | The ExecutionTrace flow and the operation_trace evidence obligation; the captured fields are not typed. |
| 7.3e | a change to success criteria, governing meaning or policy starts a new authorized evaluation version | policy:criteria_change_needs_new_version | inferred | S2 | Covers success criteria only; meaning and policy changes are not modeled. |
| 7.4 | Assignment brief (B:2238-2264): problem the agent may adopt; domain meaning required | - | unknown | L2, S7 | A ten-part list-valued record; not declared. |
| 7.5a | RealityTrace links the frozen expectation to independent observation | operation:nav_claim, entity:EvidenceSettlement | declared | - | Input requires `episode_ref` and `observation_ref`. |
| 7.5b | Classify the epistemic result S established success, F established failure, U unresolved, C contradictory | pattern:EpistemicResult, entity:EvidenceSettlement | declared | - | Closed to S, F, U, C by pattern. |
| 7.5c | Record settlement reliability and source independence separately | entity:EvidenceSettlement | declared | - | Two separate required fields. |
| 7.5d | The authorized actor chooses close-satisfied, close-failed, continue, revise, escalate or suspend | enum:DispositionKind, operation:record_disposition, role:DispositionAuthority | declared | - | Closed enum. |
| 7.5e | Update case, residuals, next affordances, handoff | - | unknown | S1 | An after-effect of disposition; postcondition enforcement is rejected. |
| 7.5f | A completed tool call is not a settled outcome | enum:ExecutionStatus, pattern:EpistemicResult, policy:episode_level_never_closes_case | inferred | S3 | Execution status and epistemic result are separate types; "not automatically" is a negative. |
| 7.6a | Four status axes stay distinct | enum:CaseLifecycle, enum:ExecutionStatus, pattern:EpistemicResult, enum:DispositionKind | declared | - | One type per axis. |
| 7.6b | close-satisfied and close-failed set lifecycle closed | policy:close_disposition_sets_closed, policy:suspend_sets_suspended, policy:continue_revise_escalate_leave_open | declared | S2 | Three policies on record_disposition, case-level only. |
| 7.6c | Episode-level outcomes never close a case | policy:episode_level_never_closes_case | declared | S2 | Precondition on level and lifecycle_after. |
| 7.6d | Lifecycle closed requires a recorded case-level close disposition and a terminal case record | policy:closed_only_by_case_level_close, policy:closed_requires_terminal_record | declared | S2 | The terminal record is a caller-supplied flag. |
| 7.7 | Reliability dimensions, applied only where the evaluation contract sets thresholds | entity:ReliabilityDimension, policy:seven_reliability_dimensions | declared | S6 | Seven dimensions; thresholds are evaluation-contract data, not declared. |
| §8 | Nine evidence gates | entity:EvidenceGate, enum:EvidenceGateName, policy:nine_evidence_gates, metric:evidence_gate_count | declared | - | Nine `EvidenceGate` instances. |
| 8.1a | (with authorized rationale) | enum:GateState, entity:GateAssessment, operation:assess_gate, policy:not_applicable_needs_authorized_rationale | declared | S2 | Four gate states; not_applicable needs an authorized rationale. |
| 8.1b | Not applicable never waives a hard governance condition | policy:not_applicable_never_waives_hard_condition | declared | S2 | Precondition on the gate assessment. |
| 8.1c | Reassess when supporting evidence changes. No automatic advance | - | unknown | S3 | A temporal trigger and an absence of automatic transitions. |
| 8.1/Choose Problem | sponsor-owned need, JTBD outcome, plausible value, scope | instance:gate_choose_problem | declared | - | Gate row. |
| 8.1/Discover Reality | work-as-done observations, baseline, exceptions, competing interpretations, missing knowledge | instance:gate_discover_reality | declared | - | Gate row. |
| 8.1/Allocate Authority | roles, scope, policy, approval, recovery ownership, permission boundaries | instance:gate_allocate_authority | declared | - | Gate row. |
| 8.1/Prove Value | frozen hypothesis and evaluator, representative observation, baseline comparison, measured actual payment | instance:gate_prove_value | declared | - | Gate row. |
| 8.1/Prove Reliability | independent repeated settlement under variation, edge cases, recovery, known limits | instance:gate_prove_reliability | declared | - | Gate row. |
| 8.1/Integrate/Govern | approved real connections, authority enforcement, observability, rollback, security, operational owner | instance:gate_integrate_govern | declared | - | Gate row. |
| 8.1/Earn Trust | actual operator/client understanding, informed adoption, escalation and recovery behaviour | instance:gate_earn_trust | declared | - | Gate row. |
| 8.1/Measure | benefit and all payment currencies, debt, externalities, confounders, limits | instance:gate_measure | declared | - | Gate row. |
| 8.1/Productize | versioned pattern, transfer hypothesis, repeatability/variation/recovery evidence, business case | instance:gate_productize | declared | - | Gate row. |
| 8.2 | A case may end with the evidenced decision not to deploy, or reopen discovery after a promising pilot | - | unknown | S5 | A permission with no restriction to express. |
| 8.3 | Measure evidence includes: benefit, time, money, attention, risk, coordination, debt and externalities | instance:gate_measure | inferred | L2 | The gate's minimum evidence is prose; the list is not typed. |
| §9 | Error, structural revision and developmental memory | namespace:godspeed-fde-sop | inferred | - | Heading. |
| 9.1 | Two comparisons, never merged | - | unknown | S3, S7 | Episode and structural comparison need observed representations; no comparison operation is declared, and "never merged" is a negative. |
| 9.2 | Diagnose a consequential difference as: observation/instrumentation problem, mistaken reference, unsupported inference | enum:DifferenceDiagnosis | inferred | - | The six diagnoses are an enum; no operation records one. |
| 9.3 | Recurrence triggers for structural review (B:1757-1764) | - | unknown | S3 | Triggers are recurrence over history. |
| 9.4 | A structural revision records: candidate semantic change, rationale, retained invariants, migration and recovery path | - | unknown | L2, S7 | A seven-part record; promotion of a new `world_ref` is outside the `.sea` model. |
| 9.5 | A reusable asset is a candidate until it survives variation and transfer | operation:act_compound | inferred | L7 | The activity exists; the candidate to reusable transition is not expressible. |
| §10 | Continuity, handoff and disposition | namespace:godspeed-fde-sop | inferred | - | Heading. |
| 10.1 | Case continuation, derived from source-owned records | - | unknown | L2, S7 | A computed read of many fields; case_open_or_resume returns three. |
| 10.2 | Receiver-specific handoff (B:326, 328) | - | unknown | L2, S7 | Two receiver-specific lists; not declared. |
| 10.3 | Close only by authorized disposition, with provenance, economic/outcome account, remaining liabilities and recoverability | policy:closed_only_by_case_level_close, operation:accept_client_outcome, entity:OutcomeSettlement | inferred | S2, S7 | Closure only through a case-level close is declared; the outcome account is `account_ref`; provenance, liabilities and recoverability are not declared. |
| 10.4 | The reusable handoff is: case summary, semantic references, evidence bundle, decision record | - | unknown | L2, S7 | A six-part list; not declared. |
| §11 | Acceptance of this SOP: first internal Field Case | entity:AcceptanceRequirement, policy:eight_acceptance_requirements | declared | - | Eight requirement instances; PROVEN, PARTIAL or BLOCKED outcomes are runtime results. |
| 11#1 | reconstruct or validate the initial IDM from this SOP and its source evidence | instance:acceptance_1 | declared | - | Statement recorded. |
| 11#2 | resume the case from authoritative records and select a real or blocked affordance | instance:acceptance_2 | declared | - | Statement recorded. |
| 11#3 | run a bounded governed inquiry against an expectation frozen beforehand and inspect execution and evidence | instance:acceptance_3 | declared | - | Statement recorded. |
| 11#4 | derive an observed interaction representation, compare it with the declared version | instance:acceptance_4 | declared | - | Statement recorded. |
| 11#5 | independently settle an episode, authorize its disposition | instance:acceptance_5 | declared | - | Statement recorded. |
| 11#6 | where justified, validate and authorize a candidate model and test a subsequent episode under it | instance:acceptance_6 | declared | - | Statement recorded. |
| 11#7 | inspect the same case, differences, authority and evidence through Horizon and programmatic records without inconsistent verdicts | instance:acceptance_7 | declared | - | Statement recorded. |
| 11#8 | transfer the templates and instrumentation to a second bounded situation | instance:acceptance_8 | declared | - | Statement recorded. |
| 11.1 | without reconstructing hidden context | instance:acceptance_2, operation:case_open_or_resume | declared | - | The expanded wording is in the acceptance_2 statement. |
| §12 | Records and owners | entity:RecordOwner, policy:six_record_owners | declared | - | Six `RecordOwner` instances; the closing sentence that projections never replace validated `.sea` is a negative (S3). |
| §13 | Conflicts resolved in this synthesis | namespace:godspeed-fde-sop.evidence, namespace:godspeed-fde-sop.outcome | inferred | - | A resolution log. Its settlement row is embodied by the two non-unifying settlement namespaces; the other rows are history and are not modeled. |
