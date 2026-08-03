# DomainForge Validation Diagnostics

## Scope and Source Identity

The committed Task 6 base is `cc9e3f55bd720d33228bcdba3cc1632e50e851dc`. The working-tree `interaction-model.sea` differs from that commit only in the user-owned comment at lines 4–6. Task 6 did not edit, stage, revert, or commit that file. The required working-tree validation and RDF projection used the current path because comments do not enter the semantic graph. For reproducible AST evidence, Task 6 materialized the exact `cc9e3f5:.sea/interaction/interaction-model.sea` blob under a temporary canonical `.sea/interaction/` path, verified its Git object ID as `e72347845b0304c55f89e74428d75c4b15784e76`, and parsed that 1,059-line source to a 174,179-byte `--out` AST. Its semantic AST nodes equal the current parse after removing source-location fields.

The binary was `/home/sprime01/projects/domainforge/target/debug/domainforge`, which reported `domainforge 0.15.0`. Human and JSON validation both passed with zero violations. The complete concise command transcript is in `domainforge-output.txt`.

## AST Inspection

The AST was generated in `/tmp`, validated with `python3 -m json.tool`, and not committed. It contained namespace `sea_forge.interaction`, version `0.1.0`, zero imports, and 148 declarations:

| Declaration | Count |
| --- | ---: |
| Entity | 37 |
| Flow | 16 |
| Instance | 76 |
| Policy | 2 |
| Relation | 3 |
| Resource | 5 |
| Role | 9 |

The 76 instances divide into 12 `CanonicalJourney`, 7 `JourneyStep`, 19 `JourneyVariant`, and 38 `InterfaceProjection` instances.

Representative semantic survival was inspected rather than inferred from the exit status:

- `cj01_establish_trusted_cell_context` retained its `CanonicalJourney` type and all 17 fields, including `journey_id`, intention, conditions, transition, evidence, recovery, decisions, variants, and maturity.
- `workbench_readiness_routes` retained all seven interface-binding fields: `interface_kind`, `canonical_journey_ids`, `entry_or_method`, `binding_role`, `implementation_maturity`, `source_ref`, and `limitations`.
- The CJ01 transformation flow retained `from`/`to`, `InteractionProgress`, quantity, and all seven annotations: `journey_id`, `action_sequence`, `capabilities`, `evidence`, `completion_condition`, `recovery`, and `next_decision`.
- `has_interaction_flow` retained a `count(flows) > 0` expression tree. `has_governed_evidence_resource` retained its existential quantifier and `r.name = "GovernedEvidence"` member comparison.
- `Sponsorship` retained the `AgentSponsor` subject, authored predicate, and `AutomatedActor` object.
- Imports did not survive because the canonical source deliberately has none; this is the single-source workaround described below, not a parser loss.

## Exact Same-Namespace Import Failure and Workaround

Task 3 tested both supported relative import forms against DomainForge 0.15.0. The source module was:

```sea
@namespace "sea_forge.interaction"

export entity "ReusableStep"
```

The wildcard importing module was:

```sea
@namespace "sea_forge.interaction"

import * as domain from "./domain.sea"

instance context_selection of "ReusableStep"
```

The exact validation result was exit status 1:

```text
Error: Parse failed for .superpowers/sdd/task-3-import-proof.MiQzA5/model.sea: Grammar error: Failed to add entity instance 'context_selection': Entity 'ReusableStep' not found in namespace 'sea_forge.interaction'
```

The named importing module changed only the import:

```sea
@namespace "sea_forge.interaction"

import { ReusableStep } from "./domain.sea"

instance context_selection of "ReusableStep"
```

It returned the same diagnostic and exit status 1:

```text
Error: Parse failed for .superpowers/sdd/task-3-import-proof.MiQzA5/model.sea: Grammar error: Failed to add entity instance 'context_selection': Entity 'ReusableStep' not found in namespace 'sea_forge.interaction'
```

The approved workaround is exactly one semantic source, `interaction-model.sea`, with no imports or duplicated declarations. The four smaller `.sea` files are declaration-free explanatory indexes. This preserves validation today at the cost of a 1,059-line committed model and prevents independent semantic modules until the resolver is corrected or a different namespace design is approved.

## RDF Projection Inspection

The output directory came from `mktemp -d /tmp/sea-forge-interaction-task6-rdf.XXXXXX`, was resolved with `realpath -e`, and was accepted only when it matched `/tmp/sea-forge-interaction-task6-rdf.*`. Projection used the fixed timestamp `2026-08-03T00:00:00Z`. The three-file inventory, byte counts, and SHA-256 hashes are recorded in `domainforge-output.txt`; `model.jsonld` also passed `python3 -m json.tool`.

The projection is meaningful but partial:

- Turtle and JSON-LD retained all 37 entity identities, 9 role identities, 5 resource identities, 3 role-level relations, and 16 flow identities with `from`, `to`, resource, and quantity.
- The representative `Sponsorship` relation retained `AgentSponsor`, `AutomatedActor`, and `is accountable sponsor for`.
- The representative CJ01 edge retained `CJ01EntryState -> CJ01CompletionState` with `InteractionProgress`, but the flow was renamed to the generated identity `flow_d63efd98bd7f4004`.
- JSON-LD serialized the same 70 graph nodes: 37 entity, 9 role, 5 resource, 3 relation, and 16 flow nodes.
- OWL retained the entity, role, and resource vocabulary as classes and named individuals, plus the generic flow and relation object properties. It included the fixed timestamp in its provenance comment and content-derived `owl:versionInfo` value `e5f459e4048fe2ae`.

The inspected semantic losses are material:

- All 76 authored instances are absent from Turtle, JSON-LD, and OWL. Consequently the 12 journey records, 7 reusable step records, 19 classification/dimension records, and 38 interface bindings do not project.
- Both policies and their expressions are absent.
- All flow annotations are absent, including `journey_id`, ordered action narration, capabilities, evidence, completion condition, recovery, and next decision.
- Because generated flow identities omit the `journey_id` annotation, the RDF edge between CJ01 state entities cannot be identified as CJ01 without returning to SEA source.
- OWL additionally omits the 16 flow and 3 relation individuals that Turtle and JSON-LD retain. It therefore describes the ontology vocabulary, not the complete projected graph.
- Untyped instance fields remain unrepresented, consistent with DomainForge's documented RDF v1 non-goal, but this means RDF cannot serve as a lossless interaction-model projection.

The RDF output was evidence only and was not committed. After inspection, Task 6 removed only the resolved temporary directory it created.

## CMMN Decision

CMMN generation was intentionally skipped. DomainForge documents the current lowering as every resource to a case-file item, every role to a case role, every entity to a human task, and every policy to a milestone and sentry. Applied here, that would produce 5 case-file items, 9 case roles, 37 human tasks, and 2 milestones. The 37 tasks would include ontology categories such as `CanonicalJourney` and `Evidence` plus entry/completion state markers; they are not executable human work. The two milestones would represent generic graph-integrity checks (`count(flows) > 0` and presence of `GovernedEvidence`), not case achievements. The current text grammar also cannot bind roles to entities, so the fabricated human tasks would generally lack performers.

Generating XSD-valid CMMN would therefore add format evidence while materially distorting the model. Skipping it is the honest result; no CMMN output was generated or committed.

## Checks Deliberately Not Claimed

- `just context-check` was owner-skipped as required.
- No CMMN validity claim is made because CMMN was not generated.
- No RDF reasoning or lossless round-trip claim is made; the documented RDF operator does not preserve the model's instances, policies, or annotations.
- No application, grammar, dependency, CI, deployment, `CURRENT_STATUS`, or `STATUS_CACHE` file changed in Task 6.
