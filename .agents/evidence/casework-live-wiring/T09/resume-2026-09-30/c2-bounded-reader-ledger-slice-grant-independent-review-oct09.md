# Bounded ledger reader slice grant: independent review

Date: 2026-10-09

## Verdict

**HOLD the source grant pending one normative boundary clarification.** The
grant correctly preserves the approved registration, lock, byte and row
budgets, continuation validation, and existing typed hash protocol. It does
not yet assign validation of known `sfwp_event` payload fields before a row
checkpoint can be treated as acknowledged progress. No source, tests, gates,
compiler, or Git state changed for this review.

## Reviewed identities and evidence

- Full grant reviewed: `c2-bounded-reader-ledger-slice-root-grant-oct09.md`,
  SHA-256 `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`.
- Governing supplement: `.agents/specs/casework-live-cursor-v4-spec.yaml`,
  SHA-256 `8ccf30be4131de57e6118bdbdd9f272c20edfd47bf0915f0fe74c7c5edfe3b11`;
  parent `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml`.
- Approved reader basis: revision 2 proposal and its independent HOLD;
  revision 3 correction and HOLD; revision 4 correction and proposal-only
  approval; bounded-reader policy approval; and the Oct. 9 registration/input
  accounting root decision, independent review, and normative traceability
  review. The operator approvals are recommendation/policy approvals, not
  approval of an exact proposal or grant hash.
- Source anchors: `crates/sea-forge-ledger/src/types.rs:46-61,191-211,409-413,
  694-723,765-802,1005-1059,1092-1145`; fixed server registration is
  `crates/sea-forge-server/src/sfwp/events.rs:43-45`.

## Findings

### Blocking: event-payload validation is not placed before acknowledgement

The normative `REQ-C2-RANGE-003` requires known required fields and types to be
validated. For `sfwp_event`, `kind` must be a string, `detail` must be present
(and may be null), and `case_id`/`run_id`, when non-null, must be strings
(`casework-live-cursor-v4-spec.yaml:258-270`). The grant instead defines the
checkpoint returned for each row as the position after that “fully validated
row” and says the server chooses the acknowledged prefix (grant lines 59-65).
Its validation list names ledger metadata, ordinal/linkage, payload hash, and
entry hash (lines 81-88), but never the normative `sfwp_event` field checks or
which layer must complete them.

This omission matters at the acknowledgement boundary: a payload can be a
valid hashed JSON `Value` and still violate the C2 event contract. The proposed
ledger-only interface could therefore return a checkpoint that a caller
mistakes for an acknowledgeable row before event validation. The current
`LedgerEntry.payload` is an unrestricted `Value` (`types.rs:191-211`), and the
existing generic ledger hash/verify paths validate hashes and chain structure,
not this event-specific shape (`types.rs:1092-1145`).

Clarify that the ledger checkpoint proves only ledger-row integrity, and that
for `record_kind == "sfwp_event"` the server/caller MUST validate the exact
REQ-C2-RANGE-003 payload fields before including the row in any acknowledged
prefix/frontier or emitting a frame. Alternatively, explicitly include that
validation in the slice and its tests. The first option keeps the source slice
bounded, but requires an explicit downstream contract; the current grant has
no server integration authorization to demonstrate it end-to-end.

## Approved boundaries that the grant preserves

- **Registration and authority:** source access is only via `&LedgerStream`,
  matching the fixed `ServerState.events_ledger` created by
  `open_events_ledger`. It adds no arbitrary path, registry, dependency,
  authority mechanism, server route, wire DTO, or kernel verb. Empty completion
  is limited to successful absent/zero-byte proof for the registered stream
  under lock. Nonregular/unreadable/corrupt data is rejected, with no invented
  symlink ban. This matches the Oct. 9 root decision and independent review.
- **Locking:** bounded `try_lock` plus monotonic 50 ms deadline, snapshot under
  lock and forward reads after release match the approved cooperative stream
  boundary. The existing writer lock is blocking (`types.rs:701-718`); the
  grant correctly prohibits relying on it for bounded reader acquisition.
- **Budgets:** every actual read, including repeated pin/predecessor reads,
  probes, non-events and lookahead, shares one 4 MiB page budget. The 500
  forward-scanned complete-row cap includes non-events and lookahead; auxiliary
  validation does not advance coverage or consume a forward slot. Row cap is
  2 MiB including LF. No cap reset, extra auxiliary allowance, fake budget
  override, or claim of RSS/I/O-latency guarantees is authorized. This is
  consistent with the approved policy and Oct. 9 accounting clarification.
- **Hashes and compatibility:** the internal checksum is SHA-256 over exact raw
  row bytes including LF and is limited to continuation-byte/boundary binding.
  It remains distinct from `payload_hash(&entry.payload)` and typed
  `entry_hash(&entry)`. This matches revision 4 and the current helpers
  (`types.rs:46-61`); unknown top-level fields remain tolerated by typed Serde,
  while unknown payload fields remain hashed. No raw-JSON entry-hash fallback,
  persisted hash protocol, MMR/root claim, or wire checksum encoding is added.
- **Resume/completion:** both the original pinned head and acknowledged
  predecessor are validated before seeking; appends after the immutable pin
  are excluded. Distinct row/complete/limit outcomes prevent limit exhaustion
  from becoming EOF. Partial rows, malformed data, and undelivered lookahead
  cannot advance progress. Checked native offsets/ordinals and no fabricated
  origin are explicit.
- **Scope/evidence:** no dependency, schema, core error, public wire protocol,
  async kernel primitive, generated file, or Git mutation is allowed. Exact
  boundary tests include auxiliary+forward byte totals, repeated reads,
  row/lookahead coverage, cap-plus-one, resume corruption, and checked
  arithmetic, without a mutable global test hook. The required TDD and gates
  are explicit; this slice makes no readiness or settlement claim.

## Material differences and limits

The grant turns proposal ambiguities into stricter implementation choices:
auxiliary reads are charged to the same 4 MiB budget; auxiliary rows do not
consume the 500 forward-row slots; and the raw checksum is explicitly internal
and includes LF. These are consistent with the approved total raw-input cap,
the continuation checksum requirement, and the distinction between pin checks
and forward coverage. The registered-only API and absence/zero-byte rule are
also consistent with the Oct. 9 clarification. No other material policy
expansion was found.

This is a design/grant review only. It does not inspect or approve future
implementation, test evidence, or runtime behavior. Graft was used before
source inspection to locate the ledger registration, lock, hash, reader, writer
and verifier anchors; it saved approximately 50,597 tokens in one retrieval
call.
