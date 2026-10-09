# Bounded ledger reader proposal: revision 3 correction overlay

Date: 2026-10-08  
Base proposal: `run-observation-bounded-ledger-reader-design-proposal-revision2-oct08.md`  
Base SHA-256: `4ff005dd073c186f159cf1e1f47432fb0e119106d105cd0b8b547ca04aa71227`  
Prior HOLD: `run-observation-bounded-ledger-reader-design-proposal-revision2-independent-review-oct08.md`  
Prior HOLD SHA-256: `1641b2ca1a26db550d86cd6c63b2897f4f0323d9ed6aa12ffc24c4e2484956d7`  
Known-field clarification: `c2-bounded-reader-known-field-compatibility-clarification-oct08.md`  
Clarification SHA-256: `e61df326baab37703e179e4c24d6a4a2d2a1535d0505d3f25aa54cb071e0f0f0`

This is an additive overlay, not a replacement of the immutable revision 2.
Apply only the paragraph replacements below. All unmentioned revision 2
choices remain unchanged, including the process-only signer, token limits,
raw-row/page budgets, head-null representation, cursor-filter binding,
restart behavior, error boundary, and pending operator/spec approval status.
This correction approves no implementation.

## Replacement for revision 2, “Process-lifetime continuation trust,” steps 2 and 4

Replace the claim that all token payload values are ASCII and that canonical
JSON alone preserves stream identity with:

> The fixed-field private continuation payload may use canonical JSON for its
> fixed ASCII fields, generated cursors, canonical decimal strings, and
> lowercase digests. Bind `ledger_id` by a separate, domain-separated
> SHA-256 over an unambiguous exact-byte encoding:
> `domain || presence_tag || checked_u64_be(utf8_byte_length) || exact_utf8_bytes`.
> Require the stream-ID presence tag and checked length to be valid, then
> compare the digest against the actual target `LedgerStream.ledger_id` bytes
> before seeking. Do not normalize, recode, parse, or derive identity from the
> digest, and do not add an ID grammar. This exact-byte binding is independent
> of canonical JSON string normalization. Continue binding `from_cursor` and
> `to_cursor` using their already specified presence-tag/checked-length/exact-
> UTF-8 digests. Token integrity is not request authority.

For payload field 4, replace the direct exact `ledger_id` field with its
`ledger_id_sha256` binding above. Keep all existing pinned-head, predecessor,
offset, checksum, and filter fields and signature coverage. The private
continuation remains strict about unknown fields; this correction changes no
token size cap or token grammar.

The current path guard happens to limit ledger IDs to nonempty ASCII
`[A-Za-z0-9_-]` strings of at most 128 bytes (`crates/sea-forge-core/src/path.rs:102–110`,
called by `LedgerStream::open` at `crates/sea-forge-ledger/src/types.rs:479–501`).
The token must nevertheless bind the actual UTF-8 bytes instead of relying on
that current restriction or on canonical JSON normalization. No new ID
grammar is proposed.

## Replacement for revision 2, “Bounded input, row validation, and event projection”

Replace the paragraph that says to reject unknown `LedgerEntry` fields and
the paragraph that specifies a strict no-unknown-fields `StoredEvent` with:

> Preserve existing unknown-field tolerance for both `LedgerEntry` and stored
> event payloads on legacy and v2 reads. Parse the raw row as JSON and validate
> known `LedgerEntry` fields and their types through the existing typed shape;
> do not add `deny_unknown_fields`. Retain the raw JSON object for integrity
> hashing. Compute the row entry hash from the raw canonical row object with
> only `entry_hash` removed, so unknown top-level fields covered by the
> producer’s entry hash are not discarded. Compute `payload_hash` over the
> complete `payload` JSON value, including unknown payload fields, before
> projecting an event. Require the known mandatory fields, field types,
> version/canonicalization/hash identifiers, checked ordinal/linkage, exact
> cursor identity, and both hashes. Continue failing closed on malformed known
> fields or broken hashes. The new private continuation payload remains
> strict about unknown fields because it has no legacy reader compatibility
> contract.
>
> For `record_kind == "sfwp_event"`, validate the known event fields without
> rejecting extra payload fields. Require `kind` to be a JSON string and the
> `detail` key to be present with any JSON value; explicit `null` is valid.
> `case_id` and `run_id` may be absent or null, otherwise they must be strings.
> Do not add a nonempty-kind rule or another kind grammar. Unknown event
> payload fields remain in the payload used for integrity hashes, even though
> the projected frame does not expose them. Do not use the legacy
> `frame_from_entry` defaults to conceal a missing required known field.

This preserves behavior supported by the existing Serde types: `LedgerEntry`
at `crates/sea-forge-ledger/src/types.rs:191–216` and `StoredEvent` at
`crates/sea-forge-server/src/sfwp/events.rs:47–59` have no
`deny_unknown_fields` attribute. `serde_json::Value` retains unknown fields
inside `payload`. The current `entry_hash` helper serializes a typed
`LedgerEntry` before removing `entry_hash`
(`crates/sea-forge-ledger/src/types.rs:53–61`), which cannot retain unknown
top-level fields after typed deserialization. The bounded reader therefore
must hash the raw row object, using the same canonical JSON profile and hash
domain, or otherwise preserve those fields through hash computation. For
current rows with no unknown top-level fields, this is the same canonical
preimage as the current helper.

The existing event writer creates `kind`, optional `case_id`, optional
`run_id`, and `detail` (`crates/sea-forge-server/src/sfwp/events.rs:47–59,61–89`).
The correction requires the known required fields while preserving unknown
payload-field acceptance and explicit-null `detail`. No old response fields,
types, or defaults change.

## Replacement for the source-basis entry-ulid statement

In revision 2, “Existing source basis and exact code anchors,” replace the
sentence claiming the append constructor “copies each exact unseen ID into its
ledger” with:

> `append_under_lock` assigns `append_ordinal` from the prior row, generates
> `entry_ulid` with `ulid()`, copies it to `record_ulid`, computes the hashes,
> and appends the serialized entry (`crates/sea-forge-ledger/src/types.rs:1005–1085`).
> It does not maintain an exact-unseen-ID ledger. Duplicate-ULID detection
> exists in `LedgerStream::verify()` as a local `HashSet` during a full scan
> (`types.rs:1099,1120–1124`). Keep exact cursor identity checks for rows,
> cursors, continuation boundaries, and pinned heads; do not attribute a
> cross-page unseen-ID guarantee to the append path. Any stronger global
> uniqueness requirement needs a separately specified bounded proof.

## Material differences from revision 2

1. Existing rows and event payloads keep forward-compatible unknown-field
   tolerance in both legacy and v2 paths. Known-field validation remains
   strict; `detail: null` is valid; `kind` has no new grammar. Integrity
   verification must retain unknown fields in raw hash inputs.
2. The continuation binds stream identity by a presence- and length-checked
   digest of exact UTF-8 bytes rather than an ASCII/canonical-JSON assumption;
   no identifier grammar is added.
3. The source-basis claim about Rust append-time unseen-ID tracking is removed.
   Source shows ID generation during append and duplicate detection only in a
   full `verify()` scan.

These are the only changes in this overlay. The operator-approval boundary and
all other revision 2 budgets and choices remain as written. No source, tests,
schema, compiler, gate, or Git state was changed.
