# Bounded ledger reader: synchronous slice design and grant

Root architecture decision, 2026-10-09. Implementation is **HELD until an
independent critic accepts this grant**. Private Next is locally accepted and
committed as `608309fab24973b9b231a666c1ce6bc9c7669d6b`. Exact permission to
publish that commit is pending; this external-write restriction does not
invalidate local verification or forbid separately reviewed local work.

## Authority and scope

Read applicable AGENTS, the approved C2 supplement and parent spec, the bounded
reader revision-2 proposal with revision-3/revision-4 overlays and reviews,
operator policy receipts, and the Oct. 9 registration/input-accounting
decision and its independent/normative traceability reviews. Those documents
govern; this grant cannot change their limits or policies.

The source slice is only `crates/sea-forge-ledger/src/types.rs`, including its
existing unit-test module. Its existing public `types` module makes root
reexports unnecessary. A new immutable builder record is allowed in this
evidence directory. Use native apply_patch for persistent edits; read each
affected file and nearby implementation/tests first. No dependency, persisted
schema, core error variant, server route, public wire DTO, token encoding,
async primitive, generated file, unrelated formatting, or Git mutation.
Root owns the sole compiler; actual host RAM/swap/process guard and six
actual captures/root comparisons apply to every gate.

## Two implementation choices inside the approved boundary

The 500-row limit counts forward-scanned complete global rows, including
non-events and undelivered lookahead. Auxiliary pin/predecessor checks do not
advance that scanned coverage and do not consume an additional forward-row
slot. Every actual byte read in either phase, including repeated reads and
boundary probes, consumes the SAME 4 MiB per-request budget. No auxiliary
byte allowance or reset is permitted. This makes explicit the proposal's
distinction between pin/predecessor validation and forward scanning.

Use the existing SHA-256 implementation for an internal fixed-size checksum
of the exact raw row bytes INCLUDING LF. This checksum binds continuation
bytes/boundaries only. It does not replace or modify authoritative
`payload_hash(&entry.payload)` or `entry_hash(&entry)`. Revision 4 withdraws
recomputing entry hashes from raw JSON objects; its separate exact raw-row
checksum requirement remains. No persisted ledger identity/hash protocol,
raw-object canonicalization fallback, or stable wire checksum encoding is
introduced by this slice. Server token encoding remains separately held.

## Small additive Rust interface

Add a session obtained ONLY from `&LedgerStream`; no path/stream-ID parameter
and no caller-overridable limits. A fresh request pins the real tail. Resume
input carries BOTH the original pinned-head row position and last acknowledged
row position, never just a next offset or a newly sampled head. Positions
contain checked native u64 byte start/end, append ordinal, exact entry_ulid,
entry hash and raw checksum. Do not derive JSON serialization for these
internal Rust types: future server DTOs own canonical-decimal adapters and
signature/filter bindings. Caller-provided positions are not proofs until
the reader validates the actual rows; server token authentication is a
separate prerequisite at the future external boundary.

The session owns its file, immutable pin, private remaining-byte counter,
forward-row counter and validated position. It exposes the pin and a single
next-step operation with DISTINCT row, complete, and limit-reached outcomes;
never represent limit exhaustion as EOF. A returned row owns its typed entry
and the checkpoint AFTER exactly that fully validated row. Box the row arm if
needed to keep the enum compact. The server will choose the acknowledged
prefix; internal reads/lookahead must not silently advance a public ACK.

Acquire the cooperative stream lock using try_lock and a monotonic 50 ms
deadline. Under lock prove registered entries absent/zero-byte, or validate
the bounded real tail for fresh requests. Resume validates the ORIGINAL pin
and acknowledged predecessor bytes, cursor/ordinal/hash/checksum/boundaries
before seeking. Release the lock before forward scanning. Legitimate appends
after the pin neither move it nor become part of this page. Reject unreadable,
nonregular, malformed, whitespace and torn history; do not invent a symlink
ban. Only actual absent/zero-byte proof under lock permits empty completion.

Use fixed bounded reads/row accumulation. Charge before every read, bound
allocation/growth before use, cap each raw row at 2 MiB including LF, and
never call unbounded read_entries/read_last_entry/verify/lines/read_line.
Budget exhaustion may expose only prior fully validated progress; partial
rows are discarded and never acknowledged. A malformed/oversized row or
integrity failure is an error, not completion. Preserve unknown top-level
Serde tolerance and complete payload Value hashing. Validate registered
ledger/version/canonicalization/algorithm, successive checked ordinals from
zero/no predecessor (or validated resume predecessor), predecessor linkage,
typed payload/entry hashes and exact pinned-head identity on completion.
No cursor parsing/ordering, fabricated origin, MMR/root verification claim,
unbounded identity cache or full-ledger uniqueness claim is authorized.
Reuse existing ForgeError categories; server maps reader failures later.

## TDD and evidence

Builder first supplies focused tests and minimal compiling declarations if
needed. Freeze and report before a compiler grant. Root runs focused expected
RED; compilation/setup failure is not behavioral RED. After root accepts RED,
the builder may implement this same bounded slice. Never weaken existing tests.

Tests must prove origin/hash/linkage/constants; unknown top-level tolerance
versus hashed payload extensions; absent/zero versus torn/whitespace/nonregular
and I/O errors; lock timeout; exact row cap/cap-plus-one; total auxiliary plus
forward raw budget exact-cap/cap-plus-one and repeated-read charging; 500-row
forward cap including non-events/lookahead; immutable pin despite appends;
resume predecessor/head/checksum/offset corruption; no partial-row progress or
cap-as-completion; and native checked arithmetic boundaries. Prefer actual
files, cooperative locks and deterministic counters over sleeps/fake state.
If exact boundary vectors need a private byte-reader seam, ask root before
adding it; no mutable global hook, budget override or alternate implementation.

Actual required gates after implementation: focused ledger tests, full ledger
crate tests, `just crate-check sea-forge-ledger`, `just check`, and eventually
the governing milestone/CI/proof gates before broader completion. No source
approval without an independent critic reviewing this FULL grant, the exact
implementation and its own direct verification. Any rejection requires a
fresh builder for the findings, then re-review. Public reader integration,
legacy bounded fallback, tokens/filters/ACK DTOs and T09 settlement remain
separate work; this slice alone cannot claim them complete.
