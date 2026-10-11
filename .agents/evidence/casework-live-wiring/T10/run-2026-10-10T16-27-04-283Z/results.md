# E2E Test Results

| Journey | Status | Depends On | Settles | Steps | Notes |
|---------|--------|-----------|---------|-------|-------|
| L0 | PASS | — | the stack is the real, non-fixture, authenticated one | 5/5 | — |
| L1 | PASS | L0 | a case exists in the kernel because the UI committed it | 6/6 | — |
| L2 | PASS | L1 | the UI shows the case's real standing, not an invented one | 5/5 | — |
| L3 | PASS | L2 | an operator-proposed item is durably in the plan with its proposer, and the UI shows it | 5/5 | — |
| L4 | PASS | L2, L3 | an executed item is durably active, settled and completed, and its downstream unlocks over SSE without a reload | 5/5 | — |
| L5 | PASS | L4 | an approval opened by the operator's execution is resolved only by a distinct R-SO principal, with both principals visible in the durable record | 9/9 | — |
| L6 | PASS | L4 | a settled item's captured artifact opens in the dock with a verified digest, and only the needed renderer chunk is fetched | 5/5 | — |
| L7 | PASS | L5, L6 | a case closes when its work completes, reopens with a reason, and a second case is terminated with a reason, all as durable kernel events offered only by the snapshot | 8/8 | — |