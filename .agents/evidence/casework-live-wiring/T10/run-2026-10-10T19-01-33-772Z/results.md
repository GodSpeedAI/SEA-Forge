# E2E Test Results

| Journey | Status | Depends On | Settles | Steps | Notes |
|---------|--------|-----------|---------|-------|-------|
| L5 | FAIL | L4 | an approval opened by the operator's execution is resolved only by a distinct R-SO principal, with both principals visible in the durable record | 4/5 | Failed: guard: after both sign in, the two sessions hold different cookies and different principals |