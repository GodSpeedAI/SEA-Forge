# Current status

**Status revision:** 237

**Stage:** FDE integration branch: GS-FDE-SOP-001 v0.4.1 declared model, translation map and checks (WP-13)

**Summary:** fde/integration: promoted the SOP draft v0.4.1 (pending owner ratification OD-2) to docs/sop with its sha256, and declared its model under .sea/interaction/fde-sop/ (world godspeed-fde-sop): 18 operations, 7 role bindings, Obligation/Prohibition policies for freeze-before-move, no self-settlement and episode identity, and two non-unifying Settlement concepts in separate namespaces. The map has 115 rows over 66 numbered clauses: 60 declared, 23 inferred, 32 unknown, each unknown naming a limitation. Revisions 234 and 235 of this branch were renumbered 235 and 236 so they follow main's 234.

**Verified:**
- scripts/check-fde-sop-model.sh: validate --application 0 violations (18 operations); envelope --emit cep world_ref identical across three runs; two Settlement concepts have different ConceptIds and a same-namespace merge is refused; six negative controls reject for the reason claimed (DomainForge 0.21.2)
- scripts/check-fde-sop-map.sh: 66 numbered clauses all covered; every excerpt is a verbatim SOP substring; every named element exists in the model
- scripts/check-agent-context.sh passes

**Limits:**
- Operation-bound policies are scope-checked by validate, not evaluated; every S2 row is insufficient until the DomainForge evaluator (WP-12) runs it
- DomainForge 0.21.2 rejects postcondition and invariant enforcement points, so only precondition policies are declared
- 32 of 115 map rows are unknown; the 4.2 field groups, 7.4 assignment brief and 10.x handoff contents are not declared in this version
- The SOP is a draft pending owner ratification; the model and map must be re-derived if its hash changes
- I-3 key (case, plan item, n) comes from the integration architecture, not the SOP text; episode_key_unique tests a caller-supplied count
- domainforge test was not used as evidence; no cargo build or just ci was run for this change
- Integration branch only: not merged to main, no PR

**Next:** Owner ratifies or amends the SOP (OD-2); WP-12 evaluates the bound policies; WP-11 and WP-20 bind Casework to the model.
