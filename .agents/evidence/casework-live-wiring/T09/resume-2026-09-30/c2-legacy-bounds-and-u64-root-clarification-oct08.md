# C2 legacy bounded-reader and full-u64 clarification

Date: 2026-10-08. Root semantic clarification within the operator-approved
bounded-reader and ordinal-contract recommendations. No implementation proof.

## Legacy response

An omitted range_version preserves the existing vector response shape. It does
not provide a continuation or completion field. Therefore the legacy reader
must validate its bounded scan to the real pinned head before returning a
successful vector. If row, serialized-byte, lock or time ceilings prevent that
validation, return typed unavailable and no success vector or partial frames.
Do not silently stop at the scan ceiling, label a prefix complete, or exceed
the bounds for compatibility. Requested result limits may retain their existing
prefix-selection behavior only after the bounded scan proves coverage to that
head. Large histories require version2 paging; the old vector cannot claim
complete coverage it has not validated. Unknown versions remain typed failures.

## TypeScript ordinal handling

On the wire, append/cursor ordinals are canonical unsigned decimal strings,
grammar `0|[1-9][0-9]*`, bounded to u64 (maximum18446744073709551615).
TypeScript validates grammar, length (at most20 digits) and checked BigInt
range before using an ordinal. Compare validated BigInts only; never convert
through Number, parseInt, parseFloat or lexical string order. Values above
Number.MAX_SAFE_INTEGER must remain exact. Reject signs, whitespace, leading
zeros, decimals, exponents, overflow and nonstrings. If a client has no ordinal
comparison responsibility, preserve the validated decimal string opaquely.
Opaque event cursors and capture digests are always exact identities; do not
parse, normalize, order or derive an ordinal from them. Pin tests at zero,
MAX_SAFE_INTEGER and its successor, u64 maximum and overflow, malformed
representations and exact wire round trips.

Revision7 must preserve the entire revision6 candidate and its prior decisions,
add these rules in the reader/client/verification sections, and retain all
runtime, complete-writer audit and authored-spec verification requirements.
Independent review must verify the repaired complete candidate against the
original instructions and operator-approved recommendations. Approval of the
recommendations remains valid; the design rejection does not revoke it.
