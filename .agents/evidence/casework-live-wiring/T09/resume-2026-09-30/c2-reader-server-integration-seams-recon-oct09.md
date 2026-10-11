# C2 bounded-reader server integration seams recon

Read-only source inventory, 2026-10-09. This identifies current seams against
the approved C2 supplement and Oct. 9 registration/input-accounting decisions.
It grants no implementation, schema, protocol, or runtime-release authority.

## Smallest server seams

- **Process startup/state:** `crates/sea-forge-server/src/lib.rs:67-113`
  defines `ServerState`, including the one `events_ledger` and `event_bus`.
  `ServerState::new` at `:150-201` calls the fixed
  `sfwp::events::open_events_ledger(&config.root)` at `:165` and puts its
  `Arc<LedgerStream>` in the state at `:199`. `run` constructs the state at
  `:1199`. This is the existing registered-stream/startup seam; no registry or
  caller-selected path exists. `conformance_sfwp.rs` supplies real socket/state
  fixtures and existing range coverage at `:491-522`; lifecycle coverage also
  calls `events_get_range` at `conformance_lifecycle.rs:468`.
- **Request and dispatch:** `lib.rs:774-781` defines flat
  `Request::EventsGetRange` with optional `from_cursor`, `to_cursor`, and
  `limit`. Dispatch at `:2349-2361` invokes `ServerState::events_get_range`.
  That method at `:236-251` clones the registered ledger and uses
  `spawn_blocking` to call `sfwp::events::get_range`. The approved supplement
  requires additive range-version selection while omitted version preserves
  the current vector response (`REQ-C2-RANGE-002`, spec `:242-252`); it also
  prohibits a new kernel verb (`REQ-C2-RANGE-001`, `:235-240`).
- **Current range implementation/response owner:**
  `crates/sea-forge-server/src/sfwp/events.rs:18-33` owns `EventFrame`, and
  `:43-45` opens the fixed `events` ledger with owner `sea-forge-server`.
  `get_range` at `:169-208` calls unbounded `LedgerStream::read_entries()`,
  resolves cursor ordinals over the materialized entries, filters event rows,
  and returns `Vec<EventFrame>`. Dispatch serializes this as the existing
  `{ "events": [...] }` response (`lib.rs:2353-2355`). There is no current
  page DTO or continuation token. Approved placement says the ledger should
  own bounded raw access, cooperative locking, and typed hash validation;
  server state owns the ephemeral signer and token/filter semantics, with
  separate V2 response DTOs (`c2-reader-registration-input-accounting-root-decisions-oct09.md`).
  The spec binds continuation issuance to a validated acknowledged prefix and
  says only the caller's acknowledged frontier can resume; lookahead,
  filtered count, partial rows, and failed pages do not advance it
  (`REQ-C2-RANGE-003/004`, spec `:262-300`).
- **Ledger seam:** `crates/sea-forge-ledger/src/types.rs:694-724` is the
  existing private file-lock path; `:785-802` is the full-file
  `read_entries` path. The bounded reader therefore needs a new ledger-facing
  raw-page seam rather than routing the approved C2 path through
  `read_entries`. Registration remains the `ServerState.events_ledger`
  handle, not an arbitrary path. One 4 MiB raw-input/page budget charges every
  actual read, including repeated pin and predecessor reads, non-events, and
  lookahead; row cap remains 2 MiB including LF, lock wait 50 ms, and complete
  response 1 MiB (approved spec `REQ-C2-RANGE-004`, `:274-300`, and Oct. 9
  registration/accounting decision).

## Signing and dependency inventory

- `sea-forge-server` already directly depends on `getrandom` and `zeroize`
  (`crates/sea-forge-server/Cargo.toml:13,37`). Existing server use of
  `getrandom::fill` is `src/transcript_seal.rs:30-35`; `Zeroizing` is already
  used in `src/delegation.rs` and `src/agent_probe.rs`.
- `sea-forge-ledger::signing` re-exports `ed25519_dalek::SigningKey` and
  `VerifyingKey` (`crates/sea-forge-ledger/src/signing.rs:5`), and exposes
  `sign_bytes` and `verify_signature` at `:63-83`. The representation is
  `ed25519:` plus Base64. C2 requires fixed 96-byte ASCII signature
  representation and input caps before variable decoding; the existing
  verifier decodes before it checks the decoded signature length, so that
  helper alone does not implement the C2 bounded parsing contract.
- There is currently no continuation signer/key field in `ServerState`
  (`lib.rs:67-113`). The approved spec requires a fresh process-memory key at
  startup, fail-closed on entropy failure, and immediate zeroization of the
  seed (`REQ-C2-RANGE-004`, `:274-282`). This describes a future state/startup
  seam, not permission to add it in this recon.

## DTO/schema surface and named gates

- `EventFrame` is an existing `JsonSchema` type; `sfwp::SCHEMA_TYPES` in
  `crates/sea-forge-server/src/sfwp/mod.rs:345-458` is the list shared with
  `gen_sfwp_schema`. `crates/sea-forge-server/src/bin/gen_sfwp_schema.rs`
  contains the explicit type imports and generated schema list. The Request
  enum is not in that generated DTO list. Any separately typed V2 page response
  would touch its DTO definition, generator entry, `SCHEMA_TYPES`, committed
  JSON schema, and generated TS/AJV contract projections; no schema should be
  hand-edited.
- `tests/conformance_sfwp.rs:725-802` regenerates schemas into a temporary
  directory and checks committed files plus exact agreement between generator
  output and `SCHEMA_TYPES`. `just workbench-contracts-generate` is the
  generator recipe (`justfile:637-641`); `just workbench-contracts-gate`
  checks generated Workbench projection drift (`justfile:650+`). Applicable
  Rust checks remain the repository's named `just check`/`just test`; this
  recon ran none of them.

## Existing event-shape mismatch to account for

The C2 contract requires `sfwp_event.kind` to be a string and `detail` to be
present (null is allowed); `case_id` and `run_id` may be absent/null or strings,
and unknown payload fields remain part of the typed-hash input
(`REQ-C2-RANGE-003`, spec `:260-270`). Current `frame_from_entry` in
`sfwp/events.rs:96-111` uses `unwrap_or_default()` for a missing or non-string
`kind`, silently producing `""`; it substitutes JSON null when `detail` is
missing, making missing detail indistinguishable from permitted null. For
`case_id`/`run_id`, `.as_str().map(...)` turns absent, null, and wrong-type
values all into `None`. These are current legacy projection defaults, not
validation evidence for the C2 reader. The bounded path must validate known
field presence/types while preserving the approved typed hash over the full
payload Value and tolerating unknown payload fields. Any adjustment to legacy
behavior or DTO defaults remains an architectural decision outside this
inventory.

## Boundaries and unresolved choices

The approved registration, lock wait, total raw-input accounting, limits,
signature contract, and filter-digest semantics are already policy. The exact
internal token codec/field layout, how the new reader is exposed across the
ledger/server module boundary, and precise placement of V2 response fields
remain implementation design choices. The existing EventFrame/vector path is
the compatibility surface. The active ledger fixture work owns
`crates/sea-forge-ledger/src/types.rs` and the CW37 preservation fixtures; this
recon makes no edits to or claims about that moving work. No compile, tests,
gates, formatter, or Git operation was run.
