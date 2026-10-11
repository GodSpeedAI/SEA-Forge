# Present-context guard Phase 2 — partial runtime verdict

Date: 2026-10-06  
Verdict: **RUNTIME VERIFICATION INCOMPLETE / REJECTED at the canonical gate.** The focused, full-server, and full-module Go gates passed after authorized local-listener retries where standard sandbox setup failed. The final required `just casework-go-check` exited before running tests because it found the frozen test file would be reformatted by gofmt. No source or test was modified; stop at that first unexpected failure. Root assigned a different fixture-only formatting builder. This record does not claim the full required runtime sequence is complete.

## Frozen implementation and fixture

- Source `apps/godspeed-casework-go/internal/server/run_observation_present_context.go`: SHA-256 `19bd9a574a25beab1e76824ddf136a3091e418d77e7573068c8d3e9464a3b1c2`.
- Fixture `apps/godspeed-casework-go/internal/server/run_observation_present_context_test.go`: SHA-256 `fffa033505fa19b937da12963df91a0cdc2af30f39ecb50031fc2a17ec64a67b`.
- The same hashes were present in all captured per-gate preflights and after the canonical failure. No source/test edit was made by this reviewer.
- Source Phase 2 approval remains limited to static code review. This record covers only the gates below; it releases no caller, manager, public contract, continuous freshness, or overall T09 completion.

All gate commands used `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1`. The canonical recipe additionally used `JUST_TEMPDIR=/tmp`.

## Gate results and boundaries

### 1. Focused guard — PASS

Command from `apps/godspeed-casework-go`: `go test -race -count=1 -parallel=1 -run '^TestCheckRunObservationPresentContext' -v ./internal/server`. Actual session 43421 joined, Go exit 0. All seven top-level test functions passed, including all 13 independent identity cases, newest-only rejection, four relay-gap cases, empty/populated copy ownership, result repetition, advancing history, parent removal, and invalid-input handling. No implementation behavior remains unproven by the fixture except its explicit untested typed-nil dependency branch noted in the Phase 2 source review. This is one focused race test, not a full runtime gate.

Fresh corrected preflight `/tmp/sea-casework-20261006-guard-green01-focused-preflight-02.raw`: SHA-256 `7a8fcc00f9fa663cecb81c14f081448ffb665101e141adf9624b1219cdc7fa9a`; at `20:43:47Z`, `MemAvailable=3,028,840,448`, `SwapFree=1,546,006,528` bytes. Source and fixture hashes matched the freeze. Test stdout/stderr `/tmp/sea-casework-20261006-guard-green01-focused-run.raw`: SHA-256 `73f5161b8ec3ee23726ff6a51725c48f9efb051310f3af7ddd20dbe108e65b37`; Go exit file `/tmp/sea-casework-20261006-guard-green01-focused-exit.raw`: SHA-256 `19eaf43821a7660ec323a87c8457bf74823beb296c39f5e01aa8a683aa50f061` (`exit=0`). The tool-only wrapper launch line was `launch_utc=2026-10-06T20:43:59Z`; it is not part of the redirected Go stdout/stderr file.

An earlier preflight `/tmp/sea-casework-20261006-guard-green01-focused-preflight.raw` (SHA-256 `bddfdf7aed21e6c0edd902bf336be003a233382fade1f68c593872baa4eb9fc9`) printed resources above the floor but a faulty local awk threshold parser returned 1 because it parsed `key=value` as whitespace fields. **No test command was launched from that preflight.** It is preserved and not counted as a gate attempt. The corrected fresh preflight above passed.

Root independently read and direct-copied the actual focused Go-output file to `present-context-red-01-run-root-direct-original-copy.raw`; root reports `cmp` exit 0 and the same SHA-256. Earlier malformed/failed archive attempts were not used as evidence.

### 2. Full server — PASS after local-listener setup retry

Command from `apps/godspeed-casework-go`: `go test -race -count=1 -parallel=1 ./internal/server`.

The first standard-sandbox session 18912 joined with Go exit 1 because `httptest.NewServer` could not bind `[::1]:0` (`operation not permitted`). This was a test setup denial, not an assertion failure. Its preflight `/tmp/sea-casework-20261006-guard-green02-server-preflight.raw` has SHA-256 `5c6837b06cf0e03c91df9bfb48d007cffc2f8e8859e984970650ff4c3cf201ef` (`MemAvailable=2,934,243,328`, `SwapFree=1,653,706,752` bytes); Go output `/tmp/sea-casework-20261006-guard-green02-server-run.raw` SHA-256 `707d04317c778ff985543667421f8e05dc2ae35cda6bf8a02261d086281f1d55`; exit `/tmp/sea-casework-20261006-guard-green02-server-exit.raw` SHA-256 `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22` (`exit=1`). Wrapper launch: `2026-10-06T20:46:48Z`.

After that actual denial, the same gate was rerun with the explicitly authorized local test listener permission only. Escalated session 92459 joined with Go exit 0 (`ok .../internal/server 15.388s`). Its new preflight `/tmp/sea-casework-20261006-guard-green02-server-escalated-preflight.raw` SHA-256 `2a5da1bbfaf5db8d5b342be25a600f191a10fe379ba11679f507e7c948e7f168` (`MemAvailable=2,582,695,936`, `SwapFree=1,726,603,264` bytes); output `/tmp/sea-casework-20261006-guard-green02-server-escalated-run.raw` SHA-256 `399bb2861c558b3e6d7f8e8a685443260412055e7ca8b17a80f343e29014f648`; exit `/tmp/sea-casework-20261006-guard-green02-server-escalated-exit.raw` SHA-256 `19eaf43821a7660ec323a87c8457bf74823beb296c39f5e01aa8a683aa50f061` (`exit=0`). Wrapper launch: `2026-10-06T20:48:15Z`. No external network was used.

### 3. Full module — PASS after local Unix/loopback setup retry

Command from `apps/godspeed-casework-go`: `go test -race -count=1 -parallel=1 ./...`.

The first standard-sandbox session 84633 joined with Go exit 1. The captured errors were test setup denials creating local Unix sockets (`setsockopt: operation not permitted`) and IPv6 loopback `httptest.NewServer` listeners (`operation not permitted`). Other listed packages passed; no source-level assertion failure was observed in the complete raw output. Its preflight `/tmp/sea-casework-20261006-guard-green03-module-preflight.raw` SHA-256 `54dc813554d22720aedfe9a407f6c4d0704567d3518f055f0739ed80268a6b98` (`MemAvailable=2,714,234,880`, `SwapFree=1,671,446,528` bytes); output `/tmp/sea-casework-20261006-guard-green03-module-run.raw` SHA-256 `5f3d31be68f7b1ce53b0560dae8a5a171b1047a2fc119f5f3931346a20042b3f`; exit `/tmp/sea-casework-20261006-guard-green03-module-exit.raw` SHA-256 `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22` (`exit=1`). Wrapper launch: `2026-10-06T20:49:08Z`.

After the observed denial, the same full-module command was rerun with permission for test-local Unix sockets and loopback listeners only. Escalated session 98629 joined with Go exit 0. All listed packages passed (`sfwp 13.797s`, `auth 33.129s`, `server 14.167s`; packages without tests reported accordingly). Fresh preflight `/tmp/sea-casework-20261006-guard-green03-module-escalated-preflight.raw` SHA-256 `7b98dfdd0b35632913ed77a9f8b5310abe47c94f8fc1bcc59fa8916c3a3fdf01` (`MemAvailable=2,742,751,232`, `SwapFree=1,682,067,456` bytes); output `/tmp/sea-casework-20261006-guard-green03-module-escalated-run.raw` SHA-256 `63525a09d7180a84bfc22015db7d2bb6b06af96c48a3d63a88cf30d3c408ce6f`; exit `/tmp/sea-casework-20261006-guard-green03-module-escalated-exit.raw` SHA-256 `19eaf43821a7660ec323a87c8457bf74823beb296c39f5e01aa8a683aa50f061` (`exit=0`). Wrapper launch: `2026-10-06T20:50:37Z`. No external network was used.

### 4. Canonical recipe — FAIL before Go tests

Command from repository root: `JUST_TEMPDIR=/tmp just casework-go-check`, with all assigned caps. The synchronous command completed (no session ID); recipe exit was 1 before running Go tests. Output: `casework-go-check: gofmt would rewrite: internal/server/run_observation_present_context_test.go`, followed by recipe failure. This is the first unexpected verification failure, so no more gates were run. No formatting or fixture change was made by this reviewer.

Preflight `/tmp/sea-casework-20261006-guard-green04-canonical-preflight.raw`: SHA-256 `14731d84662df6e2c1ab5773f904f12e3c9fb4a147c690937022f132b8d6b8d5` (`MemAvailable=2,770,440,192`, `SwapFree=1,679,966,208` bytes). Output `/tmp/sea-casework-20261006-guard-green04-canonical-run.raw`: SHA-256 `884f29c725041b36e2d1e6b6a6cd0b1cc141e2eb765de7f73b88b8adc7c7993b`; exit `/tmp/sea-casework-20261006-guard-green04-canonical-exit.raw`: SHA-256 `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22` (`exit=1`). Wrapper launch: `2026-10-06T20:52:31Z`.

## Capture handling, deviations, and non-claims

- Every listed `/tmp` capture remained intact; root will perform the exact native repository copies and byte comparisons. No manually transcribed run output is represented as original evidence. Wrapper `launch_utc` lines were tool-only metadata, not part of the redirected Go output. Fullserver/module sandbox failures and their escalated retries have distinct captures; neither failed attempt was overwritten.
- The focused gate had one earlier resource-parser mistake; the printed resources were above threshold, the parser exit was 1, and no gate launched. The corrected second focused preflight passed and immediately preceded the sole focused test run.
- The initial native archive action for the focused run was rejected by automatic review because the proposed repository capture omitted the tool-only wrapper timestamp. No incorrect repository copy was created by that rejected action. Root later copied the original Go-output bytes directly and verified exact identity.
- Source/fixture hashes remained frozen at `19bd9a…a3b1c2` and `fffa03…a67b` through the canonical failure. The canonical gofmt finding remains unresolved here; root assigned a different fixture-only formatting builder and independent review. This reviewer made no source or test changes and ran no further command.
- The three Go gates passed only as recorded above. The canonical required recipe is not green, so the overall four-gate runtime request is **not complete or accepted**. No full T09, CI, publication, or deployment proof is claimed.
