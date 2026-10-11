# Existing fixture correction after the broad focused gate

Date: 2026-10-08
Builder: fresh Luna retained_helper_format_builder, different from last
manager/worker builder. Critic: renderer_resume_critic.

Read FULL original manager Unit1 assignment/revision6/lifecycle preregistration,
algorithm granta886842c/resultb1c7e713/source review425909+erratum843dc54f,
focused gate grant5cc99e87 and actual resultdab62acb/layout0d2cebf.
Root independently compared all six actual archives and read actual failures;
53 top-level tests ran:50 pass/3 fail;22 nested pass. All seven authority/
terminal groups passed. The current implementation is not lifecycle GREEN.

Root source-grounded diagnosis: (1) stale fixture sets BOTH Store revision
and Relay head to cursor-old, so they agree and are not stale; the real guard
has no basis to reject a label containing 'old'. (2) empty-list retry expects
one perspective call, conflicting with the newer mandatory initial/post-list/
pre-attachment/final-handoff checks. (3) cancellation fixture waits only for
the second actual read; all workers are launched before wait, but scheduling
does not guarantee the first read began before cancellation. Its two-each
count needs an actual first-read gate. Fix these or explain contrary evidence;
NEVER add production cursor magic, cache authorization, serialize workers or
force reads after Stop/cancel to satisfy a false test oracle.

Allowed behavior edits ONLY in three named tests within
apps/godspeed-casework-go/internal/server/run_observation_manager_test.go,
starting exact SHAaf2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4
/55815B. FIRST exact native JSON preimage with path/source/actualsha/bytes,
NEW run-observation-existing-fixture-correction-preimage-oct08.json.

1. TestRunObservationManagerGuardRefusesBeforeListButValidParentReachesRead:
   keep Store snapshot cursor-old, make Relay cursor-current BEFORE Prepare
   using the existing fake fields. Preserve every zero-DTO/nil-lease/unavailable,
   zero-list/read and valid-parent positive assertion. No special production
   check against fixture names or arbitrary cursor strings.
2. TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList:
   preserve pre-admission refusal's ZERO perspective/list work, exact retry DTO,
   one list, zero trace, successful lease/drain. For successful empty retry
   assert FOUR perspective checks, corresponding to required boundaries above;
   update only that exact expected count/diagnostic, do not weaken to a vague
   nonzero/minimum or remove the refusal/DTO assertions.
3. TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry:
   add an actual first-read signal inside the existing fake trace callback,
   guarded to signal only on its first invocation. Wait with the existing
   bounded semantic-failure helper for BOTH actual first and second read starts
   before cancel. Preserve exactly two-each final list/read count and all
   zeroDTO/nillease/actualreturn/JOIN/distinctretry/capacity assertions. No sleeps,
   lifecycle mutation, production hooks or assumption that launched means read.

No other test/function/source behavior changes. Manager22005e5c, workerb0fde3fe,
authority fixture889f9648, failurefixtureccbbe234 and six primitives frozen.
No compiler/test/vet/scanner/formatter/build/Git/status/debt/public changes;
root retains sole compiler token IDLE. Native apply_patch sole persistent writer.
Use existing tab style; report formatting obstacle rather than gofmt -w.
Write one concise NEW run-observation-existing-fixture-correction-result-oct08.md
with actual machine-derived preimage/final/frozen identities and bounded diff.
Explain every material difference and why each assertion is strengthened or
made reachable, not weakened. Different independent critic gets FULL this and
original instructions plus resulting implementation, exact preimage/diff.
Insufficient evidence means REJECT/fresh builder. Source approval precedes
fresh sole-owner full focused race rerun; actual captures before next gate.
