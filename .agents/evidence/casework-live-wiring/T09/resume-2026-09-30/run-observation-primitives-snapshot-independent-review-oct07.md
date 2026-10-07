# Run observation primitive snapshot — independent source review

## Verdict

**Snapshot source review ready for root's isolated Go verification grant.** All six source/test files match their primary originals byte-for-byte. The detached snapshot is at the assigned HEAD and has exactly the six intended new untracked paths; no tracked path is modified or deleted. The six-file dependency closure is self-contained and contains no manager implementation, manager fixture, manager fake, or hidden helper dependency.

This review makes no formatter, compile, test, or runtime claim.

## Assignment and result

Reviewed the full snapshot-copy assignment `run-observation-primitives-snapshot-copy-assignment-oct07.md` (SHA-256 `cc66a0069b0cf06e30531260316502e10bb907767cc2f91b32e4d4054d241bce`), its result `run-observation-primitives-snapshot-copy-result-oct07.md` (SHA-256 `6c76814e15d8769ed12b005ea204b10dece014cdf663f213c0f851365c5d15b6`), and the complete key-extraction assignment/review and six-file source closure. The result's six-file scope is exact.

## Snapshot identity and status

The isolated worktree is `/tmp/sea-casework-primitives-74f86f0-oct07`; read-only `git rev-parse HEAD` returned `74f86f0964f92c5b1715eb0703a9b880d28cccae`. Full read-only `git status --short --untracked-files=all` listed exactly these six untracked paths and no other status entries:

- `apps/godspeed-casework-go/internal/server/run_observation_key.go`
- `apps/godspeed-casework-go/internal/server/run_observation_retained_version.go`
- `apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go`
- `apps/godspeed-casework-go/internal/server/run_observation_retained_policy_test.go`
- `apps/godspeed-casework-go/internal/server/run_observation_poller_image.go`
- `apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go`

Each original/snapshot pair was independently compared (`cmp` exit 0), and both hashes and byte counts match:

| File | SHA-256 (both copies) | Bytes (each) |
| --- | --- | ---: |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` | 193 |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` | 10,456 |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` | 26,425 |
| `run_observation_retained_policy_test.go` | `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7` | 13,200 |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` | 3,208 |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` | 11,463 |

Combined size is 64,945 bytes per set. The primary workspace's earlier key move is the only permitted difference from the former manager-scaffold baseline; that move is not present in this detached HEAD's tracked source because this snapshot was intentionally based on HEAD `74f86f0` and adds only the released six files.

## Dependency closure and excluded manager files

Graft's exhaustive `runObservationPollerKey` search found references only in these six primitive files plus the two existing manager scaffold files (`run_observation_manager.go` and `run_observation_manager_test.go`) in the primary workspace. In the isolated snapshot, those two scaffolds are absent because they are not part of the proposed six-file commit at the assigned HEAD. They are not removed, modified, or suppressed tracked tests: full `git status` shows no deletion or modification, and the assignment expressly excludes them from this primitive closure.

Within the six files, `run_observation_key.go` contains only the package-private three-string key. `run_observation_retained_version.go` owns the candidate, copy, marker, and canonical retained-image helpers; the image encoder uses that helper and key. The included retained-version fixture defines the shared `retainedTestKey`, frame/snapshot/state constructors, clone helper, and accepted-candidate assertion used by the policy and image fixtures. The policy fixture defines its own `requireRetainedPolicySafeCopy`. No manager helper or unpublished fake is required. A source search for `runObservationManager`, `Manager`, `Fake`, and `fake` returned no hits across the six snapshot files. The image test file name includes “manager”, but its tests exercise only the image encoder and retained-state helpers.

The fixture sources retain their tests; none is deleted, skipped, altered, or replaced by this snapshot. The excluded unwired manager files remain a separate scope question for lifecycle work and do not weaken or stand in for the primitive verification.

## Limits

This review confirms identity, Git snapshot cleanliness, and source dependency boundaries only. It does not claim canonical formatting, vet, race tests, full module tests, or runtime behavior. Those isolated gates require root's separate resource-preflight and capture grant.
