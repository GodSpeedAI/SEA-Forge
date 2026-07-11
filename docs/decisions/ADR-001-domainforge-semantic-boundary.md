# ADR-001: Use DomainForge as SEA Forge's semantic engine

## Status

Accepted

## Date

2026-07-11

## Context

SEA Forge governs capability execution: it authorizes actions before side
effects, isolates execution, records evidence, and settles declared outcomes.
Its full-system specification also says that `.sea` defines the world in which
that governed work occurs.

DomainForge already provides the canonical Rust implementation of SEA. Its
`domainforge-core` library owns the grammar, parser, semantic graph, concept
identities, semantic validation, policy and authority evaluation, and
deterministic projections. It also exposes in-memory projection sinks. The SEA
Forge v0.1 demo instead writes a JSON fixture named `model.sea` and validates it
with a private shape check. That fixture proves SEA Forge's lifecycle but does
not implement DomainForge compatibility.

The previous full-system design treated DomainForge mostly as a late projection
adapter and described `.sea` as one of its outputs. That direction is incomplete:
DomainForge primarily consumes `.sea`, constructs a validated semantic graph,
and projects that graph to other formats.

## Decision

DomainForge owns semantic meaning; SEA Forge owns governed execution.

SEA Forge will add a first-party `sea-forge-domainforge` crate at M0. That crate
will depend on an exact reviewed `domainforge-core` version with default features
disabled and only required features enabled. DomainForge's `signing` feature is
mandatory when an action-gating configuration requires cryptographic fact
verification. The adapter will remain outside `sea-forge-core`, so the
synchronous minimum kernel retains its existing dependency and proof boundary.

The adapter will provide three side-effect-free capabilities:

1. Load an authored or synthesized `.sea` source set through DomainForge's
   namespace-aware parser and semantic validator.
2. Translate DomainForge authority results and traces into candidate SEA Forge
   governance verdicts and evidence.
3. Return DomainForge projection artifacts through in-memory library APIs.

SEA Forge will authorize and materialize every returned artifact. The built-in
adapter will not invoke the `domainforge` CLI, write projection files directly,
run external projection tools, or access the network.

SEA Forge will record a hash-pinned `DomainModelRef` wherever a plan, authority
decision, projection, or semantic envelope depends on a `.sea` world. The ref
will identify all resolved sources, the DomainForge and adapter versions, the
canonical semantic-model hash, consulted concept IDs, and validation evidence.
Changing any source or version creates a new ref.

DomainForge decisions remain candidate evaluations. SEA Forge's authority
mediator applies its hard boundaries and cross-engine precedence and owns the
final `AuthorityDecision`. DomainForge `Reject` and `Deny` normalize to `deny`,
`Escalate` to `escalate`, and `Allow` to `allow`. `NotApplicable` contributes no
candidate and becomes `deny` when policy requires DomainForge for that surface.

Milestones split the integration by responsibility:

- M0 adds real `.sea` loading, semantic validation, model identity, and authority
  normalization.
- M2 resolves plan concept references against the pinned semantic model before
  authority or case activation.
- M5 adds governed `.sea` synthesis and DomainForge projections. Synthesis and
  DomainForge validation remain separate stages.

The v0.1 JSON fixture and self-validator remain unchanged as minimum-kernel
conformance machinery. Documentation must label them as a stub rather than a
DomainForge model.

## Alternatives Considered

### Reimplement SEA semantics in SEA Forge

Rejected. Two parsers, graph models, concept-ID schemes, and projection engines
would drift. SEA Forge would stop governing the DomainForge world and start
creating a competing one.

### Invoke the DomainForge CLI as the built-in integration

Rejected as the primary path. The CLI owns filesystem writes and some projection
options can invoke external tools. Wrapping it as a child process would make
side-effect attribution and artifact authorization coarser than the available
in-memory library boundary. A separately governed CLI operation may still exist
for compatibility or operator workflows.

### Let DomainForge issue the final authority decision

Rejected. DomainForge evaluates semantic and authority rules within the modeled
world; SEA Forge also enforces filesystem, process, sandbox, API, git, approval,
evidence, and settlement boundaries. Giving either evaluator a bypass would
violate the one-authority-fabric invariant.

### Defer all DomainForge integration to M5

Rejected. Code projection can wait until M5, but a system that claims to govern
a `.sea`-defined world must load, validate, identify, and reference that world
before it plans or authorizes work. Those foundations belong in M0 and M2.

## Consequences

- Full SEA Forge gains a direct, explicit `domainforge-core` dependency in the
  adapter crate; v0.1 does not.
- DomainForge version and semantic-model identity become evidence-bearing inputs
  to planning, authority, projection, and settlement.
- Invalid syntax, unresolved imports, semantic errors, source drift, incompatible
  versions, and unknown concept IDs fail before side effects.
- SEA Forge must test verdict normalization and prove that the built-in adapter
  performs no direct filesystem, network, CLI, or external-tool side effects.
- `.sea` synthesis becomes a governed source-producing stage. DomainForge parse
  and semantic validation determine whether that source can feed projections.
- DomainForge outputs and SEA Forge indexes remain derived views. `.sea` owns
  semantic truth; append-only SEA Forge records own governance truth.

## References

- `.agents/specs/spec-full.md` §§2.2, 6.2, 7.0a, 10.4a, 12, and 17
- `.agents/specs/spec-minimum.md` §11
- DomainForge `domainforge-core/src/lib.rs`
- DomainForge `docs/specs/ADR-001-sea-dsl-semantic-source-of-truth.md`
