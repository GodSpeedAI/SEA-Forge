# E2E Test Results

| Journey | Status | Depends On | Settles | Steps | Notes |
|---------|--------|-----------|---------|-------|-------|
| L-RECOV | FAIL | L1 | the UI is honest and consistent with the durable files across a corrupt artifact, an SSE drop and kernel/gateway crashes | 1/2 | Failed: (a) corrupt artifact: bytes edited inside the cell -> typed digest-mismatch card, nothing rendered from the bad bytes; restored afterwards |