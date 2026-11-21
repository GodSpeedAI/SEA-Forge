# FDE contracts changelog

The v1 files (`frozen-reference`, `anchor`, `case-record-append`) are closed and digest-pinned. A change is a new file and a new `DIGESTS` line. Newest first.

## WP-23 follow-up: the binding carries the frozen world and kind (`assignment-binding.v1.json` re-pinned)

`Binding` gains required `world_ref` (the frozen reference body's world) and `episode_kind` (the frozen kind), so a consumer composes its case context from the read alone. `assignment_binding_get` (README section 11) serves the joined binding to exactly the link's delegate or delegator; every hidden path returns one constant unknown-assignment answer. The binding was not yet consumed elsewhere, so v1 is amended in place and re-pinned.

## WP-23 delegated run admission: the versioned assignment binding (new `assignment-binding.v1.json`; `execution-episode-ref.v1.json` re-pinned in DIGESTS)

A Field Case `delegate` carries the approved assignment, not the identity alone. `assignment` (`AssignmentApproval`: explicit `domain_meaning`, `context_refs`, five SOP 5.1 ceilings, per-currency `"unknown"` preserved) is required on a Field Case delegate, new or resume; `adoptable_problem` is server-supplied from the freeze (`criteria.intention`), `instruction` stays on the request. The admitted run records the composed full `Assignment` as `binding` `{version, assignment, link_id, run_id, binding_digest, recorded_at}` in the same guarded batch as link and run; a resume must match the recorded binding; the trusted read joins case, episode, assignment, link and run and fails closed on legacy rows and partial appends. New codes: `ASSIGNMENT_APPROVAL_REQUIRED`, `ASSIGNMENT_APPROVAL_INVALID`, `ASSIGNMENT_APPROVAL_MISMATCH`, `ASSIGNMENT_BINDING_INVALID`. Accepted-response semantics unchanged. Migration: the four accepted delegate scenarios of `execution-episode-ref` now carry `assignment`; refusal scenarios are unaffected (the approval is checked at R10, after the earlier refusals).

## WP-21a owner decisions D39–D44 (contracts edited in place; `delegation-link.v1.json` re-pinned in DIGESTS)

| Decision | Change | Contract(s) |
|---|---|---|
| D39 | README: how SEA-Forge realises the program-execution decision (CEP resource_id `execution:sha256:<digest>`, TTL, run counter) | reserved-operation (README only) |
| D40 | `delegation-link.v1.json` `Error`: `invariant` (I-2) is required except on `VERIFIER_CONTEXT_MISMATCH`, which carries none; fixture `delegation-link/sibling-self-asserted-fresh-is-refused` no longer expects invariant I-2; new scenario `case-record-append-v2/independent-verifier-declares-fresh-in-shared-workspace` (53 → 54) | delegation-link, case-record-append.v2 |
| D41, D42 | README decision rows | case-record-append.v2 |
| D43 | README precedence: idempotency before `EPISODE_REF_DIGEST_MISMATCH`, placed where freeze and settlement check it | case-record-append.v2 |
| D44 | README: verifier runs are delegations to `verifier_endpoints` | delegation-link |

## S2 review fixes (independent review of d2f8189e..6c633811; contracts only, edited in place)

The S2 contracts were not yet consumed, so they are changed in place and re-pinned in `DIGESTS`. v1 files unchanged.

| Finding | Change | Contract(s) |
|---|---|---|
| C1 | `delegate` has a required `case_id`; the case template is resolved by the server; `CASE_UNRESOLVED` fails closed; I-1 is checked on the server-resolved identity | execution-episode-ref |
| C2 | executor set = actor and on_behalf_of of every executor run, link delegates and the delegator closure; `RunRecord`; link written before the run; run with no actor or delegated run with no link refuses settlement | delegation-link |
| H1 | the `contract` marker selects the response dialect only; every v2-only code has a refusal fallback (`AUTHORITY_DENIED` included); test that no v1-dialect scenario is accepted | case-record-append.v2, oracle-codes |
| H2 | verifier context derived from run records; `VERIFIER_CONTEXT_MISMATCH`; sibling cannot settle S or F without derived `fresh` | delegation-link, case-record-append.v2 |
| H3 | per-operation required `params` with `record_digest` in the digest; `ApprovalRecord`; single-use, same case, digest-bound approvals | reserved-operation, case-record-append.v2 |
| H4 | `settle_episode` split into `settle_episode_agent` (not reserved) and `settle_episode_human` (reserved) | reserved-operation |
| H5 | program decision binds the nine-member execution object; `max_runs` and `runs_used` | reserved-operation |
| H6 | R12: `EPISODE_SUPERSEDED`, `EPISODE_SETTLED` | execution-episode-ref |
| H7 | `transition_items` with `transition_only: true`; `TRANSITION_ONLY_INVALID`; `CLOSE_DISPOSITION_REQUIRED` on every terminal path; `HumanTaskCompleteRequest` | execution-episode-ref |
| M1 | read verbs: `actor` required, `AUTHORITY_DENIED`, unknown-case answer for unauthorized callers, `pending_id` only to case lead and approver | continuation |
| M2 | per-section `availability`; `allow`/`deny` spelling | continuation, reserved-operation |
| M3 | `CASE_CLOSED`; closed is final; registry `suspended_rule` and `closed_rule` | case-record-append.v2, case-record-kinds |
| M4 | `REFERENCE_DERIVED_FROM_RESULT` documented as best-effort | README |
| M5 | pin source named (world pin record); read-only boundary paths, `revise_frozen_boundary_field` | case-record-kinds, case-record-append.v2, reserved-operation |
| M6 | anchor publisher checked at freeze when configured | case-record-append.v2 |
| M7 | gateway actor and on-behalf-of recorded; canonical actor comparison | delegation-link |
| M8 | one definition of `FREEZE_MISSING` and `FREEZE_INCOMPLETE`; anchor-only fixture | README, execution-episode-ref |
| M9 | resume versus new `k`; resume after link expiry; id length; `ASSIGNMENT_BUDGET_EXCEEDED`; `unit` enum per currency | assignment, execution-episode-ref |
| LOW | `maxLength`, `maxItems` everywhere; non-hard `not_applicable` needs an approval; `observed` cannot cite operator or SOP | all S2 |
| Owner D23, D12, D8 | P3 for case-level close dispositions and hard-gate waivers; `assessor` back in the payload; suspended case refuses settlement and new dispositions except lift or close | reserved-operation, case-record-append.v2 |

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
| `reserved-operation.v1` | WP-31 (A3 check, decision API); WP-21 (program-execution decision on the existing approval path, approval records); WP-32 (Horizon surface) | NT-12, NT-24, NT-34; AT-09 (human settlement, PARTIAL until S3) |
| `assignment.v1` | WP-23 (session binding, budgets, resume); WP-21 (id validation, `k` allocation) | AT-17 |
| Independent-observation fixtures | WP-21 (emits the SEA-Forge events), WP-17 (rule already in RealityTrace) | NT-19 and NT-20 context; AT-09 |

Frames in `fixtures/*/frames/` are expected frames. WP-21 captures the goldens into `apps/godspeed-casework-go/internal/adapters/sfwp/testdata/frames.json` and mirrors them in the Go client and generated TypeScript.

## S1: WP-11

`frozen-reference.v1`, `anchor.v1`, `case-record-append.v1` (see `README.md` sections 1 to 3 and `evidence/s1/wp-11.md`).
