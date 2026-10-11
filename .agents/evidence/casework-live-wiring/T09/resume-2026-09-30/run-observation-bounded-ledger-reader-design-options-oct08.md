# Bounded ledger reader: concrete design options

**Status:** read-only architecture recommendation for the existing Rust ledger
and SFWP `events.get_range` only. No code, wire schema, or Git state changed;
no tests or gates were run. This does not approve implementation.

## 1. Continuation integrity

There is no existing HMAC/token-authentication facility in the indexed crates
(`graft` exhaustive searches for `Hmac` and `Mac` found no hits). The ledger
does have Ed25519 signing and verification in
`crates/sea-forge-ledger/src/signing.rs:1–65`; the signing key type and
`sign_bytes`/`verify_signature` are exported there. Checkpoints at
`types.rs:220–235, 853–920` sign an ordinal range, MMR root, and checkpoint
chain. They do not include a byte offset, row cursor, or page continuation.
`prove_entry` at `types.rs:927+` obtains entries with the full-ledger reader,
so it is not a bounded substitute. The SFWP server has no checkpoint signer
field in `ServerState` (`crates/sea-forge-server/src/lib.rs:70–120`).

**Recommended smallest stateless option:** add a private, process-lifetime
continuation signer to `ServerState`. Generate a fresh 32-byte seed at startup
with the server’s existing `getrandom` dependency (`crates/sea-forge-server/Cargo.toml:13`),
construct the existing re-exported `sea_forge_ledger::signing::SigningKey`,
and sign a fixed, versioned continuation payload with the already available
Ed25519 implementation. Verify with the matching public key. This adds no
dependency and does not reuse a persisted ledger/checkpoint signing key. Use a
domain separator such as `sea-forge/events-range-continuation/v1` and an
unambiguous fixed-field encoding. Bind at least: token version, exact ledger
ID, pinned head cursor/ordinal/entry hash/end offset, last validated cursor/
ordinal/entry hash/next offset, and raw preceding-row checksum. On resume,
verify the signature first, require the token’s ledger ID and pinned head to
match the request, reread and verify the complete row immediately before the
offset against its signed checksum and identity, then continue with checked
successive ordinals and hash links. Do not accept a caller-supplied offset or
checksum without a valid server signature. The signature proves the server
issued the continuation after validating its prefix; it does not authorize the
caller, change event access policy, or replace row validation after the
continuation.

The existing Ed25519 crate is already a workspace dependency through the
ledger, and `getrandom` is already a direct server dependency. The server can
use the ledger’s exported signing type/functions without adding a direct
crypto dependency. Key generation and token signing are new behavior and need
explicit approval, but do not require a persisted secret or new dependency.
Process restart discards the key: reject old tokens as stale/unavailable, and
let a v2 client restart at origin and page forward. This remains bounded per
request; legacy mode still refuses when it cannot prove coverage through the
pinned head. If restart recovery must preserve outstanding tokens instead,
that requires a durable key lifecycle and separate approval. A bounded
server-side token cache is a possible alternative but adds eviction, capacity,
and restart semantics; it is larger than the stateless signer.

The current C2 wording requires continuation binding and predecessor
verification but does not specify how the server proves that a continuation
was issued by a prior successful scan. The signature option fills that
specific trust gap. If signed tokens are not approved, the spec must select
another trusted source of verified-prefix state; re-reading the whole prefix
on each page or trusting caller fields is not an acceptable fallback.

## 2. Pinned head and the 50 ms lock bound

`LedgerStream::with_exclusive_lock` at `crates/sea-forge-ledger/src/types.rs:702–724`
uses blocking `File::lock()`. Appends use that stream lock before
`append_under_lock` (`types.rs:1005+`), and the server already demonstrates
nonblocking `File::try_lock()` handling in `crates/sea-forge-server/src/lib.rs:1097–1113,1158–1174`.

Add a synchronous ledger helper that retries `try_lock()` only until a
monotonic 50 ms deadline, returning unavailable on `WouldBlock` expiry. While
holding the lock, open the entries file and snapshot the last complete row,
its exact identity/hash, and byte end offset; bound reverse-tail scanning by
the raw-row limit below. Release the lock before page scanning. Cooperative
writers append under the same lock and never rewrite the existing prefix, so
the pinned byte end bounds the page even if later rows append. Each page must
stop at the token’s pinned end and verify the exact head pair there; it must
not chase a moving tail. A malformed or incomplete tail fails closed. Do not
hold the exclusive writer lock while scanning up to 500 rows. This uses the
existing synchronous kernel boundary and no new lock crate.

This assumes the existing supported-writer contract: appends cooperate with
the ledger lock and prior rows are immutable. The lock cannot fence an
out-of-tree writer that edits old bytes. Such mutation must be detected for
rows the page reads; this design does not claim global detection of edits to
already-scanned bytes. If the product requires that stronger guarantee, it
needs a durable authenticated frontier/index and a separate architecture
decision.

## 3. Bound input allocation separately from response size

The current C2 `event_page` bounds serialized output, but `read_entries()` at
`types.rs:785–802` calls `BufRead::lines()` and allocates each full row; output
size does not bound input allocation. Define explicit new limits for raw
ledger input, separate from the existing 1 MiB serialized-response cap. A
conservative starting proposal is **1 MiB per raw JSONL row and 4 MiB total
raw bytes parsed per page** (including non-event rows). Reject an over-limit
row/page as unavailable before growing the row buffer beyond its cap. The
reader should consume through a bounded byte buffer, not `read_line()` and
then check length. The reverse-tail head sampler uses the same per-row bound.

These are proposed values, not values already specified by C2. They tighten
which histories are readable and therefore require normative/operator
approval. If valid serialized event rows at the 1 MiB retained-event ceiling
need more envelope space, choose and document a larger per-row raw limit
before implementation; do not silently treat response bytes as an allocation
cap. The 4 MiB aggregate input budget can stop earlier than 500 rows; in that
case issue a signed continuation at the last fully validated row and report
incomplete progress for v2, or fail the legacy vector atomically.

## 4. Failures and v2 DTO boundary

Keep the existing no-version request and response unchanged. Keep
`EventsGetRange` and its existing `from_cursor`, `to_cursor`, and `limit`
fields in `crates/sea-forge-server/src/lib.rs:748–790`; add only the approved
defaulted optional `range_version`. Do not add a verb. Return a distinct v2
page DTO and v2 event DTO rather than changing `EventFrame`, which is also
used by live subscriptions and `replay_after` (`lib.rs:34, 113, 166, 214;
events.rs:19–32`). Add any new public DTO consistently to
`sfwp::SCHEMA_TYPES` (`sfwp/mod.rs:345–410`) and
`gen_sfwp_schema.rs:35,55–66`; retain the old EventFrame schema and generated
contract. The current method is already catalogued at `sfwp/mod.rs:108–136`.

Use a method-local typed error with at least `UnknownCursor` and
`Unavailable(reason-class)` cases. Preserve unknown cursor behavior as
`input_error` (`events.rs:110–130`, conformance oracle
`tests/conformance_sfwp.rs:1160–1185`). Map bounded-integrity, lock-timeout,
invalid-continuation, and resource-limit failures to the approved
`frontier_rebuilding`/unavailable error class in the existing
`{"error":...,"error_class":...}` envelope. Avoid adding a public Core
`ForgeError` variant merely to distinguish this one method; current
`ForgeError` does not define a frontier-unavailable case
(`crates/sea-forge-core/src/errors.rs:4–27,33–66`). `ForgeError::Plan` can
carry a class string, but using that semantically unrelated variant would
couple a ledger read contract to planner errors. A local error mapped at the
SFWP boundary is narrower. Any HTTP 503 mapping belongs to the higher-level
C2 owner; this recon covers the NDJSON SFWP boundary only.

## Decisions still requiring approval

1. Adopt signed, process-lifetime Ed25519 continuations, including stale-token
   behavior after restart; otherwise name the trusted frontier source.
2. Add explicit raw-row and aggregate scan-byte limits, choosing final values
   based on the supported 1 MiB event payload envelope.
3. Approve the short exclusive-lock head snapshot and cooperative append-only
   assumption, or require a stronger durable frontier.
4. Keep v2 response/frame types separate and map typed unavailability at the
   SFWP boundary without changing the Core error enum.

No source, schema, generated file, or tests were edited or run. Graft was used
before source inspection; this turn’s three billed calls saved approximately
56,225 tokens (less than $0.05).
