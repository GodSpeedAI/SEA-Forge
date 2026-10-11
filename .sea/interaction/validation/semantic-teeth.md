# Semantic Teeth

An invariant has teeth when the toolchain rejects its violation. Every case
below mutates a copy of the real canonical model — never a toy fixture — and
requires DomainForge to fail. The harness is
`validation/semantic-teeth.sh`; its captured run is
`validation/semantic-teeth-output.txt`.

```bash
DOMAINFORGE=/home/sprime01/projects/domainforge/target/release/domainforge \
  .sea/interaction/validation/semantic-teeth.sh
```

Result at capture time: **31 passed, 0 failed**, against
`domainforge 0.16.0` built from the local worktree.

The canonical model is never modified: each case works on a temporary copy.

## Typed instance validation

| Case | Mutation | Rejected because |
| --- | --- | --- |
| T00 | none (baseline) | model validates clean |
| T01 | `implementation_maturity: "shipped"` | not a member of enum `ImplementationMaturity` |
| T02 | `interface_kind: "tui"` | not a member of enum `InterfaceKind` |
| T03 | `class_value: "primary"` | not a member of enum `CanonicalizationClass` |
| T04 | binding points at journey `CJ13` | dangling typed reference |
| T05 | binding points at surface `workbench_missing_route` | dangling typed reference |
| T06 | second journey re-uses `journey_id: "CJ01"` | entity key must be unique for `CanonicalJourney` |
| T07 | journey gains an `owner_team` field | unknown field for typed entity |
| T08 | journey loses its `slug` field | missing required field |
| T09 | `journey_id: "CJ99"` | violates its `pattern` constraint (`JourneyId`) |
| T10 | `binding_id: "B90"` | violates its `pattern` constraint (`InterfaceBindingId`) |
| T11 | `observed_story_count: "fifteen"` | does not satisfy its `int` type |
| T12 | step `ordinal: 9` | violates its `max` constraint |
| T13 | `slug: "exec"` | violates its `min_length` constraint |

T01–T13 were all impossible to detect under the previous model, where every one
of these values was an unvalidated string.

## Graph invariants

These are the sixteen policies in the model. Each row shows what breaks it.

| Case | Mutation | Rejected because |
| --- | --- | --- |
| T14 | delete the `CJ12` journey instance | dangling typed reference from its bindings — referential integrity fires before the count policy, which is the stronger guard |
| T14b | renumber a journey to `CJ13` | `JourneyId` pattern admits `CJ01`–`CJ12` only, so a thirteenth identity is structurally impossible |
| T14c | re-point `CJ12` bindings to `CJ11`, then delete `CJ12` | `twelve_canonical_journeys` violated — with no reference left to trip, the count policy is what rejects an eleven-journey catalog |
| T15 | `observed_story_count: 15` → `14` | `all_observed_stories_accounted` violated (128) |
| T16 | `exercised_story_count: 5` → `4` | `exercised_stories_reconcile` violated (85) |
| T17 | `interface_binding_count: 5` → `6` | `interface_bindings_reconcile` violated |
| T18 | delete the `preflight` step | `eight_interaction_steps` violated |
| T19 | duplicate a step ordinal | `interaction_step_ordinals_complete` violated (sum 36) |
| T20 | delete a bound interface surface | dangling typed reference from its bindings |
| T20b | add an unbound 39th surface | `thirty_eight_interface_surfaces` violated |
| T21 | delete a variation dimension | `eleven_variation_dimensions` violated |
| T22 | `observed_rows: 69` → `68` | `classification_rows_account_for_all_stories` violated (128) |
| T23 | delete the `unsupported` classification | `eight_canonicalization_classifications` violated |
| T23b | add a second `ProductPurpose` | `exactly_one_product_purpose` violated — the model cannot hold two competing statements of what SEA Forge is for |

The twelve-journey invariant is guarded three times over — by the `JourneyId`
pattern, by entity-key uniqueness, and by the count policy — and each guard was
proven independently (T14b, T06, T14c).

`interface_bindings_reconcile` deserves note: it asserts that the sum of every
journey's declared `interface_binding_count` equals the number of declared
`InterfaceBinding` instances. Combined with the `min 1` field constraint on that
count, it proves that **every canonical journey is projected by at least one
interface surface and no binding is orphaned** — a claim the previous model made
in prose and could not check.

## Module closure

| Case | Subject | Outcome |
| --- | --- | --- |
| T24 | an instance in one module of an entity type imported from another module under the same namespace | accepted — the capability DomainForge 0.15.0 lacked |
| T25 | the same instance with an illegal enum wire value | rejected across the import boundary, so closure resolution does not weaken validation |
| T26 | `import { ProbeJourney } from "./absent.sea"` | rejected — unresolved specifier |

T24 is the direct retest of the previous report's limitation 9. It is fixed.

## Invariants outside DomainForge's enforcement surface

Stated honestly rather than left implied.

| Invariant | Why DomainForge cannot enforce it | Where it is enforced instead |
| --- | --- | --- |
| Each journey's four maturity counts sum to its `observed_story_count` | arithmetic in a policy `where` predicate silently matches nothing (`limitations.md` L5) | `reconcile.py` |
| Model counts equal the actual `canonicalization-matrix.csv` rows | DomainForge cannot read the CSV | `reconcile.py` |
| Every observed story maps to a declared journey | same | `reconcile.py` |
| CSV classification and maturity values stay inside the model's enums | same | `reconcile.py` |
| `get_canonical_journey` remains a valid operation | no CLI subcommand resolves the Application Contract (`limitations.md` L4) | `application-contract-harness.rs` |
| A journey has a reachable terminal or recovery path | transitions are prose (`limitations.md` L7) | human review against the catalog |
| An interface surface's cited `source_ref` still exists in the repository | outside SEA's scope | human review; last verified in `README.md` "Interface reality check" |

## Reproducing everything

```bash
df=/home/sprime01/projects/domainforge/target/release/domainforge

# 1. semantic validation
"$df" validate --format human --no-color .sea/interaction/interaction-model.sea

# 2. semantic teeth (31 cases)
DOMAINFORGE="$df" .sea/interaction/validation/semantic-teeth.sh

# 3. model <-> matrix reconciliation
python3 .sea/interaction/validation/reconcile.py

# 4. application contract and semantic envelope
#    (build the harness first — see application-contract-harness.rs)
dfharness envelope .sea/interaction/interaction-model.sea
dfharness contract .sea/interaction/interaction-model.sea
```

All four must pass. Step 1 alone is necessary but not sufficient.
