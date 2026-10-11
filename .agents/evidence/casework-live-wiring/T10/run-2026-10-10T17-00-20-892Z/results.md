# E2E Test Results

| Journey | Status | Depends On | Settles | Steps | Notes |
|---------|--------|-----------|---------|-------|-------|
| L0 | FAIL | — | the stack is the real, non-fixture, authenticated one | 1/2 | Failed: gateway is the live non-fixture build and the kernel is ready |
| L1 | BLOCKED | L0 | a case exists in the kernel because the UI committed it | — | Blocked by L0 |
| L2 | BLOCKED | L1 | the UI shows the case's real standing, not an invented one | — | Blocked by L1 |
| L3 | BLOCKED | L2 | an operator-proposed item is durably in the plan with its proposer, and the UI shows it | — | Blocked by L2 |
| L4 | BLOCKED | L2, L3 | an executed item is durably active, settled and completed, and its downstream unlocks over SSE without a reload | — | Blocked by L2 |
| L5 | BLOCKED | L4 | an approval opened by the operator's execution is resolved only by a distinct R-SO principal, with both principals visible in the durable record | — | Blocked by L4 |
| L6 | BLOCKED | L4 | a settled item's captured artifact opens in the dock with a verified digest, and only the needed renderer chunk is fetched | — | Blocked by L4 |
| L7 | BLOCKED | L5, L6 | a case closes when its work completes, reopens with a reason, and a second case is terminated with a reason, all as durable kernel events offered only by the snapshot | — | Blocked by L5 |
| L8 | BLOCKED | L7 | a typed question is answered by the real kernel, narrated from the returned disclosure only, interruptible and resumable without a second ask | — | Blocked by L7 |
| L9 | BLOCKED | L8 | the whole live ladder ran in one continuous session per user with no error-level console output and no page error | — | Blocked by L8 |
| L-RECOV | BLOCKED | L8 | the UI is honest and consistent with the durable files across a corrupt artifact, an SSE drop and kernel/gateway crashes | — | Blocked by L8 |