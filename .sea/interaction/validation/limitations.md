# Interaction Grammar and Projection Limitations

## Assessment Rule

A future grammar direction is recommended only when the concept recurs, carries an enforceable invariant, benefits multiple projections, and is represented poorly today. The assessment below separates general grammar candidates from resolver and projector defects. It does not propose SEA Forge-specific syntax where a general typed construct would suffice.

## 1. First-Class Journey, Step, and Composition Identity

- **interaction concept:** Canonical journey identity, reusable step identity, and composition between journeys.
- **desired semantics:** A journey should own typed steps and composition links while retaining one stable identity across UI, API, CLI, agent, RDF, process, and case projections.
- **current workaround:** Twelve `CanonicalJourney` instances, seven `JourneyStep` instances, `JourneyVariant` records with `classification: "composition"`, and string-valued fields and flow annotations.
- **cost:** The model cannot prove that a journey uses a declared step or composes declared journeys; RDF drops all journey and step instances; downstream consumers must reconstruct identity from conventions and prose.
- **local or recurring:** Recurring locally across all 12 journeys, 7 steps, 21 composition-classified observed stories, 12 transformation flows, and 38 interface bindings. Cross-domain recurrence has not yet been demonstrated.
- **evidence for first-class support:** Identity is stable, repeated, projection-relevant, and currently lossy, but the evidence does not yet justify a domain-specific `journey` keyword in the base grammar. General typed references and transition constructs below can address most invariants first.
- **possible future direction:** **Not yet a dedicated grammar candidate.** Add the general reference, transition, condition, and binding primitives below; revisit first-class journey syntax only if the same aggregate recurs outside this interaction model.

## 2. Instance-to-Instance Typed Relations and References

- **interaction concept:** References from journeys to reusable steps, variants, projections, capabilities, evidence, and other journey instances.
- **desired semantics:** Instance fields and relations should point to declared instances or types, fail on dangling references, and survive semantic projections.
- **current workaround:** Semicolon-separated identifiers such as `"CJ01; CJ02"` and descriptive strings such as `reusable_steps`, plus external checks against the CSV and catalog.
- **cost:** Validation accepts misspelled or nonexistent IDs; relations cannot be traversed reliably; 76 instance records disappear from RDF; every consumer must parse conventions independently.
- **local or recurring:** Recurring across 12 journeys, 19 variant/dimension records, 38 interface bindings, and the exhaustive 128-row mapping.
- **evidence for first-class support:** Referential integrity is enforceable, useful to RDF/OWL, CMMN, BPMN, API/schema, and graph consumers, and poorly represented as strings.
- **possible future direction:** **Credible general grammar candidate:** typed instance references and instance-to-instance relations with declared cardinality and namespace-aware resolution.

## 3. Ordered Steps and Conditional or Recovery Transitions

- **interaction concept:** Ordered journey steps, branching authority outcomes, denial/escalation paths, and recovery transitions.
- **desired semantics:** A transition graph should express sequence, branch guards, outcome kinds, retry/recovery edges, and composition without treating order as narration.
- **current workaround:** Each canonical flow stores `action_sequence` and `recovery` as long annotations; each journey stores a prose `state_transition`.
- **cost:** DomainForge cannot validate order or reachability; RDF drops the annotations; BPMN/CMMN cannot lower the real journey structure; recovery is unqueryable prose.
- **local or recurring:** Recurring in every canonical journey and throughout denial, escalation, timeout, cancellation, stale, quarantine, retry, replan, and terminal variants.
- **evidence for first-class support:** Order and branching encode safety invariants, and the same structure would improve graph, process, case, verification, event, and interface projections.
- **possible future direction:** **Credible general grammar candidate:** typed transition nodes and edges with explicit order, outcome, guard, recovery, and composition semantics.

## 4. Entry and Completion Conditions and Terminal Decisions

- **interaction concept:** Preconditions, entry conditions, acceptance conditions, accountable decisions, next decisions, and explicit terminal outcomes.
- **desired semantics:** Entry and completion should be evaluable expressions over typed state and evidence; terminal decisions should be distinguishable from recoverable states and execution termination.
- **current workaround:** `preconditions`, `completion_condition`, and `next_decisions` string fields duplicated in flow annotations, with only two unrelated graph-wide policies evaluated.
- **cost:** Validation cannot prove that a journey has a terminal or recovery path, cannot distinguish settlement from termination structurally, and cannot project real sentries or acceptance criteria.
- **local or recurring:** Recurring across all 12 journeys and the common interaction skeleton; central to authority, settlement, approval, quarantine, adoption, and recovery.
- **evidence for first-class support:** These are enforceable state and evidence predicates with value across CMMN, BPMN, TLA+, Alloy, Gauge, Cedar, event, and UI affordance projections.
- **possible future direction:** **Credible general grammar candidate:** typed entry, completion, and terminal-decision expressions referencing declared state, evidence, and policy concepts.

## 5. Cardinality and Integrity Policies over Instance Fields

- **interaction concept:** Exactly 12 uniquely identified canonical journeys, valid `CJ01`–`CJ12` bindings, required interface fields, and valid classification/maturity values.
- **desired semantics:** Policies should quantify over instances, inspect typed fields, enforce uniqueness and cardinality, and validate referenced instance identities.
- **current workaround:** External Python and `jq` checks enforce counts and vocabularies. SEA policies are limited to `count(flows) > 0` and existence of the `GovernedEvidence` resource.
- **cost:** DomainForge validation can report zero violations even if a journey ID is duplicated, an interface points to `CJ99`, or a required instance field is absent or malformed.
- **local or recurring:** Recurring across the ontology, all 76 instances, all 128 mappings, and every downstream interface binding.
- **evidence for first-class support:** The invariants are deterministic and enforceable, and one policy surface would benefit validation plus RDF/SHACL, schema, formal verification, and code projections.
- **possible future direction:** **Credible general evaluator/grammar candidate:** expose instance collections and typed fields to policy expressions; add uniqueness, required-field, reference, and cardinality predicates rather than interaction-specific policy syntax.

## 6. UI, API, CLI, and Agent Projection Targets

- **interaction concept:** Interface surfaces binding to canonical journeys and steps without becoming canonical identity or availability proof.
- **desired semantics:** Typed interface kinds, entries or methods, source evidence, maturity, limitations, and references to the journey semantics they project.
- **current workaround:** Thirty-eight ordinary `InterfaceProjection` instances with seven untyped string fields. DomainForge's projection target list names output generators, not these product-interface bindings.
- **cost:** Binding IDs and maturity values are unchecked; target-versus-implemented naming drift remains prose; RDF drops all bindings; consumers cannot distinguish an unavailable preview from an exercised surface without custom parsing.
- **local or recurring:** Recurring across 17 Workbench routes, 23 advertised SFWP methods, major CLI families, and Thoth/delegation agent surfaces.
- **evidence for first-class support:** The same typed binding has enforceable reference and vocabulary rules and serves RDF, documentation, API catalogs, UI navigation, capability maps, and availability reporting.
- **possible future direction:** **Credible general grammar candidate:** a typed interface-binding declaration separate from code-generation `projection` targets, with references, availability/maturity state, source evidence, and limitations.

## 7. Role-to-Entity Binding

- **interaction concept:** Binding actors and accountable roles to interaction concepts, journey steps, decisions, or executable work.
- **desired semantics:** A declared role-to-entity association that can identify performers, owners, approvers, sponsors, and auditors without pretending all relations are role-to-role responsibilities.
- **current workaround:** Three honest role-to-role `Relation` declarations plus actor and responsibility prose inside journey and interface instances.
- **cost:** DomainForge's graph supports entity role bindings but the SEA text surface cannot author them; CMMN `performerRef` is therefore normally absent; role responsibilities cannot be validated or projected consistently.
- **local or recurring:** Recurring across 9 roles, every governed journey, approval and separation-of-duty rules, agent sponsorship, audit, and human work.
- **evidence for first-class support:** The binding is enforceable, already supported in the graph model, and directly benefits CMMN, BPMN, Cedar, RDF, API, and UI responsibility projections.
- **possible future direction:** **Credible general grammar candidate:** expose the existing graph's role-to-entity binding through text syntax with explicit cardinality and responsibility kind.

## 8. Classification and Maturity Vocabularies

- **interaction concept:** The closed classification axis (`canonical`, `specialization`, `variant`, `composition`, `duplicate`, `obsolete`, `ambiguous`, `unsupported`) and independent maturity axis (`declared`, `specified`, `implemented`, `exercised`, `evidenced`).
- **desired semantics:** Closed, typed values with allowed transitions and no accidental promotion of classification into implementation proof.
- **current workaround:** Nineteen `JourneyVariant` instances and string fields in journey, interface, CSV, and Markdown artifacts; external scripts validate the CSV vocabulary.
- **cost:** SEA validation cannot reject a misspelling or illegal maturity transition, and RDF omits the vocabulary instances. Separate axes are maintained only by author discipline and external checks.
- **local or recurring:** Recurring across all 128 observed stories, 12 journey summaries, and 38 interface bindings; closed vocabularies also recur broadly outside interaction modeling.
- **evidence for first-class support:** Closed values and transitions are enforceable and projection-relevant, but they do not justify interaction-specific keywords.
- **possible future direction:** **Credible general grammar candidate:** reusable enum/value declarations and constrained typed fields, with transition rules expressed through the general policy surface.

## 9. Same-Namespace Imported-Instance Resolution

- **interaction concept:** Modular reuse of an entity type by an instance declared in another file under the same namespace.
- **desired semantics:** Both named and wildcard relative imports should make an exported entity available to same-namespace instance resolution exactly as documented.
- **current workaround:** One 1,059-line committed semantic source with no imports; four declaration-free companion indexes.
- **cost:** Semantic modules cannot be maintained independently; source navigation and ownership are poorer; any future declaration must be added to the consolidated file.
- **local or recurring:** Immediately local to this model but fundamental to every modular model that shares a namespace and instances exported entity types.
- **evidence for first-class support:** Both wildcard and named imports parse but fail graph construction with the exact same `Entity 'ReusableStep' not found` diagnostic. The grammar already accepts the intended syntax.
- **possible future direction:** **Not a grammar candidate:** correct namespace-aware imported symbol resolution and add a regression test for named and wildcard same-namespace imported instances. Do not invent another import form.

## 10. RDF Preservation of Instances, Policies, and Annotations

- **interaction concept:** Semantic graph projection of authored journeys, reusable steps, interface bindings, policy constraints, and annotated flow meaning.
- **desired semantics:** RDF/OWL should retain stable instance identities, typed fields and references, policies, and flow annotations so projected journey meaning remains traceable to source.
- **current workaround:** Use RDF only for the ontology spine, role relations, resources, and state-to-state flow edges; return to SEA AST, CSV, and Markdown for journey identity and interaction semantics.
- **cost:** All 76 instances, 2 policies, and all flow annotations disappear. The 12 journey edges receive generated flow IDs without `journey_id`; OWL additionally omits flow and relation individuals.
- **local or recurring:** Recurring for any instance-heavy or annotation-heavy SEA model and every consumer expecting RDF to represent more than declared entity/role/resource topology.
- **evidence for first-class support:** Fixed-time inspection found 70 Turtle/JSON-LD nodes but none of the representative journey, step, interface, policy, or annotation identifiers. The AST proves those constructs were parsed before projection.
- **possible future direction:** **Not primarily a grammar candidate:** extend the RDF projection IR and vocabulary to serialize instances, typed fields/references, policies, and annotations; add projection teeth checks before claiming lossless semantic-graph output.

## Candidate Summary

Seven general additions have credible evidence now: typed instance references, typed transition graphs, typed entry/completion/terminal conditions, instance-aware integrity policies, interface-binding declarations, role-to-entity binding syntax, and reusable constrained vocabularies. A dedicated journey keyword remains premature; same-namespace imports need a resolver fix; and RDF loss needs a projector fix.
