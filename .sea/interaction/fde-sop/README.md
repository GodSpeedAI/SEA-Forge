# GS-FDE-SOP-001 declared model

The Interaction Domain Model of the GodSpeed FDE operating procedure v0.4.1
(draft, pending owner ratification OD-2). World `godspeed-fde-sop`. Work package
WP-13. Built with the same method as the parent interaction model: one declared
`.sea` world, a source translation map, assumptions and unknowns, and captured
validation evidence.

| File | Role |
|---|---|
| `fde-sop.sea` | Entry point and world: roles, role bindings, entities, instances, flows, 18 operations, policies, metrics. |
| `evidence.sea` | Namespace `godspeed-fde-sop.evidence`: `Settlement` as the epistemic result of an episode (S/F/U/C). |
| `outcome.sea` | Namespace `godspeed-fde-sop.outcome`: `Settlement` as the client-accepted business result. |
| `source-translation-map.md` | One row per numbered SOP clause: excerpt, model element, `declared / inferred / unknown`, limitation, reason. |
| `assumptions-and-unknowns.md` | Readings the SOP does not settle, and the unknown groups. |
| `validation/limitations.md` | Limitations re-tested on DomainForge 0.21.2 and model-specific ones (S1-S7). |
| `validation/domainforge-output.txt` | Captured `validate --application`, `envelope --emit cep` (repeated), distinctness and negative controls. |
| `validation/map-check-output.txt` | Captured output of the map check. |

Source: `docs/sop/GS-FDE-SOP-001-v0.4.1.md`, sha256 in `docs/sop/README.md`.

## Two settlements, never unified

The SOP uses "settlement" for two things. Evidence settlement (SOP 7.5, 13) is
independent evidence against a frozen expectation. Client outcome (SOP 3.1,
10.3) is the business result the sponsor accepts. SOP 13 rejects equating them.
They are two entities named `Settlement` in two namespaces. DomainForge derives
the ConceptId from `namespace::name`, so the ids differ, and moving both into
one namespace is refused as a symbol collision. The entry file imports them
under the distinct local names `EvidenceSettlement` and `OutcomeSettlement`.

## Checks

```sh
scripts/check-fde-sop-map.sh      # AT-02: map completeness and provenance
scripts/check-fde-sop-model.sh    # AT-01: validate, envelope, world_ref, distinctness
```

Recapture the evidence after any model change:

```sh
scripts/check-fde-sop-model.sh | tee .sea/interaction/fde-sop/validation/domainforge-output.txt
scripts/check-fde-sop-map.sh   | tee .sea/interaction/fde-sop/validation/map-check-output.txt
```

## Reading the result

Bound policies are checked for scope, not evaluated (limitation S2): the
evaluator is WP-12. A clause marked S2 is `declared` and `insufficient`
until then. Nothing here is enforced by Casework yet (WP-11, WP-20).
