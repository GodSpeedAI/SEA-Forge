# FDE contracts changelog

The v1 files (`frozen-reference`, `anchor`, `case-record-append`) are closed and digest-pinned. A change is a new file and a new `DIGESTS` line. Newest first.

## S2: WP-20 contracts package (contracts only; no server behaviour changed)

New files. No v1 byte changed.

| Addition | v1 limit it closes | Source |
|---|---|---|
| `field-case.v1.json`: the Field Case document, 45 SOP 4.2 fields in six groups, each with `status` and `source_ref`; checked against the SOP text and the translation map by `scripts/check-fde-field-case-schema.sh` | S1 had no Field Case document at all (AT-03); SOP 4.2 content lived only in prose | E section 3 |
| `case-record-kinds.v1.json`: registry of six kinds with payload schema, idempotency-key rule, natural key, refusal codes, immutability | v1 knew two kinds; no place stated which codes a kind can return | E section 3, 13 |
| `case-record-append.v2.json`: six kinds; `contract` dialect marker; 12 new codes (`WORLD_NOT_PINNED`, `SETTLEMENT_EXISTS`, `KIND_IMMUTABLE`, `REFERENCE_DERIVED_FROM_RESULT`, `MODEL_ANCHOR_LATE`, `AUTHORITY_DENIED`, `ANCHOR_PUBLISHER_INVALID`, `STORE_INTEGRITY`, `PAYLOAD_TOO_LARGE`, `HARD_GATE_NOT_WAIVABLE`, `CASE_SUSPENDED`, `REVISION_CONFLICT`); `FREEZE_INCOMPLETE` extended to NT-33 a, b, c | S1 F8 (freeze did not consume the world pin); settlement under another key was not a distinct code; NT-13, NT-17, NT-18, NT-34, NT-48 had no code; closed-contract rule (the v1 Error enum could not grow) | S1 STATUS 2026-11-14 20:30Z; G |
| `execution-episode-ref.v1.json`: `episode_ref` on `item_execute`, `case_advance`, `agent_run.delegate`; rule table R1-R11; frames | S1 F1(b): I-1 "freeze before move" was enforced only at settlement; the three execution verbs carried no `n`/`episode_ref` | E I-1; S1 STATUS |
| `delegation-link.v1.json`: delegator, delegate, episode, assignment, issued, expiry; I-2 closure rule | S1 F7: "delegate then settle" by the delegator was not caught; the gateway hop existed only with a gateway configured | E I-2; S1 STATUS |
| `continuation.v1.json`: `case.continuation` read model (SOP 10.1) and the `case.field.get` and `case.continuation` verbs | S1 had no continuation or field read; AT-15 had nothing to read | E section 3 |
| `reserved-operation.v1.json`: operation, class, evidence; pending operation, A3 proof minimum, decision; the program-execution decision | S1 limit: no authority decision for program execution (RealityTrace could not resolve `authority.decision_id`); no interface for human-reserved decisions | E section 8, RD-06, RD-21 |
| `assignment.v1.json`: `assign:<caseId>:<stepId>:<n>:<k>`, brief, budget per currency, lifecycle events | S1 had no assignment payload; AT-17 | E sections 2, 3 |
| `oracle-codes.json`, `scripts/check-fde-oracle-codes.sh`: every G oracle code of the S2 tests is defined in exactly one table | S1 codes were chosen per contract; nothing checked that G's codes exist | G |
| README section 12: independent-observation requirement; `fixtures/independent-observation/` | S1 finding: executor-only evidence cannot confirm (`executor_only_evidence`) | E section 5; STATUS 2026-11-15 01:30Z |
| `scripts/check-fde-contract-digests.sh` pins `oracle-codes.json` too | | |

Still open after WP-20 (not contract gaps): remote push time of anchors (OD-6); the A3 channel and WebAuthn verification (WP-31); the shared identity contract (S3).

## Contract to implementing WP to frozen tests it serves

| Contract | Implementing WP | Frozen tests served |
|---|---|---|
| `field-case.v1` | WP-21 (document, template check) | AT-03 |
| `case-record-kinds.v1` | WP-21 | AT-03, AT-10, AT-11 |
| `case-record-append.v2` | WP-21 (kinds, lifecycle, hard gate, suspension); WP-14 follow-up for `WORLD_NOT_PINNED`, `MODEL_ANCHOR_LATE`, `ANCHOR_PUBLISHER_INVALID` on the write path | AT-03, AT-09, AT-10, AT-11; NT-10, NT-11, NT-13, NT-17, NT-18, NT-22, NT-23, NT-34, NT-48 |
| `execution-episode-ref.v1` | WP-21 (gate on the three verbs); WP-23 (Workbench sends the field) | NT-32, NT-33 (a, b, c), NT-22, NT-23, NT-13, NT-34, NT-48; AT-17; the I-1 half of AT-04 |
| `delegation-link.v1` | WP-21 (I-2 on the SEA-Forge delegation chain) | NT-10, AT-09 |
| `continuation.v1` (`case.field.get`, `case.continuation`) | WP-21 (read model); WP-41 (Horizon consumes) | AT-03, AT-10, AT-15 |
| `reserved-operation.v1` | WP-31 (A3 check, decision API); WP-21 (program-execution decision on the existing approval path); WP-32 (Horizon surface) | NT-12, NT-24, NT-34; AT-09 (human settlement, PARTIAL until S3) |
| `assignment.v1` | WP-23 (session binding, budgets, resume); WP-21 (id validation, `k` allocation) | AT-17 |
| Independent-observation fixtures | WP-21 (emits the SEA-Forge events), WP-17 (rule already in RealityTrace) | NT-19 and NT-20 context; AT-09 |

Frames in `fixtures/*/frames/` are expected frames. WP-21 captures the goldens into `apps/godspeed-casework-go/internal/adapters/sfwp/testdata/frames.json` and mirrors them in the Go client and generated TypeScript.

## S1: WP-11

`frozen-reference.v1`, `anchor.v1`, `case-record-append.v1` (see `README.md` sections 1 to 3 and `evidence/s1/wp-11.md`).
