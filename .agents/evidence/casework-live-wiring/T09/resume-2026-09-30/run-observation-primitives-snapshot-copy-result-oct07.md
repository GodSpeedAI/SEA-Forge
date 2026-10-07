# Run observation primitive snapshot-copy result — 2026-10-07

## Provenance and scope

This immutable copy result follows `run-observation-primitives-snapshot-copy-assignment-oct07.md` (SHA-256 `cc66a0069b0cf06e30531260316502e10bb907767cc2f91b32e4d4054d241bce`). Exactly the six released current primitive files were added to the existing detached snapshot worktree at `/tmp/sea-casework-primitives-74f86f0-oct07/apps/godspeed-casework-go/internal/server/`. Before the Add operations, all six destination paths were checked absent. Each destination was created through native `apply_patch` using the full original file contents read directly from the primary workspace.

No primary source file changed. No manager scaffold, manager fixture, worker, other source file, dependency, AGENTS file, hook, or Git state was edited. No formatter, test, compiler, scanner, build, or Git command was run. No deviations from the six-file copy scope.

## Exact original/snapshot comparison

Each per-file `cmp -s` returned exit code 0. SHA-256 and byte counts match for each original and its snapshot:

| File | SHA-256 (original = snapshot) | Bytes (original = snapshot) | `cmp` |
| --- | --- | ---: | ---: |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` | 193 | 0 |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` | 10,456 | 0 |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` | 26,425 | 0 |
| `run_observation_retained_policy_test.go` | `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7` | 13,200 | 0 |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` | 3,208 | 0 |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` | 11,463 | 0 |

Combined bytes: 64,945. The total original and snapshot counts each report 129,890 because `wc -c` covered both sets.

## Verification boundary

This proves byte-identical copying only. An independent renderer critic must verify the six-file closure and exact copy before root runs isolated full Go gates. No test, package, module, or runtime result is claimed here.
