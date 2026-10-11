# Next projection test format repair — independent review

Date: 2026-10-08  
Verdict: **APPROVE format-only source repair.** This verifies the test-file
formatting change only; it does not approve algorithm behavior, runtime,
manager/Next integration, or T09 completion.

## Authority and identities

Read the complete bounded grant and its identity erratum in
`run-observation-next-projection-test-format-result-oct08.md` and
`run-observation-next-projection-test-format-result-identity-erratum-oct08.md`,
plus the lossless preimage
`next-projection-test-format-preimage-oct08.json`.

The preimage archive is 20,532 bytes, SHA-256
`92be57b9eaf99391f0851ef1b09cd0ac2a84a206d1fcc3a1e06583cec113ff35`. The
embedded source decodes to 15,232 bytes, SHA-256
`66e368ffdea1839aca74a6c242cb947f6d97fca45a2e228574ec4d1c3adeedbc`.
The erratum correctly supersedes the original result's transcribed archive
size of 20,493 bytes; it does not change the source preimage identity.

Current `run_observation_delta_test.go` is 15,263 bytes, SHA-256
`48c6d28e658b8bd4e8fd3c6650ff26c73b0c791f35feff83d056ff8f501c89ed`.
I decoded the archived preimage and verified its exact length/hash, then ran
read-only `gofmt` on that decoded file and byte-compared the formatter output
to the current source; they match exactly. A separate read-only `gofmt -d`
exited 0 with empty stdout and stderr.

## Exact diff and boundaries

The complete preimage/current diff contains only three whitespace alignments
in the `runObservationWatermark` fixture literal at current test lines 127–129:
the `execution`, `settlement`, and `observationState` fields were padded to
the formatter's alignment. No assertions, setup, literals, tests, or semantics
changed.

The algorithm remains frozen at `run_observation_delta.go` SHA-256
`5241e656d65f86a315e37df44b48d5eaae865c92cf80cc306c9bf4fc0fd7e128`. All
eleven other observation source/test files match their recorded frozen hashes.
The previous canonical gate had stopped at this exact format check before
running tests; its actual captures were archived separately. No compiler,
tests, canonical gate, formatter write mode, or Git command ran in this review.
The next canonical and full-module verification steps remain under root's
separate release control.
