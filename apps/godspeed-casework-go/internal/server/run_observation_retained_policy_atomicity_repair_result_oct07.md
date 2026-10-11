# Retained policy fixture atomicity repair — result receipt

Date: 2026-10-07. Source-only. No tests, compiler, scanner, formatter,
typecheck, runtime, or Git command was run.

## Frozen files and preimage

| File | SHA-256 | Bytes |
|---|---|---:|
| `run_observation_retained_policy_test.go` | `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7` | 13,200 |
| `run_observation_retained_policy_atomicity_repair_assignment_oct07.md` | `dce74dd7af4e80297aa71d4dd2f643d9468ee63be8a366e2468a9c3dbde74f57` | — |
| `run_observation_retained_version.go` | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` | unchanged |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` | unchanged |
| `run_observation_manager.go` | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` | unchanged |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` | unchanged |

Before editing, the root-supplied JSON archive's `exact_text` decoded to 12,288
UTF-8 bytes with SHA-256
`d1b919e4e432afc42940fb6b4da632c8d631a9ff016ed8e0e2eea0075523c8f0`, exactly
equal to the live test. The assignment record was written before the test edit.

## Exact repair scope

Only the new policy test file changed. The safe-copy assertion helper now takes
the caller's pre-call deep snapshot and canonical image as explicit arguments;
it compares the original caller object and canonical bytes after candidate
construction and again after mutating the returned clone. Every relevant
ordinal overflow, generation overflow, terminality, and final-marker case
captures both baselines before calling the candidate builder.

The generation-overflow test now performs the marker image-length assertion
before calling the mutation-based safe-copy probe. The probe checks full state
and canonical image equality before mutating the result; no image/length
assertion follows the mutation. Terminality image nonretention is also checked
before the returned safe copy is mutated.

Read-only unified comparison against the exact archived preimage shows changes
only in the safe-copy helper and its calls/baseline captures in those tests.
The existing assertions, expected availability statuses, recovery, terminal
classification, response counts, and invalid-input cases remain present and
unchanged. No production helper, manager, original focused helper test, or
other fixture changed.

## Review and verification boundary

This addresses the two exact findings in the independent REJECT review. The
new test candidate remains pending a different critic's review. No RED or
GREEN result is claimed. The helper is still an unavailable stub and no
algorithm implementation is authorized by this repair. Root retains focused
RED, compiler, and implementation release decisions.
