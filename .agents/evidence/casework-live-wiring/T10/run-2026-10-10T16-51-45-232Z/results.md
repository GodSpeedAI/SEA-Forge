# E2E Test Results

| Journey | Status | Depends On | Settles | Steps | Notes |
|---------|--------|-----------|---------|-------|-------|
| L-RECOV | FAIL | L1 | the UI is honest and consistent with the durable files across a corrupt artifact, an SSE drop and kernel/gateway crashes | 2/3 | Failed: (d) SSE drop: while the stream is down another session executes; the pill says Reconnecting (no progress), then resumes from the cursor with no duplicate revision |