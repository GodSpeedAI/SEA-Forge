# SEA Forge Interaction Model Completion Report

Date: 2026-08-03
Status: complete with documented DomainForge limitations

## Outcome and authority

The governed Workbench UX epic supplies 128 observed stories. The canonical
interaction source accounts for all of them with 12 human-intention journeys,
without changing application code, the SEA grammar, DomainForge, dependencies,
schemas, public interfaces, CI, or deployment.

Use the [interaction directory](../../.sea/interaction/README.md) as the source
authority. In particular, use the [128-row matrix](../../.sea/interaction/canonicalization-matrix.csv)
for exhaustive mappings, the [catalog](../../.sea/interaction/canonical-journey-catalog.md)
for human-readable journey semantics, the [grammar](../../.sea/interaction/journey-grammar.md)
for composition rules, and the committed `cc9e3f5` version of
[interaction-model.sea](../../.sea/interaction/interaction-model.sea) for
machine-readable evidence. Reports summarize those artifacts; they do not fork
them.

The working copy of `interaction-model.sea` contains a concurrent user-owned,
uncommitted comment edit at lines 4–6. This task excluded that edit from review,
staging, and commit. The committed `cc9e3f5` model, Git blob
`e72347845b0304c55f89e74428d75c4b15784e76`, remains authoritative for
reproducible evidence. A mechanical AST comparison found the working copy and
committed model semantically equal after removing line and source-location
fields; that comparison does not adopt or review the comment text.

## Exhaustive reconciliation

The final reconciliation compared the epic headings, every CSV row, every
catalog entry, and the DomainForge AST:

- 128 epic headings equal the 128 ordered `(Observed Journey ID, Observed
  Journey)` CSV pairs; all IDs are unique and nonempty.
- Every CSV row names exactly one existing `CJ01`–`CJ12` catalog entry and SEA
  `CanonicalJourney` instance. Each catalog coverage list equals its CSV group,
  and the twelve lists contain all 128 IDs exactly once.
- All 12 SEA journeys contain exactly 17 nonempty literal fields. Their IDs,
  names, and 15 shared semantic fields equal the catalog; catalog maturity
  distributions equal the mapped CSV rows. The 12 journey tuples are distinct.
- All 12 journey flows use the matching entry and completion states and match
  the catalog across all seven annotations: journey ID, steps, capabilities,
  evidence, completion, recovery, and next decision.

Coverage by journey is CJ01 15, CJ02 16, CJ03 7, CJ04 10, CJ05 11, CJ06 7,
CJ07 6, CJ08 12, CJ09 17, CJ10 10, CJ11 10, and CJ12 7. Classification totals
are 12 canonical, 69 specialization, 26 variant, 21 composition, and zero
duplicate, obsolete, ambiguous, or unsupported. Maturity totals are zero
declared, 13 specified, 6 implemented, 85 exercised, and 24 evidenced.
Confidence is 125 high, 3 medium, and zero low.

All 128 rows are explained, so coverage is 100% and unresolved semantic journey
IDs are `[]`. Compressing 128 observed stories to 12 identities removes 116
identities, a 90.625% reduction, and yields `128 / 12 = 10.666...`, reported as
10.67:1. Stable intent remains modeled even when its current implementation
maturity is only specified.

Variation frequency uses the method defined in the
[coverage report](../../.sea/interaction/coverage-report.md): split each row's
semicolon-separated `key=value` entries, count the key before the first `=`,
and count keys independently. Counts are non-exclusive. The resulting leading
frequencies are subject 47, artifact 33, evidence 28, interface 26, assurance
22, actor 21, next decision 19, initiating condition 17, authority outcome 14,
work source 11, recovery path 10, state transition 10, and completion 9.

## Projection inventory and binding integrity

The AST contains 38 ordinary `InterfaceProjection` instances. Every instance
has exactly seven nonempty fields, and exhaustive splitting of all
`canonical_journey_ids` values proves every binding resolves to `CJ01`–`CJ12`.
No route, method, command, provider, or projection creates canonical identity.

| Kind | Count | Exact instance inventory |
| --- | ---: | --- |
| Workbench web UI | 11 | `workbench_readiness_routes`, `workbench_thoth_route`, `workbench_assets_route`, `workbench_delegation_route`, `workbench_domain_models_route`, `workbench_case_routes`, `workbench_inbox_route`, `workbench_operations_route`, `workbench_evidence_and_run_routes`, `workbench_unavailable_record_routes`, `workbench_administration_route` |
| SFWP API | 11 | `sfwp_system_family`, `sfwp_request_correlation_family`, `sfwp_event_family`, `sfwp_context_family`, `sfwp_thoth_family`, `sfwp_case_authoring_family`, `sfwp_case_navigation_family`, `sfwp_approval_family`, `sfwp_run_record_family`, `sfwp_asset_family`, `sfwp_delegation_family` |
| CLI | 11 | `cli_execution_family`, `cli_run_inspection_family`, `cli_memory_family`, `cli_cell_and_self_model_family`, `cli_semantic_and_environment_family`, `cli_case_and_human_family`, `cli_ledger_family`, `cli_federation_family`, `cli_artifact_family`, `cli_thoth_family`, `cli_agent_family` |
| Agent | 5 | `agent_thoth_epistemic_surface`, `agent_thoth_manager_surface`, `agent_endpoint_surface`, `agent_delegation_execution_surface`, `agent_delegation_control_surface` |

The inventory retains all 17 Workbench routes, 23 advertised SFWP methods, and
21 visible top-level CLI commands. Preview, unavailable, naming-drift, and
compatible-host limits remain in the binding fields instead of becoming
invented maturity claims.

## DomainForge findings

`/home/sprime01/projects/domainforge/target/debug/domainforge` reports version
0.15.0. Current-worktree human and JSON validation both pass with zero
violations. Current and committed AST outputs parse as JSON and retain namespace
`sea_forge.interaction`, version `0.1.0`, zero imports, and 148 declarations:
37 entities, 16 flows, 76 instances, 2 policies, 3 relations, 5 resources, and
9 roles. The instances divide into 12 journeys, 7 reusable steps, 19
classification/dimension records, and 38 interface bindings.

The [validation diagnostics](../../.sea/interaction/validation/diagnostics.md)
and [raw transcript](../../.sea/interaction/validation/domainforge-output.txt)
record the projection evidence. Fixed-time RDF produced exactly
`ontology.owl.ttl` (6,272 bytes), `model.ttl` (14,522 bytes), and
`model.jsonld` (15,384 bytes). Turtle and JSON-LD retain 70 ontology/topology
nodes, but all 76 instances, both policies, and all flow annotations are lost;
OWL also omits flow and relation individuals. RDF is therefore meaningful but
not a lossless journey projection. CMMN was not generated because its current
lowering would fabricate 37 human tasks and 2 case milestones from ontology
and graph-integrity declarations, generally without performers.

The [limitations assessment](../../.sea/interaction/validation/limitations.md)
contains ten fully structured findings. Their candidate distinction is:

1. Journey/step/composition identity recurs, but a dedicated journey keyword is
   not yet credible.
2. Typed instance references are a credible general grammar candidate.
3. Typed ordered and recovery transitions are a credible general candidate.
4. Typed entry, completion, and terminal-decision conditions are credible.
5. Instance-aware integrity policies are a credible evaluator/grammar candidate.
6. Typed interface bindings are a credible general grammar candidate.
7. Role-to-entity binding syntax is a credible general grammar candidate.
8. Reusable constrained vocabularies are a credible general grammar candidate.
9. Same-namespace imported-instance resolution is a resolver defect, not a new
   grammar candidate.
10. RDF instance, policy, and annotation loss is a projector defect, not
    primarily a grammar candidate.

These ten DomainForge limitations are known tooling and representation
boundaries. They are not unresolved semantic journeys and do not create a
thirteenth canonical identity.

## Semantic and scope review

The review found no duplicate canonical semantics, route-shaped journey
identity, unexplained row, invented maturity, decorative policy, invalid SEA,
forbidden incomplete marker, credential-like value, or Task 7 application or
grammar change. Classification and maturity remain independent. Identity and
authority remain cross-cutting governed steps; Thoth and maintenance remain
compositions; execution remains distinct from settlement; derived projections
remain subordinate to source truth.

The three medium-confidence rows—3.5, 11.3, and 16.3—retain explicit evidence
breadth limits. Remaining issues are the ten DomainForge limitations, partial
or preview-only implementation surfaces recorded in the matrix and bindings,
non-executable repository hooks, and operator-dependent compatible-host agent
evidence. None changes the 128/128 semantic reconciliation.

## Files, commits, and checks

The interaction commit spine is:

- `443af8a` matrix; `b83b25d` maturity correction; `ab64570` catalog, grammar,
  and coverage;
- `20ad2ef` approved single-source plan adaptation; `120eea2` ontology;
  `d1a8f55` journeys and variants; `cc9e3f5` interface projections and source
  handoff; and
- `4b9b068` DomainForge validation, followed by this report and status-cache
  commit, `docs(interaction): hand off canonical journey model`.

Tasks 1–6 changed only the linked `.sea/interaction/` sources and validation
artifacts. Task 7 changes only this report and `.agents/STATUS_CACHE.md`. The
uncommitted model comment and `.jolli/jollimemory/debug.log` are excluded and
untouched.

Final commands included DomainForge `--version`, human and JSON `validate`,
`parse --ast --format json` against both current and committed sources,
`python3 -m json.tool`, semantic AST comparison, exhaustive Python
epic/CSV/catalog/AST/flow/projection reconciliation, the forbidden-marker scan,
`git diff --check`, staged-name and staged-diff checks, a staged sensitive-value
scan, and post-commit scope checks. `just context-check` was owner-skipped and
is not claimed. Git ignored non-executable repository hooks, so the explicit
checks provide the recorded evidence.

## Exact next-agent handoff

The Journey & Capability Map agent should start at the
[interaction README](../../.sea/interaction/README.md), use `CJ01`–`CJ12` as the
only identity spine, join all 128 observed stories through the matrix, take
journey semantics and maturity from the catalog, take composition and variation
rules from the grammar, and bind capabilities, evidence, and interface
affordances through the committed model. Do not redo canonicalization, rename a
journey from a route or method, treat a projection as truth, promote specified
behavior, collapse execution into settlement, or silently change the map when
new evidence arrives. If evidence materially changes a normalized intention
tuple, update and revalidate the matrix, catalog, model, coverage, and bindings
before changing the map.

The interaction model is sufficiently stable to use as the canonical basis for the Journey & Capability Map.
