# Safe-trace Phase1 fixture: fresh independent source review

Date: 2026-10-05. Verdict: **REJECT; do not run the Go RED yet.** This source-only review evaluates the fresh builder result against the original test-first assignment and binding accepted v3 contract. No Go compiler or test was run, and no runtime RED is claimed.

## Reviewed inputs and frozen identities

Reviewed the original assignment `safe-trace-test-first-root-assignment-oct05.md`, original proposal and review history (including v1 rejection, anchor clarification, v2 proposal/rejection), accepted v3 proposal/review, both prior independent fixture rejection records and supplement, root contradiction finding, fresh builder record, and actual source.

The fixture SHA matches fresh builder record `safe-trace-fixture-fresh-builder-oct05.md` (`d6c0f8f56f099026ca943386595fc7c6b41067c4c252776d239adbf66efa9db0`):

* `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go`: `a3c13aba36756d3f9f3e7657aeb3846b865dc430624e41f97c85640ae24806dc`.
* `apps/godspeed-casework-go/internal/ports/run_trace.go`: `88191de13d6a21df988dcc3cb834df9015cdb6c2c2aa73a555e10583b108daf5`.
* `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go`: `944cf97fd4d6b055f6ec3940086eb3f7ccbce28b7ad0642d023c1a80870c0e51`.

The separate port remains semantic-only (no wire JSON tags/raw DTO coupling); adapter remains a typed-unavailable test-first stub without production decoder/transport/projection. The builder says only the test fixture changed and reports `gofmt -d` empty. I did not run formatting, compiler, or tests.

## Blocking source findings

1. **Blank expected identities are asserted both invalid-before-dial and unavailable-after-wire.** At `run_trace_test.go:276-289`, each expected case/run/item matrix includes `""` and `" \t"` and routes them through `assertRunTraceCase` expecting `apperr.KindUnavailable`; that helper at `:206-229` also requires exactly one `run_get` line. V3 and the original assignment require blank expected identities to return typed invalid before dialing. The dedicated test at `:553-578` correctly asserts invalid, zero snapshot, zero dial and zero wire request for each field—but so far it tests only whitespace (`" \n"`, `"\t"`, and `" "`). Thus the whitespace inputs are directly contradictory; the literal-empty inputs are not correctly tested at all and are missing from the dedicated predial coverage. The correction is to retain only foreign nonblank expected IDs in the network/unavailable loops, and add literal-empty inputs for all three fields to the dedicated typed-invalid/zero-snapshot/zero-dial/zero-wire matrix while retaining its whitespace cases. Root's contradiction note slightly overstates literal-empty overlap: the dedicated test has three whitespace examples and no literal-empty examples.

2. **The unknown-settlement case duplicates unknown execution.** At `run_trace_test.go:303-311`, `{"unknown execution", "mystery", "accepted"}` and `{"unknown settlement", "mystery", "accepted"}` pass identical execution/settlement JSON to the test at line 314. The latter therefore tests a second unknown execution, not an unknown settlement. V3 requires unknown standings to be unavailable; missing and wrong-shaped settlement cases exist, but no valid execution paired with an unknown settlement does. This leaves the unknown-settlement rule untested.

Both findings are test-source defects, not production behavior claims. No source change or execution result is inferred.

## Prior rejection findings: status

The fresh builder closes the earlier source findings:

* `runTraceIdentityWithout` at `:173-186` builds valid JSON with exactly one response identity key omitted, covering each missing run/case/item response identity (`:270-273`). Response-side blank, whitespace, and foreign values remain unavailable cases with valid expected IDs.
* `assertRunTraceCase` at `:206-229` requires all error results to equal the exact expected zero snapshot.
* Selected rows now cover missing/null/numeric event IDs; missing/null/numeric timestamps; and null/scalar array rows (`:350-365`). Unknown string kinds now have missing/null/numeric event IDs alongside malformed timestamp/payload and selected-ID collision cases (`:331-348`).
* Trace presence adds missing/null/wrong-type `present` and null `bytes` cases; prior required/duplicate/absent/false/malformed/non-integer/negative/u64-boundary cases remain (`:408-439`).
* Raw invalid JSON now has a direct unavailable case (`:461-463`). The `missing payload` case uses `runTraceRowWithoutPayload` at `:199-204`, and the table dispatch at `:471-512` selects it, so the member is actually absent; null, scalar, array, and object payload cases remain.

The owned peer still has one listener and worker, a four-second peer read/write deadline, client/listener/accepted-connection cleanup, and a five-second worker join bound (`:35-123`). The single request assertion checks one received line, `run_get`, exact requested run ID and exactly two request keys (`:125-149`). Positive assertions precede request waiting, so the typed-unavailable stub reaches the value assertion without waiting for a request; runtime behavior is unverified.

The other v3 matrix remains materially represented: all ten safe kinds, all 24 valid standing pairs, response ownership identity, valid foreign nonblank expected identities, duplicate selected IDs across retention, selected timestamp/ID validation, source order, returned-count 0/1024/1027 and newest retention, required trace presence with non-atomic zero/positive byte combinations, `u64` bounds, command execution optional metadata and safe-integer bounds, refusal class, and synthetic raw-field exclusion. Prefix assertions disclaim source completeness and no byte/row consistency rule is introduced.

## Review correction and limits

The prior independent fixture critic (this reviewer) failed to catch the blank-expected-input contradiction in its earlier review and supplement. Root independently found it. This re-review verifies that finding and refines it as above. I also found the distinct duplicated unknown-settlement test row during this fresh pass. The original review artifacts remain unchanged.

No fixture, source, status, debt, or Git edit was made. The compiler token remains with root; no Go RED, test, runtime, or cleanup execution is claimed. Graft retrieval preceded direct source inspection: approximately 27,037 tokens saved (~$0.02) over two calls this turn.
