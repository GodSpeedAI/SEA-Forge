# E2E Test Results

| Journey | Status | Depends On | Settles | Steps | Notes |
|---------|--------|-----------|---------|-------|-------|
| L0 | PASS | — | the stack is the real, non-fixture, authenticated one | 5/5 | — |
| L1 | PASS | L0 | a case exists in the kernel because the UI committed it | 6/6 | — |
| L2 | PASS | L1 | the UI shows the case's real standing, not an invented one | 5/5 | — |
| L3 | FAIL | L2 | an operator-proposed item is durably in the plan with its proposer, and the UI shows it | 1/2 | Failed: an empty proposal is refused client-side with no durable effect |
| L4 | BLOCKED | L2, L3 | an executed item is durably active, settled and completed, and its downstream unlocks over SSE without a reload | — | Blocked by L3 |