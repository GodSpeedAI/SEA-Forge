# SEA Forge Journey & Capability Map

This map connects the twelve canonical journeys to their invoked capabilities,
their evidence, and their interface bindings. It is built entirely from
existing canonical source — `canonicalization-matrix.csv`,
`canonical-journey-catalog.md`, `journey-grammar.md`, and the
`InterfaceProjection` instances in `interaction-model.sea` — per the
[Journey & Capability Map Handoff](README.md#journey--capability-map-handoff).
It performs no new canonicalization: every `CJ01`–`CJ12` identity, story
mapping, classification, maturity value, and interface binding below is read
from source, not re-derived. The interface groupings were extracted
programmatically from the 38 `InterfaceProjection` instances' declared
`canonical_journey_ids` fields, not assembled by hand.

It is deliberately **not** built from RDF projection. `domainforge-core`
0.16.0 still drops all authored `Instance` and `Policy` declarations from
Turtle, JSON-LD, and OWL output (verified 2026-08-03 against this repository's
own `interaction-model.sea`: zero of the 76 instance identifiers and zero of
the 2 policies appear in any projected format; see `validation/limitations.md`
§10). RDF remains usable only for the ontology spine — entities, roles,
resources, and untyped flow edges — so it cannot carry journey identity,
capability, or interface detail. This map exists precisely because that gap
has to be closed from SEA source and the matrix, not from a projection.

Like the other companion files, this map is a declaration-free index. It adds
no new `.sea` declarations and is not a validation input.

## Summary

| CJ | Journey | Stories | Web UI | API | CLI | Agent | Maturity (spec / impl / exercised / evidenced) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| CJ01 | Establish Trusted Cell Context | 15 | 2 | 2 | 1 | 0 | 1 / 2 / 5 / 7 |
| CJ02 | Discover Lawful Affordances | 16 | 5 | 5 | 3 | 2 | 4 / 1 / 9 / 2 |
| CJ03 | Ground Work in Semantic Meaning | 7 | 1 | 0 | 1 | 0 | 1 / 0 / 6 / 0 |
| CJ04 | Form and Commit a Governed Case | 10 | 1 | 2 | 2 | 2 | 1 / 0 / 6 / 3 |
| CJ05 | Navigate and Adapt a Live Case | 11 | 1 | 1 | 1 | 1 | 0 / 0 / 10 / 1 |
| CJ06 | Resolve Human Judgment and Approval | 7 | 2 | 2 | 1 | 1 | 2 / 0 / 3 / 2 |
| CJ07 | Execute Governed Work | 6 | 1 | 1 | 2 | 2 | 0 / 0 / 3 / 3 |
| CJ08 | Monitor, Intervene, and Recover | 12 | 5 | 5 | 4 | 2 | 1 / 1 / 9 / 1 |
| CJ09 | Evaluate, Settle, and Audit Outcomes | 17 | 5 | 4 | 5 | 3 | 1 / 0 / 11 / 5 |
| CJ10 | Reuse Demonstrated Knowledge and Capability | 10 | 2 | 1 | 2 | 1 | 2 / 0 / 8 / 0 |
| CJ11 | Transform and Mature Governed Artifacts | 10 | 2 | 0 | 2 | 0 | 0 / 1 / 9 / 0 |
| CJ12 | Transfer and Adopt Governed Assets | 7 | 1 | 0 | 1 | 0 | 0 / 1 / 6 / 0 |

Story, classification, and maturity counts are read directly from
`canonicalization-matrix.csv` (128 rows, reconciled in `coverage-report.md`).
Interface counts are read directly from the 38 `InterfaceProjection` instances
in `interaction-model.sea`; a binding that names more than one journey in
`canonical_journey_ids` is counted once per journey it names, so column totals
exceed 38.

## CJ01 — Establish Trusted Cell Context

- **Intention / job:** Enter a cell whose identity, configuration, integrity,
  self-model, and readiness are explicit; establish a trustworthy starting
  context without overwriting history or treating stale state as current.
- **Completion condition:** The actor is attributable, governing snapshots and
  integrity state are explicit, and the cell states whether the intended
  operation is ready, degraded, stale, or blocked with evidence.
- **Invoked capabilities:** Cell initialization and bundle-aware migration,
  startup validation, `identity.get`, `readiness.get`, policy inspection,
  self-model validation or rebuild, lifecycle inspection, version-skew
  detection.
- **Interface bindings:**
  - Web UI: `workbench_readiness_routes`, `workbench_administration_route`
  - API: `sfwp_system_family`, `sfwp_context_family`
  - CLI: `cli_cell_and_self_model_family`
- **Evidence anchor:** Governed UX epic §§1, 2.1–2.4, 16.1–16.5; `spec-minimum.md`
  §§7.3.3, 8, 8.5, 10.2, 14; `spec-full.md` §§7.0b–7.0c, 7.4; cell,
  self-model, lifecycle, identity, readiness, version-skew conformance tests.
- **Coverage:** 15 stories — 1 canonical, 3 variant, 8 specialization, 3
  composition; maturity 1 specified, 2 implemented, 5 exercised, 7 evidenced.
  IDs: 1.1–1.7, 2.1–2.4, 16.1, 16.2, 16.3, 16.5.

## CJ02 — Discover Lawful Affordances

- **Intention / job:** Understand what exists, is usable, is allowed, and why
  a path is unavailable; choose a spendable path without confusing catalog
  presence with authority or settlement evidence.
- **Completion condition:** The user receives a source-backed set of
  currently visible, reachable, permitted, and settleable actions, including
  explicit blockers, uncertainty, freshness, and limitations.
- **Invoked capabilities:** `thoth.ask`, asset and endpoint inspection,
  endpoint probe, capability projection, environment and adapter inspection,
  authority preflight, denial explanation.
- **Interface bindings:**
  - Web UI: `workbench_readiness_routes`, `workbench_thoth_route`,
    `workbench_assets_route`, `workbench_delegation_route`,
    `workbench_administration_route`
  - API: `sfwp_system_family`, `sfwp_context_family`, `sfwp_thoth_family`,
    `sfwp_asset_family`, `sfwp_delegation_family`
  - CLI: `cli_semantic_and_environment_family`, `cli_thoth_family`,
    `cli_agent_family`
  - Agent: `agent_thoth_epistemic_surface`, `agent_endpoint_surface`
- **Evidence anchor:** Governed UX epic §§2.5–2.6, 3.1–3.6, 3.9, 4.1–4.6, 4.8;
  `spec-minimum.md` §10.2; `spec-full.md` §§7.0b, 7.3, 7.6, 10.0; Thoth,
  asset, endpoint, topology, authority, case-episode tests.
- **Coverage:** 16 stories — 1 canonical, 3 variant, 10 specialization, 2
  composition; maturity 4 specified, 1 implemented, 9 exercised, 2 evidenced.
  IDs: 2.5, 2.6, 3.1–3.6, 3.9, 4.1–4.6, 4.8.

## CJ03 — Ground Work in Semantic Meaning

- **Intention / job:** Bind work to validated, reproducible `.sea` domain
  meaning before authority or execution can spend it; keep derived output
  subordinate to source.
- **Completion condition:** All semantic references resolve against an exact
  validated model and adapter snapshot, or activation stays blocked with
  layer-specific diagnostics and a lawful repair path.
- **Invoked capabilities:** DomainForge parsing and semantic validation, model
  inspection, plan reference resolution, hash pinning, version-skew checks,
  deterministic projection, output validation.
- **Interface bindings:**
  - Web UI: `workbench_domain_models_route`
  - CLI: `cli_semantic_and_environment_family`
- **Evidence anchor:** Governed UX epic §5; `spec-minimum.md` §10.1;
  `spec-full.md` §§7.0a, 10.4a; DomainForge, planner, projection,
  criteria-provenance, version-skew, spec-pipeline conformance tests.
- **Coverage:** 7 stories — 1 canonical, 1 variant, 3 specialization, 2
  composition; maturity 1 specified, 6 exercised. IDs: 5.1–5.7.

## CJ04 — Form and Commit a Governed Case

- **Intention / job:** Turn intent, a template, or an untrusted proposal into
  a validated, previewed, immutable case plan before any execution side
  effect.
- **Completion condition:** The exact accepted plan, parameters, criteria,
  provenance, configuration digests, and model references are committed
  before execution.
- **Invoked capabilities:** Intent planning, template instantiation, proposal
  validation, topology construction, pipeline planning, delegation preview,
  `case.preflight`, `case.commit`.
- **Interface bindings:**
  - Web UI: `workbench_case_routes`
  - API: `sfwp_request_correlation_family`, `sfwp_case_authoring_family`
  - CLI: `cli_execution_family`, `cli_case_and_human_family`
  - Agent: `agent_thoth_manager_surface`, `agent_delegation_execution_surface`
- **Evidence anchor:** Governed UX epic §6, §10.3; `spec-minimum.md`
  §§7.3.1–7.3.2; `spec-full.md` §§7.6, 8.6, 10.2, 10.7; ADLC/Thoth and
  agent-orchestration specs; lifecycle, planner, case-authoring, topology,
  delegation-preview, pipeline, manager-loop tests.
- **Coverage:** 10 stories — 1 canonical, 2 variant, 5 specialization, 2
  composition; maturity 1 specified, 6 exercised, 3 evidenced. IDs: 6.1–6.9,
  10.3.

## CJ05 — Navigate and Adapt a Live Case

- **Intention / job:** Understand a committed case's purpose, horizon,
  history, and lawful adaptation paths without rewriting prior truth.
- **Completion condition:** The user can identify the governing outcome,
  current spendable and blocked work, evidence for every state, and each
  presently lawful action.
- **Invoked capabilities:** `case.get_overview`, `case.get_horizon`, run
  listing and detail, discretionary-item admission, reopen, replan,
  terminate, manager-loop judgment, ordinary plan mutation.
- **Interface bindings:**
  - Web UI: `workbench_case_routes`
  - API: `sfwp_case_navigation_family`
  - CLI: `cli_case_and_human_family`
  - Agent: `agent_thoth_manager_surface`
- **Evidence anchor:** Governed UX epic §7, §§10.1–10.2, 10.4; `spec-minimum.md`
  §9; `spec-full.md` §§7.1, 9; agent-orchestration spec §§7.6, 10.3, 17.4;
  case-view, run-view, run-locator, case-runner, topology, manager-loop tests.
- **Coverage:** 11 stories — 1 canonical, 3 variant, 5 specialization, 2
  composition; maturity 10 exercised, 1 evidenced. IDs: 7.1–7.8, 10.1, 10.2,
  10.4.

## CJ06 — Resolve Human Judgment and Approval

- **Intention / job:** Admit accountable human decisions and work into the
  governed record through the same authority and evidence fabric as other
  work.
- **Completion condition:** An eligible human decision or contribution is
  committed with actor, context, evidence, rationale, and downstream effect,
  or refusal/denial/expiry is recorded without unauthorized side effects.
- **Invoked capabilities:** `approval.list`, approval context inspection,
  `approval.decide`, human-task listing and completion, ACP permission
  handling, identity and separation-of-duty checks, case reevaluation.
- **Interface bindings:**
  - Web UI: `workbench_case_routes`, `workbench_inbox_route`
  - API: `sfwp_request_correlation_family`, `sfwp_approval_family`
  - CLI: `cli_case_and_human_family`
  - Agent: `agent_delegation_control_surface`
- **Evidence anchor:** Governed UX epic §8; `spec-full.md` §§7.2, 7.2.1, 10.2,
  10.3; agent-orchestration spec §10.4; approval, identity, ACP,
  human-waiting, CLI approval tests.
- **Coverage:** 7 stories — 1 canonical, 3 variant, 3 specialization; maturity
  2 specified, 3 exercised, 2 evidenced. IDs: 8.1–8.7.

## CJ07 — Execute Governed Work

- **Intention / job:** Perform human, command, projection, transition, or
  agent work only within the exact granted boundary, retaining evidence for
  later settlement.
- **Completion condition:** Execution terminates within the granted boundary
  and leaves complete outcome evidence for settlement, including governed
  denial or failure; only settlement may accept the work.
- **Invoked capabilities:** Authority decision, sandbox realization,
  environment provision, command and projection execution,
  `delegation.preview`, HTTP and ACP agent execution, SWE_SEED route and
  proof harvesting, artifact capture, settlement handoff.
- **Interface bindings:**
  - Web UI: `workbench_delegation_route`
  - API: `sfwp_delegation_family`
  - CLI: `cli_execution_family`, `cli_agent_family`
  - Agent: `agent_endpoint_surface`, `agent_delegation_execution_surface`
- **Evidence anchor:** Governed UX epic §§9.1–9.6; `spec-minimum.md` §§7.3.6,
  10.2, 10.3; agent-orchestration spec §§7.3, 10.2, 10.4, 17.5; authority,
  sandbox, delegation-preview, HTTP, ACP, SWE_SEED portable tests.
- **Coverage:** 6 stories — 1 canonical, 2 variant, 3 specialization;
  maturity 3 exercised, 3 evidenced. IDs: 9.1–9.6.

## CJ08 — Monitor, Intervene, and Recover

- **Intention / job:** Keep concurrent and interrupted work visible,
  controllable, and recoverable without changing committed meaning or
  affecting unrelated work.
- **Completion condition:** The operator sees authoritative operational
  standing and either completes a scoped intervention or reaches an explicit
  resume, retry, replan, repair, endpoint, environment, escalation, or
  terminal decision.
- **Invoked capabilities:** Delegation listing, `events.subscribe`,
  `events.get_range`, concurrency and capacity projection, scoped
  cancellation, case resume, restart reconciliation, ADLC reactivation,
  failure diagnosis, lifecycle-snapshot preservation, maintenance-debt
  inspection.
- **Interface bindings:**
  - Web UI: `workbench_delegation_route`, `workbench_case_routes`,
    `workbench_operations_route`, `workbench_evidence_and_run_routes`,
    `workbench_administration_route`
  - API: `sfwp_request_correlation_family`, `sfwp_event_family`,
    `sfwp_case_navigation_family`, `sfwp_run_record_family`,
    `sfwp_delegation_family`
  - CLI: `cli_execution_family`, `cli_run_inspection_family`,
    `cli_case_and_human_family`, `cli_agent_family`
  - Agent: `agent_thoth_manager_surface`, `agent_delegation_control_surface`
- **Evidence anchor:** Governed UX epic §9.7, §10.5, §11, §§16.6–16.7;
  `spec-minimum.md` §§14, 14.3; `spec-full.md` §8.4; ADLC/Thoth and
  agent-orchestration recovery rules; event, topology, concurrency,
  cancellation, lifecycle, run-locator, run-view, delegation, manager-loop
  tests.
- **Coverage:** 12 stories — 1 canonical, 4 composition, 7 specialization;
  maturity 1 specified, 1 implemented, 9 exercised, 1 evidenced. IDs: 9.7,
  10.5, 11.1–11.8, 16.6, 16.7.

## CJ09 — Evaluate, Settle, and Audit Outcomes

- **Intention / job:** Prove what was intended, authorized, attempted,
  observed, accepted, and promoted from immutable evidence.
- **Completion condition:** Every outcome or assurance claim resolves to
  attributable, integrity-checked evidence and an independent settlement or
  explicit unavailability; execution termination stays separately visible.
- **Invoked capabilities:** `run.get`, evidence and claim explanation,
  artifact and transcript verification, settlement evaluation, declaration
  weighting, provenance checks, ledger inclusion and consistency proof,
  deterministic replay, audit search, disclosure history, source-record
  access.
- **Interface bindings:**
  - Web UI: `workbench_thoth_route`, `workbench_delegation_route`,
    `workbench_operations_route`, `workbench_evidence_and_run_routes`,
    `workbench_administration_route`
  - API: `sfwp_event_family`, `sfwp_thoth_family`, `sfwp_run_record_family`,
    `sfwp_delegation_family`
  - CLI: `cli_run_inspection_family`, `cli_cell_and_self_model_family`,
    `cli_ledger_family`, `cli_thoth_family`, `cli_agent_family`
  - Agent: `agent_thoth_epistemic_surface`, `agent_delegation_execution_surface`,
    `agent_delegation_control_surface`
- **Evidence anchor:** Governed UX epic §§2.7, 3.7–3.8, 9.8–9.9, 10.6, 12,
  16.8; `spec-minimum.md` §§10.4, 10.6, 12.1, 14; `spec-full.md` §§7.0c,
  7.2.1, 10.4; agent-orchestration evidence and separation rules; run-view,
  settlement, capability, ledger, replay, manager separation tests.
- **Coverage:** 17 stories — 1 canonical, 2 variant, 11 specialization, 3
  composition; maturity 1 specified, 11 exercised, 5 evidenced. IDs: 2.7, 3.7,
  3.8, 9.8, 9.9, 10.6, 12.1–12.10, 16.8.

## CJ10 — Reuse Demonstrated Knowledge and Capability

- **Intention / job:** Reuse settled history and evidence-backed capability
  without allowing indexes, summaries, or stale projections to become
  authoritative.
- **Completion condition:** Reused knowledge or capability is
  disclosure-safe, traceable to source settlements, explicit about scope and
  weakness, and linked to the resulting plan, decision, answer, or
  settlement.
- **Invoked capabilities:** Governed memory recall, source-record fallback,
  influence tracking, capability derivation, promotion and contraction,
  capability browse and detail, deterministic rebuild, routing gates, Thoth
  claim derivation.
- **Interface bindings:**
  - Web UI: `workbench_thoth_route`, `workbench_unavailable_record_routes`
  - API: `sfwp_thoth_family`
  - CLI: `cli_memory_family`, `cli_thoth_family`
  - Agent: `agent_thoth_epistemic_surface`
- **Evidence anchor:** Governed UX epic §4.7, §13, §16.4; `spec-full.md`
  §§7.3, 7.5, 10.4, 10.5; ADLC/Thoth claim-derivation rules; capability,
  memory, CLI recall, index-deletion, projection tests.
- **Coverage:** 10 stories — 1 canonical, 2 variant, 4 specialization, 3
  composition; maturity 2 specified, 8 exercised. IDs: 4.7, 13.1–13.8, 16.4.

## CJ11 — Transform and Mature Governed Artifacts

- **Intention / job:** Derive, project, promote, or quarantine artifacts
  through explicit, evidence-backed maturity gates without skipping stages.
- **Completion condition:** Output is validated, hash-linked, and settled at
  its actual maturity, or quarantined with typed reasons; every transition
  preserves provenance and no file creation alone claims runtime readiness.
- **Invoked capabilities:** Deterministic DomainForge projection,
  spec-pipeline transformation, artifact registration and lineage
  inspection, gate evaluation, derive/promote transition, quarantine,
  separation-of-duty checks, `TransitionToken` validation.
- **Interface bindings:**
  - Web UI: `workbench_domain_models_route`,
    `workbench_unavailable_record_routes`
  - CLI: `cli_semantic_and_environment_family`, `cli_artifact_family`
- **Evidence anchor:** Governed UX epic §14; `spec-full.md` §§7.8a, 7.9,
  10.4a, 10.7, 10.8; DomainForge projection, spec-pipeline, artifact-IP, CLI
  artifact conformance tests; `ARCHITECTURAL_TRUTH.md` "What Usable Means."
- **Coverage:** 10 stories — 1 canonical, 4 variant, 5 specialization;
  maturity 1 implemented, 9 exercised. IDs: 14.1–14.10.

## CJ12 — Transfer and Adopt Governed Assets

- **Intention / job:** Move verifiable records and assets across cell
  boundaries while preserving provenance and withholding local usability
  until governed adoption.
- **Completion condition:** The transfer is rejected atomically or admitted
  with provenance intact and assets inert; a separately authorized adoption
  is required before local usability.
- **Invoked capabilities:** Bundle export and verification, atomic import,
  imported-history isolation, inactive-asset enforcement, manifest
  inspection, compatibility checks, governed template or environment
  adoption, bounded rejection.
- **Interface bindings:**
  - Web UI: `workbench_unavailable_record_routes`
  - CLI: `cli_federation_family`
- **Evidence anchor:** Governed UX epic §15; `spec-full.md` §§7.4, 10.9, 14;
  `sea-forge-cell` bundle and size-bound tests; CLI closeout tests; bundle
  manifest implementation.
- **Coverage:** 7 stories — 1 canonical, 1 variant, 5 specialization;
  maturity 1 implemented, 6 exercised. IDs: 15.1–15.7.

## Cross-cutting compositions

Per `journey-grammar.md`, three surfaces are compositions of the above twelve
rather than independent journeys, and are not given their own row:

- **Thoth** composes CJ02 (discovery), CJ09 (audit), CJ04 (bounded proposal
  admission), CJ05 (manager-loop adaptation), and CJ10 (capability/memory
  consultation). It creates no private authority, execution, approval,
  settlement, or plan-mutation channel.
- **Maintenance** composes CJ01 (self-model rebuild, compatibility), CJ10
  (derived-store repair), CJ08 (in-flight protection, interruption
  recovery, debt resolution), and CJ09 (assurance-label evidence).
- **Identity and authority** are cross-cutting steps invoked by every
  journey above rather than a thirteenth journey; they gate entry (CJ01),
  discovery (CJ02), and execution (CJ07) without separate identity.

## Boundaries this map does not cross

- It does not reassign any observed story to a different `CJ01`–`CJ12` — the
  matrix remains the row-level authority.
- It does not promote an unavailable, preview-only, or naming-drifted
  interface binding into a working capability. `workbench_unavailable_record_routes`
  spans CJ10–CJ12 precisely because those record surfaces are explicit
  unavailable routes, not implemented affordances — see "Meaning of
  Interface Bindings" in `README.md`.
- It does not treat this document, or any projection, as canonical. Identity
  lives in `interaction-model.sea`; this map is a navigable synthesis of it.
- If future evidence changes a normalized intention tuple, the canonical
  matrix and catalog must be updated first; this map is regenerated from
  them, never edited to diverge from source.
