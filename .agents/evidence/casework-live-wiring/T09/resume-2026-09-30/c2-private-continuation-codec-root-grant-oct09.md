# Private continuation codec grant proposal

**Status: draft pending independent review and explicit root implementation
release.** This document records the root's proposed architecture for the
next private token-codec slice. It grants no source implementation, compilation,
tests, public DTO/schema work, startup-key integration, ledger I/O, or runtime
release.

## Governing inputs

- Normative supplement: `.agents/specs/casework-live-cursor-v4-spec.yaml`,
  SHA-256 `8ccf30be4131de57e6118bdbdd9f272c20edfd47bf0915f0fe74c7c5edfe3b11`.
  Relevant requirements: REQ-C2-RANGE-001 (`:228-239`), -003 (`:253-271`),
  -004 (`:272-314`), -005 (`:315-340`), and verification cases V-C2-RANGE-01
  and -03.
- Approved registration/accounting clarification:
  `c2-reader-registration-input-accounting-root-decisions-oct09.md`, SHA-256
  `f0a5ec83e9a84e7ffa45ad69e1df743082f07fc26f45413d7cb5a3077f2c32c8`.
- Approved design basis: `live-cursor-v4-complete-candidate-revision7-oct08.md`
  (`:19,23`); proposal context only, not runtime or implementation evidence.

The parent GodSpeed casework specification remains authoritative outside this
supplement's scope. Existing C2 approvals do not approve the exact token codec
or any exact source hash; this proposal does not change policy or authority.

## Proposed private format and validation order

Use an opaque UTF-8 token consisting of the exact compact signed JSON payload
bytes, one `.` separator, and the existing fixed 96-byte ASCII
`ed25519:`-prefixed signature representation. Locate the separator at
`token_len - 97`; do not search/split on periods, since the signed JSON payload
may itself contain periods. This is an internal encoding choice only; no public
DTO or dispatch integration is in scope.

Admission must proceed in this order:

1. Measure the token's UTF-8 byte length and reject above 4,096 before slicing,
   splitting, decoding, or allocating payload state.
2. Use checked subtraction to locate the fixed suffix and separator. Require a
   nonempty payload of at most 2,048 bytes and a suffix of exactly 96 ASCII
   bytes with the `ed25519:` prefix. Reject invalid UTF-8 or malformed layout.
3. Borrow the payload and signature slices. Verify the signature over the
   exact payload bytes before JSON parsing or any use/exposure of position data.
   The existing verifier may be used after the fixed suffix precheck; its
   decoder then receives only the bounded fixed-size signature. Invalid input
   maps to a private typed token error for later unavailable mapping, without
   fallback.
4. Only after successful signature verification, deserialize the private
   version-1 payload. Reject unknown JSON fields. Validate canonical number
   strings, digest/checksum encodings, filter state, and position relationships
   before returning any decoded state to a caller.

Do not add a Base64 payload layer or a dependency. The direct server dependency
already includes `sha2`; the token payload remains UTF-8 JSON. This avoids
variable decoded payload allocation before admission and signature verification.

## Signed payload semantics

The private version-1 payload carries:

- `version`;
- `stream_digest`, `from_digest`, and `to_digest`;
- the complete `pinned_head` and acknowledged row positions;
- nullable `resolved_from_ordinal` and `resolved_to_ordinal` values.

Each position carries canonical unsigned decimal strings for `start_offset`,
`end_offset`, and `append_ordinal`; exact opaque `entry_ulid` and `entry_hash`;
and a lowercase 64-character hex raw-row checksum. Digests are lowercase
64-character hex strings. This duplicates the semantic data needed to resume;
it does not add serde derives to public ledger reader types.

Use these three distinct fixed ASCII domain tags:

- `sea-forge/casework/continuation/stream/v1`
- `sea-forge/casework/continuation/from/v1`
- `sea-forge/casework/continuation/to/v1`

For each digest, hash `domain_bytes || presence_byte || length_u64_be ||
exact_utf8_bytes`. The length is checked before conversion; absent values use
presence byte zero and zero length, present values use byte one and the exact
UTF-8 byte length and bytes. Do not normalize or parse identifiers. Recompute
and compare stream and filter digests against the registered stream and
requested filters before returning any position or resolved-filter state to
seek logic.

Absent filters require a null resolved ordinal. Present-but-unresolved filters
also carry null; their distinct filter digest records presence. Non-null
resolved ordinals must be canonical u64 decimal strings no later than the
acknowledged ordinal. In particular, ordinal zero is `"0"`, never a null
sentinel. Require acknowledged ordinal no later than pinned-head ordinal;
reject overflow, reversed/impossible offsets, positions beyond the pin, and
unequal acknowledged/head positions when their ordinals are equal.

The codec does not establish ledger integrity or acknowledgement eligibility.
Issuance is allowed only from a caller-supplied already validated and
ACK-eligible prefix. On resume, the ledger reader must independently revalidate
the original pin and acknowledged row bytes/checksum before seeking. A token,
digest, cursor, or ordinal grants no authority.

## Scope and existing seams

The implementation, if separately released, is limited to one private module
under `crates/sea-forge-server/src/sfwp/` and its private module declaration in
`sfwp/mod.rs`. Keep token types/helpers unexported and place focused unit tests
beside them. No `SCHEMA_TYPES`, generator, committed schema, public request/page
DTO, dispatch, handler, startup `ServerState`, signer lifecycle, ledger source,
or filter-resolution integration is authorized by this codec proposal.

The server directly depends on `sha2`, `getrandom`, and `zeroize`, but has no
direct Base64 dependency. Existing signing APIs are
`sea-forge-ledger::signing::sign_bytes` and `verify_signature`. Existing
`LedgerReadPosition` has public fields but intentionally has no serde derives;
the server codec must define its own private encoded position shape. Future
startup integration must separately generate a fresh process-memory Ed25519
key with `getrandom::fill`, fail startup on entropy failure, and zeroize the
seed immediately after key construction. This codec slice only accepts
references to signing/verifying keys in focused unit vectors.

## Focused TDD vectors for a later implementation release

The future private tests should cover:

- valid round-trip of exact payload fields and fixed signature suffix, including
  payload periods;
- signature tampering and wrong-key rejection;
- token 4,096-byte and payload 2,048-byte exact/cap-plus-one cases;
- verification-before-parse behavior using separately asserted invalid-JSON
  signature failure versus validly signed invalid-JSON parse failure;
- wrong signature prefix/length, non-ASCII suffix, invalid Base64, and decoded
  signature length rejection;
- distinct stream/from/to digest golden vectors, absent versus present-empty,
  and exact UTF-8 byte handling;
- canonical u64 strings, overflow, malformed digests/checksums, filter presence
  and unresolved state, ordinal/frontier/offset relationships;
- rejection of unknown payload JSON fields and malformed layout.

Tests must use fixed local inputs and actual codec functions. No mutable hooks,
global budget overrides, fake alternate lifecycle, or real ledger reads are
needed for this private codec slice. Do not claim that parsing proves a
position's ledger integrity or makes it ACK-eligible.

## Release boundary

This proposal remains held until an independent reviewer accepts the exact
document and root explicitly releases implementation. No source edit, compiler,
test, formatter, gate, Git operation, public contract change, dependency,
startup change, or runtime claim follows from this draft alone.
