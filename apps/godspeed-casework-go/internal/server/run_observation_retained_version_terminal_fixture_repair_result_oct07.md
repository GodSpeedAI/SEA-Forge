# Retained-version terminal fixture repair — result receipt

Date: 2026-10-07. Preserve the original assignment and prior review records.

## Frozen identities

| File | SHA-256 | Bytes |
|---|---|---:|
| `run_observation_retained_version.go` | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` | 5,836 |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` | 26,425 |
| `run_observation_retained_version_terminal_fixture_repair_assignment_oct07.md` | recorded in its own immutable file | — |

The source hash is unchanged from the reviewed scaffold. The test preimage was
verified before edit: archived JSON content decoded to 26,472 bytes and hash
`6ba0ab0ddbd80614d25c48d6485f1af70a75e8f670a72fac535c8ec128652d00`, exactly
matching the live file bytes. A read-only unified comparison afterward showed
only these three line replacements:

```diff
-candidateStatus := "candidate-only-status"
+candidateStatus := "timed_out"
-over.Execution = "candidate-only-execution"
+over.Execution = "failed"
-over.Settlement = "candidate-only-settlement"
+over.Settlement = "rejected"
```

No assertion, helper code, other fixture, or production source changed. These
values are valid trace vocabularies and are absent from the fixture's prior
image values. `failed` is a terminal execution standing; the settlement value
is not used to infer terminality. The candidate command status is valid for a
`command_finished` frame. Existing exact-state/image/length/ledger/pointer/
counter assertions remain in place.

## Review boundary

This repairs only the invalid terminal test inputs identified by the
independent source review and its value-matching correction. The candidate
publisher remains the unavailable stub. No tests, compiler, scanner,
formatter, typecheck, runtime, or Git command was run; no RED/GREEN result is
claimed. The files are frozen for a different guard critic. Root must review
all original instructions and this result before any test execution or
implementation release.
