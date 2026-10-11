# C2 reader recon provenance correction

The immutable recon was first written with native `apply_patch` as a new file.
Its first post-write SHA-256 was `b24936dc176b821624ee9a4b293e727a9ab49cbdae8038d9b22cd6fb8e71a751`.
After that hash was captured, a second native `apply_patch` changed only the
accepted-policy paragraph at lines 99-108 of the recon: it added the approved
repair-review citation and its final spec/ADR hashes, and changed the following
sentence to distinguish the original authoring result from the accepted
repair review. The current recon SHA-256 is
`594c4f3c9e50940d0e36d3890de30fb0e7a79ee234b58ccc2e41e0f51e42c7e1`.

The first-write preimage was not separately saved as a file or artifact. Its
exact original text remains recoverable from the actual first `apply_patch`
ADD payload in the conversation record; it is reproduced below from that
payload. This is a transcription of the captured patch input, not a claim that
the current recon itself was never edited. The existing recon is now frozen.

## First-write text from the native ADD payload

```markdown
# C2 continuation-signing and reader source recon

Read-only bounded recon after accepted C2 V4 spec/ADR. No source, schema, test,
compiler, gate, Git, or status operation was run. Root retains architectural
placement and implementation decisions.

## Existing request, dispatch, DTO, and stream path

- `crates/sea-forge-server/src/lib.rs:643-781`: `Request` is the flat
  `#[serde(tag = "verb")]` NDJSON enum. Existing `EventsGetRange` accepts
  optional `from_cursor`, `to_cursor`, and `limit`; it has no V2 selector or
  continuation field yet. Requests are deserialized at `:1352`; subscribe and
  unsubscribe are connection-scoped at `:1365-1378`; range goes through the
  shared dispatch at `:2349-2361`.
- `crates/sea-forge-server/src/sfwp/events.rs:18-33`: existing response DTO is
  `EventFrame`. `:169-208` implements `get_range` by calling
  `ledger.read_entries()`, resolving cursors against the complete entry vector,
  filtering, then returning `Vec<EventFrame>`. Current wire response is
  `{"events": [...]}` (`lib.rs:2353-2355`). There is no existing v2 page DTO.
- `crates/sea-forge-server/src/bin/gen_sfwp_schema.rs:59-75` emits `EventFrame`
  and other SFWP DTO schemas; `sfwp/mod.rs:345-383` lists those schemas for
  `system.get_schema`. The `Request` enum itself is not in that generated
  schema list. A future V2 page DTO therefore needs its own DTO schema entry,
  generator type, generated schema, and generated client contract. Preserve
  `EventFrame` and the legacy vector response.
- `ServerState::new` opens one event ledger with
  `sfwp::events::open_events_ledger(&config.root)` (`lib.rs:151-166`); that
  helper calls `LedgerStream::open(root, "events", "sea-forge-server")`
  (`sfwp/events.rs:43-45`). The state stores it as `Arc<LedgerStream>`
  (`lib.rs:107-111,185-200`). `events_get_range` and `events_replay_after`
  clone it and use `spawn_blocking` (`lib.rs:237-264`). The live path subscribes
  to `event_bus`, replays from the ledger, then forwards broadcast frames
  (`lib.rs:1996-2072`).
- `LedgerStream::read_entries` (`crates/sea-forge-ledger/src/types.rs:785-802`)
  reads every line with `BufRead::lines()` and materializes every parsed row
  into a `Vec`; it has no raw-row or page-input cap. Its exclusive `File::lock`
  helper is private (`types.rs:694-724`); no public registered-stream registry
  or bounded page-reader API was found. The current open/read path also treats
  a missing entries file as an empty vector, so it does not prove the accepted
  verified-empty predicate by itself. A bounded reader integration will need
  the approved cooperative-lock, registered-stream, and incremental-read
  treatment; this is a distinct concern from the signing primitive.

## Existing signing, entropy, zeroization, and process state

- `sea-forge-ledger::signing` publicly re-exports `ed25519_dalek::{SigningKey,
  VerifyingKey}` (`crates/sea-forge-ledger/src/signing.rs:1-5`) and exports
  `sign_bytes` (`:63-66`) and `verify_signature` (`:68-83`). The signature
  representation is `ed25519:` plus Base64; for an Ed25519 signature it is
  96 ASCII bytes. `verify_signature` verifies exact supplied bytes but does
  not itself impose the C2 token/payload limits or fixed encoded-length check.
- The server already directly depends on `getrandom` and `zeroize`
  (`crates/sea-forge-server/Cargo.toml:13,37`).
  `transcript_seal.rs:30-35` uses `getrandom::fill` and maps failure to a
  fail-closed `ForgeError`; this function has no injectable entropy seam.
  `Zeroizing` is already used in server modules (`delegation.rs`,
  `agent_probe.rs`). There is no existing process-memory continuation signer.
- `ServerState` has no signing-key field (`lib.rs:67-112`). `ServerState::new`
  is synchronous and returns `Result<Self, ForgeError>` (`:151-200`), so the
  accepted startup-failure policy can be exercised at this boundary when
  runtime wiring is later granted. Do not persist or log the fresh key.

## Smallest signing-primitive TDD unit

An isolated private continuation-signing helper can be reviewed and tested
without wiring `ServerState`, changing `Request`, or defining a public DTO:

1. Generate a 32-byte seed through `getrandom::fill`, hold it in existing
   `Zeroizing<[u8; 32]>`, construct `SigningKey::from_bytes`, and return only
   the key. A private `*_with(fill)` helper taking a fill closure gives a
   deterministic entropy-error test; production passes `getrandom::fill`,
   while a test closure returns an injected error. No new dependency or global
   mutable RNG hook is needed.
2. Test that a fixed test seed produces a `sign_bytes` token signature and that
   `verify_signature` accepts the exact payload bytes, rejects changed bytes,
   wrong key, bad prefix/base64, and rejects non-96-byte signature text before
   calling the existing decoder. Keep token/payload byte-cap checks before
   splitting or allocating decoded state, as required by REQ-C2-RANGE-004.
3. Keep any token envelope/field parsing outside this minimal primitive until
   its exact internal codec is assigned. No envelope codec currently exists;
   the reusable, established pieces are Ed25519 signing/verification and the
   `ed25519:` signature encoding. The primitive must sign/verify exact payload
   bytes, not parse-and-reserialize them.

This is a scoped source/TDD suggestion, not an implementation or placement
grant. The larger reader still needs raw-row/page caps, locked pinning, and
registered-empty proof. No public shape or runtime wiring should be included
in the signing-primitive unit.

## Accepted policy anchors

- `.agents/specs/casework-live-cursor-v4-spec.yaml:278-299` (`REQ-C2-RANGE-004`)
  requires fresh process-memory Ed25519 at startup, fail-closed entropy,
  immediate local-seed zeroization, signature-before-payload parsing, exact
  96-byte signature representation, and token/payload caps.
- The same spec `:303-343` (`REQ-C2-RANGE-005`) requires presence-aware,
  field-specific domain-separated SHA-256 filter digests and acknowledged-only
  resolved filters.
- `c2-bounded-reader-additional-policy-operator-approval-oct08.md:5-11`
  records approval for a fresh memory-only startup key and verified-empty
  behavior. `casework-live-cursor-v4-normative-spec-adr-result-oct08.md` records
  the authored spec and ADR. The earlier
  `c2-bounded-reader-normative-independent-review-oct08.md` is a rejected
  pre-repair review; do not treat it as current authority.

No implementation grant, source placement, or runtime-readiness conclusion is
claimed by this recon.
```
