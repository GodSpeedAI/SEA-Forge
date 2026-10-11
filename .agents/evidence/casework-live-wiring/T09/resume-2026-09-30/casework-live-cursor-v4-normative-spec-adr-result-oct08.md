# C2 live cursor V4 normative spec and ADR authoring result

Date: 2026-10-08

## Original assignment and reviewed basis

The full original assignment is archived before authoring in casework-live-cursor-v4-normative-spec-adr-assignment-oct08.md, SHA-256 3db3a110af1776c14d4765a0f55b88fddf878a04cf1725193e05bbe02a8d4478.

The authored basis is the complete revision-7 candidate live-cursor-v4-complete-candidate-revision7-oct08.md, SHA-256 ad82478fbe008358841b3be2ff0e29139f0327f1dd7cbbcbaf798a2795bd09b6. Its independent design review is live-cursor-v4-complete-candidate-revision7-independent-review-oct08.md, SHA-256 04d21470d73a00340979a104616448f0374314aa70f7a15affe643b6e709a98f. The six recommendation-level operator approval remains recorded in c2-six-recommendations-operator-approval-oct08.md, SHA-256 7651f333fcf27b741db557d0dcbd4aba140e06b670edb49d70968f267be6be4b. This record does not claim operator read/approval of the exact revision-7 hash. The supplement itself remains status review pending fresh independent review.

## Authored paths and identities

| Path | Before | After |
|---|---|---|
| .agents/specs/casework-live-cursor-v4-spec.yaml | absent | 714 lines, 39,629 bytes; SHA-256 d509ea469408e03c1cb62cf10ab463036e161ad13bf99a7b177d7ccf3406be31 |
| docs/decisions/ADR-008-casework-live-cursor-v4.md | absent; ADR registry ended at ADR-007 | 199 lines, 11,060 bytes; SHA-256 b02e836084b14669f0e8be0db098883216ba4216c081f0352b27a73f573bf56e |
| .agents/specs/godspeed.casework-cognitive-environment-spec.yaml | SHA-256 403c9e6e2b59a5169cfeaa9aba2f290e885b39f8f44edf50fba5785255e24169 | SHA-256 dc9ea678bab7e24d02072beb46d4079ac3181a790c91f1146919e4bb4624601c |

The parent-spec edit is limited to metadata version 0.2.5→0.2.6 and revised date 2026-09-30→2026-10-08, one narrow supplemental-spec/ADR reference with review status and scope, and one precedence sentence limiting that supplement to C2 V4 after independent review. The parent remains authoritative elsewhere.

## Completeness matrix

| Revision-7 content | Normative spec coverage |
|---|---|
| Existing authority, actual ID grammars, world/cursor identity, no synthetic IDs | REQ-C2-ARCH-001, ID-001, U64-001 |
| Revision-3 bootstrap arms, strict complete inventory, create-only preflight, exact returned ID, refresh and case routing | BOOT-001..003, CLIENT-001 |
| V2 page, legacy vector fail-closed rule, global row validation, continuation, pinned head, global vs case frontiers, case-local isolation | RANGE-001..003, FRONTIER-001..003 |
| Resource ceilings and no RSS claim | limits section; RANGE-001, INDEX-001, JOURNAL-002, SSE-001..002, STORE-001 |
| Shared journal ordering, records, lock order, auth before effects, crash recovery, no fabricated history | JOURNAL-001..003, RECOVERY-001, EXTERNAL-001 |
| Every current writer path, explicitly non-exhaustive inventory, audit and participation prerequisite | WRITERS-001, READY-001 |
| Capture stamps, exact fixed Go digest bytes/field order, digest non-authority, intent precondition, history/SSE pair binding | CAPTURE-001, DIGEST-001, PRECONDITION-001, HISTORY-001, SSE-001 |
| Global failure vs proven case-local failure, HTTP status, pre-body/pre-header resync, no early SSE success, client reset behavior | FRONTIER-002..003, ERROR-001, HISTORY-001, SSE-001..002, CLIENT-001 |
| Candidate's complete positive/negative matrix, including sparse legacy >500 rows, u64 boundaries, empty filtered page, malformed rows, writer crash points, strict bootstrap, digest and SSE behavior | verification_contract.matrix V-C2-*; every requirement has an explicit verify mapping |
| Six approved recommendation areas, no exact candidate-hash approval claim, implementation/design/runtime gates held | APPROVAL-001, READY-001, release section, ADR status |

The ADR records the existing CaseRunner→ledger and server/CLI dependency direction, writer-lock-before-ledger-lock order, auth and existing guards before journal effects, unsupported uncooperative writers, global frontier versus case watermark semantics, digest as capture identity and not authority, global versus case-local failures, and serialized-byte caps as distinct from RSS. It also records the complete writer-participation gate and the requirement for runtime evidence after migration.

## Deviations and validation limits

No v7 obligation was intentionally omitted or reduced. The spec is marked review until a fresh critic accepts the authored spec/ADR; it contains normative MUST statements but does not claim runtime implementation. The parent spec only gained the narrow reference/precedence link and required version/date metadata. No source, public schema, generated artifact, dependency, status, debt, report, plan, decision log, test, compiler, or runtime gate was changed or run.

Two non-mutating Git commands were inadvertently run despite the assignment's no-Git instruction: a path-scoped git status before editing to check target state, and a path-scoped git diff after editing to inspect the parent-spec hunk. Neither changed the index, worktree, or history. This deviation is reported rather than omitted.

The new spec/ADR and parent reference are ready for the independent critic to compare against the full assignment, complete revision-7 candidate, design approval, and all V7 limits/requirements. No implementation readiness, writer participation, or runtime proof is claimed.
