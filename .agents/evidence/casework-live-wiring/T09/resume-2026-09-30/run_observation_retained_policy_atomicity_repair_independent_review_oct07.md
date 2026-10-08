# Independent review: retained policy fixture atomicity repair

Date: 2026-10-07  
Disposition: **APPROVE the new policy fixture source for a focused semantic RED attempt only**

## Scope and identities

Reviewed the full fresh-repair assignment/result, complete original policy-fixture assignment, original helper assignment and algorithm preregistration, root algorithm decisions, prior policy-fixture rejection, full repaired test, exact archived preimage, and frozen helper/manager source/test identities. This is source-only: no test, compiler, scanner, formatter, typecheck, runtime, or Git command was run.

| Artifact | SHA-256 |
|---|---|
| Fresh-repair assignment | `dce74dd7af4e80297aa71d4dd2f643d9468ee63be8a366e2468a9c3dbde74f57` |
| Fresh-repair result | `4e3f98bed27582bbd7e1b439e342dca0bab89a0b75d24e4ff43d1fd674cef4fe` |
| Original policy-fixture assignment | `ed290103a075fae97b438d3ed755ac4ef6e3553ed8712a973efbb5f607504a83` |
| Current policy fixture | `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7` (13,200 bytes) |
| Retained helper, frozen | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` |
| Original helper fixture, frozen | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| Manager source/test, frozen | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` / `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| Prior rejection | `a429d0cc6a3865ff14115e9470c14f873681ae0d9feee4ec2a235382e584cc93` |
| Root algorithm decisions | `9dcf3886f98d4416fd1bbcded12ecc939a9f5281b9bebbe3e929dd865185aa3a` |

The policy fixture is the only code-adjacent file changed by the repair. I decoded the root's `exact_text` UTF-8 preimage for the original `d1b919...` file and independently diffed it against the live file. The diff is limited to the safe-copy assertion helper signature/body, its pre-call snapshot/image captures and arguments, and moving the generation length assertion before the helper's mutation probe. The preimage decoded to 12,288 bytes with SHA-256 `d1b919e4e432afc42940fb6b4da632c8d631a9ff016ed8e0e2eea0075523c8f0`; current bytes/hash match the repair result.

## Prior findings closed

1. **Caller prior mutation is now checked against pre-call values.** The helper takes `priorBefore` and `priorImage` explicitly (`run_observation_retained_policy_test.go:13-18`), checks complete struct and canonical bytes before and after mutating the returned copy (`:24-42,55-63`). Every marker/overflow call site captures the clone and marshaled image before `buildRunObservationRetainedCandidate`: ordinal overflow `:85-102`, generation overflow `:106-128`, terminality `:145-167`, and final-marker refusal `:175-194`. This closes the post-call-baseline defect from the prior rejection.

2. **Generation marker budget assertion now precedes result mutation.** Generation overflow captures the candidate result image and compares its length to the prior image at `:121-127`; only after that check does it call the mutation-based deep-copy assertion at `:128`. No subsequent assertion depends on the deliberately mutated result. The corrected helper compares the returned state and its image to the expected pre-call prior plus requested marker before its alias probe.

## Full policy and fixture assessment

The repaired file keeps the original four policy fixes intact:

* Exact nonterminal ordinal, terminal ordinal, and generation-overflow availability outcomes are asserted (`:67-129`). Caller state is cloned and serialized before each operation. The return state is compared to a full deep copy with only the required marker, and the caller object/image must remain identical.
* Terminal classification is execution-only across all three terminal standings with unsettled settlement, and active with both accepted and rejected settlements (`:131-170`). The generated command frame uses valid `completed` metadata; all execution/settlement values are in `internal/contract/contract.go:426-436`. Oversize derives from a very long exact opaque event ID, not invalid enum/status/timestamp data.
* Terminal-retention-failure and stop-scheduling markers reject a later fitting candidate and preserve safe values, marker, generation, timestamp and ledger (`:172-197`). The mutation probe is last in each case.
* The fitting total-count case preserves `total=5`, `retained=1`, `omitted=4`, `truncated=true`, distinguishes it from ledger size, and accepts a later lower total (`:199-235`). This matches `RunTraceSnapshot`'s meaning: total allowlisted frame count before port retention, not source-journal completeness (`internal/ports/run_trace.go:10-20`).

The separately authorized recovery/key/count tests remain present (`:237-290`): read-unavailable recovers to current; requested/snapshot and prior/requested key mismatch return invalid/nil; negative and below-window totals return invalid/nil. They do not add adapter validation, production code or lifecycle tests. All five test names retain the `TestRunObservationRetainedPolicy` prefix, so the planned selector reaches this file. Helpers are synchronous, use no channels/timers/waits, and check nil pointers/states before dereferencing; no setup deadlock or failure-path panic is apparent from source inspection.

These tests align with root decisions: ordinal overflow returns retention-unavailable unless the refused candidate is terminal, generation overflow returns stop-scheduling, final markers refuse subsequent publication, read-unavailable can recover, key/count validation is local, and candidate construction cannot mutate the prior. The fixture does not broaden the helper beyond its bounded trusted `RunTracePort` input contract.

## Disposition and limits

Approve this repaired new fixture as suitable for one focused semantic RED attempt using `^TestRunObservationRetainedPolicy` against the deliberate helper stub. This source approval clears the two findings in review `a429d0cc...` and restores the four policy dimensions that review said were missing. It is not evidence that the file compiles or has run. The focused RED remains held until the exclusive compiler owner captures and independently reviews a current run.

This also clears the fixture blockers from the bounded algorithm design review, but only at source-design level: the root-ratified algorithm remains unimplemented and cannot be approved by this review. Do not treat the earlier three-manager/eight-helper negative evidence bundle as independently verified; its separate capture-comparison provenance gap remains. No manager lifecycle, actual retained algorithm, lifecycle integration, `Next`, SSE, production behavior, or T09 settlement is approved here.

No test/compiler/runtime result is claimed. Graft retrieval saved ~33,970 tokens (~$0.03) this turn.
