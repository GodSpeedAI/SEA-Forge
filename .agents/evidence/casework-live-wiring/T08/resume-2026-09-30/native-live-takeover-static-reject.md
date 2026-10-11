# T08 independent takeover static rejection

Independent Luna critic t07_live_confirmation reviewed the original harness instructions
and result after the prior verifier's usage failure. No compile/runtime command ran for
this static review. The import-only repair does not address these findings.

1. native-events-live.ts phase one requires retained Head equal EXECUTE_ITEM response cursor.
   Execution legitimately emits several case frames; postMutationCursor returns the first
   retained advance, and later frames can move Head past that receipt before the test polls.
   This can hang. Wait for actual durable terminal trace and published event-tail state, then
   capture actual retained K; never invent a cursor or weaken retention/resync assertions.
2. Both phases require ADD_DISCRETIONARY_WORK to be offered. Current projection actionsFor
   offers item execute/complete/approve/reject and lifecycle actions; ADD is not yet exposed.
   The protected T06 API supports ADD, so the test may explicitly drive that real API to
   produce one frame without claiming it is a UI descriptor. Phase-one execute eligibility
   must be checked on its exact target, not the global action union.
3. Shared live runner uses fixed /tmp/t08-live-gw and kills children without awaiting exits.
   Use unique owned output paths and bounded actual cleanup; preserve foreign listeners/data.
   Its explicit test-only ledger assertions are durable-state evidence, not gateway authority.

Verdict: **REJECT runtime readiness.** Root assigned fresh TS harness builder t09_wiring_scope
and fresh shared-runner cleanup builder production_import_guard_builder with disjoint files.
Neither owns the compile token. Root temporarily owns it until stable source then independent
critic transfer. Original native/fetch assertions and real-kernel mutations remain required.

Correction to the prepared invocation note: its TMPDIR suggestion does not isolate a hardcoded
absolute /tmp/t08-live-gw path. The runner source must use an actual unique binary path.
No prior preparation note is a passing gate or implementation approval.
