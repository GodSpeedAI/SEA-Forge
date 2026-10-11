# T09 test-first assignments

T08 is settled at af362f5. Approved T09 proposal and contract-source-preparation.md govern
these bounded stages. Root holds the sole compiler token; neither builder may run tests
or compile before independent fixture review. File sets are disjoint.

## Canonical contract tests — t08_type_builder

Own only interface-contracts/tests/contract-conformance.test.ts and necessary new fixtures/
goldens. Read actual Rust views and nearby schema/golden patterns. No canonical type/schema/
normative implementation edits yet, and no Go/UI edits. Add meaningful positive/negative
tests for the complete protected Ask request/answer, exact question/claim/status/disposition/
freshness values and disclosures; assurance/authority_notice remain strings. Add observation
cohort and safe trace metadata tests, bounded runs, separate standing/read states, unavailable
count absence and exact event-schema parity including existing interrupted/error.

Use root naming/read-state choices. Do not invent frame IDs/progress or silently relax required
fields. Schema cannot enforce UTF-8 purpose-byte or raw HTTP-body limits; those need route tests
in their later unit. No dependency additions. Report exact expected baseline failures and
freeze for independent review/RED. Canonical implementation follows only after that review;
Go/UI mirrors are a subsequent separate bounded builder unit.

## SFWP cap tests — shared_replay_builder

Own focused tests under internal/adapters/sfwp, grounded in actual client reader/caller graph.
No implementation edits yet. Prove incremental default32MiB line cap before decode on every
response/event/recovery read path, configured lower limits and hard-ceiling refusal, fragmented
input, exact/over-limit boundary and truncated EOF. Overflow closes/poisons the owned connection
and surfaces typed unavailable; fail pending calls using existing connection ownership idioms.

Preserve inspect's one fresh-connection retry and uncertain mutation correlation recovery.
Never resend a mutation after an oversized response: outcome is unknown until request status
resolves it. Tests must measure actual relevant request counts and uncertainty, not fabricate
success. A later Ask call writes durable records and must use the non-resending path; report any
verb-classification concern without implementing Ask here. No aggregate wire/kernel-work/total
allocation claim follows from the per-line cap. No dependency/API/identity changes outside the
approved per-client Config cap extension. Freeze for independent fixture review/RED.

## Independent loop

Critic receives each builder's original assignment plus actual tests and source context,
reviews whether fixtures really distinguish correct behavior, then runs expected RED with
actual-host RAM/compiler preflights. Root transfers one compiler token explicitly. Builders
implement only after fixture approval; independent critic then checks source, focused GREEN,
broader gates and material differences. Fresh builder after rejected implementation. Root
retains semantic decisions, cross-cutting authority, integration, status and commits.
