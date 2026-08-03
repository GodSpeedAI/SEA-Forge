# SEA Forge Interaction Model

This directory is the canonical source for SEA Forge's human-intention interaction model. It compresses all 128 durable stories in the governed Workbench UX epic into 12 canonical journeys while retaining classification, variation, implementation maturity, evidence, recovery, and interface traceability.

Routes, SFWP methods, CLI commands, and agent surfaces are bindings to this model. They do not define canonical identity, and their existence does not prove that a journey is complete.

## Source Authority

The observed catalog is the 128 `X.Y` stories in `.agents/specs/frontend/sea-forge-governed-workbench-ux-epic-v0.1.md`. The reviewed `canonicalization-matrix.csv` maps every one of those stories to exactly one `CJ01`–`CJ12` identity using actor, intention, initiating condition, transition, capability, artifact, evidence, completion condition, and next decision.

`.sea/interaction/` is canonical because it holds the exhaustive mapping and the sole validated semantic model together. Reports elsewhere may summarize or link to these files, but they must not fork the journey catalog, reclassify rows, or make a route or method authoritative. Canonical records and runtime behavior remain governed by their own repository specifications; this model describes their interaction meaning and evidence-backed bindings.

## Entry Point and Artifact Responsibilities

| Artifact | Responsibility |
| --- | --- |
| `interaction-model.sea` | Sole semantic source and validation entry point: ontology, reusable steps, policies, 12 journeys and flows, variants, and interface projections. It intentionally has no imports. |
| `interaction-domain.sea` | Declaration-free index to roles, entities, resources, relations, reusable steps, and graph invariants in the entry point. |
| `canonical-journeys.sea` | Declaration-free index to `CJ01`–`CJ12`, their unique states, and their `InteractionProgress` flows. |
| `journey-variants.sea` | Declaration-free index to the eight classification values and recurring variation families. |
| `interaction-projections.sea` | Declaration-free index to the UI, API, CLI, and agent binding section. |
| `canonicalization-matrix.csv` | Exhaustive evidence ledger: one nonempty, uniquely identified row for each of the 128 observed stories. |
| `canonical-journey-catalog.md` | Human-readable specification of every canonical journey, including covered story IDs and maturity. |
| `journey-grammar.md` | Stable verbs, nouns, composition rules, variation dimensions, and the projection boundary. |
| `coverage-report.md` | Reconciled classification, maturity, confidence, coverage, and compression counts. |

DomainForge 0.15.0 cannot resolve an instance whose entity type is imported from another same-namespace file. The canonical entry point therefore consolidates declarations, while the four smaller SEA companions remain valid, import-free, and declaration-free indexes. They are modules for navigation, not competing models.

## The 128-Story Mapping

`canonicalization-matrix.csv` is the exhaustive layer. It contains one row for every observed story ID and preserves:

- its `CJ01`–`CJ12` mapping and one closed classification;
- material variation dimensions instead of creating route-shaped journeys;
- direct repository evidence, confidence, notes, and implementation maturity; and
- specified, unavailable, preview-only, implemented, exercised, and evidenced distinctions without treating stable intent as shipped behavior.

The SEA source intentionally does not copy the matrix into 128 instances. It carries stable ontology and reusable semantics; the CSV retains row-level distinctions. `coverage-report.md` reconciles the result to 128 unique observed stories, 12 represented canonical journeys, and 100% explained mapping without deleting the original distinctions.

## Validate and Inspect

From the repository root:

```bash
df=/home/sprime01/projects/domainforge/target/debug/domainforge
"$df" validate --format human --no-color .sea/interaction/interaction-model.sea
"$df" validate --format human --no-color .sea/interaction/interaction-projections.sea
"$df" parse --ast --format json .sea/interaction/interaction-model.sea \
  > /tmp/sea-forge-interaction.ast.json
python3 -m json.tool /tmp/sea-forge-interaction.ast.json >/dev/null
```

Inspect the AST rather than trusting process exit alone. Confirm that `InterfaceProjection` instances retain all seven binding fields, that every `canonical_journey_ids` value contains only existing `CJ01`–`CJ12` IDs, that all 17 Workbench routes and all 23 methods in `IMPLEMENTED_METHODS` remain traceable, and that the companion AST contains zero declarations. Keep generated AST or projection output temporary; it is not a source artifact.

## Meaning of Interface Bindings

Each ordinary `InterfaceProjection` instance has:

- `interface_kind`: `web_ui`, `api`, `cli`, or `agent`;
- `canonical_journey_ids`: existing CJ identifiers whose steps the interface projects;
- `entry_or_method`: exact routes, advertised methods, commands, or agent entry surfaces;
- `binding_role`: what the interface lets an actor inspect or do;
- `implementation_maturity`: the evidence-backed standing of that binding;
- `source_ref`: repository definitions, handlers, screens, or tests; and
- `limitations`: unavailable behavior, preview ceilings, proof gaps, and target-versus-implementation naming drift.

Bindings never mint a journey ID. A preview does not grant authority, a catalog entry does not establish availability, an execution surface does not imply accepted settlement, and a UI projection never becomes ledger truth. In particular, the current model preserves these implementation facts:

- `/models`, `/memory`, `/capabilities`, `/artifacts`, and `/federation` are explicit unavailable surfaces;
- `/delegate` provides live preview, roster, and scoped cancellation but deliberately offers no delegation commit action;
- implemented `case.entry_options` differs from target `case.get_entry_options`;
- the model route probes `domain.list_models`, while the target catalog names `domain_model.list`, and neither is advertised;
- `identity.get` and the two advertised delegation inspect methods are implemented additions not present in the target method catalog; and
- delegation execution and cancellation still use legacy wire verbs outside the 23-method advertised SFWP catalog.

## Journey & Capability Map Handoff

The next Journey & Capability Map agent must consume these artifacts without redoing canonicalization:

1. Use `canonicalization-matrix.csv` as the exhaustive observed-story-to-CJ lookup. Do not reassign rows because a newer screen or method name looks more convenient.
2. Use `canonical-journey-catalog.md` for each journey's intention, job, conditions, steps, capabilities, artifacts, evidence, completion, recovery, next decisions, maturity, and observed IDs.
3. Use `journey-grammar.md` for shared steps, composition rules, cross-cutting identity and authority, and variation dimensions.
4. Use `interaction-model.sea` for stable machine-readable IDs, transitions, reusable instances, invariants, and interface bindings.
5. Use `interaction-projections.sea` only as an index into the canonical source; it declares nothing.
6. Use `coverage-report.md` to reconcile counts and surface maturity or confidence boundaries.
7. Treat interface projection maturity and limitations as current repository evidence, not as permission to rename a CJ, collapse execution into settlement, or promote an unavailable target into a capability.

Build the map with `CJ01`–`CJ12` as the identity spine, then connect capabilities, evidence, and interface affordances through the recorded fields. If future evidence materially changes the normalized intention tuple, update the canonical matrix and catalog deliberately before changing the map; do not silently canonicalize inside the map.
