# Independent review: retained candidate algorithm implementation

Date: 2026-10-07  
Disposition: **APPROVE current helper source for the independently owned focused GREEN attempt only**

## Scope and identities

Reviewed the complete pre-edit algorithm-release assignment/result, original helper assignment, algorithm preregistration, root decisions/decomposition, prior source/design review, accepted direct-capture RED review, the complete helper implementation, frozen base/policy fixtures, manager source/tests, `RunTracePort`, and its SFWP adapter. No tests, compiler, scanner, formatter, typecheck, runtime, Git, or source edit was performed.

| Artifact | SHA-256 |
|---|---|
| Algorithm release assignment | `756984af7a3e2d1909ddd54b92f49c5d2bed00687c82d7d4d5473c31af78b54c` |
| Algorithm result | `a610c6015e43ddadb6dd7f7c4e08b760a5fa75e19aac8fa67911a9edb6c38dc5` |
| Current retained helper | `38ca7fdaf45b016fb8a55fdb72a32b15cad100fb5b31585410af943dddf7447a` (10,389 bytes) |
| Frozen base helper test | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| Frozen policy fixture | `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7` |
| Frozen manager source/test | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` / `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| Accepted direct-capture RED review | `8963e3a8ad78adc7e626f20dcae4a932ce52a97bb80c0c085b36055988cacc47` |
| Root algorithm decisions | `9dcf3886f98d4416fd1bbcded12ecc939a9f5281b9bebbe3e929dd865185aa3a` |

The root's lossless JSON content preimage decodes to the prior 5,836 source bytes, SHA-256 `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331`. I independently decoded and compared the exact preimage against the complete current file. The complete diff replaces only the candidate-builder comment/body and adds `runObservationSnapshotIsTerminal`, `cloneRunTraceFrame`, `cloneRunObservationRetainedState`, and `retainedMarkerCopy`. The canonical retained-image types, field order, fixed-width serializer, and marshal logic are unchanged. The five frozen manager/helper/policy source and test identities match current files. Graft found no same-name helpers elsewhere in the Go app.

## Candidate validation and input boundary

The builder rejects requested/snapshot key mismatch and prior/requested key mismatch before mutation (`run_observation_retained_version.go:76-80`). It rejects negative `TotalFrameCount`, totals below the returned window length, windows over 1,024, and duplicate exact IDs within a window (`:82-92`). It neither normalizes nor sorts event IDs and preserves frame order in the candidate loop (`:120-143`). Decreasing source totals remain accepted; the helper derives omitted/truncated values in the existing serializer without treating totals as sequence or ledger length (`:299-310`).

The validation boundary matches the release assignment/root decision: the helper trusts a well-formed safe `RunTracePort` projection and does not reproduce all enum/frame metadata validation. The actual SFWP adapter checks requested identity, response identity, execution and settlement allowlists, trace-record presence, frame kind, nonblank event ID/timestamp, RFC3339 syntax, duplicate IDs, command status/exit metadata, and the 1,024-frame tail projection (`internal/adapters/sfwp/run_trace.go:67-176,254-292`). The port contract defines `TotalFrameCount` before its retention and says it is not source-journal completeness (`internal/ports/run_trace.go:10-20`). The code's comment at helper lines 67-69 accurately confines its claim to that safe projection. `RunTracePort` is an interface, so arbitrary custom implementations are not proved safe by this review; the root-ratified production-adapter trust precondition is the boundary.

The adapter decodes JSON into Go strings and contains no separate raw-wire UTF-8 validation. Root's decision explicitly places malformed UTF-8/custom-port sanitization outside this unit's guarantee; this review likewise makes no claim for malformed wire bytes. No additional validation framework or duplicated enum checks are warranted within this released helper scope.

## State construction, ordinals, and atomicity

Final terminal-retention and stop-scheduling markers return a deep clone and reject later valid candidates (`:94-100`). Generation maximum is checked before increment, returns the typed generation-overflow outcome, and changes only the copied availability marker to stop-scheduling (`:101-105`). Initial accepted generation is 1; replacement generation increments once.

For other previous states, construction starts with a fresh ledger and copies the full prior map (`:108-118`). Each source frame reuses its exact existing ordinal or receives the next ordinal in source order. The `MaxUint64` guard occurs before increment; on overflow, locally accumulated state is discarded and the untouched prior is copied with retention-unavailable, or terminal-retention-failure when the snapshot execution is terminal (`:120-143`). Because the caller's prior map/window are not modified, this path is atomic even if an overflow is encountered after earlier candidate-local new IDs. Lifetime entries are never evicted; reorder, disappearance/reappearance, shorter windows and rewritten payloads preserve first ordinals.

The candidate uses the supplied key, exactly `acceptedAt.Format(time.RFC3339Nano)`, snapshot execution/settlement/counts, fixed derived `validated` observation state, a fresh source-order frame slice, the full copied/extended ledger, and current availability (`:145-152`). `cloneRunTraceFrame` copies both optional pointer values (`:179-190`); `cloneRunObservationRetainedState` copies every retained frame pointer, frame slice, and ledger map (`:192-212`). Marker copies change only availability after deep cloning (`:214-220`). No prior mutable field is updated by this pure transform, and no candidate metadata is retained on refusal.

## Budget and refusal behavior

The complete candidate is marshaled using the existing explicit canonical image; it is accepted iff serialization succeeds and the serialized image is at most exactly `1<<20` bytes (`:153-156`, image limit at `:14-16`). There is no frame or ledger pruning to fit. A first over-budget candidate returns rejected/nil (`:158-160`). With prior state, budget refusal returns only a deep safe copy plus retention-unavailable, or terminal-retention-failure based solely on execution (`:161-167,170-177`). The schema keeps fixed-width availability and 20-digit uint64 generation/ordinal values in the unchanged canonicalizer (`:236-310`); test fixtures assert exact/+1-byte images and marker-width preservation.

These branches align with root decisions: generation overflow stops scheduling; ordinal overflow can recover when later candidates use already-seen IDs; terminality is execution-only; final markers block later publication; read/retention-unavailable prior states can recover to current; and rejection preserves prior timestamp, generation, metadata, frames, high-water and ledger. The repaired policy tests provide focused source assertions for exact markers, caller prior/image immutability before/after returned-copy mutation, terminality, later final-marker refusal, count semantics, recovery, identity mismatch and count invalidation. The frozen base tests cover ID reordering/shorter/reappearance, clone behavior, exact budget, terminal leakage, fixed widths, duplicate IDs and the frame window. Source review does not claim these tests passed.

## Scope differences and limits

No material implementation deviation from the release assignment was found. The implementation changes only the authorized private helper file and leaves schemas, image shape, field order, manager, fixtures, ports, adapters, and public surfaces unchanged. The package-local result/preimage placement is documented as a read-only `.agents/evidence` location constraint. Existing image struct formatting predates this implementation; no formatting verification or claim is made.

Approve this source for root's separate focused GREEN attempt against the frozen helper/policy fixtures. Approval is limited to source readiness and does not establish compilation or behavior. It does not approve manager lifecycle behavior, leases/cancellation/join/capacity, integration, `Next`, SSE, production claims, full-package results, or T09 settlement. The per-poller 1 MiB claim remains scoped to this helper's canonical image; manager-held controls outside this helper must be accounted before a whole-poller bound is claimed. Root retains the compiler run and all subsequent lifecycle decisions.

No test/compiler/runtime result is claimed. Graft retrieval saved approximately 49,233 tokens (about $0.04, plus less than $0.01 for the small grep result) this turn.
