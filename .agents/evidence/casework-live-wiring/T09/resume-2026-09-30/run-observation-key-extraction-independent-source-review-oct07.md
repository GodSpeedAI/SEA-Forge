# Run observation poller key extraction — independent source review

## Verdict

**Source review ready for root's separately authorized focused verification.** The three-string private key type and its comment moved unchanged into a same-package source file. The current manager file equals the exact recorded pre-edit manager content after removing only that comment/type block and its adjacent blank separator. The six frozen pure/fixture files retain their assigned hashes. No test, compiler, formatter, scanner, Git, or build command was run in this review.

## Original instruction and proposal alignment

Reviewed the full key-extraction assignment `run-observation-key-extraction-assignment-oct07.md` (SHA-256 `ae053d50a93421ba320ef0f2ff94d5ba895ab75bce316bd38669ffc6e3427994`), its result `run-observation-key-extraction-result-oct07.md` (SHA-256 `1b77194d29533c6e8ba4b25dcd9871dc44cb2a65da45f4c74569e62f60b39b11`), accepted encoder GREEN record, and concrete read-start proposal 270042 (`manager-read-start-testability-proposal-oct07.md`, SHA-256 `2700427c0e63f9780c47062fba2f5c7d04c07992551498b6323a015ed08790bb`). The proposal explicitly keeps this key extraction outside the future worker/read-start unit and says it must remain a separately reviewed primitive change. The new assignment follows that separation: it introduces no read-start behavior, lifecycle state, hook, method, or public API.

## Exact source proof

The result identities match current files:

- New key source [`run_observation_key.go`](/home/sprime01/projects/sea-rs/apps/godspeed-casework-go/internal/server/run_observation_key.go): `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b`.
- Manager [`run_observation_manager.go`](/home/sprime01/projects/sea-rs/apps/godspeed-casework-go/internal/server/run_observation_manager.go): `ab9f1c35757aecd8c8c02de300a95205a1bd07aa5e39ebe8ae056dbc8878fe62`.

The stored pre-list manager snapshot `run_observation_manager.go.before-list-fixture-oct07.json` contains a `content` value whose exact extracted bytes hash to the assigned pre-edit SHA-256 `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`. A direct range comparison of that snapshot's key comment/type declaration against the new file returned `cmp=0`. Removing that original comment/type range and its adjacent blank separator from the exact pre-edit snapshot yielded the current manager file byte-for-byte (`cmp=0`), with current hash `ab9f1c35757aecd8c8c02de300a95205a1bd07aa5e39ebe8ae056dbc8878fe62`.

The moved declaration remains exactly `runObservationPollerKey` with private `string` fields `caseID`, `runID`, and `planItemID`, in the original order and spelling. The original comment is unchanged. The new file contains only `package server`, that comment, and that type declaration. The manager retains the same-package map and poller references, with no replacement alias, method, field, import, signature, or semantic alteration.

The sole textual caveat is the physical diff removes the declaration/comment plus the adjacent blank separator (seven removed diff lines: comment, five-line type declaration, and blank line). That is the expected source-file separation and has no semantic effect. The assignment's “five source lines” wording counts only the type block and omits its comment/blank; its operative direction and result correctly say to move/remove the comment and declaration. This is the only difference in phrasing, not a source deviation.

## Frozen closure identities

All six frozen files match the exact assignment/result hashes:

- `run_observation_manager_test.go`: `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.
- `run_observation_retained_version.go`: `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`.
- `run_observation_retained_version_test.go`: `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`.
- `run_observation_retained_policy_test.go`: `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.
- `run_observation_poller_image.go`: `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`.
- `run_observation_manager_retained_image_test.go`: `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`.

The package-private type remains the shared key used by the retained helper and encoder; moving its declaration does not change identity, key grammar, or closure behavior. The unchanged manager fixture remains deliberately unwired. This review approves source readiness only; formatting and focused/full-closure test evidence await root's new grant.
