# Safe-trace Phase1 fixture: final independent source review

Date: 2026-10-05. Verdict: **REJECT for fixture approval; do not run the Go RED yet.** This source-only review evaluates the repair against the original test-first assignment, accepted v3 contract, and complete review/rejection history. No Go compiler, test, or runtime execution was performed.

## Inputs and frozen source identities

Reviewed `safe-trace-test-first-root-assignment-oct05.md`; original safe-trace proposal and its independent rejection; anchor clarification; v2 proposal and rejection; v3 proposal and accepted source-only review; initial fixture rejection and supplement; fresh fixture rejection; root blank-identity contradiction note; fresh repair builder record; and all three actual Go files.

Current SHA-256 identities match the fresh builder handoff:

* `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go`: `c39468439b51ef57f5ab8b04772063901e9cc67415eae284d7f7bfd51e19f410`.
* `apps/godspeed-casework-go/internal/ports/run_trace.go`: `88191de13d6a21df988dcc3cb834df9015cdb6c2c2aa73a555e10583b108daf5`.
* `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go`: `944cf97fd4d6b055f6ec3940086eb3f7ccbce28b7ad0642d023c1a80870c0e51`.

The semantic port remains separate, lacks wire JSON tags and raw DTO coupling, and contains the approved safe fields. The adapter remains the typed-unavailable test-first stub with no decoder, transport request, or projection.

## Repair verification and earlier rejection closure

The repair record claims exactly three edits to the test file. I independently reversed those three edits in memory—restored the former blank/whitespace expected-identity loop elements, restored the old unknown-settlement tuple, and removed the three new literal-empty predial cases. The reconstructed source SHA-256 matched the prior frozen fixture `a3c13aba36756d3f9f3e7657aeb3846b865dc430624e41f97c85640ae24806dc` exactly. This confirms the actual test-file delta is limited to the three reported corrections; the test file was not written during this audit. The port and adapter identities are also unchanged.

The three corrections are sound:

1. Expected-input network/unavailable loops at `run_trace_test.go:276-289` now contain only each field's foreign nonblank value. Response-side blank/whitespace/mismatch and valid-JSON missing-response-identity cases remain unavailable with an empty snapshot and exactly one `run_get`.
2. The predial test at `:553-581` now covers literal-empty and whitespace values for case, run, and item independently. It requires typed invalid, exact zero snapshot, zero dial attempts, and zero wire requests.
3. The unknown-settlement case at `:309` now pairs valid `completed` execution with `mystery` settlement, distinct from unknown execution.

The repair resolves all prior fixture blockers: missing response identity is valid JSON (`:173-187,270-273`); all error assertions require an empty safe result (`:206-229`); selected malformed IDs/timestamps and null/scalar trace rows are covered (`:352-367`); unknown string kinds with missing/null/numeric IDs and malformed ignored timestamp/payload are omitted alongside an unknown/selected ID collision (`:329-349`); presence shapes include missing/null/wrong-type `present`, null/missing/invalid bytes and exact `u64` edges (`:408-439`); raw invalid JSON is tested (`:461-463`); and the absent-payload row helper omits the payload member rather than substituting `{}` (`:199-204,471-512`).

The test peer owns one listener and worker, four-second read/write deadlines, closes client/listener/accepted connection and joins the worker within five seconds (`:35-123`). Its request assertion checks one received line, `run_get`, the requested run ID and exactly two keys (`:125-149`). Positive return assertions precede any request assertion, so the immediate unavailable stub reaches a behavior assertion without waiting for a request. These are source observations only; no runtime behavior is proven.

## Remaining blocking coverage gaps

1. **“Missing execution/settlement” cases use JSON null, not absent members.** The standing table is at `:303-315`; entries labelled `missing execution` and `missing settlement` pass `null`. `runTraceResponse` at `:151-171` always emits both standing members and has no omission path. The v3 contract requires execution and settlement to be present with one of their allowed values and calls for malformed-standing coverage. Null is a malformed supplied value; it does not exercise an omitted required field. Rename these cases accurately and add actual absent-member cases for both standings before claiming the missing-shape matrix is complete.

2. **Missing/null/wrong-typed event `kind` shapes are not tested.** Unknown-kind omission is specifically for unknown **string** kinds (`safe-trace-test-first-root-assignment-oct05.md`, lines 46-48; v3 projection rules). Current unknown rows all use string kinds (`:329-349`); malformed selected rows omit/alter ID or timestamp but always have a valid string kind (`:352-367`). Missing, null, or non-string kind is not an unknown string kind and falls under malformed-response/projection handling, yet no case asserts unavailable and zero safe result for those shapes. Clarify this boundary in the fixture with representative malformed-kind cases.

3. **Valid JSON with a wrong top-level response shape is absent.** `runTraceResponse` always emits a JSON object (`:151-171`); the raw-response test at `:461-463` covers only syntactically invalid JSON. V3 requires malformed successful responses to be unavailable. Add at least a valid non-object root case (for example `null` or an array) so response-envelope shape handling is represented separately from syntax errors and nested projection shapes.

4. **Malformed `records` array entries are not exhaustively represented.** The suite tests a numeric `record` value (`:437`) and malformed top-level `records` shapes (`:421-423`), but has no null/scalar array element and no entry with `record` missing or null. V3 requires malformed trace-presence entries to be unavailable and requires exactly one named `trace.jsonl` entry. Add representative null/scalar entry shape plus missing/null record-name cases, or narrow the documented claim if the accepted contract does not require these shapes to fail.

These are test-matrix gaps, not claims about production adapter behavior. In particular, the `missing ...` standing labels are materially inaccurate because the JSON contains explicit nulls.

## Disposition and limits

The prior root contradiction note's literal-empty shorthand was imprecise: before this repair, whitespace inputs overlapped between network and predial cases, while literal empty appeared only in the network/unavailable cases. The earlier independent reviewer (this reviewer) failed to identify that contradiction in the first fixture review; root found it. This repair correctly addresses it. The prior missing-payload, malformed unknown-ID, response-identity, zero-snapshot, selected-row, presence, and raw-invalid-JSON rejection findings are closed as described above.

The remaining malformed standing, event-kind, top-level response, and record-entry shapes prevent READY. A new fixture repair and independent review are needed before the intended RED. No source, test, status, debt, or Git file was edited; no compiler token was used. Graft retrieval preceded source verification and saved approximately 38,934 tokens (~$0.03) this turn.
