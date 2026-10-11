# Unit5B safe trace Phase1 fixture builder record

Date: 2026-10-05. Source-only builder record for the released Phase1 assignment.

## Authority and scope

The binding assignment is `safe-trace-test-first-root-assignment-oct05.md`;
the prerequisite release is `cancellation-root-acceptance-oct05.md`. Root's
follow-up resolved an initial path ambiguity explicitly: only the three new Go
files below are authorized. No Rust path was created or edited. V3 is the
semantic test specification; the original proposal and rejection history remain
part of the review envelope. V3 proposal review accepted the proposal only, not
fixtures, implementation, compilation, runtime behavior, or T09 settlement.

Created files (all absent before this builder assignment):

| File | SHA-256 | Purpose |
| --- | --- | --- |
| `apps/godspeed-casework-go/internal/ports/run_trace.go` | `88191de13d6a21df988dcc3cb834df9015cdb6c2c2aa73a555e10583b108daf5` | Application-owned `RunTracePort`, snapshot and safe-frame semantic types; no JSON tags or adapter/wire imports. |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go` | `944cf97fd4d6b055f6ec3940086eb3f7ccbce28b7ad0642d023c1a80870c0e51` | Compile-time port conformance assertion and temporary typed-unavailable method stub. |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` | `3ffab96a1bdf8235db876d4ada4cc6978279952b0db676ba15b8594e991bcf64` | Focused v3 matrix with bounded owned Unix-socket peers, exact request evidence, and bounded worker cleanup. |

No existing Go, Rust, schema, DTO, authorization, server, UI, status, debt,
Git, or immutable evidence files were changed. No source behavior was implemented.

## Fixture matrix

The six top-level test functions cover:

- Exact expected ownership and exact single `run_get` request; missing, blank,
  and mismatched response identities; blank expected identities rejected before
  any dial; all six execution and four settlement standings; malformed or
  unknown standings.
- All ten allowlisted kinds, unknown-kind omission before validating malformed
  ignored fields, selected/unknown ID collision acceptance, selected ID and
  RFC3339 timestamp validation, original timestamp text/source order, duplicate
  selected IDs, and a selected duplicate spanning the retained/dropped 1024
  boundary.
- Returned safe-row counts 0, 1024, and 1027, with newest-row retention kept in
  source order. Count expectations are explicitly scoped to rows returned by
  `run.get`.
- Exactly-one trace-record presence validation: missing/null/wrong-shaped
  records or trace, absent/false/malformed/missing-byte metadata, duplicate
  trace entries, negative/fractional/string/boolean bytes, exact `u64` maximum,
  and overflow. Empty returned rows with zero or positive bytes and nonempty
  rows with zero or positive bytes are accepted after presence and ownership
  validation; there is no byte/row consistency inference.
- A returned prefix is counted without asserting source-journal completeness.
  Comments record that current `run.get` has no parse-completeness signal and
  may return empty rows after read/cap failure or a valid prefix after malformed
  JSONL. Presence and bytes establish no completeness fact.
- `command_finished` missing/null/scalar/array payloads and missing/null/object
  execution behavior; all five canonical statuses; absent/null optionals,
  JavaScript-safe integer boundaries, unsafe adjacent values, fractional,
  string, boolean, and `int64` overflow rejection. Non-command payload metadata
  is ignored. Semantic serialization checks that raw/synthetic markers do not
  cross the port and explicitly disclaim canonical wire representation.
- Authority refusal class preservation. The existing artifact provenance and
  authorization implementation/tests remain untouched.

The fixture harness owns one listener and one server worker per case. Cleanup
closes the client, listener, and accepted peer connection, then joins the worker
with a five-second bound. Successful-result assertions run before exact request
capture checks, so the temporary stub reaches the expected behavior assertion
RED without waiting on an absent request. Negative cases check the expected
typed behavior before checking that the peer observed exactly one request.

## Material semantic changes retained from original proposal through v3

The original proposal/rejection history and accepted v3 specification identify
these material v3 repairs, all represented in fixtures:

1. Unknown kinds are omitted before inspecting IDs or payloads; duplicate-ID
   checks apply to selected rows across the full returned array, before
   retention. A selected duplicate beyond retention remains invalid.
2. Trace-record metadata requires exactly one `trace.jsonl` entry, `present:
   true`, and a nonnegative exact JSON `u64` byte value. This records filesystem
   presence, not successful parsing or completeness.
3. Empty/nonempty trace rows are not rejected based on their byte-size
   relationship. The upstream read and later metadata collection are not an
   atomic snapshot.
4. `TotalFrameCount` is returned-array allowlisted-frame count before retention,
   not a source journal count. No parse-completeness or source-total claim is
   added.
5. Missing, null, scalar, or array whole payload on `command_finished` is
   treated as absent command metadata; supplied nonnull `execution` must be an
   object. Supplied status and exit-code values remain exact-validated.

The runtime implementation stays deliberately unavailable in Phase1. It does
not call `Client.Do`, decode a response, retry, project trace fields, alter the
artifact path, or implement pollers/SSE/UI. This record does not approve the
port semantics or production method; independent source review and the separate
compiler-token RED gate are still required.

## Source-only verification and limitations

`gofmt -d` on all three new Go files produced no output after formatting
corrections. No compiler, test, Cargo, or runtime command was run. The required
assertion RED is not claimed as executed or proven here. No runtime/decode
behavior, test compilation, or test execution result is claimed.

Graft retrieval preceded source inspection. Retrieved the neighboring Go port
shape, SFWP `Authority`/`NewRunGet` and refusal mapping, and existing bounded
socket fixture patterns before writing. Graft reported approximately 146,768
tokens saved (~$0.12) across this turn's retrieval calls.
