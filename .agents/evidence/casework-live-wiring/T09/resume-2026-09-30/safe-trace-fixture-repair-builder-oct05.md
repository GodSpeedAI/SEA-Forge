# Safe-trace fixture repair builder handoff — 2026-10-05

## Binding scope

This source-only repair follows the original
`safe-trace-test-first-root-assignment-oct05.md`, binding accepted V3
`observation-safe-trace-port-root-proposal-v3.md` (SHA-256
`e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6`), and
its proposal-only acceptance. I reviewed the original proposal/review and
clarification history, the prior fixture rejection and supplement, the fresh
independent rejection `safe-trace-fixture-independent-source-review-fresh-oct05.md`
(which identifies the frozen test file SHA as
`a3c13aba36756d3f9f3e7657aeb3846b865dc430624e41f97c85640ae24806dc`), and
`safe-trace-fixture-root-contradiction-oct05.md` before editing. The root
contradiction note remains unchanged.

Only `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` was
edited. The semantic port, typed-unavailable adapter stub, client, Rust files,
other tests, status/debt records, and Git state were not edited by this task.

## Exact changes

Baseline fixture SHA-256:
`a3c13aba36756d3f9f3e7657aeb3846b865dc430624e41f97c85640ae24806dc`

Result fixture SHA-256:
`c39468439b51ef57f5ab8b04772063901e9cc67415eae284d7f7bfd51e19f410`

The diff is limited to these matrix corrections:

1. The three **expected-input** case/run/item mismatch loops now contain only
   their respective foreign nonblank ID. These still use the network fixture
   and expect typed unavailable. All response-side missing/blank/whitespace/
   foreign identity tests remain in the network matrix and still expect typed
   unavailable with an empty snapshot and exactly one `run_get` request.
2. The dedicated pre-dial invalid-input table now adds literal-empty case, run,
   and item values alongside the pre-existing whitespace values. Its assertions
   still require typed invalid, an exact zero snapshot, zero dial attempts, and
   zero wire requests for every case.
3. The `unknown settlement` malformed-standing row now uses valid execution
   `completed` paired with actual unknown settlement `mystery`; the existing
   `unknown execution` row remains separate.

All other fixture cases, assertions, helpers, bounded peer cleanup, request
shape checks, and matrix behavior are preserved.

## Contradiction-note precision

The root contradiction record accurately identified the conflicting
**whitespace** expected inputs: those appeared in the network/unavailable loops
and in the typed-invalid-before-dial test. Its shorthand that literal-empty
inputs were also duplicated overstates the source: literal empty appeared in
the network/unavailable loops, but the dedicated pre-dial test had only
whitespace examples. This repair removes literal empty from the network loops
and adds literal empty to the dedicated pre-dial matrix. The original fresh
independent rejection also called out the missing literal-empty pre-dial
coverage; it remains immutable and is not superseded as an approval.

## Source-only check and handoff

`gofmt -d apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go`
produced no output. No Go compiler or test was run, so this does not establish
an assertion RED or runtime behavior. This handoff requests a fresh independent
review against the original assignment and V3 before any RED run. It is not
production authorization, fixture approval, or test-pass evidence.

Material deviations from the bounded assignment: none. No prior closed fixture
finding was undone, and no unrelated change was made.
