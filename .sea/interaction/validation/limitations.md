# Interaction Model Limitations

Every entry below was re-tested against the local DomainForge build described in
`domainforge-change-review.md`. Nothing is carried forward from the previous
report on the strength of that report alone. Each limitation is classified as a
**grammar**, **resolver**, **evaluator**, **projector**, **tooling**, or
**product-model** limitation, because the right owner differs in each case.

## Resolved since the previous report

These are recorded so the next agent does not re-derive them or reinstate a
workaround. Proof for each is in `semantic-teeth.md`.

| Previous limitation | Status | Proof |
| --- | --- | --- |
| 2 — instance-to-instance typed relations | **Resolved.** `ref<Entity>` with dangling-reference rejection replaced `"CJ01; CJ02"` strings | T04, T05, T14, T20 |
| 5 — cardinality and integrity policies over instances | **Resolved.** `entity_instances` gives 14 enforced counting invariants | T14c–T23b |
| 8 — classification and maturity vocabularies | **Resolved.** Four closed enums; illegal values rejected | T01–T03 |
| 9 — same-namespace imported-instance resolution | **Resolved.** Deterministic transitive filesystem closure | T24–T26 |
| 1 — journey and step identity | **Substantially resolved.** Typed keys, patterns, and required fields give journeys and steps enforced identity. A dedicated `journey` keyword remains unjustified and is not recommended | T06, T08, T09 |
| 4 — entry and completion conditions | **Partly resolved.** Presence and non-emptiness are enforced by `min_length`; the conditions themselves remain prose. See L6 | T13 |
| 6 — interface binding declarations | **Resolved for integrity.** `InterfaceSurface` + `InterfaceBinding` give typed kinds, closed maturity, validated references, and enforced counts. A dedicated binding keyword remains unjustified | T02, T05, T17, T20b |
| 3 — ordered steps and transitions | **Superseded by the model.** The eight-phase skeleton is ordered by an enforced `ordinal`; branching remains prose. See L7 | T12, T19 |

## Current limitations

### L1 — RDF still drops instances and policies in released DomainForge

- **Class:** projector.
- **What was tested:** `domainforge project --format rdf` on the canonical model
  with both the released `~/.cargo/bin/domainforge` 0.16.0 and the local
  worktree build.
- **Result:** released 0.16.0 emits 333 lines of `model.ttl` and contains none
  of `workbench_readiness_routes`, `twelve_canonical_journeys`, or `IB001`. The
  local build on branch `feat/rdf-instance-policy-projection` emits 2101 lines
  and contains all of them.
- **Consequence:** instance and policy preservation is real but **unreleased**.
  Until that branch lands, RDF is usable only for the ontology spine — entities,
  roles, resources, relations, and flow edges. It is never a source for journey,
  binding, capability, or policy identity.
- **Use instead:** the Canonical Semantic Envelope, which preserves all 266
  semantic declarations including every instance and policy.
- **Owner:** DomainForge. No SEA Forge change is required; do not model around it.

### L2 — `list<T>` fields cannot be instantiated

- **Class:** grammar.
- **What was tested:** an instance field holding `["CJ01"]`.
- **Result:** `Syntax error: expected primary_expr`. The `expression` rule
  reaches `literal`, which has no array form (`sea.pest` 444); `string_array`
  exists only for annotations and policy metadata.
- **Consequence:** an entity may declare `list<ref<T>>` or `list<string>`, but no
  concrete instance can supply a value, so the field is unusable in practice.
- **How this model responds:** multi-valued relationships are normalized into
  their own entity. `InterfaceBinding` exists because one surface projecting
  four journeys cannot be one instance with a list. This is a better model in any
  case — it makes each binding independently addressable and validated — so the
  limitation is currently costless here.
- **Credible general grammar candidate:** an array literal in `expression`, so
  declared `list` types are authorable. This is a real gap in the language, not a
  SEA Forge-specific need.

### L3 — role-to-entity binding is still not authorable in SEA text

- **Class:** grammar.
- **What was tested:** `entity "Case" in governance` alongside `role "Approver"`.
- **Result:** parses and validates, but `graph.entity_roles` is `{}`. The `in
  <identifier>` clause sets a domain, not a role binding. The graph model
  supports entity role bindings; the text surface cannot author them.
- **Consequence:** the model states role responsibilities through three honest
  role-to-role `relation` declarations and through prose on journeys and
  surfaces. CMMN `performerRef` is normally absent from projections, and
  responsibility cannot be validated.
- **Credible general grammar candidate, unchanged:** expose the graph's existing
  role-to-entity binding through text syntax with explicit cardinality and
  responsibility kind. Recurring well beyond interaction modeling.

### L4 — the Application Contract and Semantic Envelope have no CLI path

- **Class:** tooling.
- **What was tested:** every `domainforge` subcommand and
  `grep -rn "application\|envelope" domainforge-core/src/cli/*.rs`.
- **Result:** `parse`, `validate`, and `project` call
  `application::resolve::resolve_filesystem_graph` for module closure only. No
  subcommand calls `resolve_application_contract` or `resolve_semantic_envelope`.
  `domainforge parse --format json` returns a Graph with no `operations` or
  `records` key; `domainforge validate` therefore never emits an `APP` diagnostic.
- **Consequence:** the declared `get_canonical_journey` operation is **not**
  checked by the command an ordinary editor runs. It is checked only by the
  harness in `application-contract-harness.rs`. An editor who breaks the
  operation will see `Validation succeeded` from the CLI.
- **Mitigation in this repository:** `semantic-teeth.sh` and the harness are
  documented together in `README.md`; both must be run before the model is
  considered validated. Treat CLI-green as necessary, not sufficient.
- **Owner:** DomainForge. A `domainforge contract` / `domainforge envelope`
  subcommand would close this.

### L5 — arithmetic inside a policy `where` predicate silently matches nothing

- **Class:** evaluator. This is a correctness defect, not a missing feature.
- **What was tested:**
  `count(i in entity_instances where i.entity = "J" and i.a + i.b != i.total: i.jid) = 0`
  over data that violates the predicate.
- **Result:** the policy **passes**. Field-to-field and field-to-literal
  comparisons in `where` work correctly (verified separately); only arithmetic
  over instance fields fails, and it fails silently by matching no rows. The same
  arithmetic in the projection position fails loudly with
  `Projection in aggregation comprehension must reduce to a literal`.
- **Consequence:** per-row arithmetic invariants cannot be trusted in SEA. A
  policy written that way looks green and proves nothing.
- **How this model responds:** no policy uses arithmetic in a `where` predicate.
  The one per-row arithmetic invariant this model needs — that each journey's
  four maturity counts sum to its `observed_story_count` — is enforced by
  `reconcile.py` instead, and the reason is recorded there.
- **Owner:** DomainForge. Silent falsehood is worse than an error; this should
  either evaluate or raise.

### L6 — `forall` / `exists` over `entity_instances` cannot be scoped to one entity type

- **Class:** evaluator.
- **What was tested:** `forall i in entity_instances: (i.rank > 0)` in a model
  with two entity types, and the guarded form
  `forall i in entity_instances: (i.entity != "Journey" or i.rank > 0)`.
- **Result:** both evaluate to `UNKNOWN (NULL)` and fail validation, because a
  field absent on other entity types produces NULL and the `or` guard does not
  rescue it. Only the aggregation form with a `where` filter
  (`count(i in entity_instances where i.entity = "X": i.field)`) filters before
  projection and therefore works.
- **Consequence:** every invariant in this model is expressed as a counting or
  summing aggregation. Genuine universal quantification over a typed field —
  "every canonical journey has a non-empty completion condition" — is not
  expressible as a policy; it is enforced by the `min_length` field constraint
  instead, which is stronger anyway.
- **Owner:** DomainForge. A typed collection (`entity_instances of "X"`) or
  Kleene-correct `or` would resolve it.

### L7 — branching, guards, and recovery edges remain prose

- **Class:** grammar, with a product-model component.
- **Current representation:** the eight-phase skeleton is ordered and enforced
  (`ordinal`, `interaction_step_ordinals_complete`). Within a journey,
  `state_transition`, `recovery_paths`, and `next_decisions` are validated
  non-empty strings but are not a transition graph. DomainForge cannot prove that
  a journey has a terminal or a recovery path, and cannot distinguish settlement
  from execution termination structurally.
- **Why no workaround is attempted:** encoding a transition graph in typed
  entities would produce a large table that DomainForge still could not check for
  reachability, guard satisfaction, or terminal coverage. That is churn without
  teeth.
- **Credible general grammar candidate, unchanged:** typed transition nodes and
  edges with explicit order, outcome, guard, recovery, and composition semantics.
  This would also improve BPMN, CMMN, TLA+, and event projections.

### L8 — per-journey step emphasis is not provable

- **Class:** product-model.
- **Statement:** all twelve journeys traverse all eight skeleton phases; what
  varies is emphasis, not membership. The model therefore declares the ordered
  skeleton once and does not emit ninety-six journey-to-step rows that would
  carry no information.
- **Consequence:** the per-journey `reusable_steps` prose is the only record of
  which phases dominate a given journey, and it is not machine-checkable beyond
  non-emptiness.
- **When to revisit:** if a future journey is found that legitimately skips a
  phase, the universal claim breaks and an explicit `JourneyStepUse` binding
  entity becomes justified. Until then it would be churn.

### L9 — an operation's `policy_governed` access breaks graph validation

- **Class:** evaluator, surfacing as a real authoring constraint.
- **What was tested:** DomainForge's own flagship fixture
  `fixtures/application_generation/flagship/command-write.sea`, and a minimal
  reproduction.
- **Result:** both fail `domainforge validate` with
  `Policy 'order_total_within_limit' evaluation is UNKNOWN (NULL)`. A policy
  written to be evaluated in an operation's typed precondition context is not
  evaluable against the semantic graph, and the graph validator evaluates every
  policy in the file.
- **Consequence:** in one `.sea` file you may have graph-evaluable policies or
  operation-precondition policies, not both. This model declares
  `access public` on its single read operation, so no conflict arises, and it
  keeps all fifteen policies graph-evaluable.
- **Owner:** DomainForge. Policies bound to an operation should be excluded from
  graph evaluation, or marked so the graph validator can skip them.

### L10 — an entity key must be `string` or `uuid`

- **Class:** grammar, by design (APP005).
- **Effect here:** `CanonicalizationClassification` carries both
  `classification_id: string` (the key) and `class_value: CanonicalizationClass`
  (the enum-checked value), because the enum cannot itself be the key. The
  redundancy is small and the enum still has teeth (T03).
- **Assessment:** not a defect. Recorded so the duplication is not mistaken for
  an oversight.

## Not limitations, recorded to prevent regression

- **A green `domainforge validate` is not sufficient.** It does not check the
  operation (L4), and it will happily accept a policy whose `where` arithmetic is
  meaningless (L5). The model is validated only when `semantic-teeth.sh`,
  `reconcile.py`, and the harness have all run.
- **Consolidating the model into one file is now a choice, not a constraint.**
  The 0.15.0 resolver bug that forced consolidation is fixed and proven fixed
  (T24–T26). Consolidation is retained on the architectural grounds in
  `README.md`, and can be revisited on its merits.
- **A generated artifact does not prove runtime behavior.** Interface maturity
  values describe the surface, never the journey.
