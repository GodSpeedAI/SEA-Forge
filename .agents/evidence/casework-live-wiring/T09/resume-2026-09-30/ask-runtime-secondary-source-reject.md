# Ask runtime secondary source review — REJECT

Date: 2026-10-01

## Verdict

REJECT source completeness pending fresh-builder corrections. This is an independent source
review against the original runtime assignment and its supplements. No compiler or tests were
run by this reviewer; root owns the compiler token. The primary critic's separate compile
finding is not duplicated here.

## Assignment and frozen result

Reviewed `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/ask-runtime-implementation-assignment.md`,
`ask-runtime-root-integration-supplement.md`, and
`ask-live-ledger-projection-clarification.md` against the Ask runtime implementation and
fixtures. The supplements permit the adapter, live test, and narrow integration changes; the
existing two independently frozen fixture files remain unchanged. Scope is otherwise bounded
to the assigned Ask runtime and integration files. No Rust, UI, schema, golden, or dependency
change was part of this review.

## Blocking findings

1. **Malformed raw UTF-8 is not rejected before JSON decoding.** In
   `apps/godspeed-casework-go/internal/server/ask.go:58-83`, the raw body is size-checked and
   decoded without validating its byte encoding. The later check at lines 95-105 validates the
   decoded purpose string with `utf8.ValidString`; Go's JSON decoder replaces malformed UTF-8
   in JSON strings with U+FFFD, so that check cannot establish that the input bytes were valid.
   The original assignment requires strict UTF-8 input. Validate `raw` with `utf8.Valid` before
   decoding and reject without invoking either Ask or perspective verification. Add a focused
   malformed-raw-UTF-8 test asserting both call counts remain zero.

2. **SFWP comments conflate correlation-tracked mutation with every protected write and omit
   Ask's no-recovery semantics.** `internal/adapters/sfwp/frame.go:86-88` says `IsMutation`
   identifies protected side-effecting verbs, although Ask is protected and record-writing but
   intentionally has no request ID or status recovery. `internal/adapters/sfwp/client.go:260-261`
   says any request crossing the wire can be settled only through the authority correlation
   store. Lines 362-368 document inspect retry and correlated mutation recovery but omit Ask's
   typed-unavailable, no-resend/no-status behavior and the separate explicit busy retry. The
   original assignment explicitly requires correcting comments that equate protected operations
   with durable correlation. Update only these comments to describe correlation-tracked
   mutations, read-only inspect retry, uncorrelated Ask behavior, and the distinct pre-admission
   busy retry; preserve all behavior and the frozen transport fixtures.

## Material scope/evidence notes

The adapter and live integration additions are allowed by the root supplement, but source review
does not establish their runtime behavior. The live ledger clarification requires equality after
the canonical projection, retaining all eleven answer fields and checking each projected claim
field and optional reference; the three ledger-only claim fields must be recorded as an evidence
shape difference. No assertion of successful live proof is made here. The earlier fixture review
and frozen expected-RED evidence remain separate records.

## Source hashes at review

```text
4eb5440cd23e8e8a2cf0d929147ce2b71d756bf2fa9e870f3f5b259e983362b3  apps/godspeed-casework-go/internal/server/ask.go
75702c8577327b84d7cafeb23171c400e914d1a60f575e9e47588f701e8749eb  apps/godspeed-casework-go/internal/adapters/sfwp/ask.go
17a9156ec9c21d2a50a816c97fa10b3916cc756fc42eaaeffbbc690f7da94c2b  apps/godspeed-casework-go/internal/adapters/sfwp/frame.go
b21c1f018607eb28d41fc2dd7b27f2da43f2818c0f8091d11d7905566b8b277c  apps/godspeed-casework-go/internal/adapters/sfwp/client.go
09c6af57138e52fe0e5dfe504b08a6a4517fa8d65955a7cb508e463f884de6b6  apps/godspeed-casework-go/internal/ports/ask.go
90612d084a10fce6ec4871faae563790c7de5267d2ef60369bc1fc23bd538cbe  apps/godspeed-casework-go/internal/server/ask_test.go
94764ad34ca808bfa92ed96bfd91acd23fb8767c5fa9645515e57f91395426cc  apps/godspeed-casework-go/internal/adapters/sfwp/ask_transport_test.go
7564a7ef04c2b0af242b6a7f270b4a9aef701221ec8ce39ec3f6c51cb4678200  apps/godspeed-casework-go/internal/adapters/sfwp/ask_adapter_test.go
8f3613930cea8052671611549646419c6e6f6e02007a7d28631417f1d3d7eb0f  apps/godspeed-casework-go/internal/server/ask_live_test.go
```
