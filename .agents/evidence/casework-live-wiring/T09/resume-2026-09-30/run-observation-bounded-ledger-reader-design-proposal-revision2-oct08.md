# Bounded ledger reader: corrected design proposal, revision 2

**Status: proposal only.** This revises the read-only options after the
independent HOLD and root clarifications. It authorizes no source, schema,
generated-file, test, or runtime work. The specific signer lifecycle, token
and raw-input budgets, and verified-empty v2 wire arm remain subject to a
separate operator decision and normative spec/ADR update.

## Inputs and authority

Read in full:

| Input | SHA-256 |
|---|---|
| Original bounded-reader options, immutable | `a4d2aa1c7c7a023b2bf08e75b4bd9d59e3e35d87088014a31cc7463644ed7d03` |
| Full source recon | `168bbd5a11d8d427a7fe84c9fb0a3667c29a296e9ac79f9aed1a05d3c662ff97` |
| Independent HOLD of original options | `b3a832dcc49ed5fc578934064db466a30e4f08fbe5b626293b03db9287c1f94a` |
| Root bounded-reader clarification | `c6ffb613f5eeffe2a80183e9d1de73634884c9ebd4ff73cf565852c2af7b5696` |
| Root frontier-boundary addendum | `7055688af770af4de8c003901d908ddc7e7f9f2435757a4fbb2946346910e1d9` |
| Current normative C2 spec | `d509ea469408e03c1cb62cf10ab463036e161ad13bf99a7b177d7ccf3406be31` |

The full HOLD review reproduces the original architecture review assignment
and identifies the concrete repairs at its findings 1–6. Root clarification
and frontier addendum select the choices below. The six previously approved
recommendation areas remain approved at recommendation level; that approval
does not cover these additional wire/security/resource decisions or this
proposal's exact hash. This document is the new revision-2 proposal result;
the original options and HOLD remain unchanged.

## Compatibility and API shape

Keep the existing `events.get_range` method and all legacy request fields,
types, and serde defaults. Add only `#[serde(default)] range_version` to the
request; represent it as an optional unsigned integer wide enough to route an
unknown supported numeric value to a typed version error. Omitted version
continues to return the exact legacy `{"events":[...]}` vector wrapper and
the existing `EventFrame`. Do not add a verb, dependency, authority rule, ID
grammar, or persisted history. Do not modify `EventFrame`, which is also used
for live broadcast and subscriptions.

Use separate public v2 DTOs if and when implementation is separately
authorized:

* `EventFrameV2` contains the existing frame values plus the exact cursor and
  its canonical decimal-string ordinal.
* `EventPageV2` contains `range_version: 2`, `frames`, nullable real
  `pinned_head` cursor/ordinal pair, nullable `scanned_through_cursor` and
  `scanned_through_ordinal`, nullable `continuation`, `complete`, and optional
  `case_state_stamp`. Ordinals are strings, never JSON numbers.
* A nonempty stream's real first row has ordinal `"0"`. Never synthesize a
  cursor or assume the first ordinal is one.
* A supported, readable stream that is physically absent or zero bytes under
  its stream lock has the explicitly selected verified-empty v2 form:
  `frames: []`, `pinned_head: null`, both scanned-through values null,
  `continuation: null`, and `complete: true`. It has no fabricated head,
  cursor, ledger event, or case authority. A nonempty whitespace-only file,
  malformed tail, or torn row is unavailable, not empty.
* Add the new public DTOs to both SFWP schema registries and the existing
  conformance oracle in one future authorized change; do not edit generated
  Workbench schema output by hand.

For v2, `from_cursor` remains exclusive and `to_cursor` inclusive. Both values
are exact opaque cursor strings and filter emitted frames only after each
global ledger row is validated. Resolve their ordinal only by exact row
identity; never parse, normalize, or lexically order a cursor. A `to_cursor`
does not stop validation before the pinned head. Continuation payloads bind
the exact nullable `from_cursor` and `to_cursor` byte values by separate,
domain-separated SHA-256 digests over a presence tag, checked fixed-width byte
length, and exact UTF-8 bytes. Resumed requests must repeat both filters
exactly; the page frame `limit` may vary because it does not change range
membership.

The root-selected v2 unknown-cursor rule is deferred until a scan reaches the
real pinned head. Until its exact `from_cursor` is discovered, emit no matching
frames. If `to_cursor` is still absent at the pinned head, the completing
request returns typed `input_error` and no complete page. Earlier incomplete
pages may contain validated matching frames; each is only a page-prefix, never
proof that the requested range or global head is complete. Legacy behavior
remains all-or-nothing: scan through the pinned head first, classify both
cursors, then return the existing frame vector; any unknown cursor or
incomplete proof returns an error with no frames. This bounded difference is
an explicit v2 clarification, not a silent change to legacy behavior.

## Process-lifetime continuation trust

Use the existing Ed25519 and JSON primitives; do not add a token codec
dependency or reuse a persisted checkpoint key:

1. Before accepting requests, fill a fresh 32-byte seed using the server's
   existing `getrandom::fill`, construct the already exported
   `sea_forge_ledger::signing::SigningKey::from_bytes`, then clear the local
   seed using the server's existing `zeroize` dependency. Entropy failure
   fails server startup closed. Keep the key only in process memory. Never
   persist or log the seed, key, continuation, or payload.
2. Serialize a fixed-field private continuation payload with existing
   `serde_json`, then use the ledger's exported `canonical_json` for its
   canonical signed bytes. Every payload field is an ASCII version, stream
   identity, exact generated cursor, canonical decimal string, or lowercase
   digest; raw filter values are represented by the exact-byte digests above,
   so canonical JSON's existing NFC normalization cannot change cursor
   identity. Sign `domain || canonical_payload` with the existing
   `sea_forge_ledger::signing::sign_bytes`.
3. Use the unambiguous token text form
   `v2.<signature>.<canonical-json-payload>`. The payload is raw canonical
   JSON bytes inside the SFWP string; it is not an unbounded base64 blob. The
   existing signer emits the `ed25519:` signature using its existing RFC 4648
   base64 codec. The `.` delimiters do not occur in that signature. No
   visibility change to the separate private base64 helpers is needed.
4. Bind payload version, exact `ledger_id`, pinned real head cursor/ordinal/
   entry hash/end byte offset, last fully validated row cursor/ordinal/entry
   hash/start and end offsets, next byte offset, exact-row checksum, and the
   two exact filter digests. Encode every ordinal and offset as canonical
   decimal strings. At origin, last-row fields are null and next offset is
   exactly zero; this nullable predecessor is permitted only at origin.
5. On input, reject token text longer than 4,096 bytes before splitting or
   decoding. Check the fixed `ed25519:` signature representation is exactly
   96 ASCII bytes: prefix, 86 base64 alphabet characters, and `==` padding,
   which can encode only 64 signature bytes. Reject any other shape before
   calling the existing bounded `verify_signature` decoder. Check payload
   bytes are at most 2,048 before parsing; verify the domain-separated
   signature over those exact bytes before deserializing or using any seek
   field. Then parse with a strict no-unknown-fields payload type and require
   that canonical re-encoding exactly equals the signed bytes. Validate
   canonical decimal spelling, checked offset arithmetic, field sizes, stream
   and filter binding before seeking. Signature/format failure is typed
   unavailable, never a caller-controlled offset or fallback to legacy scan.

The token is stateless and has no persistent cache. Process restart rotates
the key, so prior-process tokens fail typed unavailable. A direct SFWP caller
must restart at actual origin. The trusted Go global-scan owner must accept
only the exact continuation and pinned head it expects. Failure of that
expected continuation/head binding means no acknowledgement from the failed
page, cell-wide global unavailable/drain, and bounded rebuild from actual
origin; it must not serve old ready bytes or silently fall back to a partial
legacy result. Restore readiness only after complete validated coverage to a
real head and the required index, inventory, and journal checks. An arbitrary
SFWP caller's malformed token is only a failed request and cannot mutate Go
owner state or bypass authorization.

The C2 owner still enforces the approved two-page/500 ms reconciliation
scheduling-turn bound. That is not a new per-reader page deadline. A scheduler
deadline or cancellation cannot acknowledge an unvalidated result or switch
to partial success; it must await/rejoin the actual scanner result according
to the owner lifecycle. A blocking OS read is not preempted by that scheduling
budget.

## Pinned head and cooperative append boundary

Add a synchronous ledger-layer pinned-head/read-page primitive. The ledger
layer owns offsets, LF row boundaries, row parsing, and the byte checksum; the
SFWP layer owns event projection. Keep it synchronous and invoke it through
the existing blocking-work boundary.

On a fresh request, open the existing stream lock and retry `try_lock()` only
until a monotonic 50 ms deadline. `WouldBlock` past that bound is typed
unavailable; never call blocking `.lock()` for this path. Under the lock,
sample the exact current file end and bounded last complete row, including its
real `entry_ulid`, ordinal, entry hash, and ending offset. The reverse-tail
read is at most one 2 MiB row. Release the lock before scanning pages. If the
file is absent/empty, use the verified-empty arm above. A nonempty file must
end with a complete LF-terminated row; malformed or torn tail fails closed.

The token's pinned head is stable across cooperative appends. A later append
does not move the old pin and is excluded from that request. While scanning,
the last row must match the exact pinned cursor/ordinal/hash/end offset before
the page may be complete. On resume, verify signature first; reread and
validate the complete predecessor row immediately before its signed next
offset, including its exact row checksum, cursor, ordinal, hash, and end
boundary. Only then seek. For origin offset zero, verify actual origin and
allow a null predecessor without inventing ordinal or cursor.

The row checksum covers the exact raw row bytes including the LF terminator;
start/end offsets are byte offsets and all additions are checked. This binds
both row contents and boundary. The lock fences only supported appends that
use the same stream lock and preserve old bytes. Manual, plugin, and other
out-of-tree writers remain unsupported under `REQ-C2-EXTERNAL-001`; this design
does not claim it can detect every edit to bytes already scanned. Any mutation
detected in rows read is global unavailable, with unknown case impact.

## Bounded input, row validation, and event projection

Use **2 MiB maximum raw bytes per JSONL row** and **4 MiB maximum raw input
bytes per page**, counting every row including non-event rows, every LF, and
all bytes read for lookahead. These are separate proposed admission limits;
neither is inferred from the 1 MiB output cap. The raw row cap includes its
terminating LF. Empty file is special; an empty/whitespace row within a
nonempty file is malformed. A torn final row is unavailable.

Read through a fixed-size byte buffer and a row accumulator that checks the
remaining row/page budget before extending. Never call `BufRead::lines()` or
`read_line()` and check length afterward. Only a complete LF-terminated row
within both budgets is passed to `serde_json::from_slice`. At most 500 global
rows are scanned in one page, including non-event rows; no more than 500
frames are emitted. The 4 MiB aggregate budget can stop earlier. If the page
budget is reached partway through a next row, count all consumed bytes, discard
that partial row, and return only the prior fully validated frontier with a
continuation before the row. A row above 2 MiB fails typed unavailable.

For each complete row, strictly validate its current `LedgerEntry` JSON shape
and field types before filtering. Reject unknown top-level fields; parse the
typed entry; require current ledger ID/version/canonicalization/hash
algorithm, checked successive append ordinals beginning at zero, exact opaque
`entry_ulid` identity, correct first-row/no-predecessor and subsequent
`previous_entry_hash`, recomputed `payload_hash` and `entry_hash` through the
existing ledger helpers, and exact pinned-head identity at the pinned end.
Never parse or order cursors to derive ordinals. Non-event rows count toward
all input and scan limits and advance the global frontier after validation,
but never create an event frame. These checks bound a whole-ledger row
validation pass to the rows actually read; the existing unbounded `verify()`
remains a different operation.

For `record_kind == "sfwp_event"`, validate payload shape before applying
`from_cursor`, `to_cursor`, or any caller frame filter. Use a private strict
stored-event DTO matching the actual writer at
`crates/sea-forge-server/src/sfwp/events.rs:47-59`: required `kind: String`,
required `detail: serde_json::Value` (explicit `null` is valid), optional
`case_id` and `run_id` strings (absent/null allowed), no unknown fields.
Do not add a nonempty-kind rule absent from the current writer contract. The
legacy `frame_from_entry` defaults malformed/missing `kind` to `""` and
missing `detail` to null; v2 must not reuse that permissive conversion. A
malformed event payload is global unavailable, not an empty or partial event.

The row/page limits bound serialized input, not RSS, allocator overhead, or OS
I/O latency. Existing C2 bytes are availability limits rather than a heap
guarantee. The SFWP transport already caps one JSON request line at 1 MiB
before parsing; the 4,096-byte continuation cap applies before its own token
split/verification, and the payload cap before payload decoding. No new
request-line policy is proposed.

## Page completion and response-byte boundary

Serialize the complete prospective response using the same existing
`serde_json` response path used by SFWP, then measure its exact byte length.
The 1,048,576-byte limit includes the full page envelope, frame fields,
continuation token, and JSON escaping. Do not estimate from `detail` length or
truncate/split/drop a frame. The current allowed 1 MiB event/detail does not
necessarily fit with its frame and page metadata. If one matching frame plus
the smallest required page metadata and continuation cannot fit, return typed
unavailable with no page and no acknowledgement.

For a normal incomplete page, stop at a whole-row boundary. When the next
validated matching frame would exceed the 500-frame cap or complete serialized
response budget, do not acknowledge that event row: return the previous
fully validated row as `scanned_through` and sign a continuation whose next
offset is immediately before the undelivered row. It must be reread and
validated next page. Non-event and filtered rows may advance the frontier.
All bytes read during that lookahead count against the current 4 MiB budget.
If even the empty-prefix page's necessary continuation plus that one frame
cannot fit, fail unavailable rather than repeat forever at origin. A frame
that fits with minimum metadata but not alongside earlier page frames remains
undelivered for the next page.

If no row has been acknowledged, preserve origin offset zero and a nullable
predecessor in the signed continuation. Do not manufacture a prior row or
ordinal. `scanned_through` is null at origin. `complete` is true only after
the validated scan reaches the pinned head and required cursor-bound checks
finish; reaching a frame count, event count, `to_cursor`, or filtered count is
not global completion. Any row-integrity, token, lock, cap, or payload error
returns only the typed error envelope—no page fields or partial frames for
that request.

The legacy path uses the same bounded scanner but is all-or-nothing. It stages
at most 500 requested `EventFrame`s under the 1 MiB exact serialized response
cap, and returns `{"events":[...]}` only after complete proof through its real
pinned head. If 500 rows, 4 MiB raw input, the 50 ms lock wait, or a response
limit prevents head proof, or if any cursor/payload/row is invalid, it returns
no vector and no partial frames. A requested prefix of frames does not shorten
the required global scan.

## Errors and trusted ownership

Keep range errors local to the SFWP reader: `UnknownCursor`,
`UnsupportedVersion`, and `Unavailable(reason-class)`. Map unknown cursor to
the existing fixed `events_unknown_cursor`/`input_error` behavior; unknown
version is a typed `input_error`. Map row/hash/continuation/head/lock/resource
failures to `frontier_rebuilding` without exposing token, payload, or raw-row
contents. Do not add a public Core `ForgeError` variant or reuse planner
errors. At the SFWP dispatcher, preserve the existing JSON error envelope
`{"error":...,"error_class":...}` while distinguishing unavailable from
unknown input. No HTTP mapping is selected here.

The Rust reader never mutates Go `Store` state. A trusted Go global owner
retains its expected pinned head and exact continuation; an incomplete page
may advance only the global frontier to the exact returned fully validated
row and never a case watermark past an undelivered matching frame. It accepts
completion only on a complete page matching the pinned head. Failure of the
expected token or pin is unknown-impact global failure: acknowledge nothing
from that failed page, make dependent live/bootstrap/replay unavailable and
drain streams, then rebuild boundedly from actual origin. Rebuild exhaustion
remains unavailable for operational review. Arbitrary public SFWP callers do
not supply or modify the trusted owner's expected token or readiness state.

## Existing source basis and exact code anchors

These are existing capabilities; the proposal adds no dependency:

* Ed25519 `SigningKey`, `sign_bytes`, and `verify_signature` are public at
  `crates/sea-forge-ledger/src/signing.rs:5,63-83`, re-exported through the
  public `sea_forge_ledger::signing` module. The signer returns an
  `ed25519:` standard-base64 signature; the decoder is private and allocates
  in proportion to its input, which is why the exact 96-byte signature check
  comes first (`signing.rs:85-123`).
* `getrandom::fill` already supplies OS entropy in the server at
  `crates/sea-forge-server/src/transcript_seal.rs:32-40`; direct server
  dependencies already include `getrandom`, `serde_json`, `sha2`, and
  `zeroize` (`crates/sea-forge-server/Cargo.toml`).
* The ledger exports canonical JSON, payload hash, and entry hash helpers at
  `crates/sea-forge-ledger/src/lib.rs:5-10` and
  `crates/sea-forge-ledger/src/types.rs:35-61`. The canonical JSON codec is
  used for deterministic payloads and hashes; token payload fields are
  ASCII-only, while raw filter strings are bound by exact-byte hashes.
* `LedgerEntry` actual fields are at `crates/sea-forge-ledger/src/types.rs:191-216`;
  writer construction sets first ordinal zero and copies each exact unseen ID
  into its ledger at `types.rs:1014-1058`; `write_entry` serializes the row and
  appends LF at `types.rs:1071-1084`.
* The stream writer lock currently blocks at
  `crates/sea-forge-ledger/src/types.rs:702-724`. Nonblocking
  `File::try_lock()`/`WouldBlock` handling already exists in the server at
  `crates/sea-forge-server/src/lib.rs:1092-1114,1152-1175`.
* Current `read_entries()` uses unbounded `BufRead::lines()` at
  `crates/sea-forge-ledger/src/types.rs:785-802`; current `get_range`
  materializes the entire ledger before cursor lookup/filtering at
  `crates/sea-forge-server/src/sfwp/events.rs:169-208`.
* Stored event payload writer fields are at
  `crates/sea-forge-server/src/sfwp/events.rs:47-59`. Current malformed-field
  defaults are at `events.rs:92-107`, so the new strict v2 decoder must not
  call `frame_from_entry` for unvalidated rows. Current `EventFrame` fields
  are at `events.rs:19-32`; it remains unchanged.
* The request line cap is applied before JSON request parsing at
  `crates/sea-forge-server/src/lib.rs:1302-1356`. Current request fields and
  defaults are `lib.rs:773-781`; current `events.get_range` response/error
  branch is `lib.rs:2349-2360`. Unknown cursor conformance is
  `crates/sea-forge-server/tests/conformance_sfwp.rs:1152-1185`.
* Normative C2 page limits and clauses are `.agents/specs/casework-live-cursor-v4-spec.yaml:90-104,203-230`; the trusted global failure rule is
  `REQ-C2-FRONTIER-002` at lines 243-252, and unknown/unavailable wire classes
  are `REQ-C2-ERROR-001` at lines 421-430.

## Unresolved approval boundary and next move

This revision concretely recommends, but does not authorize:

1. a fresh process-only Ed25519 continuation key, exact token format and
   4,096-byte/2,048-byte token/payload caps, including restart invalidation;
2. the 2 MiB raw-row and 4 MiB aggregate-input admission limits; these values
   can reject a row whose serialized metadata exceeds the chosen cap, and
   existing writers do not all enforce a matching raw-envelope size before
   append;
3. the nullable-head verified-empty v2 page and origin continuation rules;
4. deferred v2 unknown-cursor refusal and exact filter binding described
   above.

The normative page limit already fixes 500 scanned rows, 500 frames, 1 MiB
serialized response, and 50 ms lock wait. It defines no separate elapsed
per-page wall-clock deadline; this proposal adds none. The Go owner's
2-page/500-ms reconciliation scheduling turn remains separate. Its deadline
does not preempt a blocking OS read.

Before any source grant, an independent critic must accept this proposal and
the operator must specifically approve these additional design choices, with
spec/ADR updates. Then issue a separate bounded TDD assignment for the
synchronous ledger reader, SFWP v2 DTO/method branch, exact malformed and
boundary tests, and compatibility tests. No code, schema, tests, compiler,
gate, Graft build, Git operation, migration, or runtime action was performed
for this proposal.
