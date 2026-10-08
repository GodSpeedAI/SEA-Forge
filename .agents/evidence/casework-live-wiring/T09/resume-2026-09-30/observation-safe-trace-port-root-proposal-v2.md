# Unit5 safe trace port: repaired root proposal for independent source critique

Preparation only. This is a proposal for a later independent source review; it
releases no fixture or production writes and gives no runtime approval. It
implements only the already approved trace disclosure projection. It does not
authorize session sharing, pollers, SSE, UI, or full T09 settlement.

## Semantic boundary

Add a separate application `ports.RunTracePort` with
`ReadRunTrace(ctx context.Context, caseID, runID, planItemID string)` returning
`ports.RunTraceSnapshot` and error. The snapshot contains exact run/case/item
identity, execution and settlement strings, safe frames, and the total number
of safe frames in the `trace` array returned by this successful `run.get`
response. A safe frame contains event ID, kind, original timestamp, optional
execution status, and optional int64 exit code only. Domain structs have no
JSON wire tags, following neighboring semantic ports. The gateway stamps the
later observation time; it never replaces a frame's recorded timestamp.

Implement on the existing SFWP Authority using exactly one logical
`Client.Do` with `NewRunGet(runID)`. Preserve the existing inspect transport
retry inside `Do`. Keep `RunArtifactProvenanceView` and artifact authorization
unchanged; use a separate decoder/projection for this disclosure. No kernel
verb or dependency. Do not put this data in CaseFacts, Store, or captured world
snapshots.

Reject blank expected identities before dialing, using the existing invalid
input error convention. For a completed read, require nonblank and exact run
ID, case ID, and plan item ID matching all three expected values. Unknown
ownership or mismatch is unavailable with no safe result. Require execution
in pending/enabled/active/completed/failed/terminated and settlement in
unsettled/accepted/rejected/escalated. Preserve authority refusal classes;
malformed responses and invalid disclosure projections are unavailable.

## Trace presence and completeness boundary

Require `records` to be an array containing exactly one entry whose
`record` is exactly `trace.jsonl`. That entry must have `present: true` and a
`bytes` value represented as a nonnegative exact JSON integer. Missing,
duplicate, absent, false, malformed, or missing-byte trace metadata makes the
read unavailable. Do not infer trace presence from `trace` being an array.
This metadata establishes filesystem presence only; it is not evidence that
the journal was fully read or parsed.

Require `trace` to be an array. Missing/null/wrong-shaped trace is unavailable.
An empty array is safe zero only when the exact trace-record metadata above
reports `present: true` and `bytes: 0`, and only after all requested identity,
standing, and response validation succeeds. Empty `trace` with positive byte
size is unavailable because the response does not support a safe-zero claim.
A nonempty trace with zero byte size is inconsistent metadata and unavailable.

`run.get` intentionally returns a valid prefix when JSONL parsing encounters a
malformed line, and can return no rows on read failure or over-cap fallback.
Its current response has no parse-completeness signal. Therefore both the
safe-frame count and any later cohort completeness/retained/omitted/truncated
accounting refer only to allowlisted safe rows in the `trace` array received
from this successful `run.get` response. Compute the safe-frame count before
the port's 1024-row retention. A nonempty returned prefix may be projected
under this explicit returned-row boundary, but must not be called a complete
source trace or used to claim a source-journal total. Trace-record presence
and byte size do not cure this limitation. Do not fabricate a source count,
parse status, or UI claim of source completeness. Resolving source completeness
requires a separately authorized kernel contract/behavior change and is out of
scope here.

## Projection and count rules

Select only the ten canonical safe kinds in `types.ts:306-316`. Unknown kinds
are omitted entirely, including their payloads. For each selected row, require
a nonblank original event ID, RFC3339/RFC3339Nano timestamp, and no duplicate
selected event ID anywhere in the full returned array, including rows later
omitted by retention. Duplicate checking is performed before kind filtering
and retention so a selected ID repeated in a returned row omitted by kind
filtering or retention cannot evade the full-array uniqueness rule. Unknown
rows otherwise contribute no safe frame and their payload is ignored. Preserve
original source order and timestamp text. No sorting, manufactured IDs, or
manufactured times.

The total safe-frame count is the number of validated allowlisted rows in this
returned array before retention; unknown kinds are excluded. Retain the newest
at most 1024 safe rows in source order. Later cohort serialization may retain
fewer to satisfy the approved one-MiB envelope bound, but must preserve this
returned-array safe count and report accurate retained/omitted/truncated
fields within that same boundary. This port makes no claim that the upstream
trace journal or total cohort wire bytes are bounded by 1024.

For `command_finished`, a nonnull `payload` must be an object. Inspect
`payload.execution.status` and `exit_code`. Missing/null raw `execution`,
`status`, or `exit_code` means no recorded optional value; output omits the
corresponding field. A nonnull `execution` must be an object; a supplied
nonnull status must be a canonical command execution status, and a supplied
nonnull `exit_code` must be a JSON integer within both int64 and the JavaScript
exact interval [-9007199254740991, 9007199254740991]. Fractional, string,
boolean, overflow, or unsafe recorded values make the entire read unavailable.
Never silently discard or round a supplied nonnull value. Outgoing canonical
optional fields remain nonnull when present. Do not interpret payload metadata
on other kinds. Actor, arguments, output, environment, evidence, and all other
raw payload fields are ignored and cannot exist in the semantic DTO.

## Test-first decomposition after cancellation prerequisite

Once the cancellation fixture/repair is independently verified, release a
fresh builder for tests and any explicitly named minimal declarations needed
for a compiling assertion RED. Then an independent critic receives this v2
proposal plus the actual result and must prove the intended RED before
production implementation. Required matrix:

- exact request and all three ownership identities; blank expected identity
  rejected before dialing; all allowed execution/settlement standings;
- all ten safe kinds, unknown-kind omission and payload exclusion, selected-row
  ID/timestamp validation, source order, and duplicate selected IDs across the
  full returned array, including a duplicate found in a row omitted by kind
  filtering or retention;
- valid empty trace plus exactly one present `trace.jsonl` record with zero
  bytes produces safe zero only after identity/standing checks; missing,
  duplicate, absent/false, malformed, or missing-byte record metadata is
  unavailable; missing/null/wrong-shaped `records` and trace-presence entries
  are unavailable;
- empty trace with positive bytes is unavailable; nonempty trace with zero
  bytes is unavailable; nonempty valid returned rows with positive bytes are
  counted under the returned-row boundary;
- demonstrate/document that a nonempty returned prefix can be counted without
  asserting source-journal completeness or source totals;
- zero/1024/1027 returned safe-row counts and newest-row retention order;
- command payload missing/null handling, nonnull payload/execution shape,
  optional metadata absent/null versus outgoing absence, safe integer
  boundaries, unsafe adjacent values, fraction/string/boolean, and int64
  overflow;
- malformed standing/trace shape; public synthetic ignored-field markers
  absent from semantic serialization; existing artifact authorization and
  authority refusal behavior unchanged.

This matrix preserves the original proposal's cases and adds executable
presence, duplicate-source-row, and returned-prefix boundary cases. Root
retains integration and count semantics; the critic must identify remaining
contradictions or missing cases. This document is a proposal, not passing
evidence.

## Material changes from the original proposal

1. Added an exact-one `records` entry requirement for `trace.jsonl`, requiring
   `present: true` and nonnegative exact-integer `bytes`; missing, duplicate,
   absent, or incomplete metadata now makes the read unavailable.
2. Tightened the empty-trace rule: only a present zero-byte journal paired
   with an empty returned array may yield safe zero, after ownership and
   standing validation. Positive-byte empty responses and zero-byte nonempty
   responses are unavailable.
3. Defined safe count and subsequent cohort completeness/retention accounting
   against safe rows in the current successful `run.get` returned array, never
   the source journal. Nonempty prefixes may validate under this boundary.
4. Explicitly documented that current `run.get` cannot report JSONL
   parse/read completeness; presence and byte size do not establish it. No
   source-total, full-source-completeness, or invented UI claim is permitted.
   Kernel/schema changes remain separately authorized scope.
5. Added the command-finished nonnull payload-object requirement while
   retaining the nonnull execution-object rule and all prior exact optional
   integer/status validation.
6. Made duplicate checking explicit across the full returned array before
   kind filtering and retention, and added the filtered-row duplicate test.
7. Extended the test matrix with trace metadata cardinality/shape, zero versus
   nonzero bytes, and returned-prefix-count cases. All other original identity,
   refusal, ten-kind, timestamp, ID, duplicate, newest-1024, exact optional
   safe-integer, no-sensitive-fields, and unchanged-artifact rules remain.

## Source anchors

- The original proposal's ports, transport, identity, projection, and optional
  metadata rules: `observation-safe-trace-port-root-proposal.md`.
- Independent review and its append-only clarification:
  `observation-safe-trace-port-proposal-independent-review.md` and
  `observation-safe-trace-port-proposal-independent-review-anchor-clarification.md`.
- `crates/sea-forge-server/src/sfwp/run_views.rs:425-436`: `read_jsonl` can
  return empty on cap/read failure and stops at the valid prefix on malformed
  JSONL. `:842-855` shows `get` uses it and may still return a run when a plan
  or settlement exists.
- `crates/sea-forge-server/src/sfwp/run_views.rs:85-95,240-250,283-326,870-880`:
  canonical record inventory; `RecordPresence` filename/present/optional bytes;
  returned trace and records fields; and metadata population.
- Canonical safe frame/status fields:
  `.agents/reports/interface-contracts/typescript/types.ts:306-335`.
- Run identity and standing source shape:
  `crates/sea-forge-server/src/sfwp/run_views.rs:283-326`.
- Existing semantic port and narrow artifact view:
  `apps/godspeed-casework-go/internal/ports/ports.go` and
  `apps/godspeed-casework-go/internal/adapters/sfwp/frame.go:686-694`.
- Normative informational boundary:
  `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:82-105`.

Graft retrieval preceded source validation. The current Rust source anchors
above were then read directly. No source code, fixture, test, or runtime was
changed or approved by this proposal.
