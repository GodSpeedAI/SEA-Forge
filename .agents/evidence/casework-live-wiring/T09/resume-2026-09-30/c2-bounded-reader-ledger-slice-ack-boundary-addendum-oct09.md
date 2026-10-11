# Bounded ledger reader slice: acknowledgement-boundary addendum

Date: 2026-10-09. This immutable clarification supplements the source grant and
its independent review; it does not replace either record or authorize source
implementation.

## Reviewed records and authority

- Root grant: `c2-bounded-reader-ledger-slice-root-grant-oct09.md`, SHA-256
  `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`.
- Independent review: `c2-bounded-reader-ledger-slice-grant-independent-review-oct09.md`,
  SHA-256
  `60062b3413251632bb9981916fb90f99b2c456118cbbf0570a18e20fb929b8e8`.
- Governing supplement: `.agents/specs/casework-live-cursor-v4-spec.yaml`,
  SHA-256
  `8ccf30be4131de57e6118bdbdd9f272c20edfd47bf0915f0fe74c7c5edfe3b11`;
  parent spec `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml`,
  SHA-256
  `39739d09d5af3e116494f484db7e80a86a2bfa1a51c9d7c0a0a9bd4a646e7002`.
- Normative anchor: `REQ-C2-RANGE-003`, supplement lines 253–270. It requires
  validation of every complete global row before filtering, names the known
  `sfwp_event` payload fields and their types, and prohibits a failed page,
  filtered count, or undelivered lookahead from advancing the acknowledged
  frontier.
- Grant anchors: lines 59–65 define the ledger checkpoint and acknowledged
  prefix; lines 79–88 define the ledger checks and defer server mapping.
- Review finding: “Blocking: event-payload validation is not placed before
  acknowledgement.” The review requests an explicit contract assigning the
  validation to the server/caller while keeping this source slice ledger-only.

## Required acknowledgement boundary

“Fully validated row” in the grant means **ledger-integrity validated only**:
the bounded reader has validated the typed ledger row, its registered stream
identity and protocol constants, ordinal and predecessor linkage, existing
payload and entry hashes, raw row boundary/checksum, and any applicable pin or
resume position. The returned position is a candidate checkpoint. It proves
neither C2 event validity nor eligibility for acknowledgement.

Before a caller includes any row in an acknowledged prefix/global frontier or
emits a frame, it MUST additionally apply `REQ-C2-RANGE-003` to every
`record_kind == "sfwp_event"` row in that prefix, including rows later removed
by `from_cursor`/`to_cursor` filtering or other event filtering:

- `kind` MUST be a string; no additional kind grammar is introduced.
- `detail` MUST be present and MAY contain any JSON value, including `null`.
- `case_id` and `run_id` MAY be absent or `null`; otherwise each MUST be a
  string.
- Unknown payload fields remain tolerated. No extra-field rejection is added.

If any event row in the candidate prefix fails those checks, the whole page
fails and advances no acknowledgement/frontier. Filtering cannot conceal an
invalid event row. An undelivered lookahead row is never acknowledged; it must
not advance the acknowledged prefix or any cursor-resolution state. This
requirement does not move event-shape validation into `sea-forge-ledger`:
validation remains server/caller-owned, after ledger integrity validation and
before ACK eligibility.

## Scope and status

This addendum grants no server integration, token/filter/DTO change, source or
test edit, compiler/gate execution, or runtime implementation. The source
grant remains **HELD** pending independent re-review of this clarification.
The ledger slice alone cannot claim an event row is ACK-eligible; that requires
the separately held server/caller validation boundary above.

Material difference from the original grant: terminology is narrowed so the
reader's “fully validated” checkpoint means ledger integrity only, and the
downstream ACK prerequisite is made explicit for all event rows, including
filtered-out rows. Limits, hashes, schemas, ownership, and other grant terms
are unchanged.
