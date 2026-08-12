# DomainForge Validation Diagnostics

What was run, what the resulting semantic objects actually contain, and which
errors were corrected on the way. A green exit status is reported here only
alongside the object it produced.

## Toolchain identity

| Property | Value |
| --- | --- |
| Binary | `/home/sprime01/projects/domainforge/target/release/domainforge` |
| Reported version | `domainforge 0.16.0` |
| Built from | branch `feat/rdf-instance-policy-projection`, HEAD `c8f7300`, **dirty worktree** |
| Build command | `cargo build --release --bin domainforge --features cli` |
| Library harness | `validation/application-contract-harness.rs` against `domainforge-core` at the same path |

The `--features cli` flag is required; `cargo build --release --bin domainforge`
alone fails with `requires the features: cli`.

The released `~/.cargo/bin/domainforge` also reports `0.16.0` but differs in RDF
output (`limitations.md` L1). The local build is the authority for this evidence.

## Semantic validation

```
$ domainforge validate --format human --no-color .sea/interaction/interaction-model.sea
Validation succeeded: 0 violations total
```

Zero violations means all sixteen policies evaluated to satisfied **and** every
one of the 168 typed instances passed field, type, constraint, key-uniqueness,
and reference checks. Under the previous model the same command proved far less,
because 76 instances were schemaless and only two policies existed.

## Semantic graph inspection

`domainforge parse --format json` was inspected, not merely generated:

| Collection | Count |
| --- | ---: |
| entities | 39 |
| entity_contracts | 7 |
| enum_contracts | 4 |
| patterns | 3 |
| roles | 9 |
| resources | 5 |
| relations | 3 |
| flows | 16 |
| entity_instances | 168 |
| policies | 16 |

Entity instances by type: 12 `CanonicalJourney`, 90 `InterfaceBinding`,
38 `InterfaceSurface`, 11 `VariationDimension`, 8 `JourneyStep`,
8 `CanonicalizationClassification`, 1 `ProductPurpose`.

The 39 entities are 7 typed entities, 8 schemaless ontology entities, and 24
journey entry/completion states. `entity_contracts` = 7 confirms that exactly
the seven intended entities carry typed bodies and that the remaining ontology
entities are deliberately schemaless.

## AST inspection

`domainforge parse --ast --format json` yields declarations that match the graph
and additionally retain the application-contract nodes the graph does not carry:

| Node | Count |
| --- | ---: |
| Entity | 39 |
| Instance | 168 |
| Flow | 16 |
| Policy | 16 |
| Role | 9 |
| Resource | 5 |
| Enum | 4 |
| Relation | 3 |
| Pattern | 3 |
| Record | 2 |
| Operation | 1 |
| Export | 33 |

`Operation` and `Record` appear in the AST but **not** in the semantic graph.
That asymmetry is the direct evidence for `limitations.md` L4.

## Canonical Semantic Envelope inspection

Resolved through the library harness, since no CLI path exists.

| Field | Value |
| --- | --- |
| `schema_version` | `domainforge-semantic-envelope/v1` |
| `producer` | `domainforge-core 0.16.0` |
| `language_schema_version` | `domainforge-ast/v3` |
| `interpretation_version` | `domainforge-interpretation/v1` |
| `source_set_hash` | `sha256:6f5bb9d515304c561c82e17bb4f5b84c84a53f7452911aeb10ff61d5b02ebaca` |
| `semantic_closure_hash` | `sha256:437e7ee2c0cf08ecd13d2d0beb2aa38830a84202131f4ed342accd6b7a1c71fc` |
| `self_hash` | `sha256:96b75ef044d42550d36c5cb8597d2523daa43f88fe3db2fdb5402fa320634a96` |
| modules | `interaction-model.sea`, content hash `sha256:f36c876172a7699e5f85144147eb94ea484eefeb4c0c9e305336f28c88d3d833` |
| `resolved_references` | 14 |
| `semantic_declarations` | **266** |

Declaration kinds in the envelope: 168 `instance`, 39 `entity`, 16 `flow`,
16 `policy`, 9 `role`, 5 `resource`, 4 `enum`, 3 `pattern`, 3 `relation`,
2 `record`, 1 `operation`.

Two properties were checked rather than assumed:

- **Instances survive.** A sampled instance declaration retains its entity-type
  reference and all authored field values as typed literals. This is the
  capability RDF lacks in released DomainForge.
- **Policies survive.** A sampled policy retains its full expression tree,
  including the aggregation function, the `where` predicate, and the member
  accesses. Policy meaning is recoverable from the envelope alone.

The `semantic_closure_hash` is the model's stable contract identity. Any change
to declared meaning changes it; comments, formatting, and absolute paths do not.

## Application Contract inspection

| Field | Value |
| --- | --- |
| `schema_version` | `domainforge-application-contract/v1` |
| `self_hash` | `sha256:138e5982bba5f7fdfd1fa40c0adce497d647b60385627b629081e3a07be61673` |
| enums | `CanonicalizationClass`, `ImplementationMaturity`, `InteractionPhase`, `InterfaceKind` |
| records | `GetCanonicalJourneyInput`, `GetCanonicalJourneyOutput` |
| typed entities | `ProductPurpose`, `JourneyStep`, `VariationDimension`, `CanonicalizationClassification`, `CanonicalJourney`, `InterfaceBinding`, `InterfaceSurface` |
| operations | `get_canonical_journey` |

Every clause of the operation resolved: `direction inbound`, `actor anonymous`,
`access public`, input and output resolved to records, `state` resolved to the
`CanonicalJourney` concept ID, `effect reads`, `transaction read_only`,
`idempotency inherent`, `concurrency read_snapshot`, `evidence operation_trace`,
`lifecycle synchronous_request_response`, and both declared failure codes
(`invalid_journey_id`, `canonical_journey_not_found`).

## Errors encountered and corrected

Recorded because they are instructive, not because they remain.

1. **`APP011 output field cannot project a same-named compatible state field`**
   — the first draft of `GetCanonicalJourneyOutput` declared `journey_id: string`
   while the entity declared `journey_id: string (pattern JourneyId)`. Output
   record fields must match the state field's constraints, not only its scalar
   type. Corrected by repeating the constraints on every projected output field.

2. **`min_length` violation on `value_derived_concepts`** — the first typed draft
   required 20 characters on a field whose shortest real value is 9. Corrected by
   lowering the floor to 5 rather than by padding the data. The field is also
   `optional`, because only 2 of the 11 variation dimensions carry it; the
   previous untyped model could not express that at all.

3. **`Policy evaluation is UNKNOWN (NULL)`** on a first-draft
   `forall i in entity_instances: (...)` invariant — a field absent on other
   entity types produces NULL, and an `or` guard does not rescue it. Every
   invariant was rewritten as a `count`/`sum` aggregation with a `where` filter,
   which filters before projection. Recorded as `limitations.md` L6.

4. **Silent pass on an arithmetic `where` predicate** — a draft invariant
   asserting per-journey maturity arithmetic passed against data that violated
   it. Removed from SEA entirely and moved to `reconcile.py`. Recorded as
   `limitations.md` L5. This one is the reason a green validate is not treated
   as sufficient anywhere in this package.

5. **Two semantic-teeth cases asserted the wrong guard** — deleting a journey or
   a surface trips referential integrity before the count policy. The cases were
   corrected to assert the guard that actually fires, and separate cases (T14c,
   T20b) were added to prove the count policies independently.

## Reconciliation against the observed catalog

```
$ python3 .sea/interaction/validation/reconcile.py
observed stories:      128
canonical journeys:    12
interface bindings:    90
classification values: 8

model and matrix agree on every reconciled quantity
```

This checks the six quantities listed in `semantic-teeth.md` under "Invariants
outside DomainForge's enforcement surface", including per-journey maturity
distributions row by row against `canonicalization-matrix.csv`.
