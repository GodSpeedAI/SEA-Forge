# C2 bounded reader root clarification for proposal repair

Scope: architecture decisions for a corrected proposal and independent review,
not a source grant or operator approval. Read with the normative C2 spec,
bounded-reader recon, options note and independent HOLD reviewb3a832dc.

## Continuation trust and parsing

Recommend a new process-lifetime Ed25519 signer using existing ledger signing
primitives and the server's existing getrandom dependency. Generate a fresh
seed before accepting requests; entropy failure fails startup closed. Do not
persist or log seed/key/token payloads, reuse checkpoint keys, add dependencies,
or change caller authorization. Issue tokens only after a validated global
prefix. Sign a domain-separated fixed-field encoding binding stream identity,
pinned head cursor/ordinal/hash/end, last complete row cursor/ordinal/hash,
next offset and preceding raw-row checksum. Verify signature before trusting
any seek input, then reread and verify the signed preceding row. Appends remain
cooperative and prefix-immutable; unsupported external edits remain unsupported.

Recommend4096 encoded bytes per token and2048 serialized payload bytes. Check
the outer length and fixed signature encoding/decoded64-byte signature length
before decoding; bound payload decoding and field parsing. Over-budget metadata
returns typed unavailable instead of changing ID grammar. The exact encoding
must be unambiguous and use existing codecs; if no existing codec is usable,
surface that rather than adding a dependency.

Old process-key tokens fail typed after restart. The Rust reader never changes
Go Store state directly. The trusted Go global scan owner must treat a failed
continuation it was using as no acknowledgement and cell-wide unavailable/drain,
then rebuild boundedly from actual origin. An arbitrary SFWP caller's malformed
input cannot become the trusted owner's continuation or bypass authorization.
No successful ready/replay response is served from a failed scan.

## Origin, empty history and request bounds

First actual ledger ordinal is0. Do not assume1 or invent a cursor for an empty
stream. V2 scans from actual origin or a server-verified token; from_cursor and
to_cursor only select output bounds after validation. A cursor cannot prove a
prefix. Preserve existing exclusive-from/inclusive-to semantics and unknown
cursor input refusal. Define their behavior across pages and pinned head
explicitly, including sparse/non-event rows.

Recommend a distinct verified-empty page representation: nullable real head
and nullable scanned-through ordinal, no frames, no continuation, complete true.
Only an actual bounded snapshot of a supported stream with no rows establishes
this state; unreadable/malformed state is unavailable. This supplies no snapshot
cursor, synthetic ledger event or case authority. The Go owner must additionally
prove journal cleanliness and complete case inventory before bootstrap.no_cases.
This empty-head wire clarification needs normative acceptance before code.

## Allocation, head snapshot and response limits

Recommend explicit admission limits of2MiB per raw JSONL row and4MiB total raw
bytes per page, including non-event rows. They are not a proof that arbitrary
ledger metadata fits; over-limit inputs fail typed before unbounded allocation.
Use bounded buffered reads. A page may stop at the last fully validated row
before the aggregate budget, with signed continuation and incomplete state.
No partial-complete claim or partial legacy success is permitted.

Keep the approved500 rows/500 frames/1MiB serialized response/50ms lock wait.
Do not claim a1MiB detail or event necessarily fits its framed response. Measure
the entire prospective serialized v2 response before emitting. If an individual
frame plus required page metadata cannot fit, return typed unavailable with no
partial output or acknowledgement; do not split/drop it or increase the cap.
Document this admission consequence and test exact boundaries. A fitting page
may stop before a later fitting frame with incomplete progress, provided it has
validated the relevant rows and its continuation cannot skip an undelivered
matching frame. Define the pending-frame/frontier choice precisely in repair.

Acquire the existing stream file lock with try_lock and a monotonic50ms deadline.
Under the lock, obtain the actual complete bounded tail and pinned byte end;
release before page scanning. Empty and genesis behavior above apply. Concurrent
cooperative appends cannot move the pinned target. Malformed/torn tail or head
binding failure yields unavailable, never a manufactured head.

## Compatibility and approval boundary

Keep legacy no-version response exactly {events: [...]}, existing request
field types/defaults and unknown-cursor classification. Use distinct v2 DTOs,
not modifications to broadcast EventFrame. Method-local typed unavailable maps
at SFWP without inventing a Core planner error or new authority rule.

Signer lifecycle, explicit token/input admission budgets and verified-empty
wire clarification require a concrete reviewed operator decision and spec/ADR
update. The earlier six public recommendations and private policy approval do
not silently approve these newly identified details. No implementation is
released by this note; root will review the corrected proposal and independent
evidence before asking that targeted question.
