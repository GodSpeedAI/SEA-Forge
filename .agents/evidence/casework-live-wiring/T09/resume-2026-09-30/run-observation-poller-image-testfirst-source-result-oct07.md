# Run observation poller image TESTFIRST source result — 2026-10-07

## Provenance and release boundary

This is a new immutable result record; it does not replace or claim to reconstruct any earlier assignment, original lifecycle document, review, or test evidence. It follows the original prep assignment `run_observation_poller_image_testfirst_assignment-oct07.md` (SHA-256 `5c8b57bdf3b462fc366783bb304f1aff45af7c4f36a7e58dc3e7a55dada3a2c0`) and the explicit source release recorded in `run_observation_poller_image_testfirst_release_assignment-oct07.md` (SHA-256 `4877d97deee28b572b6df553a4e7ac47e51aec26216c22e54e998ee9de49b5e8`). The accepted scope was exactly two new files, with a compile-safe fixed generic error/nil throwing seam and a seven-case test fixture; no encoder algorithm implementation.

The source-only fixture was created under root's release. No tests, compiler, formatter, scanner, or Git command was run. The result is for a separate renderer critic before root runs the focused RED test. It is not runtime evidence and does not assert that the tests pass.

## New files and exact hashes

- `apps/godspeed-casework-go/internal/server/run_observation_poller_image.go` — SHA-256 `32f57bd8aa6edb9c20b645e3138b4b9e9fc02076518a34587253a3af31af328e` (63 lines). Defines the private schema version; fixed phase codes 0–3 and initializer-result codes 0–5; fixed generic unavailable error; branch-specific ordered wrapper structs; and `marshalRunObservationPollerImage`, which currently returns nil bytes plus the fixed error for every input. There is no encoding algorithm, manager admission, read, publication, lifecycle map, reference handling, port, cancellation, or join logic.
- `apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go` — SHA-256 `5f4e1253fb3656df570a8f941cbca1686f03a6443b56e9821a7b08e9cae06e70` (299 lines). Adds seven top-level tests sharing prefix `TestRunObservationPollerImage`.

## Frozen-file byte identity

The five release-frozen files were rehashed after the new-file edits and match the supplied baseline hashes:

- `run_observation_manager.go` — `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`
- `run_observation_manager_test.go` — `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`
- `run_observation_retained_version.go` — `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`
- `run_observation_retained_version_test.go` — `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`
- `run_observation_retained_policy_test.go` — `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`

## Fixture claims and limits

The seven fixture cases cover: (1) nil-current exact ordered shape with only schema, phase, initializer result, and key (no fake helper metadata); (2) current embedded as a raw JSON object, exact current-branch schema/control prefix, one key occurrence, deterministic bytes, and helper bound; (3) exact combined cap and one-byte-over refusal; (4) unchanged combined byte width for availability marker variants and deep-copy/non-alias checks; (5) oversized nil-current key yielding only fixed generic refusal; (6) no source-state mutation or returned-byte alias for either branch; and (7) invalid phase/result codes and current/key mismatch yielding nil bytes plus fixed generic refusal.

The exact-cap fixture intentionally differs from the nearby helper-only fixture's single-occurrence key-padding method. Here the candidate grows the retained helper frame timestamp from `2026-10-07T12:00:00.1Z` to `2026-10-07T12:00:00.12Z`, a measured one-byte growth in the helper's canonical JSON while keeping the padded key and all outer controls unchanged. Because the combined wrapper's `current` field is a `json.RawMessage` object, the wrapper adds the raw helper bytes with the same fixed prefix/suffix; the test checks `len(candidateInner) == len(exactInner)+1` and predicts combined length as `len(exact)+len(candidateInner)-len(exactInner)`, exactly cap+1. It then requires the encoder to return nil bytes and the fixed refusal. This is a direct structural size accounting assertion, not runtime proof; the algorithm stub means the accepted output assertions are expected to fail until root installs the implementation.

The marker-width fixture checks each defined retained availability code from `retainedCurrent` through `retainedStopScheduling`, selecting stopping phase for terminal-retention failure and stop-scheduling. It asserts equal serialized length to an exact-cap prior, prior deep equality, and no aliasing in the marker copy. It does not prove manager behavior or publication ordering.

The serializer-only oversized nil-current case proves a generic refusal contract for that input, not “no admission” or “no read.” Those are manager integration properties and remain untested here. The fixtures also do not cover manager initialization coalescing, waiter reference cleanup, read retirement, worker join, capacity release, ready/stop signal ownership, or stop/detach-versus-ready races; those require a separate lifecycle fixture before lifecycle implementation. Shared poller outcome must not be confused with identical per-Prepare DTOs: owner/shared read-attempt counts and capture times may differ under the recorded root clarification.

## Deviations / review notes

No scope deviation from the accepted two-file release. The +1 boundary construction uses the one-byte timestamp extension described above rather than changing a single-occurrence key, and explicitly checks both the helper delta and its one-for-one combined-wrapper effect. The current branch test asserts canonical schema/control order through its exact prefix. As noted, this source-only step deliberately leaves the encoder as a throwing seam. No claims are made about actual manager admission, polling, reads, publication, lifecycle synchronization, or runtime success.
