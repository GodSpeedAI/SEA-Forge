# Independent re-review — bounded-reader normative repair

**Disposition: APPROVE the normative package only.** The three findings in the prior rejection are closed in the final spec and ADR. This approval covers normative fidelity only; it does not authorize implementation, schemas, migration, runtime readiness, or T09 settlement.

## Reviewed basis and final identities

I reviewed the original normative update result and its reproduced grant, the full repair result and reproduced repair grant, my prior rejection, the four-policy operator receipt, the revision-2 proposal and revision-3/4 corrections, the revision-4 proposal-only approval, and the additive identity/traceability correction. The earlier six-area authorization and normative package review remain the baseline authority. The final file hashes were independently calculated:

| File | Final SHA-256 |
|---|---|
| `.agents/specs/casework-live-cursor-v4-spec.yaml` | `4bb9730c25e582be159a35c5febe0f09f8af8ef4c17b59683599d147ff5e1b67` |
| `docs/decisions/ADR-008-casework-live-cursor-v4.md` | `c66912baee08317e4819ad211a208f5e2abff5d3369053a743a0f384737cc4e5` |

The immutable repair result records its first post-edit hashes (`5ae60125…d3d7c3d` and `b6c737e8…cc3b5d`). The additive correction explains that it subsequently added `REQ-C2-RANGE-005` to `V-C2-RANGE-01` and added the corresponding digest acceptance vector to ADR-008; it gives the final hashes above. This is a disclosed post-result source-document change, not a claim that the earlier result already identified the final bytes. The correction adds no requirement or matrix ID and reports no other source change.

## Rejection findings closed

1. **Registered-empty predicate:** `REQ-C2-RANGE-005` now requires an actually registered stream whose registered path is proven absent or zero-byte by a successful check under the cooperative lock. `V-C2-RANGE-02` distinguishes registered-empty from unregistered, unreadable, and corrupt history. ADR-008 repeats the same predicate and cases. This matches the operator receipt.
2. **Exact-byte binding:** `REQ-C2-RANGE-004` and `REQ-C2-RANGE-005` specify distinct fixed ASCII domain tags and a domain-separated SHA-256 preimage of presence byte, checked UTF-8 byte length encoded as unsigned 64-bit big-endian, and exact UTF-8 bytes; absence uses its absent tag and zero length. The signed token carries digests, not raw stream/filter strings, and recomputes and compares them against the requested exact registered stream and filter values before seek or use of resolved state. `V-C2-RANGE-01` now includes both RANGE-004 and RANGE-005 and pins distinct stream/from/to domains, encoding, mismatch-before-seek, and digest-only token fields. ADR-008 specifies and vectors the same. This preserves the token and payload caps, exact identity, and no-normalization/no-new-ID-grammar boundaries of the approved proposal.
3. **Key seed handling:** `REQ-C2-RANGE-004` specifies existing `getrandom::fill`, startup refusal before accepting requests on entropy failure, and immediate zeroization of the local seed after signing-key construction using existing `zeroize`. `V-C2-RANGE-01` covers entropy failure and seed clearing; ADR-008 repeats these acceptance vectors. Fresh process-memory keys, restart invalidation, bounded recovery, and no persistence/logging remain intact.

## Scope and preserved requirements

The final spec and ADR retain the prior six-area C2 package and its authority, writer-participation, migration, and runtime-proof holds. The bounded-reader provisions retain the legacy response and `EventFrame` shape, separate v2 DTOs, existing typed `entry_hash` and complete-`Value` `payload_hash`, unknown-field compatibility, known event-field validation, signed acknowledged-prefix resolved filters, empty-bound input classification, and no-lookahead acknowledgement rule. The 2 MiB row and 4 MiB page-input admission caps remain distinct from the unchanged 500-row, 500-frame, 1 MiB complete-response, and 50 ms lock-wait caps. The documents continue to distinguish those byte caps from RSS/heap and blocking-I/O latency, and the 500 ms/two-page owner scheduling bound from any reader elapsed-time guarantee. No new authority, dependency, kernel verb, public legacy reshape, identifier grammar, or destructive owner-state fallback appears in the repair.

No source, schema, implementation, test, compiler, gate, runtime, Git, or status action was performed. No readiness or settlement conclusion follows from this normative-package approval.
