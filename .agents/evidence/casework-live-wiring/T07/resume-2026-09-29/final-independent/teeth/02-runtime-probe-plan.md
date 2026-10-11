# Pending T07 real-binary runtime probe

The intended runtime was isolated under `/tmp/t07-final.uUPtaO` with two OS processes: the existing kernel binary on `/tmp/t07-final.uUPtaO/cell/server.sock` (PID 256835) and freshly built gateway binary `/tmp/casework-resume-gateway` bound only to `127.0.0.1:44179`. The temporary cell binds gateway/service to the current uid and delegates `operator_local/operator` plus `rso_local/R-SO`; it uses only checked-in E2E templates/policy and dev auth users in `/tmp/t07-final.uUPtaO/serve.json`. No credentials are present.

Gateway process startup and loopback health/readiness probes were rejected by auto-review before execution. Do not start it until the coordinator receives user guidance or approval. The planned assertions after authorization are:

1. Fresh browser cookie jars login separately as operator and R-SO; GET `/api/session` reports the distinct configured actor/role for each.
2. Anonymous `/api/world` and `/api/events` refuse; no snapshot data. Each logged-in session's current world verifies that actor through kernel delegation.
3. Operator commits the checked-in sentry-chain template using preflight and PROPOSE_CASE. Capture cursor and old projection while dependent work is still pending.
4. Execute the ready operator work. GET `/api/world?case_id=<case>&cursor=<old>` must retain the old cursor/state; current world must show the later enabled/execution state.
5. Historical, replayed SSE, and live SSE render the authenticated actor. R-SO receives no operator execution actions; operator receives its authorized offers. A query actor/role override paired with a retained cursor and a cursor belonging to another case both return typed refusals.
6. Kernel perspective verification is denied for a cached history request and SSE subscription after the test mapping is revoked; both fail closed. An unmapped login/session cannot obtain current or retained history.
7. Re-check cookie flags, exact-origin/CSRF refusals and SFWP no-write records, intent actor overwrite and correlation id, structured single-line logs, and production refusal of `auth.mode=dev` using this fresh binary.

The reviewed T07 repair's unit tests and static code currently cover parts of these assertions, but this real-cell proof was not performed. No claim in this plan is a result.
