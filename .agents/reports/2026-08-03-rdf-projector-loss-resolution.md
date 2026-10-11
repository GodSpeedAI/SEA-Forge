# Resolving the DomainForge RDF Projector Loss (limitation 10)

Reviewed: `/home/sprime01/projects/domainforge` @ 0.16.0.
Consumer: `sea-rs` `.sea/interaction/interaction-model.sea` (76 instances, 2 policies).

## 1. Root cause — exact, not inferred

`KnowledgeGraph::from_graph` (`domainforge-core/src/kg.rs:145–427`) iterates
exactly six collections: `all_entities`, `all_roles`, `all_resources`,
`all_patterns`, `all_relations`, `all_flows`. It never reads:

- `Graph::all_entity_instances()` (`graph/mod.rs:554`) → `primitives::Instance`
- `Graph::all_instances()` (`graph/mod.rs:693`) → `primitives::ResourceInstance`
- `Graph::all_policies()` (`graph/mod.rs:728`) → `policy::Policy`

`OntologyIR::from_graph` (`projection/rdf/ontology.rs:55–170`) mints
`owl:NamedIndividual`s only from entities, roles, and resources.

The two SHACL shapes emitted at `kg.rs:380` and `kg.rs:414` are hardcoded
structural invariants (`sea:Flow`, `sea:Entity`). They are not derived from any
declared `Policy`.

This is omission, not a lowering defect. `Instance x of "Type" { … }` parses via
`parser/ast.rs:555 → parse_instance → graph.add_entity_instance` (`ast.rs:3536`),
so the data is present and intact in the `Graph`. The projector simply does not
look at it.

## 2. Why this is a completable v2, not a workaround

`docs/rdf-projections.md` "Non-goals (v1)" states:

> **No typed entity attributes.** Attributes are untyped in the IR today, so the
> ontology derives classes, relations, and named individuals only.

That was the stated reason for the omission, and 0.16.0 retired it: the I1–I3
intervention (PR #120) added typed entity fields and validation
(`graph/entity_validation.rs`, `policy/quantifier.rs`, `grammar/sea.pest`).
Emitting instances with correct `xsd:` datatypes is now possible for the first
time. Closing this is the documented roadmap continuing, not a patch bolted onto
a design that rejected it.

## 3. The determinism constraint (the one real trap)

The project's hardest RDF invariant is byte-identical output
(`docs/rdf-projections.md` §Determinism, CI `verify-rdf` job,
`tests/rdf_projection_tests.rs`). Two facts govern the fix:

- `Instance::id` is `ConceptId::from_concept(namespace, "Type:name")` — already
  content-derived and stable. Instance IRIs must therefore be minted directly
  and **must not** be routed through `canonicalize_node_ids`
  (`projection/rdf/mod.rs:32`); `is_minted_node` stays limited to `sea:flow_`
  and `sea:pattern_`.
- `Instance::fields` is a `HashMap<String, Value>`. Iterating it directly makes
  output order nondeterministic and breaks the CI gate. Field triples must be
  collected into a `BTreeMap` before emission.

By contrast `ResourceInstance::new_with_namespace`
(`primitives/resource_instance.rs`) assigns `Uuid::new_v4()` — a genuinely
nondeterministic id. Resource instances therefore cannot be emitted under the
current identity rule without either a content-derived id change or the
`canonicalize_node_ids` hashing path. Keep them out of scope.

## 4. Proposed resolution

### 4.1 Entity instances → RDF individuals (closes the 76-instance loss)

In `kg.rs::from_graph`, after the entity loop:

- `sea:<sanitize_qname(Type_name)> rdf:type sea:<EntityType>`
- `… rdfs:label "<name>"`
- `… sea:namespace "<namespace>"`
- one triple per field, keys `BTreeMap`-sorted, values typed from the declared
  entity field type when known (`xsd:string`/`xsd:decimal`/`xsd:boolean`),
  falling back to `xsd:string`

**Correction (verified during implementation):** an earlier draft of this report
claimed a collision hazard between same-named instances of different entity
types, and proposed type-qualified IRIs to defend against it. That collision
cannot occur. `Graph::insert_entity_instance` (`graph/mod.rs:495`) rejects any
instance whose name already exists anywhere in the graph, so instance names are
unique graph-wide. The IRI is therefore `sea:instance_<name>`, which is simpler
and additionally keeps an instance's identity stable when it is retyped. A test
(`instance_names_are_unique_graph_wide`) pins that invariant so the IRI scheme
is revisited if the uniqueness rule is ever relaxed.

Residual, pre-existing and unchanged: namespace is not part of any IRI here, so
two instances with the same name in different namespaces would share an IRI —
exactly as two same-named entities already do (`sea:<Name>`). This is existing
`kg.rs` behavior, not something this change introduces.

In `ontology.rs::from_graph`, add each instance as an `owl:NamedIndividual` of
its declared class, and promote declared entity field types to
`owl:DatatypeProperty` with domain/range.

### 4.2 Policies → SHACL where faithful, individuals where not

`Policy` (`policy/core.rs`) carries `name`, `namespace`, `version`, `modality`,
`kind`, `priority`, `rationale`, `tags`, and an `Expression`.

Split by expressibility:

- Quantified constraints over instance collections (the
  `entity_instances`/`instances` quantifiers added in 0.16.0) lower to real
  `ShaclShape` entries — `sh:targetClass` plus `sh:minCount`/`sh:maxCount`/
  `sh:datatype`. The machinery already exists (`kg.rs:35–48`,
  `validate_shacl` at `kg.rs:790`, `domainforge validate-kg`, oxigraph CI gate),
  so these become *executable* in RDF, not merely present.
- Everything else emits as a `sea:Policy` individual carrying modality, kind,
  priority, rationale, tags, and its source expression as a literal — declared
  and traceable, explicitly non-normative.

**Never fake-translate.** A policy whose meaning SHACL cannot carry must not be
lowered into a SHACL constraint that means something adjacent. That is the same
reasoning sea-rs already used to skip CMMN, applied here.

### 4.3 Deliberately out of scope

- **Do not extend `to_graph()`** (`kg.rs:655`). It currently reconstructs only
  entities, resources, and flows — Turtle→Graph already drops roles, relations,
  and patterns. Making instances round-trip is a separate, larger change, and
  pushing RDF toward round-trip parity would make it a second source of truth,
  which is exactly what upstream's "One Canonical Semantic World" ADR rejects.
- **Do not fork `kg.rs`.** `docs/rdf-projections.md` states `model.ttl` comes
  from the kg.rs serializer, "which is not forked." Extending `from_graph` in
  place makes `--format kg`, `Graph::export_rdf`, RDF/XML, and `validate-kg` all
  gain instances consistently and for free. A separate instance emitter would
  create precisely the drift this project has avoided.
- Resource instances (§3), OWL reasoning, RDF-star, named graphs — all remain
  the documented v1 non-goals.

## 5. Governance steps this project requires

These are not optional here; skipping them *is* the debt.

1. Amend `docs/specs/SDS-005-knowledge-graph-module.md` §4.2/§4.3 — the class
   and property mapping tables are the declared vocabulary contract. New classes
   (`sea:Policy`) and the instance-typing rule belong there before code.
2. Update `docs/rdf-projections.md`: strike the now-false "No typed entity
   attributes" non-goal, extend the artifact table.
3. Tests, mirroring existing conventions: a `turtle_instance_export_tests.rs`
   alongside `turtle_entity_export_tests.rs`; extend the determinism and
   rename-isolation checks in `rdf_projection_tests.rs`; a SHACL round of
   `validate-kg` over a fixture whose policy is expected to fail.
4. Sequencing: `.agents/next_steps.md` says do not combine new work with the
   I1–I3 intervention diff until it is reviewed. This lands after.

## 6. Estimated size

~60 lines in `kg.rs::from_graph`, ~30 in `ontology.rs`, plus tests and the two
doc amendments. No new module, no new dependency, no grammar change, no change
to `to_graph`, `canonicalize_node_ids`, or the projection registry.

## 7. Effect on sea-rs

With §4.1 alone, all 76 `InterfaceProjection` and journey instances appear in
`model.ttl`, `model.jsonld`, and `ontology.owl.ttl` with stable IRIs and their
`canonical_journey_ids` intact. RDF becomes usable for SPARQL over the journey
model. `interaction-model.sea` stays canonical; RDF stays a projection. The
STATUS_CACHE instruction "do not regenerate the map from RDF" remains correct
and should stay — a faithful projection is still not the source.
