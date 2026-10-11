# Handoff

For the next Journey & Capability Map agent, implementation agent, or reviewer.
Read this before touching anything under `.sea/interaction/`.

## State

The interaction model was re-canonicalized against DomainForge 0.16.0 on
2026-08-12. The twelve canonical journeys survived unchanged; what changed is
how they are represented and how much of the model the toolchain can prove.

| | Before (0.15.0-era) | Now |
| --- | --- | --- |
| `.sea` files | 1 canonical + 4 declaration-free companions | 1 canonical |
| Typed entities | 0 | 7 |
| Closed vocabularies | 0 (prose) | 4 enums |
| Identity patterns | 0 | 3 |
| Instances | 76, schemaless | 168, fully validated |
| Typed references | 0 (`"CJ01; CJ02"` strings) | 90 validated `ref<>` bindings |
| Enforced invariants | 2 | 16 |
| Negative tests | 0 | 31, all passing |
| Application operations | not expressible | 1 read boundary |

## What must not be redone

1. **The 128-story canonicalization.** `canonicalization-matrix.csv` is the
   reviewed row-level authority. Do not reassign a row because a newer screen or
   method name looks convenient. `reconcile.py` proves the model still agrees
   with it on every reconciled quantity.
2. **The twelve canonical journeys.** They were re-tested against current
   DomainForge semantics and current SEA Forge source. No new evidence
   disconfirmed a journey boundary, intention, transformation, completion
   condition, governance distinction, settlement distinction, or variation
   dimension. Optimize for minimum sufficient structure, never for a different
   count.
3. **The DomainForge capability review.** `validation/domainforge-change-review.md`
   records which of the last ten commits matter, what each changed, and what the
   uncommitted working tree adds. Re-read it before assuming a capability is
   missing.

## What changed in the model, and why

- **`InterfaceProjection` split into `InterfaceSurface` + `InterfaceBinding`.**
  A surface projecting four journeys could not be one instance, because
  DomainForge has no array literal for `list<ref<T>>` fields
  (`validation/limitations.md` L2). Normalizing was the better model anyway: each
  binding is independently addressable and both of its references are validated.
- **`JourneyVariant` split into `CanonicalizationClassification` +
  `VariationDimension`.** The old entity carried two unrelated shapes
  distinguished by a `variant_kind` string. They are now separate typed entities
  with separate invariants.
- **Seven journey steps became eight.** `journey-grammar.md` described eight
  narrative phases while the model declared seven steps, splitting availability
  out of authority and folding the state effect into capability invocation. The
  two artifacts now name the same eight phases, and `state_and_artifact_effect`
  is explicit. This reconciled a genuine discrepancy between two canonical
  artifacts; it did not change any journey.
- **Flow annotations were reduced to `@journey_id`.** The twelve journey flows
  previously duplicated `action_sequence`, `capabilities`, `evidence`,
  `completion_condition`, `recovery`, and `next_decision` from the journey
  instance, because untyped instances were not dependable. Journey meaning is now
  stated once, on the journey.
- **Per-journey story and maturity counts became typed integers.** The catalog's
  maturity prose ("15 mapped stories: 1 specified, 2 implemented…") is retained
  verbatim as `maturity_note`, and the numbers are now enforced.
- **`ProductPurpose` gained a typed body and its single instance.** The entity
  was previously declared and never instantiated, so the model could not state
  what SEA Forge is for, what constraint governs every journey, or what the model
  deliberately does not define. `exactly_one_product_purpose` keeps it singular.
- **The four declaration-free `.sea` companions were deleted.** Their only stated
  reason for existing was the 0.15.0 resolver bug, which is fixed.

## How to use the model

Build the map with `CJ01`–`CJ12` as the identity spine, then connect
capabilities, evidence, and interface affordances through the recorded fields.

1. `canonicalization-matrix.csv` — the exhaustive observed-story-to-CJ lookup.
2. `canonical-journey-catalog.md` — each journey's intention, job, conditions,
   steps, capabilities, artifacts, evidence, completion, recovery, next
   decisions, maturity, and observed IDs.
3. `journey-grammar.md` — the eight-phase skeleton, verbs, nouns, composition
   rules, cross-cutting identity and authority, variation dimensions, and the
   table separating what the grammar enforces from what it only describes.
4. `interaction-model.sea` — machine-readable identity, typed fields, validated
   references, and the sixteen invariants.
5. `coverage-report.md` — reconciled counts and maturity or confidence
   boundaries.

Interface maturity and limitations are current repository evidence. They are not
permission to rename a CJ, collapse execution into settlement, or promote an
unavailable target into a capability.

### Extracting bindings programmatically

Journey-to-interface groupings come from `InterfaceBinding` instances, not from
hand assembly:

```python
import json, subprocess
df = "/home/sprime01/projects/domainforge/target/release/domainforge"
g = json.loads(subprocess.run(
    [df, "parse", "--format", "json", ".sea/interaction/interaction-model.sea"],
    capture_output=True, text=True, check=True).stdout)
bindings = [i for i in g["entity_instances"].values()
            if i["entity_type"] == "InterfaceBinding"]
by_journey = {}
for b in bindings:
    by_journey.setdefault(b["fields"]["journey"], []).append(b["fields"]["surface"])
```

`journey-capability-map.md` (2026-08-03) was generated from the previous
`canonical_journey_ids` string field. Its groupings remain correct — the 90
bindings are the same 90 pairs — but regenerate its interface sections from
`InterfaceBinding` rather than hand-editing them if anything changes.

## Before you change the model

Run all four checks in `README.md` → "Validate and Inspect". A green
`domainforge validate` is necessary and **not** sufficient: it does not reach the
application operation, and it will accept a policy whose `where` arithmetic is
meaningless (`validation/limitations.md` L4, L5).

If you add a journey, a surface, a binding, a dimension, a classification, or a
second product purpose, the corresponding count policy will fail until you
update it. That is the intent:
the counts are the model's tripwire against silent drift.

The `semantic_closure_hash` in `validation/diagnostics.md` is the model's stable
contract identity. Recompute it after any change to declared meaning; comments
and formatting do not affect it.

## Open items for whoever owns DomainForge

None of these block this model. All are recorded with reproductions in
`validation/limitations.md`.

- **L5 is a correctness defect**, not a missing feature: arithmetic in a policy
  `where` predicate silently matches nothing, so an invariant written that way
  looks green and proves nothing. It should evaluate or raise.
- **L1**: land the RDF instance and policy projection work so RDF stops being a
  lossy projection of an instance-heavy model.
- **L4**: a `domainforge contract` / `domainforge envelope` subcommand would let
  ordinary validation reach the Application Contract.
- **L2**: an array literal in `expression` would make declared `list` types
  authorable.
- **L3**: expose the graph's existing role-to-entity binding through text syntax.
- **L9**: exclude operation-bound policies from graph evaluation.

The two grammar candidates this model still supports, on recurrence grounds
rather than SEA Forge convenience, are typed transition graphs
(`limitations.md` L7) and role-to-entity binding syntax (L3). A dedicated
`journey` keyword remains unjustified: typed keys, patterns, and references gave
journeys enforced identity without one.
