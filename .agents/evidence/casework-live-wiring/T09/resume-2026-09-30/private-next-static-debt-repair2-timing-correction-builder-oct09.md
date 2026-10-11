# Private Next debt repair-2 timing correction builder record

Date: 2026-10-09. This immutable record captures one source-supported wording
repair in CW-41. It makes no source, compiler, test, runtime, gate, or Next
completion claim.

## Exact fresh grant

> New VERY SMALL fresh documentation repair grant; your frozen Next source stays untouched. In .agents/DEBT.md CW41 followup, prior builder next_debt_builder wrote wrong phrase: separate post-wake test 'did assert consumption before its failed second authorization check'. Actual test asserts len(wake)==0 only AFTER nextWithProjector returns second-check error. Read full DEBT before editing; inspect actual exact test function and prior docs correction record7f598157. Native patch only change that phrase to accurate AFTER-return timing, preserve everything else. Critic is freezing narrow rejection; wait for/read private-next-static-debt-repair2-fresh-correction-independent-review-oct09.md when present, then write NEW private-next-static-debt-repair2-timing-correction-builder-oct09.md with exact grant, finalDEBTSHA and one-phrase diff, no broader claims. No source/status/gates/compile/tests/Git mutations. Do not delay on broad recon or reread all policies—one source-supported sentence only. Root will send to same independent critic, who is also reviewing source in parallel.

## Reviewed evidence

Read the full `.agents/DEBT.md` and the independent rejection
`private-next-static-debt-repair2-fresh-correction-independent-review-oct09.md`
(which identifies predecessor correction record SHA
`7f598157012e8e9c649699d9579c97ad1891ef201ac3744fc12bff9fbb8ee8a1`). Inspected
the exact `TestRunObservationNextRechecksAuthorizationAfterWake` function in
`apps/godspeed-casework-go/internal/server/run_observation_next_test.go`.
That function calls `nextWithProjector`, checks its failed second authorization
result, and only then checks `len(lease.wake) != 0`; therefore the assertion
establishes the wake is empty after the call returns, not when it was consumed
relative to the authorization callback.

## Exact phrase change

Only the inaccurate timing phrase in CW-41 was corrected:

```diff
-  post-wake test did assert consumption before its failed second authorization
-  check. The two recovery fixtures allowed a fitting third read to overwrite
+  post-wake test asserted `len(lease.wake)==0` only after `nextWithProjector`
+  returned with the failed second authorization check. The two recovery
+  fixtures allowed a fitting third read to overwrite
```

All other bytes and files were preserved. The correction does not revise or
approve any broader source-review finding.

## Final identity

`.agents/DEBT.md` SHA-256 after the single phrase correction:
`989f048300718cb94ffc9cd1d4cc9654ef32aa5f0bd73042a52bfa25e96d6149`.

No source, status, gate, compiler, test, or Git mutation was performed.
