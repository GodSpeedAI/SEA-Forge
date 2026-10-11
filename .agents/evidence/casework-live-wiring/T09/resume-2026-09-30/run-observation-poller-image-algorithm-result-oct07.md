# Poller image encoder algorithm result

Date: 2026-10-07  
Status: source-only pure encoder implementation complete; awaiting independent
source review before GREEN.

## Immutable instruction and reviewed RED evidence

- Algorithm assignment archived before editing: `run-observation-poller-image-algorithm-assignment-oct07.md`, SHA-256 `151f09cc3d187eaef91ddf08228199f86f35d08cda301d67e19bb81ac089ca65`.
- Original fixture assignment: `run_observation_poller_image_testfirst_assignment-oct07.md`, SHA-256 `5c8b57bdf3b462fc366783bb304f1aff45af7c4f36a7e58dc3e7a55dada3a2c0`.
- Original fixture/stub release: `run_observation_poller_image_testfirst_release_assignment-oct07.md`, SHA-256 `4877d97deee28b572b6df553a4e7ac47e51aec26216c22e54e998ee9de49b5e8`.
- Root combined-image decisions: `manager-combined-image-root-decisions-oct07.md`, SHA-256 `48ed5d0b01e25a080b8b2560462bfd6bdf4b196d9769ff917b6a9cb25b32fe96`.
- Accepted focused RED: `run-observation-poller-image-red-independent-acceptance-oct07.md`, SHA-256 `4d576aae33b45300f45082f93e9a6f3c72087e71510688872e802f6d849874d1`.
- RED result: `run-observation-poller-image-red-oct07-result.md`, SHA-256 `c0fbc9ea2558da6fd7ffb0b80e81330cb039b58421aa91f6173d4a17727f58a2`.
- Actual captured RED output: `run-observation-poller-image-red-output-oct07.raw`, SHA-256 `84fae8578513ee4730d204eabb78f358ab3b99a25a3e2ad723794cce77ec2600`.
- Fixture oracle repair assignment/result: `run-observation-poller-image-exact-oracle-repair-assignment-oct07.md`, SHA-256 `cf67136cbb4f5e89e571535a96847ffbcd75c9a593e189b784ef8a3181fedd78`; `run-observation-poller-image-exact-oracle-repair-result-oct07.md`, SHA-256 `d485d611c68f2e480b6f141003113749bd9abedc899703ff6cd2d1cede34822e`.

The accepted RED proves only the throwing seam's expected focused failures
and refusal/validation passes. It is not a GREEN result. The original fixture
was authored by a separate builder; this builder's earlier contribution was
limited to the two independent expected-byte test oracles. The independent
renderer critic authored no source.

## Implemented source

- `apps/godspeed-casework-go/internal/server/run_observation_poller_image.go` — SHA-256 `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`.
- Frozen fixture `apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go` — SHA-256 `8e741f7604f49bf28674e669f820ec9e696abd84ad4c60bb2a145b36123496f8` (unchanged).

The private marshaler validates the phase and initializer-result code ranges,
checks exact current/key identity for nonnil state, and delegates inner state
encoding to the unchanged helper. It wraps helper bytes as `json.RawMessage`
in the ordered current branch; the nil branch serializes the key once in its
ordered key object. It maps all errors to the fixed generic error and nil
bytes, and rejects only after measuring the complete wrapper if it exceeds
1 MiB. Successful `json.Marshal` results are newly owned bytes. No input
mutation, manager/lifecycle behavior, or extra validation was added.

## Frozen identities and material deviations

All five release-frozen files remain byte-identical:

- `run_observation_manager.go` — `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`
- `run_observation_manager_test.go` — `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`
- `run_observation_retained_version.go` — `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`
- `run_observation_retained_version_test.go` — `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`
- `run_observation_retained_policy_test.go` — `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`

No material deviation from the released pure-encoder contract. Only the
released source file changed. No test, compiler, formatter, scanner, Git, or
build command was run. No GREEN, lifecycle, manager admission, read
suppression, publication, or runtime claim is made. A different
source-independent renderer critic must review this source and result before
the sole compiler owner receives a separate GREEN authorization.
