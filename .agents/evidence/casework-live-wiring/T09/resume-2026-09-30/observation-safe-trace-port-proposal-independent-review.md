# Unit5 safe trace-port proposal: independent source review

Review result: **REJECT** (proposal only; no fixture or runtime approval).

Reviewed the frozen root proposal (SHA-256
`03a69d218e750432f01019ee1eb5f3eb6c3c874672e0d11ea5c9db740a47fd93`), the fixture
preparation (SHA-256 `f0f13da2f1e2f8fed7174dd02bb562850330cf9d20cbb818f3be006e647e60a7`),
the runtime integration decisions (SHA-256
`3658e72703464006cd7cc43f2c5e6fd8c98a3bf5788127a4a7b95c638b3f553d`), the operator-approved
T09 contract proposal (SHA-256
`0453e5b45cb56fc09b2fb29de41424aaeb9f565e98e476c627c6ba85bf4f96de`), canonical TypeScript
contract (SHA-256 `d272a25a7e799b839ac1997a4c153fc0a8697ca11f365708b100030c1b092b7b`), and
normative cognitive-environment spec (SHA-256
`403c9e6e2b59a5169cfeaa9aba2f290e885b39f8f44edf50fba5785255e24169`).

## Blocking source contradiction: an empty returned trace is not proof of a real empty trace

The proposal makes an explicit empty `trace` array a valid zero-frame result, while treating
absent/null/wrong-shaped trace as unavailable (`observation-safe-trace-port-root-proposal.md`,
“Projection and count rules”). The existing Rust `run.get` view cannot uphold that distinction
from the trace array alone:

- `crates/sea-forge-server/src/sfwp/run_views.rs:842-843` obtains trace rows through
  `read_jsonl(&dir.join("trace.jsonl"))`.
- `crates/sea-forge-server/src/sfwp/run_views.rs:77-83` defines `read_jsonl`; at
  `:77-94` it returns an empty vector when the file exceeds the journal cap, cannot be read,
  or does not exist, and `map_while` stops at the first malformed JSON line.
- `crates/sea-forge-server/src/sfwp/run_views.rs:283-326` exposes `RunRecord.trace` as a
  defaulted `Vec<TraceRow>`, so serialization emits an array in these cases. A run with a
  settlement can still be returned with empty events: the list path only calls it unreadable
  when both events are empty and settlement is absent (`:793-800`); `run.get` itself proceeds
  to build the record (`:842-850`).
- The view already has a `records` presence list, and `trace.jsonl` is one of its canonical
  entries (`:79-95`), but the proposal’s decoder requirements do not require/check this signal.

Consequently, the specified decoder can accept “no readable source trace” as a successful
zero-frame observation. It can also report a complete but shortened trace after a malformed
line because the kernel silently stops parsing and returns the valid prefix. This conflicts
with the proposal’s source-fidelity/count language and with its distinction between valid empty
trace and unavailable malformed response. The operator-approved proposal requires observations
to be real kernel trace facts and exact truncation/count reporting; it does not authorize
fabricating completeness from the lossy `run.get` projection.

Before fixture release, revise the contract to use the existing run-record presence metadata
to reject a trace journal that is absent (so the deliberate valid-empty case requires a present
trace record), and explicitly bound the claim to rows actually returned by `run.get`. Also
resolve the malformed-line prefix ambiguity: either establish an existing `run.get` signal
that proves the returned trace is complete, or make the limitation explicit and prevent the
port/cohort from labeling that result complete or reporting source-total counts. Do not silently
expand this unit into a Rust wire/schema change; that would need separate authorization.

## Confirmed portions

- The ten allowlisted kind strings and the two optional command fields match
  `.agents/reports/interface-contracts/typescript/types.ts:306-335`; the safe frame deliberately
  has no raw payload or actor fields. `TraceKind` has additional variants in
  `crates/sea-forge-core/src/types.rs:499-525`, so unknown-kind omission is necessary and
  consistent with the canonical boundary.
- The five command statuses match `crates/sea-forge-core/src/types.rs:478-492` and the
  TypeScript union at `types.ts:318-323`. Kernel `exit_code` is `Option<i64>`; validating the
  JavaScript safe interval before projecting to `number` correctly implements the comment at
  `types.ts:334-335` and `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/
  runtime-integration-decisions.md` (“Exact numeric observation metadata”). Missing/null raw
  optionals must remain omitted in the canonical outgoing DTO, which has `execution_status?`
  and `exit_code?` and no nullable form (`types.ts:325-335`).
- The three-way expected identity check is supported by `RunRecord`’s optional case/item
  identities (`crates/sea-forge-server/src/sfwp/run_views.rs:283-297`); requiring exact
  nonblank case and plan-item values avoids turning optional kernel identity into guessed
  ownership. Execution and settlement are separate fields there (`:298-326`) and their allowed
  vocabularies match the canonical types (`types.ts:339-350`).
- A separate semantic port/decoder is consistent with the neighboring application-owned ports
  and the existing narrow artifact view (`apps/godspeed-casework-go/internal/ports/ports.go`;
  `apps/godspeed-casework-go/internal/adapters/sfwp/frame.go:686-694`). Keeping trace
  observations out of `CaseFacts` and historical snapshots matches runtime decision anchors
  and normative spec `godspeed.casework-cognitive-environment-spec.yaml:82-105`.
- Calling `Client.Do` once at the adapter level while retaining its established read-only
  retry path is consistent with the approved T09 proposal’s inspect-retry rule. Authority
  refusal classes must pass through unchanged; malformed successful response/projection can
  map to typed unavailable. No new Rust verb, dependency, API identity, or artifact-provenance
  relaxation is authorized by the reviewed proposal.

## Required fixture coverage after repair

Keep the proposed matrix and add cases that make the source distinction executable:

1. Empty trace array with `records` declaring `trace.jsonl` present is accepted as zero frames;
   an empty array with missing/false trace-record presence is unavailable.
2. A trace array containing a valid prefix followed by a malformed-source condition cannot be
   silently asserted as a complete source trace. If the current wire view has no way to
   represent that condition, assert/document the narrow returned-array claim and do not claim
   source-journal completeness.
3. Cover null/missing/wrong-shaped `records` and trace-presence entries as malformed projection.
4. Retain the specified duplicate-ID test after the newest-1024 retention boundary, and add a
   selected allowlisted event whose duplicate ID occurs in a row that is later omitted by kind
   filtering, clarifying whether identity uniqueness spans all source rows or only selected
   safe rows.

Other required root-proposal cases remain appropriate: exact identities; all ten kinds;
unknown-kind payload exclusion; selected-row ID/timestamp validation; source order; 0/1024/1027
counts; optional command metadata absent/null versus nonnull wrong shapes; safe integer edges,
fraction/string/bool and overflow; malformed standing/trace shape; public ignored-field markers
absent from the semantic DTO; refusal preservation; and existing artifact authorization
unchanged. No tests or compiler were run for this source-only review.

## Source anchors

- Proposal and intended rules: `observation-safe-trace-port-root-proposal.md`, semantic boundary,
  projection/count rules, required matrix.
- Existing fixture-preparation caveats: `observation-trace-port-fixture-preparation.md`.
- Existing identity/standing and run-view shape: `crates/sea-forge-server/src/sfwp/run_views.rs:77-95,283-326,793-800,835-850`.
- Raw trace kind/status/exit types: `crates/sea-forge-core/src/types.rs:478-525`.
- Canonical safe frame, run observation, and standing DTOs:
  `.agents/reports/interface-contracts/typescript/types.ts:306-372`.
- Normative informational boundary: `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:82-105`.

Graft used: one source retrieval call, saving approximately 104,861 tokens ($0.08).
