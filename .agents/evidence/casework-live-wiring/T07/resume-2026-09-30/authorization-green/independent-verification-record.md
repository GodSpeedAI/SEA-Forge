# T07 independent verification record — 2026-09-30

## Verdict

**Approve T07 wiring after the cached-intent authorization repair.** The original T07 tests, focused race gate, global Go vet/race gates, fresh binary, live cursor receipts, revoked-session refusal, restored exact receipt, read/session isolation, and production-dev-auth refusal all passed. No production-code defect remains in the reviewed scope.

## Original repair requirement and source review

The bounded repair in `../cached-intent-authorization-builder.md` required session perspective verification after JSON decode/actor overwrite and before the dispatcher can consult its idempotency cache; fail closed when the verifier is missing, unmapped, or unavailable; preserve exact authorized replay; leave CSRF/origin/malformed-request guards ahead of verification; and prove the real handler behavior against a live cell.

Reviewed source hashes:

- `apps/godspeed-casework-go/internal/server/server.go`: SHA-256 `d6d8f0077a297bdaa85dd9252f866b71fbe33e623466b71e702e82103544b757`
- `apps/godspeed-casework-go/internal/server/intent_authorization_test.go`: SHA-256 `b398367594a4458cafcfd392627ca56f0d65d7a4de69e48299552fb53dfb877b`
- Fresh gateway `/tmp/t07-final.uUPtaO/authorization-green-2026-09-30/godspeed-casework-fixed`: SHA-256 `8c686a93802add0b5b9ba38ba14cd0e38b13fc73be5e3fa6fec82b8103c72214`

`handleIntent` preserves request protections and actor overwrite, then calls the existing `verifySessionPerspective` with the session claim and returns immediately on failure, before `s.intents.Handle`. The existing helper maps absent verification to 503, unavailable kernel verification to 502, and denied authority to 403. No identity model, dispatcher behavior, or schema was changed.

## Commands and gates

All Go compile/test commands were serialized with `GOMAXPROCS=2`, `GOFLAGS=-p=1`, and the writable cache `/tmp/t07-final.uUPtaO/go-cache`. Before each, available RAM was 2.8–2.9 GiB and the host compiler scan found no active Go/compile/link processes.

1. Focused auth tests, exit 0:

   `GOCACHE=/tmp/t07-final.uUPtaO/go-cache GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./internal/server -run 'TestIntentAuthorizationPrecedesCachedReplay|TestIntentAuthorizationFailsClosedBeforeDispatcher|TestIntentRequestGuardsPrecedePerspectiveVerification'`

   Output: `focused-race.log`.

2. Global vet, exit 0:

   `GOCACHE=/tmp/t07-final.uUPtaO/go-cache GOMAXPROCS=2 GOFLAGS=-p=1 go vet ./...`

   Output: `global-vet.log` (empty on success).

3. Full Go race suite, exit 0:

   `GOCACHE=/tmp/t07-final.uUPtaO/go-cache GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./...`

   Captured confirmation output: `global-race-confirm.log`. The tool returned `exit_code=0`. An earlier invocation yielded before its shell status marker; it is not used as the exit record.

4. Fresh gateway build, exit 0:

   `GOCACHE=/tmp/t07-final.uUPtaO/go-cache GOMAXPROCS=2 GOFLAGS=-p=1 go build -o /tmp/t07-final.uUPtaO/authorization-green-2026-09-30/godspeed-casework-fixed ./cmd/godspeed-casework`

   Output: `gateway-build.log` (empty on success).

5. `git diff --check -- apps/godspeed-casework-go/internal/server/server.go apps/godspeed-casework-go/internal/server/intent_authorization_test.go` passed with no output.

## Fresh live verification

The fresh gateway was built to a new path and ran in a supervised foreground session as PID 1017163 on `127.0.0.1:44280`, connected to the isolated cell at `/tmp/t07-final.uUPtaO/cell-green`. Health and readiness returned 200. Existing gateway PID 939247 on port 44279 and Vite PID 310487 on port 4178 stayed running. The task cell's repository-built kernel was verified by executable and lock descriptors before each authorized test-config restart; original cell config was preserved and restored byte-for-byte afterward. Restored kernel PID: 1027373.

`live-cached-replay.py` ran against the real gateway, real in-process intent handler/cache, and real SFWP kernel. It retained cookies and response bytes in memory; it printed no auth values.

- Authorized operator session submitted a valid `PROPOSE_CASE` intent with a deliberately spoofed R-SO body actor. The operator's returned history snapshot identified `operator_local`; both operator and R-SO immediately read the accepted `new_cursor` (no intervening gateway request). Case `case_20260930T152905Z_8f2e6a`, cursor `01M3SEY0NAEDY4NQ7PBFPZA3X9`, was already in trajectory.
- After removing only `operator_local` from the copied test cell's delegation allowlist and gracefully restarting only the verified task kernel, replaying the exact successful POST returned HTTP 403 `authority_denied`. Operator world and SSE also returned 403. Still-delegated R-SO world and SSE returned 200. R-SO's trajectory cursor/summary sequence remained unchanged at one point.
- After restoring the original delegation config byte-for-byte and restarting the same task kernel, the same session's exact POST returned HTTP 200 with response bytes equal to the original receipt. Operator world and SSE returned 200. R-SO trajectory remained unchanged, so the receipt replay did not create another case revision.
- Fresh-binary production posture test with `serve.production=true` and `auth.mode=dev` exited 2 with the expected refusal. The test used free port 44281; a direct listener check confirmed it remained absent. The live test gateway uses 44280, so no refusal claim is made for that occupied port.

Full sanitized command output and assertions are in `live-cached-replay.log`; exact script is `live-cached-replay.py`. Gateway/kernel startup logs are `gateway.log`, `kernel-revoked.log`, and `kernel-restored.log`; production refusal output is `production-dev-refusal.log`.

## Prior unchanged-source evidence reused

- The relay publication repair's focused repeated race tests, global Go checks and fresh accepted-cursor proposal/execute proofs are documented in `.agents/evidence/casework-live-wiring/T07/resume-2026-09-29/relay-publication-race-builder.md` and `.agents/evidence/casework-live-wiring/T07/resume-2026-09-30/relay-cursor-publication-race-red/` (red snapshot retained separately).
- Prior live T07 history/action, actor identity, cookie flags, CSRF/origin, query/cross-case cursor refusals, immutable old snapshots, and production refusal evidence is in `.agents/evidence/casework-live-wiring/T07/resume-2026-09-30/final-independent-runtime/confirmation.md` and adjacent probe scripts. The auth/session/cookie code was not changed in this repair. That prior record's accepted-cursor race was subsequently fixed and retested as above.
- Rust Cargo (594/0) and UI (255/0 plus 11 journeys) gate evidence remains under `.agents/evidence/casework-live-wiring/T07/resume-2026-09-29/final-independent/`; Rust/UI production source did not change in the reviewed repairs.

## Material limitations and deviations

- The Python harness manually replayed Secure cookies over loopback HTTP. This verifies session binding in the gateway protocol but does not prove native-browser Secure-cookie behavior; the native browser T08 check remains separate.
- The post-fix immediate proposal/execute cursor scripts and their successful outputs were captured in the tool transcript, but a separate durable raw stdout file for that earlier green run was not found. The scripts remain preserved. This record does not invent their missing output artifact.
- An initial background launch of the fresh gateway returned PID 1013150 but did not persist; at that time its log was empty and no listener existed. It was replaced with the verified supervised foreground session described above. `gateway.pid` retains the failed attempt PID; `gateway.log` contains startup output from the later successful foreground session. The successful PID 1017163 is recorded separately in `gateway-supervised.pid`.
- During a setup-pattern search I accidentally allowed `rg` to traverse the task cell tree and print two test-case `entries.jsonl` authority records. I did not use those records as evidence or read them further; the parent was notified. No credentials were exposed. All subsequent mutation/no-write claims use gateway API and trajectory observations, not direct ledger reads.
- No native-browser/WorkBench or live-tagged T08 tests were run in this T07 verification.
