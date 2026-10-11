# Independent review — C2 V4 normative spec and ADR package

**Verdict: APPROVE normative alignment and readiness for separately scoped implementation work.** This accepts the authored specification/ADR package as an accurate normative expression of the reviewed revision-7 design. It does not prove writer migration, implementation, runtime behavior, gate completion, or settlement. Source and generated schemas remain separately authorized work; readiness remains contingent on the specification's audit and verification prerequisites.

## Reviewed inputs and final identities

I read the full normative-authoring assignment and result, its ADR identity erratum, the complete revision-7 candidate and independent design review, and the operator's six-recommendation approval record. I read the complete 714-line supplemental YAML, the final 200-line ADR-008, and the complete parent-spec text around its reference/precedence change. The review used the recorded root/global-frontier and legacy/u64 decisions embodied in the v7 candidate and specification.

| Artifact | Final bytes | SHA-256 |
|---|---:|---|
| `casework-live-cursor-v4-spec.yaml` | 39,629 | `d509ea469408e03c1cb62cf10ab463036e161ad13bf99a7b177d7ccf3406be31` |
| `ADR-008-casework-live-cursor-v4.md` | 11,106 | `30556f78e37ac3f6f33b2ea7e5f976b87645ca67e1cc51e330a54deb5dad17b3` |
| `godspeed.casework-cognitive-environment-spec.yaml` | 37,670 | `dc9ea678bab7e24d02072beb46d4079ac3181a790c91f1146919e4bb4624601c` |

The candidate basis remains v7 SHA-256 `ad82478fbe008358841b3be2ff0e29139f0327f1dd7cbbcbaf798a2795bd09b6`, with design-only independent approval recorded at SHA-256 `04d21470d73a00340979a104616448f0374314aa70f7a15affe643b6e709a98f`. The operator approval record is SHA-256 `7651f333fcf27b741db557d0dcbd4aba140e06b670edb49d70968f267be6be4b`; it approves the six recommendation-level areas and expressly does not claim review of the exact v7 hash.

## Alignment and traceability

The YAML parses successfully. It contains 30 unique requirement IDs and 39 verification-matrix entries. Every requirement is referenced by at least one matrix case; each requirement's verification references exist; every matrix requirement reference resolves. No dangling or unmapped IDs were found.

The requirements and ADR cover the full material v7 contract:

- **Authority and identities:** SEA Forge remains the sole authority; no new kernel verb, dependency, permission rule, synthetic ID, or second history. Existing IDs and opaque cursors are exact. All wire ordinals use canonical decimal strings and checked u64/BigInt handling without Number or lexical comparisons.
- **Bootstrap and completeness:** revision-3 response arms, strict complete-or-error inventory, pending-journal blocking, exact returned create ID, absent-versus-explicit case routing, and validated-snapshot readiness are retained.
- **Global range and legacy compatibility:** v2 pages validate every global row through a pinned real head, with bound continuations and separate global/case progress. Non-event rows and empty filtered pages advance only global progress. Legacy vectors return a requested prefix only after bounded proof to the real head; otherwise they fail typed with no partial success or unbounded fallback. The matrix includes sparse histories beyond 500 rows, exact scan/head boundary, and byte/lock/time exhaustion.
- **Limits and recovery:** numeric page, index, inventory, journal, retention, replay, queue, stream, aggregate, and deadline limits match v7. The document distinguishes serialized-payload limits from RSS guarantees, prohibits automatic ceiling resets, and keeps absence unproven until a validated scan reaches a pinned head.
- **Journal, writers, and locking:** existing CaseRunner-to-ledger dependency direction and lock ordering are preserved; authorization and guards precede journal effects; callbacks, network I/O, capability execution, and unbounded work stay outside the journal lock. The writer inventory includes permission-broker settlement paths, is explicitly non-exhaustive, and remains a migration-readiness gate. Unsupported external writers are not represented as coordinated.
- **Capture, history, Store, and SSE:** stamps, exact Go digest preimage/order, digest non-authority, stale preconditions, exact GET/SSE cursor-digest pairing, pre-body/pre-header errors, per-case versus cell-wide failure, client reset behavior, and no-I/O-under-Store-mutex are represented. Error codes and HTTP statuses remain explicit.
- **Release evidence:** authored schemas/contracts, independent complete writer audit, focused TDD, canonical/runtime gates, and actual writer participation remain required. The documents do not claim that any of them has passed.

The v7 additions are present: the typed legacy bounded-read outcome and caller behavior are explicit in `REQ-C2-RANGE-002` and `V-C2-LEGACY-01..03`; the exact decimal grammar, 20-digit cap, checked BigInt u64 range, prohibited conversions/orderings, and boundary cases are explicit in `REQ-C2-U64-001` and `V-C2-U64-01..02`. These match v7 §§1–2 and its final verification list. The six-area approval is recorded without requesting it again or claiming exact-hash approval.

## Parent reference and authoring deviations

The parent spec's recorded preimage SHA-256 `403c9e6e2b59a5169cfeaa9aba2f290e885b39f8f44edf50fba5785255e24169` matches the baseline read from `HEAD`. Its actual diff contains only: metadata version `0.2.5` to `0.2.6`; revised date `2026-09-30` to `2026-10-08`; a supplemental-spec/ADR reference with review status and C2-only scope; and a precedence sentence making the supplement controlling only for V4 details after independent acceptance. No unrelated parent rule changed. The version/date metadata changes exceed the assignment's literal reference/precedence-only wording, but are disclosed in the result and are routine metadata reflecting the supplemental normative reference; they do not widen authority or behavior.

The initial result reported ADR-008 as 199 lines/11,060 bytes with hash `b02e836084b14669f0e8be0db098883216ba4216c081f0352b27a73f573bf56e`. The additive identity erratum correctly supersedes that identity: the final ADR is 200 lines/11,106 bytes, hash `30556f78e37ac3f6f33b2ea7e5f976b87645ca67e1cc51e330a54deb5dad17b3`, after tightening status wording to keep independent package review pending. The final file path is the authorized ADR-008 destination; the erratum preserves the candidate and parent-spec identities.

The builder result also discloses two non-mutating Git commands despite the builder assignment's no-Git rule: a path-scoped status before editing and a path-scoped diff after editing. It states neither changed the index, worktree, or history. I performed a separate read-only `git show HEAD:<parent-spec-path>` to verify the parent preimage and narrow diff, as requested for this independent review; no Git mutation occurred. No compiler, test, formatter, scanner, build, or runtime gate was run for the normative authoring task or this review.

## Disposition

The authored spec and ADR faithfully carry the v7 obligations and preserve the explicit hold conditions. The inventory is a checklist, not proof; complete writer participation and all implementation/runtime evidence remain outstanding. This approval is limited to normative alignment and allows later, separately scoped implementation preparation under the recorded approval boundary. It is not migration readiness, source authorization, runtime approval, or T09 completion.
