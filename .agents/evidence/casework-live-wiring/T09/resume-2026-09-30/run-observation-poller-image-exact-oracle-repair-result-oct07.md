# Poller image fixture exact-oracle repair result

Date: 2026-10-07  
Status: source-only fixture repair complete; sent for independent renderer
review before any focused RED run.

## Immutable task and review evidence

- Accepted original assignment: `run_observation_poller_image_testfirst_assignment-oct07.md`, SHA-256 `5c8b57bdf3b462fc366783bb304f1aff45af7c4f36a7e58dc3e7a55dada3a2c0`.
- Original source release: `run_observation_poller_image_testfirst_release_assignment-oct07.md`, SHA-256 `4877d97deee28b572b6df553a4e7ac47e51aec26216c22e54e998ee9de49b5e8`.
- Prior source result: `run-observation-poller-image-testfirst-source-result-oct07.md`, SHA-256 `4fdcde6655b358da2f6308697fc378e9738092a418c447bc50c816c7d0c01947`.
- Independent rejection repaired: `run-observation-poller-image-testfirst-source-independent-review-oct07.md`, SHA-256 `9a61fee9adc0509eedcf1c9c0478cff345bab36fdee8ab9f27cd415b742724fc`.
- Full fresh repair instruction archived before patch: `run-observation-poller-image-exact-oracle-repair-assignment-oct07.md`, SHA-256 `cf67136cbb4f5e89e571535a96847ffbcd75c9a593e189b784ef8a3181fedd78`.

## Resulting files

- `apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go` — SHA-256 `8e741f7604f49bf28674e669f820ec9e696abd84ad4c60bb2a145b36123496f8`.
- `apps/godspeed-casework-go/internal/server/run_observation_poller_image.go` — SHA-256 `32f57bd8aa6edb9c20b645e3138b4b9e9fc02076518a34587253a3af31af328e` (unchanged throwing seam).

## Exact fixture repair

The current-branch test now calls the unchanged helper marshaler on the same
state, concatenates those canonical bytes between the literal ordered wrapper
prefix and a closing brace, and compares the entire expected wrapper against
the poller encoder result. This checks the complete current value, exact
canonical helper ledger/metadata, raw JSON embedding, field order and that
`current` is last with no additional wrapper fields.

The combined +1 test still proves a one-byte helper-image increase from the
timestamp extension and that the candidate helper remains below the inner
cap. It now independently builds the expected candidate wrapper from the
literal prefix, candidate helper bytes, and closing brace and asserts that
oracle is exactly `runObservationRetainedImageMaxBytes+1`. The expected
candidate length does not use output from the poller encoder. Existing
production-call refusal assertions and all seven prefixed tests remain.

## Frozen identities and exclusions

All five frozen files match their release identities:

- `run_observation_manager.go` — `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`
- `run_observation_manager_test.go` — `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`
- `run_observation_retained_version.go` — `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`
- `run_observation_retained_version_test.go` — `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`
- `run_observation_retained_policy_test.go` — `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`

Material deviation: only the two precise independent fixture oracles required
by the rejection were added. No top-level test was removed or renamed. No
production source, algorithm, runtime behavior, manager behavior, fixture
scope, cap, error contract, or frozen file changed. No tests, compiler,
formatter, scanner, or Git command was run. No RED or runtime claim is made.

The repaired source/result pair is ready for a different renderer critic;
focused RED remains gated on that review and root's next authorization.
