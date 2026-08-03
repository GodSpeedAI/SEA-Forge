# SEA Forge Interaction Model Design

Date: 2026-08-03
Status: Approved for implementation

## Outcome

Reconstruct SEA Forge's interaction space from human intention downward and
encode it as a validated DomainForge model. The model will compress the 128
durable user stories in the governed Workbench UX epic into the smallest useful
set of canonical journeys without erasing differences in intent, transition,
capability, artifact, evidence, completion, or next decision.

The result becomes the semantic basis for a later Journey & Capability Map. It
does not include that map, grammar changes, or application implementation.

## Source Catalog and Evidence

The observed journey catalog is the 128 `X.Y` stories in
`.agents/specs/frontend/sea-forge-governed-workbench-ux-epic-v0.1.md`. The
epic's 16 journey areas organize product intent; they do not automatically
define canonical journey identity.

The analysis will correlate that catalog with:

- the minimum and full SEA Forge specifications;
- architecture decisions and execution reports;
- the Workbench routes, screens, navigation, and state machines;
- the SFWP method catalog, implemented server methods, schemas, and tests;
- CLI and kernel capabilities, records, and conformance tests;
- existing SEA sources and the SEA Forge DomainForge boundary; and
- DomainForge's current grammar, authoring guidance, CLI, examples, and
  projection behavior.

Each conclusion will distinguish declared intent, specified behavior,
implemented behavior, exercised behavior, evidenced behavior, inference, and
unresolved ambiguity. A route, screen, button, method, or implementation type
is evidence or an interface binding, never the canonical identity of a
journey.

## Canonicalization Method

Every observed story will be normalized against this tuple:

```text
actor + intention + initiating condition + state transition
+ capability + artifact + evidence + completion condition + next decision
```

Two stories may collapse only when those dimensions are materially equivalent.
Shared wording alone is insufficient. Each story will receive exactly one of
the required classifications:

- canonical;
- specialization;
- variant;
- composition;
- duplicate;
- obsolete;
- ambiguous; or
- unsupported.

Classification describes the story's relationship to the interaction model.
Implementation maturity is a separate axis:

```text
declared -> specified -> implemented -> exercised -> evidenced
```

Variation dimensions include actor, subject, entry point, interface, provider,
work source, lifecycle stage, assurance, authority outcome, recovery path, and
artifact maturity. The analysis may add a dimension only when multiple stories
need it.

## Canonical Journey Set

The model starts with 12 candidate canonical journeys. The 128-row stress test
may rename or split one when evidence proves a material semantic difference;
it may merge two only when their normalized tuples remain equivalent.

1. **Establish Trusted Cell Context** — enter a cell whose identity,
   configuration, integrity, semantic self-model, and readiness are explicit.
2. **Discover Lawful Affordances** — understand what exists, what is usable,
   what is allowed, and why a path is unavailable.
3. **Ground Work in Semantic Meaning** — supply, validate, inspect, pin, and
   project the domain model that gives work reproducible meaning.
4. **Form and Commit a Governed Case** — turn intent, template, or proposal
   into a validated, previewed, immutable case plan.
5. **Navigate and Adapt a Live Case** — understand the case horizon and change
   its lawful path without rewriting history.
6. **Resolve Human Judgment and Approval** — admit accountable human decisions
   and work through the same authority and evidence fabric.
7. **Execute Governed Work** — run human, command, projection, transition, or
   agent work within the granted boundary and retain its outcome evidence.
8. **Monitor, Intervene, and Recover** — observe concurrent work, intervene at
   the right scope, resume committed state, and construct a lawful recovery
   path.
9. **Evaluate, Settle, and Audit Outcomes** — judge immutable criteria from
   evidence, distinguish execution from settlement, and verify provenance and
   ledger truth.
10. **Reuse Demonstrated Knowledge and Capability** — recall settled history
    and use evidence-backed capability without making a derived view
    authoritative.
11. **Transform and Mature Governed Artifacts** — project, derive, promote,
    quarantine, and verify artifacts through explicit maturity gates.
12. **Transfer and Adopt Governed Assets** — export, verify, import, isolate,
    and locally adopt assets across cell boundaries.

Thoth is a bounded composition and interface projection across discovery,
proposal, case adaptation, and audit. It is not a separate interaction
grammar. Maintenance composes trusted-context establishment, recovery, and
verification. Identity and authority are cross-cutting prerequisites and
steps, not screen-shaped journeys.

## Journey Grammar

The common journey skeleton is:

```text
intention
-> select or supply context
-> determine authority and availability
-> invoke a capability
-> observe a state change or artifact
-> expose evidence and limitations
-> settle or reach a decision
-> offer the next lawful affordance or terminal condition
```

Read-only journeys may resolve authority through public or disclosure-scoped
access, but they do not bypass it. Failure, denial, escalation, expiry,
interruption, and stale state remain governed outcomes with explicit recovery
or terminal semantics.

Larger journeys compose reusable steps. A composition does not copy the
semantics of its components, and an interface variant does not redefine the
canonical journey.

## SEA Representation

The model will use current SEA primitives:

- `Role` for actor categories;
- `Entity` for interaction concepts, states, capabilities, artifacts,
  evidence, decisions, and interface bindings;
- `Instance` for canonical journeys and reusable catalog entries;
- `Flow` for meaningful transitions, with supported annotations for journey,
  action, capability, evidence, completion, and recovery metadata;
- `Relation` for relationships the current role-level relation surface can
  express honestly; and
- `Policy` only for invariants DomainForge can evaluate over the graph.

The model will not use SEA's typed `operation` declaration for an end-to-end
journey. The current application contract is intentionally limited to a
synchronous, single-aggregate request/response operation. Applying it to an
adaptive, multi-step journey would change the meaning of both constructs.

The current SEA projection-contract targets do not include UI, API, CLI, or
agent interfaces. These will be modeled as ordinary interface-projection
concepts bound to canonical steps. The limitation will be explicit.

## Artifact Layout

Durable source belongs under `.sea/interaction/`:

```text
.sea/interaction/
|-- interaction-model.sea
|-- interaction-domain.sea
|-- canonical-journeys.sea
|-- journey-variants.sea
|-- interaction-projections.sea
|-- canonicalization-matrix.csv
|-- canonical-journey-catalog.md
|-- journey-grammar.md
|-- coverage-report.md
|-- README.md
`-- validation/
    |-- domainforge-output.txt
    |-- diagnostics.md
    `-- limitations.md
```

`interaction-model.sea` is the canonical validation entry point. The other SEA
files are modules, not competing models. If DomainForge's current module
resolution cannot preserve the intended semantics, the entry file will remain
the single validated source and smaller files will become explanatory or
generated companions. No invalid SEA file will remain in the final tree.

The exhaustive 128-row mapping belongs in CSV. The SEA source contains the
stable ontology, canonical transformations, reusable variation dimensions,
representative bindings, and enforceable invariants rather than 128 noisy
copies.

A consolidated report under `.agents/reports/` will summarize results and link
to the canonical artifacts. It will not duplicate the source model.

## Required Human-Readable Outputs

The artifact set will provide:

- one classification row for every observed story ID;
- a canonical journey catalog with intentions, jobs, conditions, steps,
  capabilities, transitions, artifacts, evidence, completion, decisions,
  recovery, variants, and maturity;
- a concise journey grammar;
- coverage and compression counts reconciled to 128;
- DomainForge commands, version, diagnostics, and inspected outputs;
- grammar limitations with recurrence and enforcement evidence; and
- a handoff that lets the Journey & Capability Map agent continue without
  repeating canonicalization.

## Validation

Validation will use the DomainForge CLI from
`/home/sprime01/projects/domainforge` through its declared development
environment because the host shell does not expose `cargo` or an installed
`domainforge` binary directly.

The validation sequence will:

1. capture the CLI version and relevant help;
2. parse and semantically validate the canonical entry model;
3. inspect AST output for semantic preservation;
4. run a small semantic projection, expected to be RDF;
5. run a process or case projection only if it preserves the modeled meaning;
6. inspect generated output rather than trust process exit alone;
7. reconcile all matrix and coverage counts; and
8. run narrow repository checks appropriate to model and documentation files.

Per the owner's instruction, `just context-check` is excluded because it is not
working in this checkout. The report will record it as skipped and will not
claim that gate.

## Failure Behavior

Syntax errors, unresolved imports, ambiguous references, invalid graph
relationships, projection distortion, missing story rows, count disagreement,
and unsupported semantics block completion. The model will be revised or the
limitation made explicit; invalid or misleading artifacts will not remain.

Unknown implementation maturity stays unknown. Declared product intent will
not be presented as implemented behavior. Lack of runtime support does not, by
itself, erase a stable user intention from the canonical interaction model.

## Boundaries

This task will not:

- change the SEA grammar or DomainForge implementation;
- change SEA Forge frontend, API, backend, runtime, persisted schemas, public
  interfaces, dependencies, CI, or deployment;
- build the broader Journey & Capability Map;
- treat generated output, routes, process exit, or agent narration as proof;
  or
- invent product capabilities without marking them as intended, inferred, or
  unsupported.

The work will preserve unrelated changes in the current checkout. Any conflict
between repository specifications and the requested model will stop the work
and be reported instead of silently resolved in source.

## Completion Standard

The work is complete when all 128 stories are classified, the canonical model
validates under DomainForge, its output has been inspected, coverage and
limitations are explicit, and the next agent can build the Journey & Capability
Map without repeating this analysis.
