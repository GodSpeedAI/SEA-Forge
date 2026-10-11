# Bounded ledger reader revision 3: independent overlay review

**Disposition: HOLD.** The overlay corrects unknown-field tolerance, token stream identity, and the Rust unseen-ID claim, but its proposed raw-object `entry_hash` computation changes the existing hash contract and contradicts the root's explicit hash clarification. No source, tests, compiler, schema, or Git state changed.

## Original review assignment

> Independent re-review corrected overlay revision3 bfe581e9f5eb372b372d2a515c90d92ac5c8a932ccee2d39c77e295ef6f76f09 readwith fullrev2/originaltask/rootknownfieldclarification+priornotes. SameFULL builderinstructions asprevious + correction exactly preserveunknown tolerance existingLedgerEntry/StoredEvent whilevalidating requiredknown fields/types/hash, strictunknowns NEWprivatecontinuation only, exactstreamID bytebinding vs ASCII/NFCassumption, remove inaccurateRust unseenIDledger sourceclaim. Verify known hash semantics: payload Value includesunknownpayload fields in payloadhash; current entryhash uses typedLedgerEntry serialization so unknownTOPLEVEL fields tolerated/ignored perexistingprotocol (do not silently redefine hash). Root accepts preservation of actualhelper ratherthan incorrect implication unknowntop fields were hashed. Check overlaydiscloses thismaterialclarification. Approveproposalonly ifallheldfindingsclosed, exactsourceevidence and fullmaterialdiff. Native new concise report; no compile/source/Git.

## Reviewed identities

- Revision 3 overlay: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-bounded-ledger-reader-proposal-revision3-correction-oct08.md`, SHA-256 `bfe581e9f5eb372b372d2a515c90d92ac5c8a932ccee2d39c77e295ef6f76f09`.
- Base revision 2: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-bounded-ledger-reader-design-proposal-revision2-oct08.md`, SHA-256 `4ff005dd073c186f159cf1e1f47432fb0e119106d105cd0b8b547ca04aa71227`.
- Revision 2 independent HOLD: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-bounded-ledger-reader-design-proposal-revision2-independent-review-oct08.md`, SHA-256 `1641b2ca1a26db550d86cd6c63b2897f4f0323d9ed6aa12ffc24c4e2484956d7`.
- Known-field clarification: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-bounded-reader-known-field-compatibility-clarification-oct08.md`, SHA-256 `e61df326baab37703e179e4c24d6a4a2d2a1535d0505d3f25aa54cb071e0f0f0`.
- Governing specification: `.agents/specs/casework-live-cursor-v4-spec.yaml`, SHA-256 `d509ea469408e03c1cb62cf10ab463036e161ad13bf99a7b177d7ccf3406be31`.

The governing C2 page/continuation clauses are `REQ-C2-RANGE-001..003` and `REQ-C2-FRONTIER-001..002` (spec lines 203-252). Root's task message separately clarified the existing helper boundary: preserve `entry_hash(&typed_entry)` and `payload_hash(&complete_payload_value)`; do not add raw-hash alternate/fallback semantics.

## Findings

The overlay's other stated changes align with the known-field clarification: existing `LedgerEntry` and stored-event payload reads retain unknown-field tolerance; known required fields and types remain checked; explicit null event detail is valid; the new private continuation type remains strict; exact stream identity is bound by presence/length-checked exact UTF-8 digest without a new ID grammar; and the erroneous claim that append maintains an exact unseen-ID ledger is removed. The overlay leaves revision 2's other token, page, lock, genesis, and approval choices in place.

**Blocking hash-semantic mismatch:** the replacement paragraph directs the reader to retain the raw row object and compute `entry_hash` from that raw object after removing `entry_hash`, including unknown top-level fields. The existing helper does something different: `entry_hash(&LedgerEntry)` serializes the typed struct, removes `entry_hash`, canonicalizes, and hashes (`crates/sea-forge-ledger/src/types.rs:53-61`). `read_entries()` deserializes each JSON row into `LedgerEntry` with ordinary Serde (`types.rs:785-802`); the type has no `deny_unknown_fields` (`types.rs:191-216`). Therefore unknown top-level fields are tolerated by deserialization but are not present in the typed helper's entry-hash preimage. The writer also computes `entry_hash(&entry)` over the typed value (`types.rs:1050-1058`). Recomputing against the raw object changes the preimage for a tolerated row containing an unknown top-level field and can reject a row whose hash is valid under the existing helper. It silently introduces the hash-protocol change the root clarification expressly disallows.

The overlay correctly says to compute `payload_hash` over the complete `payload` `Value`; `payload_hash` hashes the canonical `Value` itself (`types.rs:46-50`), so unknown fields nested inside `payload` remain included. The correction must keep that behavior while using the existing typed `entry_hash` helper on the typed row. Retaining the raw object is unnecessary for the established helper semantics unless a separately approved protocol change is proposed; no such change is authorized here.

## Material differences and disposition

Revision 3 correctly changes revision 2's strict top-level/event-field rejection back to compatibility tolerance and adds exact-byte stream binding. Its raw-object `entry_hash` paragraph, however, is a new hash-preimage behavior, not merely an implementation detail for preserving unknown fields. It is inconsistent with the actual writer, typed reader, known-field clarification, and current protocol. **Do not accept the overlay or grant source work until that paragraph is corrected.** No implementation, operator approval, or runtime proof is granted by this review.

Graft was queried first for the typed ledger read/hash boundary; the cited code was then checked directly. Graft saved approximately 36,367 tokens ($0.03) in one retrieval call. No tests or gates were run.
