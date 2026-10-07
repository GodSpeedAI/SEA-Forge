# Run observation key extraction result — 2026-10-07

## Release provenance

This result follows the complete pre-edit assignment `run-observation-key-extraction-assignment-oct07.md` (SHA-256 `ae053d50a93421ba320ef0f2ff94d5ba895ab75bce316bd38669ffc6e3427994`). Root's explicit release relied on the independently accepted pure encoder GREEN record `run-observation-poller-image-green-independent-acceptance-oct07.md` (SHA-256 `ef3418e5c699f8d4d5b3f782029f7ce8eb884bcbbde7930806c54a137c9722d1`), which reports seven top-level and three nested tests passing with eight archived captures compared byte-identically. That GREEN does not cover manager lifecycle.

## Exact source movement

Only the two authorized source paths changed:

1. New `apps/godspeed-casework-go/internal/server/run_observation_key.go`, SHA-256 `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b`:

   ```go
   package server

   // runObservationPollerKey is the exact source identity used to share a poller.
   type runObservationPollerKey struct {
       caseID     string
       runID      string
       planItemID string
   }
   ```

2. Existing `apps/godspeed-casework-go/internal/server/run_observation_manager.go` changed from its pre-edit SHA-256 `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` to `ab9f1c35757aecd8c8c02de300a95205a1bd07aa5e39ebe8ae056dbc8878fe62` by removing only the identical comment and type declaration. The declaration move is the complete source deviation and the explicit exception to the former frozen manager-file identity. No other source line was intentionally changed; a different renderer critic is tasked to verify the complete original instructions and prove the manager remainder is unchanged.

## Dependency closure

Graft located the type at the prior manager declaration and found no indexed callers; direct `rg` over the exact pure closure confirms the five pure files/tests use the key only as a value/type:

- retained helper source `run_observation_retained_version.go` uses it in retained state, candidate building, and serialization;
- image encoder `run_observation_poller_image.go` uses it as the wrapper key;
- `run_observation_retained_version_test.go`, `run_observation_retained_policy_test.go`, and `run_observation_manager_retained_image_test.go` use it in test fixtures and key cases.

The helper fixture also supplies `retainedTestKey`, `retainedTestState`, and clone helpers used by the other two tests. These files contain no manager constructor, authorization, lease, poller, or stop method reference. The new key source plus manager declaration removal are the two source files in this primitive extraction unit. This establishes source dependency closure only; no new test or compilation evidence was produced.

## Frozen identities rechecked

- `run_observation_manager_test.go`: `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.
- `run_observation_retained_version.go`: `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`.
- `run_observation_retained_version_test.go`: `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`.
- `run_observation_retained_policy_test.go`: `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.
- `run_observation_poller_image.go`: `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`.
- `run_observation_manager_retained_image_test.go`: `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`.

## Verification and next boundary

No tests, compiler, formatter, scanner, Git, or build command was run. This source movement has not been independently reviewed or verified. Root plans separate isolated-HEAD verification of the primitive source closure before a coherent primitive checkpoint; that planned work is not evidence from this result. The intentional unwired manager fixture and full T09 gates remain unchanged and are not weakened or removed.
