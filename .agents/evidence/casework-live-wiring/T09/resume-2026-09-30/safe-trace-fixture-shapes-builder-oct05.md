# Safe trace fixture shape-matrix builder — 2026-10-05

## Scope and review inputs

This source-only repair follows `safe-trace-test-first-root-assignment-oct05.md` and accepted semantic proposal V3 (`observation-safe-trace-port-root-proposal-v3.md`, SHA-256 `e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6`). It re-read the original proposal, original independent rejection and anchor clarification, v2 proposal/rejection, v3 approval, and all fixture review findings through `safe-trace-fixture-independent-source-review-final-oct05.md` (SHA-256 `85554606352a754743b0b56649a1538530a4b4499ac3ccf3f23f5cfebaee79f4`). The final review's four remaining gaps are the exact target of this change. This is a fresh fixture builder pass; no production trace code is authorized.

## Exact source identity and delta

Only `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` changed in this pass.

- Before: SHA-256 `c39468439b51ef57f5ab8b04772063901e9cc67415eae284d7f7bfd51e19f410` (the frozen final-review target).
- After: SHA-256 `e8859aa2e88632f03301cb4f4d97b9039e9105a96a2eaa2c97092f2d104a1155`.
- Unchanged typed port: `apps/godspeed-casework-go/internal/ports/run_trace.go`, SHA-256 `88191de13d6a21df988dcc3cb834df9015cdb6c2c2aa73a555e10583b108daf5`.
- Unchanged temporary adapter stub: `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go`, SHA-256 `944cf97fd4d6b055f6ec3940086eb3f7ccbce28b7ad0642d023c1a80870c0e51`.

The complete source delta is limited to these hunks:

1. Added `runTraceResponseWithoutStanding` immediately after `runTraceResponse` (around lines 173-188). It assembles valid JSON members explicitly and omits exactly the named `execution` or `settlement` field. It does not route through the empty-string defaulting helper. Tests use it for actual absent-member cases.
2. Renamed the previous standing matrix labels from “missing execution/settlement” to “null execution/settlement” because those values are literal JSON `null`; added separate absent execution and absent settlement cases using the new omission builder (around lines 326-339). Unknown and wrong-shaped cases remain.
3. Added selected-row malformed-kind cases for missing `kind`, `kind: null`, numeric `kind`, and boolean `kind` (around lines 377-380). Existing future/unknown **string** kinds remain omission cases and retain their malformed IDs, timestamps, payload markers, and selected-ID collision test.
4. Added malformed `records` entries for a null row, numeric scalar row, missing `record` property, and null `record` name (around lines 452-455). Existing wrong-typed name, wrong top-level shapes, missing/duplicate/non-present records, malformed presence, and byte-boundary cases remain.
5. Expanded `TestReadRunTraceV3RejectsRawInvalidJSON` to a table that retains invalid JSON text and adds valid JSON root shapes `null`, array, string, and number (around lines 493-502). Every case uses the common assertion requiring unavailable and the exact zero snapshot, then verifies one logical `run_get` request.

## Shape audit and preserved semantics

The new absent-standing builder's source explicitly adds the other standing and omits the selected one before joining object members; neither execution nor settlement is silently defaulted in these cases. The standing cases labelled null contain literal `null`. Unknown-kind behavior is still limited to unknown strings; missing/null/nonstring kinds enter the selected malformed-response cases and expect unavailable with no safe result. Root-shape cases are syntactically valid JSON values distinct from invalid JSON syntax. The `records` cases are array elements or named-record properties, not top-level records-shape cases.

The previously reviewed ownership, exact request, pre-dial blank identity, refusal, zero-snapshot, ownership, safe-kind and selected-ID/timestamp cases remain unchanged. The unknown string collision, valid absent payload path, malformed ignored unknown IDs, selected duplicate across retention, return-row limits/order, trace presence semantics, safe integer cases, and source-completeness disclaimers remain unchanged. No byte/row consistency assertion or unknown-kind ID inspection was added.

## Formatting and limits

`gofmt -d apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` produced no output after formatting. No Go compiler or tests were run; no runtime RED or passing behavior is claimed. No port, stub, client, Rust, UI, status, debt, or Git changes were made. No material deviation from the bounded assignment was identified. This builder record awaits independent source review and does not grant RED approval.

Graft was used before inspecting source and reported approximately 9,218 tokens saved (under $0.01) this turn.
