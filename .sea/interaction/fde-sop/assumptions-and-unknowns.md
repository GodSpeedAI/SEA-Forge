# Assumptions and unknowns: GS-FDE-SOP-001 v0.4.1 declared model

The SOP is a draft pending owner ratification (OD-2). Everything below is a
reading of that draft. If the SOP changes, its hash changes, the map check
fails, and this model is re-derived against the new version.

## Assumptions

| Id | Assumption | Where it shows | Consequence if wrong |
|---|---|---|---|
| A-1 | "Settlement" has two contextual meanings. **Evidence settlement** is the epistemic result of an episode against a frozen expectation (S/F/U/C; SOP 7.5, 13). **Client outcome** is the accepted business result the sponsor finally accepts (SOP 3.1 "final acceptance", 10.3 "outcome account"). "Smallest-settlement identification" (3.3, 10.2) is read in the client-outcome sense; this is the one place the SOP does not say which meaning it intends. | `evidence.sea`, `outcome.sea`; rows 3.1, 10.3, §13 | If 3.3 and 10.2 mean evidence settlement, the outcome pack loses a clause and nothing else changes. The two packs stay distinct either way. |
| A-2 | Expressing the two meanings as two namespaces (`godspeed-fde-sop.evidence`, `godspeed-fde-sop.outcome`), each exporting an entity named `Settlement`, is how DomainForge keeps them apart: ConceptId is UUIDv5 over `namespace::name`, so they get different ids, and a same-namespace merge is refused (AT-01 section 3, T4). They live in one world, one `world_ref`; they are not separate semantic packs in the `domainforge pack` sense (no pack artifact is built). | `validation/domainforge-output.txt` | If separate signed packs are wanted later, `domainforge pack build` over each namespace is the next step; not done here. |
| A-3 | In a Prohibition policy the expression states the **permitted** condition (for example `settler_ref != executor_ref`). DomainForge treats modality as metadata and requires the expression to hold. | policies with `per Constraint Prohibition` | A reader who takes the expression as the prohibited state would invert them. Each policy's `@rationale` says which way it reads. |
| A-4 | A policy bound to an operation sees only that operation's input record (APP007), so facts such as "a freeze is recorded" or "no earlier freeze exists for the key" are caller-supplied input fields. The model states the rule; the case keeper must supply true values. DomainForge cannot check that they are true. | S2, S4 | Until the evaluator (WP-12) runs these policies, their verdict is `insufficient`, not `confirmed`. |
| A-5 | The SOP assigns no role to the ten activities. `FDE` is the actor of every `act_*` operation because the SOP makes the FDE/case lead responsible for coordination (3.1). | operations `act_*` | Replace the actor per activity if the owner assigns them. |
| A-6 | I-1 (freeze before move), I-2 (no self-settlement) and I-3 (key `(case, plan item, n)`) are design constraints from the integration architecture (E 3, RD-10). The SOP text supports I-1 (7.2), I-2 (3.2) and only the idea of n episodes per case (2.2) for I-3; the key itself is not in the SOP. The map cites the SOP clause and marks the I-3 row `inferred`. | rows 7.2e, 3.2b, 2.2c | None for validation. A reader must not take the key as ratified SOP text. |
| A-7 | The `world_ref` pattern (`world:<name>@sha256:<64 hex>`) comes from E 2, not from the SOP. | pattern `WorldRef` | Loosen the pattern if the identity format changes. |
| A-8 | Records and typed entities exist because the Application Contract requires them (an operation needs input and output records and a keyed state entity whose required fields all appear in the input). They are modeling vehicles, not the Casework storage schema (E 3 keeps the persisted plan schema unchanged). | records, entities | None for validation. |
| A-9 | Unnumbered SOP text is not a clause and has no row: the standing criterion (A:178), the sentence "Outcome per requirement: PROVEN, PARTIAL or BLOCKED", and the table captions. The standing criterion needs observed values (S6), so no metric claims it. | n/a | n/a |
| A-10 | The world name is supplied with `--world-name godspeed-fde-sop`, not written in the source; the `world_ref` hash is derived from the model identity and is independent of the working directory (checked from `/` and from the model directory). | `validation/domainforge-output.txt` | A different `--world-name` gives a different `world_ref`. |

## Unknowns

`unknown` rows in the map are listed there with their limitation. They fall into
four groups; none was forced into a policy:

- **Prescriptive prose with no decidable predicate (S5):** 1.2, 1.3, 1.5, 3.2d,
  3.6, 4.5, 5.2, 6.2, 7.1b, 7.2b, 7.3c, 8.2.
- **Negative, ordering, temporal and immutability claims (S3):** 2.1a, 3.5, 4.3,
  4.4, 6.1, 8.1c, 9.1, 9.3, and the "never" clauses inside rows marked `inferred`.
- **Not declared in this version, many-part or list-valued (S7, L2):** 1.4c, 4.2,
  7.2c, 7.4, 9.4, 10.1, 10.2, 10.4.
- **After-effects and runtime measurement (S1, S6):** 7.5e, 5.3, the thresholds
  in 7.7.

`3.3a` and `3.3b` are unknown because a role binding carries no responsibility
kind (L3).

## What this model does not prove

- A green `validate --application` shows the model parses, resolves, and that
  every bound policy references fields that exist. It does not show any bound
  policy holds (S2). Per the integration architecture (RD-08), where DomainForge cannot
  evaluate an expectation the result is `insufficient`; there is no second
  evaluator.
- `domainforge test` is not used as evidence anywhere in this directory.
- Declaring a role, operation or policy here does not make Casework enforce it.
  Enforcement is WP-11 and WP-20.
