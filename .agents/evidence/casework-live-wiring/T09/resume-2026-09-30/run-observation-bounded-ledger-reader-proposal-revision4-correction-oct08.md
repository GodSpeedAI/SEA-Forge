# Bounded ledger reader proposal: revision 4 correction overlay

Date: 2026-10-08. This adds a narrow correction to revision 3; it does not
replace either prior proposal or authorize implementation.

Base proposal: `run-observation-bounded-ledger-reader-design-proposal-revision2-oct08.md`
(SHA-256 `4ff005dd073c186f159cf1e1f47432fb0e119106d105cd0b8b547ca04aa71227`).
Revision 3 overlay: `run-observation-bounded-ledger-reader-proposal-revision3-correction-oct08.md`
(SHA-256 `bfe581e9f5eb372b372d2a515c90d92ac5c8a932ccee2d39c77e295ef6f76f09`).
Known-field compatibility clarification:
`c2-bounded-reader-known-field-compatibility-clarification-oct08.md`
(SHA-256 `e61df326baab37703e179e4c24d6a4a2d2a1535d0505d3f25aa54cb071e0f0f0`).

## Replacement for revision 3, bounded row validation and hash explanation

Replace revision 3 lines 54–67 with:

> Preserve existing Serde tolerance for unknown top-level `LedgerEntry` fields
> and unknown stored-event payload fields. Parse each bounded raw row into the
> existing typed `LedgerEntry`; validate its known required fields and types,
> but do not add `deny_unknown_fields` or a raw-row hash protocol. Recompute
> `payload_hash` with `payload_hash(&entry.payload)` and `entry_hash` with
> `entry_hash(&entry)`. The existing helper serializes the typed entry,
> removes `entry_hash`, canonicalizes that value, and applies the existing
> domain-separated hash. Unknown top-level row fields are therefore accepted
> and ignored by deserialization/reserialization, matching the established
> writer and verifier. Unknown fields inside `payload` remain in its
> `serde_json::Value` and are covered by both the payload hash and the typed
> entry hash. Continue validating known fields, ordinal/linkage, exact cursor,
> and both hashes before filtering. The separate private continuation type
> may remain strict about unknown fields.
>
> For `record_kind == "sfwp_event"`, preserve revision 3's event validation:
> require string `kind` and a present `detail` of any JSON value (including
> null); `case_id` and `run_id` may be absent/null or strings. Do not reject
> extra payload fields or add a new kind grammar. They remain in the payload
> value used by the existing hash helpers even though the projected frame
> omits them.

Replace revision 3 lines 78–89 with:

> `LedgerEntry` (`crates/sea-forge-ledger/src/types.rs:191–211`) and
> `StoredEvent` (`crates/sea-forge-server/src/sfwp/events.rs:47–57`) have no
> `deny_unknown_fields`, so their established deserializers accept unknown
> fields. After that deserialization, `entry_hash` serializes the typed
> `LedgerEntry`, removes `entry_hash`, canonicalizes it, and hashes it with the
> existing domain (`types.rs:53–61`). `payload_hash` canonicalizes the complete
> `Value` (`types.rs:46–49`). The writer and verifier use those same helpers
> (`types.rs:1056–1058,1125–1132`). Consequently, an unknown top-level row
> field is tolerated but not retained by the typed entry hash preimage;
> unknown payload members are retained and hashed. The bounded reader should
> use these existing typed helpers. The proposed raw-row-object hash and
> fallback protocol are withdrawn; no separate approval question is needed
> for that avoided change. All other pending revision 2/3 approvals remain.

## Add resolved filter ordinals to signed continuation state

In addition to the exact presence/length/UTF-8 filter bindings already
specified, the signed continuation must carry nullable canonical decimal
`resolved_from_ordinal` and `resolved_to_ordinal`. A null requested filter is
distinguished from a present-but-not-yet-discovered cursor by its signed filter
presence binding. Resolve a present cursor only when its exact row is within
the acknowledged validated prefix; persist the real `append_ordinal` there.
Do not resolve from a lookahead row that the page leaves unacknowledged. On
resume, continue discovering unresolved filters in newly validated rows and
apply the exclusive/inclusive filter using the signed resolved ordinals. A
real ordinal zero is encoded as `"0"`, never null or synthesized. Require
resolved ordinals to be checked canonical `u64` values no later than the
acknowledged frontier and consistent with the exact filter cursor when found.
Completion at the pinned head with any requested cursor still unresolved is
typed `input_error`; a verified-empty stream with a present `from_cursor` or
`to_cursor` likewise cannot return successful complete-empty. These fields
fit the existing signed-token/payload limits; no cache, codec, limit, or
endpoint change is proposed.

## Corrected material-difference summary

Replace revision 3 material-difference item 1 with: unknown top-level ledger
fields remain tolerated but are ignored by the existing typed-entry hash
protocol; unknown payload fields remain in `Value` and are hashed. Known-field
validation remains strict, explicit-null `detail` is valid, and `kind` has no
new grammar. The raw-row hash proposal is withdrawn.

Add: continuations retain resolved `from_cursor`/`to_cursor` ordinals from the
acknowledged validated prefix, so paging does not need to rescan an unbounded
prefix to recover a previously discovered filter boundary. Exact cursor
bindings, append order, no-lookahead-ack rule, and existing token limits remain.

## Source basis and limits

The relevant implementation is `crates/sea-forge-ledger/src/types.rs:46–61`
for the two hash helpers, `:191–211` for the typed row, `:777–778` for row
deserialization, `:1056–1058` for append hashes, and `:1125–1132` for verify
hashes. The stored event type is at
`crates/sea-forge-server/src/sfwp/events.rs:47–57`; its payload is a
`serde_json::Value`. These source facts correct revision 3's claim that the
existing helper cannot preserve the current hash protocol. Unknown-field
tolerance is not a promise to hash unknown top-level members.

This overlay changes only the revision 3 hash paragraphs/material summary and
adds the resolved-ordinal continuation clause. Revision 2/3's other budgets,
typed error boundaries, compatibility rules, operator-approval status, and
proposal-only status remain unchanged. No source, tests, schema, compiler,
gate, or Git state was changed.
