# Safe-trace fixture shape-matrix independent source review

Date: 2026-10-05. Verdict: **READY for the assigned assertion RED after explicit compiler-token transfer only.** This is a source-only review of the fixture; it does not approve production behavior, a test pass, the trace port, or completion of T09.

## Inputs and identities

Reviewed the original `safe-trace-test-first-root-assignment-oct05.md`, the original safe-trace proposal and reviews/clarifications, v2 proposal and review, accepted v3 proposal/review, all fixture rejection reports and supplements, the root blank-identity contradiction note, and `safe-trace-fixture-shapes-builder-oct05.md`. V3 (`observation-safe-trace-port-root-proposal-v3.md`, SHA-256 `e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6`) remains the binding semantic proposal.

Current SHA-256 values match the assigned frozen identities:

- `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go`: `e8859aa2e88632f03301cb4f4d97b9039e9105a96a2eaa2c97092f2d104a1155`.
- `apps/godspeed-casework-go/internal/ports/run_trace.go`: `88191de13d6a21df988dcc3cb834df9015cdb6c2c2aa73a555e10583b108daf5`.
- `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go`: `944cf97fd4d6b055f6ec3940086eb3f7ccbce28b7ad0642d023c1a80870c0e51`.

The port is semantic-only and has no JSON tags (`ports/run_trace.go:5-30`). The adapter is still only the typed-unavailable stub (`adapters/sfwp/run_trace.go:10-15`); there is no production decoder, transport, or projection implementation in this result.

## Re-review of prior findings and matrix

The four shape gaps in the previous rejection are now represented in source:

1. The standing cases labelled `null execution` and `null settlement` contain literal JSON null, and actual absent-member cases use `runTraceResponseWithoutStanding` (`run_trace_test.go:173-190,322-340`). The accepted and invalid standing values are also exercised: all 24 allowed execution/settlement pairs, unknown execution, unknown settlement paired with valid execution, null, wrong-array shapes, and absence.
2. Unknown-kind omission is tested only for unknown **string** kinds, including missing/null/numeric IDs, malformed timestamp/payload, and a selected-ID collision (`:353-374`). Separate missing/null/numeric/boolean kind rows are malformed selected-row shapes that require typed unavailable (`:376-395`); this preserves the v3 distinction between unknown strings and malformed kind fields.
3. Invalid JSON text and valid non-object JSON roots (`null`, array, string, number) each require unavailable and an exact empty snapshot through the common assertion (`:225-248,493-505`).
4. Malformed trace-record entries now include null/scalar array entries, missing/null record names, wrong-type name, missing/duplicate record, wrong `records` root shapes, presence and byte errors, exact `u64` maximum and overflow (`:436-481`).

The earlier blank-expected-identity contradiction is resolved in the current source: network/unavailable expected-ID cases contain only foreign nonblank values (`:278-308`), while each of case/run/item has literal-empty and whitespace variants that require invalid, zero snapshot, zero dials, and zero wire requests (`:595-623`). The earlier unknown-settlement duplicate is also repaired: `completed` execution is paired with `mystery` settlement (`:322-340`).

The remaining v3 matrix is directly represented: exact run_get method, run ID, and request key shape (`:125-149`); exact ownership and missing/blank/foreign response identities (`:261-309`); all ten safe kinds (`:343-351`); selected ID/timestamp validation, source order, duplicates including a duplicate outside newest-1024 retention, and returned-row counts 0/1024/1027 with newest retention (`:267-275,376-434`); required trace presence, zero/positive bytes with empty/nonempty returned rows, and explicit source-completeness disclaimer (`:436-491`); command-finished payload missing/null/scalar/array/object distinctions and optional execution metadata, safe-integer boundaries and malformed values (`:507-559`); ignored payload data on other kinds and synthetic-field absence in semantic serialization (`:555-589`); and preserved authority refusal class with empty snapshot (`:625-639`). The helper for absent payload emits a row without a `payload` property rather than defaulting to `{}` (`:208-223,514-551`).

The peer owns one Unix listener and worker, applies bounded read/write deadlines, closes client/listener/accepted connection, and joins the worker under a timeout (`:33-123`). Positive behavior assertions happen before request waiting in `assertRunTraceCase` (`:225-248`), so the unavailable stub reaches an assertion rather than blocking on a nonexistent request. The dedicated pre-dial test does not wait for the peer request. These are source properties only.

## Delta and material limits

The previous frozen `c394684...` file is not present as an archived source copy in the workspace or `/tmp`, so I could not independently compute a byte-for-byte diff against it. I verified the current source at every previously rejected shape and contradiction anchor, and the current test SHA matches the assigned new identity, but I do not independently certify the builder's complete-delta claim against the unavailable prior bytes. The builder record states that only the one test file changed, adding an absent-standing helper/cases and the four shape-case groups; current workspace status confirms that this test file is untracked, but status is not a complete historical diff.

The fixture's passing behavior, the intended RED, actual client retry behavior, and production authority/artifact behavior remain unverified because the assigned boundary is source-only and no compiler token was transferred. The fixture asserts one request line for each returned-response case; it does not independently prove `Client.Do` retry semantics. The approved proposal assigns that retry to the existing transport `Do`, and this fixture review makes no retry claim. Timestamp handling is covered on selected trace frames; this review does not approve or resolve the independent real-kernel cursor-format/schema mismatch recorded in `live-cursor-contract-independent-recon-oct05.md`.

No source, test, status, debt, or Git file was edited by this critic, and no compiler/test/format command was run. No fixture or feature beyond this test-first source matrix is approved. Graft retrieval saved approximately 60,672 tokens (~$0.05) across two calls this turn.
