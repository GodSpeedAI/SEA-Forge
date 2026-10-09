# Bounded ledger reader slice grant: ACK-boundary independent re-review

Date: 2026-10-09

## Verdict

**Approve the bounded ledger slice grant as clarified by the ACK-boundary
addendum.** The addendum resolves the prior finding by limiting a ledger
checkpoint to ledger-integrity evidence and requiring server/caller event
validation before ACK eligibility, frontier advancement, or frame emission.
This approves the design grant only; it does not approve future source, tests,
runtime behavior, server integration, or T09 settlement.

## Reviewed identities and authority

- Root grant: `c2-bounded-reader-ledger-slice-root-grant-oct09.md`, SHA-256
  `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`.
- ACK-boundary addendum: `c2-bounded-reader-ledger-slice-ack-boundary-addendum-oct09.md`,
  SHA-256 `1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`.
- Prior independent review: `c2-bounded-reader-ledger-slice-grant-independent-review-oct09.md`,
  SHA-256 `60062b3413251632bb9981916fb90f99b2c456118cbbf0570a18e20fb929b8e8`.
- Governing C2 supplement: `.agents/specs/casework-live-cursor-v4-spec.yaml`,
  SHA-256 `8ccf30be4131de57e6118bdbdd9f272c20edfd47bf0915f0fe74c7c5edfe3b11`;
  parent: `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml`,
  SHA-256 `39739d09d5af3e116494f484db7e80a86a2bfa1a51c9d7c0a0a9bd4a646e7002`.
- Reader basis: revision 2 proposal and review, revision 3 correction and
  review, revision 4 correction and review, approved bounded-reader policy
  receipt, and Oct. 9 registration/input-accounting decision and independent
  review. Operator approvals are recommendation/policy approvals, not exact
  grant-hash approval.

## Resolution of prior finding

The previous HOLD identified that the grant called a row “fully validated”
while only listing ledger-level checks, leaving C2 `sfwp_event` payload
validation ambiguous at the ACK boundary. The addendum now defines that term
as ledger-integrity validation only and calls the returned position a
candidate checkpoint, not proof of event validity or ACK eligibility.

Before ACKing a prefix/global frontier or emitting frames, the server/caller
must validate every `sfwp_event` row in that candidate prefix, including rows
later removed by cursor or event filters. It requires string `kind`, present
`detail` (any JSON value, including null), and absent/null/string `case_id`
and `run_id`. It preserves unknown payload fields, adds no kind grammar, and
requires an invalid candidate prefix to fail the whole page without ACK or
frontier advancement. An undelivered lookahead row is never acknowledged and
cannot advance frontier or cursor-resolution state. These rules satisfy
`REQ-C2-RANGE-003` (spec lines 253-270) and align with the existing V2
acknowledged-prefix filter rule (`REQ-C2-RANGE-004`, lines 316-355).

This placement keeps event-shape validation server/caller-owned and does not
expand the authorized ledger source slice. The separate server integration
remains held; the addendum is an explicit downstream contract, not evidence
that current `get_range` implements it. Current `frame_from_entry` defaults
missing `kind` and `detail` (`crates/sea-forge-server/src/sfwp/events.rs:92-107`),
so an implementation must add the required validation before ACK/frame
eligibility. That is a future source-review requirement, not a defect in this
design grant.

## Remaining grant boundaries and material differences

The addendum changes only ACK semantics and validation ownership: “fully
validated” is narrowed to ledger integrity; the downstream server/caller must
validate event payloads for every event row in the candidate acknowledged
prefix, including filtered-out rows; and event validation failure rejects the
candidate page without ACK. It explicitly preserves the lookahead no-ACK
rule. It adds no code, test, dependency, DTO, token, route, schema, limit,
authority, error type, or source-slice expansion.

All other reviewed grant requirements remain consistent with approved policy
and source: registered `&LedgerStream` only; cooperative try-lock with
monotonic 50 ms deadline; registered absent/zero-byte proof for empty success;
2 MiB raw-row cap including LF; one shared 4 MiB budget charging all repeated,
auxiliary, forward, non-event, and lookahead reads; 500 forward-scanned
complete-row cap including non-events/lookahead; bounded allocation and
checked positions; immutable pin and validation of both original pin and
acknowledged predecessor before seek; distinct row/complete/limit outcomes;
and no unbounded helper reads or false EOF.

Hash semantics remain exact: raw-row SHA-256 including LF is internal
continuation-boundary evidence only; authoritative payload and entry checks
continue through `payload_hash(&entry.payload)` and typed `entry_hash(&entry)`.
Unknown top-level fields remain tolerated and ignored by typed
deserialization/reserialization; unknown payload fields remain hashed. The
withdrawn revision-3 raw-object entry-hash protocol is not reintroduced. No
MMR/root verification, full-ledger uniqueness, cursor ordering, or fabricated
origin claim is added.

The original proposal/policy distinctions remain: process-memory signer and
token/filter/DTO work are outside this slice; no arbitrary path or new
registration is introduced; byte caps do not claim RSS or I/O-latency bounds;
no per-page elapsed guarantee is added; and source implementation still must
follow the grant's focused RED, implementation, tests, required crate/check
gates, and later milestone verification sequence. Existing policy approvals
do not constitute implementation or readiness evidence.

Current source anchors checked for this re-review: ledger helpers/types/read,
lock, append and verifier at `crates/sea-forge-ledger/src/types.rs:46-61,
191-211,409-413,694-723,765-802,1005-1059,1092-1145`; registration at
`crates/sea-forge-server/src/sfwp/events.rs:43-45`; and current event projection
at `events.rs:92-107`. Graft was queried first for event-range, validation and
ACK/frontier anchors; it saved approximately 84,264 tokens in one retrieval
call. No source, tests, gates, compiler, Git, status, or debt state changed.
