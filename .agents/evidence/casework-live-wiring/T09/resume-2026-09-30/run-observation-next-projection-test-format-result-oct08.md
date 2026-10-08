# Next projection test formatting result

Date: 2026-10-08

## Full bounded assignment

This was a FORMAT-ONLY change to `apps/godspeed-casework-go/internal/server/run_observation_delta_test.go`. The canonical Go gate had stopped on formatting before running tests. Only this test file could be edited. Before editing, archive its exact current bytes and SHA-256 in `next-projection-test-format-preimage-oct08.json` using a machine-derived, lossless representation. Read the source and inspect read-only `gofmt -d` output. Apply only the formatter's whitespace changes with `apply_patch`, preserving all tests, assertions, fixture names, literals, and semantics. Verify the postimage is byte-equal to formatter stdout and `gofmt -d` is empty. Record the pre/post identities, exact formatting delta, and limitations. Do not run compiler, tests, gates, Git, or change another source file.

## Source and capture identities

The pre-edit test source was 15,232 bytes, SHA-256 `66e368ffdea1839aca74a6c242cb947f6d97fca45a2e228574ec4d1c3adeedbc`. Before editing, its exact bytes were archived as a base64-wrapped JSON record at `next-projection-test-format-preimage-oct08.json`, 20,493 bytes, SHA-256 `92be57b9eaf99391f0851ef1b09cd0ac2a84a206d1fcc3a1e06583cec113ff35`. Decoding the archived payload produced a byte-for-byte match with the then-current source and its recorded length and hash.

The formatted source is 15,263 bytes, SHA-256 `48c6d28e658b8bd4e8fd3c6650ff26c73b0c791f35feff83d056ff8f501c89ed`. Its bytes exactly equal the stdout bytes from read-only `gofmt apps/godspeed-casework-go/internal/server/run_observation_delta_test.go`. A subsequent read-only `gofmt -d` exited 0 with empty stdout and stderr.

## Exact change and scope

The preimage and postimage differ only in the alignment whitespace of the three `execution`, `settlement`, and `observationState` fields in the `runObservationWatermark` literal at test lines 127–129. The captured unified diff contains only those three replacements; no test, assertion, identifier, fixture, or literal was changed. The only source file edited by this task was the authorized test file. The implementation file `run_observation_delta.go` remained at 5,599 bytes, SHA-256 `5241e656d65f86a315e37df44b48d5eaae865c92cf80cc306c9bf4fc0fd7e128`.

This is a formatting result only. No AST-level comparison, compile, test, or runtime claim is made. No compiler, test, canonical gate, formatter write mode, Git command, or other source edit was performed.
