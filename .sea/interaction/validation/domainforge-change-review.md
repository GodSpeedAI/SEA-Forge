# DomainForge Change Review

Scope: the ten most recent commits in `/home/sprime01/projects/domainforge`,
read from the local repository rather than from release notes, plus the
uncommitted working-tree delta. This review decides which previous modeling
constraints are obsolete.

## Repository state at review time

| Property | Value |
| --- | --- |
| Repository | `/home/sprime01/projects/domainforge` |
| Branch | `feat/rdf-instance-policy-projection` |
| HEAD | `c8f7300dc181824e897d06a2861b2053001a4292` (2026-08-03) |
| Reported version | `domainforge 0.16.0` |
| Worktree | **Dirty** — 41 changed paths |
| Binary used for all evidence | `target/release/domainforge`, built from this worktree with `cargo build --release --bin domainforge --features cli` |

The globally installed `/home/sprime01/.cargo/bin/domainforge` also reports
`0.16.0` but is built from the released tree. It differs materially from the
local build in one respect recorded under "Working-tree delta" below.

`CHANGELOG.md` in the DomainForge repository stops at `0.11.0` and is not a
reliable guide to current behavior. Every claim below is grounded in the
implementation, its tests, and direct execution.

## Commits reviewed

| # | SHA | Subject | Material to SEA modeling? |
| --- | --- | --- | --- |
| 1 | `c8f7300` | chore(devbox): install domainforge 0.16.0 (#121) | No — toolchain pin only |
| 2 | `0cb9c73` | chore: release main (#119) | No — release plumbing |
| 3 | `5ad44be` | feat(application): complete typed entity interaction support (#120) | **Yes — decisive** |
| 4 | `21f836a` | fix(release): align version files, fix npm package name for 0.15.0 (#118) | No — packaging |
| 5 | `212e17e` | chore: release main (#117) | No — release plumbing |
| 6 | `3802392` | feat: SEA application contract (M0) + full projection-targets line (#116) | **Yes — decisive** |
| 7 | `9a3d7d1` | chore: release main (#105) | No — release plumbing |
| 8 | `262b48b` | feat(projection): domain code-gen targets, authority signing, proof harness (#115) | Partly — new projection families, none of which changes interaction semantics |
| 9 | `f1739af` | ci(pypi): serialize sdist upload (#113) | No — CI |
| 10 | `863b586` | fix(npm): mark root package private (#112) | No — packaging |

Two commits — `5ad44be` and `3802392` — carry the entire modeling-relevant
change. Commit `262b48b` adds CloudEvents, AsyncAPI, Devbox, and related
projection targets; those broaden output formats but introduce no new way to
state interaction meaning.

### `3802392` — SEA Application Contract, Milestone 0

Introduces `record`, `enum`, and `operation` as contextual declarations,
typed entity bodies with an aggregate `key`, the shared field/type/constraint
grammar (`scalar`, `ref<T>`, `list<T>`, `quantity<U>`, `optional`, `default`,
`min_length`/`max_length`/`min`/`max`/`min_items`/`max_items`/`exclusive_min`/
`exclusive_max`/`pattern`), the `role<...>` contextual expression leaf, the
`ApplicationContract` and `CanonicalSemanticEnvelope` documents, canonical JSON
and SHA-256 hashing, and the `APP001`–`APP015` diagnostic family.

Authority: `domainforge-core/grammar/sea.pest` lines 238–321;
`docs/reference/sea-application-contract.md` (v0.1, Accepted);
`docs/specs/ADR-013-sea-application-contract.md`;
`domainforge-core/src/application/{contract,envelope,canonical,validate,diagnostic}.rs`;
fixtures `fixtures/application_generation/flagship/{command-write,query-read}.sea`.

### `5ad44be` — Typed entity interaction support

Completes the half that the old SEA Forge model actually needed:

- **Concrete typed entity instances are validated.** `graph/entity_validation.rs`
  checks required fields, rejects undeclared fields, type-checks every scalar,
  enum, list, quantity, and typed reference, applies every field constraint,
  and enforces entity-key uniqueness per entity type.
- **`ref<Entity>` dangling-reference rejection.** A reference value is the
  target's key value and must match an instance in the complete resolved
  closure.
- **Deterministic transitive filesystem module closure.** `module/resolver.rs`
  plus `application/resolve.rs::resolve_filesystem_graph`, wired into CLI
  `parse`, `validate`, and `project`. An imported entity type can now back a
  same-namespace instance declared in another file.
- **Instance-aware policies.** `policy/quantifier.rs` adds the
  `entity_instances` collection, exposing `id`, `name`, `entity`, `namespace`,
  and every authored field to quantifiers and aggregations.
- **Regression fixtures named for this model.**
  `fixtures/sea_forge_interaction_i1_i3/` and
  `domainforge-core/tests/sea_forge_interaction_fixture_tests.rs` were written
  directly against the previous limitations report.

## Working-tree delta (uncommitted)

Branch `feat/rdf-instance-policy-projection` carries uncommitted changes to
`domainforge-core/src/kg.rs`, `domainforge-core/src/projection/rdf/ontology.rs`,
`docs/rdf-projections.md`, and `docs/specs/SDS-005-knowledge-graph-module.md`.

Measured effect on this model's RDF projection:

| Binary | `model.ttl` lines | `CJ01` | `workbench_readiness_routes` | `twelve_canonical_journeys` | `IB001` |
| --- | ---: | ---: | ---: | ---: | ---: |
| Released `~/.cargo/bin/domainforge` 0.16.0 | 333 | 8 | 0 | 0 | 0 |
| Local worktree build | 2101 | 14 | 13 | 9 | 1 |

**This is unreleased work in progress.** Instance and policy preservation in
RDF is real in the local tree and absent from released 0.16.0. The canonical
model therefore must not depend on RDF carrying journey, binding, or policy
identity until that branch lands. See `limitations.md` L1.

## Capability change matrix

| Capability | Previous assumption (0.15.0-era model) | Current reality | Evidence | Modeling consequence |
| --- | --- | --- | --- | --- |
| Typed entity bodies | Unavailable; entities were name-only | Available with one aggregate `key`, required/optional fields, defaults | `sea.pest` 85–91; `sea-application-contract.md` §4 | All seven typed entities, including `ProductPurpose`, carry bodies |
| Entity keys and uniqueness | Not expressible | `key` field, unique per entity type | `entity_validation.rs` 63–74 | `journey_id`, `binding_id`, `surface_id`, `purpose_id` are real keys; teeth T06 |
| Required vs optional fields | Not expressible | `optional` marker; missing required field rejected | `entity_validation.rs` 30–41 | `value_derived_concepts` is honestly optional; teeth T08 |
| Undeclared fields | Silently accepted | Rejected | `entity_validation.rs` 43–61 | Typos cannot enter the catalog; teeth T07 |
| Scalar type checking | None | `string`/`int`/`decimal`/`bool`/`timestamp`/`uuid` checked | `entity_validation.rs` 187–206 | Counts are `int`, not prose; teeth T11 |
| Scalar constraints | None | `min_length`, `max_length`, `min`, `max`, items, exclusive bounds | `entity_validation.rs` 223–274 | Empty prose and out-of-range counts rejected; teeth T12, T13 |
| Pattern constraints | None | `(pattern <Pattern>)` against a declared SEA `Pattern` | `sea.pest` 269, 279 | `CJ13` and `B90` are structurally illegal; teeth T09, T10, T14b |
| Enums | Not expressible | `enum X { m = "wire" }`, closed, wire-checked | `sea.pest` 247–248; `entity_validation.rs` 157–173 | Classification, maturity, interface kind, and phase are closed; teeth T01–T03 |
| Typed instance references | Semicolon-separated ID strings | `ref<Entity>`, value is the target key | `entity_validation.rs` 130–156 | `InterfaceBinding` replaces `"CJ01; CJ02"` |
| Dangling-reference rejection | Impossible; `CJ99` validated clean | Rejected across the whole closure | `entity_validation.rs` 147–155 | Teeth T04, T05, T14, T20 |
| Instance-aware policies | Only `count(flows)` and resource existence | `entity_instances` collection with fields | `policy/quantifier.rs` 377–407 | 14 counting/summing invariants; teeth T14c–T23b |
| Same-namespace imported instances | Blocked (`Entity 'ReusableStep' not found`) | Resolved through the transitive closure | `cli_module_closure_tests.rs`; teeth T24, T25 | The consolidation workaround is obsolete; consolidation is now a choice |
| Records | Not expressible | `record X { field... }`, identityless | `sea.pest` 244–245 | Operation input/output |
| Application operations | Not expressible | Full §5.1 machine contract | `sea.pest` 282–321 | One read boundary declared |
| Application Contract document | Did not exist | `domainforge-application-contract/v1` with hashes | `application/resolve.rs` 97–130 | Inspected; see `domainforge-output.txt` §5 |
| Canonical Semantic Envelope | Did not exist | `domainforge-semantic-envelope/v1`, all semantic declarations plus `semantic_closure_hash` | `application/envelope.rs` 897–955 | The lossless canonical projection; 266 declarations including all 168 instances and 16 policies |
| RDF instance/policy preservation | Lossy | Lossy in released 0.16.0; preserved in the uncommitted branch | Table above | Still not dependable; `limitations.md` L1 |
| CLI reach into operations | n/a | **None** — no subcommand resolves contract or envelope | `cli/mod.rs`; `grep application cli/*.rs` | `limitations.md` L4; harness required |
| Arithmetic in policy `where` | n/a | Silently matches nothing | Probe `p13.sea`; `limitations.md` L5 | Per-row arithmetic moved to `reconcile.py` |
| List literals in instances | n/a | **Not authorable** — no array literal in `expression` | `sea.pest` 444; probe `p1.sea` | `list<ref<T>>` fields are undeclarable; drove the normalized binding entity |

## Consequence for the previous model

Of the ten limitations in the previous report, four are resolved outright
(instance references, instance-aware policies, closed vocabularies,
same-namespace imports), three are substantially resolved through general typed
constructs (journey/step identity, entry conditions, interface bindings), one is
resolved only in unreleased work (RDF preservation), one is unchanged
(role-to-entity binding syntax), and one is superseded by the modeling approach
(ordered transitions). `limitations.md` records the current, re-tested set.
