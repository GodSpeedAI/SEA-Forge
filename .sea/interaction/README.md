# SEA Forge Interaction Model

This directory is the canonical source for SEA Forge's human-intention interaction model. It compresses all 128 durable stories in the governed Workbench UX epic into 12 canonical journeys while retaining classification, variation, implementation maturity, evidence, recovery, and interface traceability.

Routes, SFWP methods, CLI commands, and agent surfaces are bindings to this model. They do not define canonical identity, and their existence does not prove that a journey is complete.

## Source Authority

The observed catalog is the 128 `X.Y` stories in `.agents/specs/frontend/sea-forge-governed-workbench-ux-epic-v0.1.md`. The reviewed `canonicalization-matrix.csv` maps every one of those stories to exactly one `CJ01`–`CJ12` identity using actor, intention, initiating condition, transition, capability, artifact, evidence, completion condition, and next decision.

`.sea/interaction/` is canonical because it holds the exhaustive mapping and the sole validated semantic model together. Reports elsewhere may summarize or link to these files, but they must not fork the journey catalog, reclassify rows, or make a route or method authoritative. Canonical records and runtime behavior remain governed by their own repository specifications; this model describes their interaction meaning and evidence-backed bindings.

## Entry Point and Artifact Responsibilities

`interaction-model.sea` is the single semantic authority. There is exactly one `.sea` file.

| Artifact | Responsibility |
| --- | --- |
| `interaction-model.sea` | Sole semantic source and validation entry point: patterns, closed vocabularies, roles, ontology, the product purpose, typed catalog entities, the eight-phase skeleton, 12 journeys and their flows, classifications, variation dimensions, interface surfaces and bindings, one application read boundary, and 16 enforced invariants. |
| `canonicalization-matrix.csv` | Exhaustive evidence ledger: one nonempty, uniquely identified row for each of the 128 observed stories. |
| `canonical-journey-catalog.md` | Human-readable specification of every canonical journey, including covered story IDs and maturity. |
| `journey-grammar.md` | Stable verbs, nouns, skeleton, composition rules, variation dimensions, the projection boundary, and what the grammar enforces versus describes. |
| `coverage-report.md` | Reconciled classification, maturity, confidence, coverage, and compression counts. |
| `journey-capability-map.md` | Synthesis binding each `CJ01`–`CJ12` to its invoked capabilities, evidence, and interface bindings. |
| `handoff.md` | What the next agent needs, and what it must not redo. |
| `validation/` | Toolchain evidence: change review, diagnostics, semantic teeth, limitations, and the two runnable harnesses. |

### Why one file

DomainForge 0.16.0 resolves a deterministic transitive module closure for `parse`, `validate`, and `project`, so an imported entity type can back a same-namespace instance. That capability is proven here, not assumed: `validation/semantic-teeth.sh` cases T24–T26 exercise it directly.

Consolidation is therefore a deliberate architectural choice, not a tooling constraint. It follows DomainForge's own `docs/explanations/one-cononical-semantic-world.md`: one canonical world means every concept has one authoritative definition, invariants can be validated against the complete world, and shared terms cannot silently diverge. Specialization belongs at the distal boundaries — projections, provider configuration, and adapters — not inside the canonical model.

The four declaration-free `.sea` companions that previously accompanied this model (`interaction-domain.sea`, `canonical-journeys.sea`, `journey-variants.sea`, `interaction-projections.sea`) have been removed. They existed only because DomainForge 0.15.0 could not resolve imported entity types for same-namespace instances, and each carried a header stating that obsolete reason. Their navigational content is covered by this README and the catalog.

## What the Model Enforces

DomainForge 0.15.0 could check almost nothing about this catalog: 76 instances were schemaless, references were semicolon-separated strings, vocabularies were unchecked prose, and only two policies existed. The current model is typed throughout.

- **Closed vocabularies** — `CanonicalizationClass`, `ImplementationMaturity`, `InterfaceKind`, and `InteractionPhase` are enums. An illegal value fails validation.
- **Identity patterns** — `JourneyId` admits `CJ01`–`CJ12` only; `InterfaceBindingId` admits `IB000`–`IB999`; `InteractionStepId` admits `IS1`–`IS8`. A thirteenth journey is structurally impossible.
- **A single product purpose** — `ProductPurpose` states what SEA Forge is for, what constraint governs every journey, and what this model deliberately does not define. `exactly_one_product_purpose` prevents a competing statement.
- **Entity keys** — `purpose_id`, `journey_id`, `surface_id`, `binding_id`, `step_id`, `dimension_id`, and `classification_id` are unique keys.
- **Typed references** — `InterfaceBinding.journey` and `InterfaceBinding.surface` are `ref<>` fields. A binding to a journey or surface that does not exist is rejected, which the previous model could not detect.
- **Required and optional fields** — every prose field has an enforced minimum length, so an empty intention or completion condition fails. `value_derived_concepts` is honestly `optional`, because only 2 of 11 variation dimensions carry it.
- **Sixteen counting invariants** — one product purpose, twelve journeys, eight phases with ordinals 1–8, eight classifications, eleven dimensions, thirty-eight surfaces, 128 stories, the four maturity totals, and the binding reconciliation.

Every one of these has a negative test proving it rejects its violation. See `validation/semantic-teeth.md` — 31 cases, all passing.

## Meaning of Interface Bindings

Interfaces are modeled in two parts, because one surface may project several journeys and several surfaces may realize one journey.

`InterfaceSurface` describes the surface itself:

- `surface_id`: stable key;
- `interface_kind`: `web_ui`, `api`, `cli`, or `agent` (closed);
- `entry_or_method`: exact routes, advertised methods, commands, or agent entry surfaces;
- `binding_role`: what the interface lets an actor inspect or do;
- `implementation_maturity`: the evidence-backed standing of **that surface** (closed);
- `source_ref`: repository definitions, handlers, screens, or tests;
- `limitations`: unavailable behavior, preview ceilings, proof gaps, and target-versus-implementation naming drift.

`InterfaceBinding` is one row per (surface, journey) pair, with both sides typed references. There are 38 surfaces and 90 bindings.

Bindings never mint a journey ID. Surface maturity describes the surface, never the journey: an exercised route proves a test or execution record drives that route, not that every journey it binds to is complete. A preview does not grant authority, a catalog entry does not establish availability, an execution surface does not imply accepted settlement, and a UI projection never becomes ledger truth.

### Interface reality check

Re-verified against the current SEA Forge working tree during this revision:

- 17 Workbench routes in `workbench/apps/desktop/src/router.tsx` (`/`, `/readiness`, `/thoth`, `/assets`, `/delegate`, `/models`, `/cases`, `/cases/new`, `/inbox`, `/operations`, `/evidence`, `/runs/$runId`, `/memory`, `/capabilities`, `/artifacts`, `/federation`, `/admin`);
- 23 advertised methods in `IMPLEMENTED_METHODS` in `crates/sea-forge-server/src/sfwp/mod.rs`;
- every cited `sea-forge` CLI family present in `crates/sea-forge-cli/src/main.rs`.

The counts and names match what the model binds. These implementation facts are preserved:

- `/models`, `/memory`, `/capabilities`, `/artifacts`, and `/federation` are explicit unavailable surfaces;
- `/delegate` provides live preview, roster, and scoped cancellation but deliberately offers no delegation commit action;
- implemented `case.entry_options` differs from target `case.get_entry_options`;
- the model route probes `domain.list_models`, while the target catalog names `domain_model.list`, and neither is advertised;
- `identity.get` and the two advertised delegation inspect methods are implemented additions not present in the target method catalog; and
- delegation execution and cancellation still use legacy wire verbs outside the 23-method advertised SFWP catalog.

## The Application Boundary

The model declares exactly one operation, `get_canonical_journey`: a public, read-only, synchronous boundary that returns one journey's canonical interaction contract. It is the stable contract that every catalog-projecting surface realizes — Workbench `/capabilities` and `/thoth`, SFWP `system.describe` and `thoth.ask`, and `sea-forge ask` are adapters over that one boundary, not separate meanings.

No write operation is declared. The catalog is authored in this file and reviewed as source; it is not mutated at runtime. SEA Forge's own governed-work records — cases, approvals, runs — are **not** modeled here. They belong to their own repository specifications, and importing them would fork their definitions into the interaction model.

DomainForge resolves operations through the Application Contract library boundary, not through the CLI. `domainforge validate` does not check the operation. See `validation/limitations.md` L4.

## Validate and Inspect

```bash
df=/home/sprime01/projects/domainforge/target/release/domainforge
# built with: cargo build --release --bin domainforge --features cli

# 1. semantic validation — necessary, not sufficient
"$df" validate --format human --no-color .sea/interaction/interaction-model.sea

# 2. semantic teeth: 30 negative cases against the real model
DOMAINFORGE="$df" .sea/interaction/validation/semantic-teeth.sh

# 3. reconcile the model against the 128-row matrix
python3 .sea/interaction/validation/reconcile.py

# 4. application contract and canonical semantic envelope
#    build the harness first — see validation/application-contract-harness.rs
dfharness envelope .sea/interaction/interaction-model.sea
dfharness contract .sea/interaction/interaction-model.sea
```

All four must pass. Step 1 alone proves less than it appears to: it does not reach the operation, and it will accept a policy whose `where` arithmetic is meaningless. Inspect the resulting objects rather than trusting an exit status; `validation/domainforge-output.txt` is the captured evidence and `validation/diagnostics.md` reads it.

Keep generated AST, RDF, or projection output temporary. It is not a source artifact.

## Known Toolchain Boundaries

Full detail in `validation/limitations.md`. The two that most affect a reader:

- **RDF still drops instances and policies in released DomainForge 0.16.0.** Preservation exists only in the uncommitted `feat/rdf-instance-policy-projection` branch. Until it lands, RDF is usable for the ontology spine only and is never a source for journey, binding, or policy identity. Use the Canonical Semantic Envelope instead, which preserves all 266 semantic declarations including every instance and policy.
- **`domainforge validate` cannot check the application operation.** No CLI subcommand resolves the Application Contract or the Semantic Envelope; the harness is required.

## The 128-Story Mapping

`canonicalization-matrix.csv` is the exhaustive layer. It contains one row for every observed story ID and preserves:

- its `CJ01`–`CJ12` mapping and one closed classification;
- material variation dimensions instead of creating route-shaped journeys;
- direct repository evidence, confidence, notes, and implementation maturity; and
- specified, unavailable, preview-only, implemented, exercised, and evidenced distinctions without treating stable intent as shipped behavior.

The SEA source does not copy 128 rows. It carries the stable ontology plus each journey's reconciled story and maturity counts, and DomainForge proves those counts still total 128 and still match the reconciled maturity distribution. `reconcile.py` closes the remaining gap by checking the model's counts against the CSV row by row, including each journey's maturity distribution. The CSV remains the row-level authority for evidence, notes, and variation dimensions; the model is the authority for identity and invariants.

## Journey & Capability Map Handoff

See `handoff.md`.
