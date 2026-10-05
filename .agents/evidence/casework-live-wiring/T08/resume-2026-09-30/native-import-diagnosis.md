# T08 native browser import diagnosis and runtime result

Scope was the two native harness files only. No production change, fallback transport, fake
EventSource, skipped assertion, or weakened recovery assertion was made. The temporary
diagnostic additions in `native_events_live_test.go` retain private Vite stdout/stderr, probe
the actual served TypeScript module over HTTP, and collect browser module/network errors without
reading cookies or request headers. The only behavior correction was renaming the second local
`resyncPayload` variable to `retryResyncPayload` in `native-events-live.ts` so both K and L
resync payload assertions compile.

## Attempts and evidence

All Go commands used the narrow race gate below unless called out, from
`apps/godspeed-casework-go`:

```sh
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 go test -p=1 -parallel=1 -race -tags=live -run '^TestNativeEventSourceResyncRetryAndRetentionReplay$' -count=1 ./internal/server
```

1. An earlier invocation with the output path incorrectly relative to the Go module exited 1
   before starting Go; details remain in `independent-runtime-2026-09-30/native-go-race-path-error.log`.
2. The first command above with the default Go cache exited 1 during setup because
   `/home/sprime01/.cache/go-build` was read-only. Log:
   `/tmp/sea-t08-native-go-race-diagnostic.log`.
3. The same command with `GOCACHE=/tmp/sea-t08-native-gocache` under the default sandbox exited 1
   after the cell's 20-second socket wait. The test framework removed its temporary server log.
   Log: `/tmp/sea-t08-native-go-race-instrumented.log`.
4. Two escalated host invocations added
   `TMPDIR=/tmp/sea-t08-native-testtmp GOCACHE=/tmp/sea-t08-native-gocache`. Both exited 1 before
   Vite because that longer temp root made `server.sock` 104 bytes, above the server's 95-byte
   Unix socket limit. The first output is
   `/tmp/sea-t08-native-artifacts/native-go-race-host-test.log`; the second run captured the
   temporary server log at `/tmp/sea-t08-native-artifacts/server-startup.log` and recorded the
   same test failure in `native-go-race-startup-capture.log`. This was a test-runner setup error;
   the attempt was repeated with the normal short `/tmp` path.
5. With the writable cache and normal `/tmp` path, the escalated host run exited 1 in the browser
   import. The direct HTTP probe returned 500 for
   `http://127.0.0.1:42367/e2e/native-events-live.ts`; retained Vite stderr identifies the actual
   cause as esbuild rejecting the duplicate `resyncPayload` declaration at TypeScript line 265.
   The browser reported `TypeError: Failed to fetch dynamically imported module`. Evidence:
   `/tmp/sea-t08-native-artifacts/native-go-race-default-tmp.log` and
   `/tmp/sea-t08-native-diagnostic-1048786612/` (`vite.stdout-stderr.log`,
   `module-http-probe.json`, and `phase1-browser-module-diagnostics.json`).
6. After the identifier rename, the same bounded escalated host command exited 1 later in Phase 1.
   The direct Vite probe now returned HTTP 200 (`text/javascript`) for the transformed module.
   The real offline K-to-L mutation then exposed a production retention defect: the Go server
   panicked with `index out of range [1] with length 1` in `projection.Store.At`, reached from
   the required stable-state route. The route returned 500, so the recovery assertions could not
   complete. Evidence:
   `/tmp/sea-t08-native-artifacts/native-go-race-after-transform-fix.log` and
   `/tmp/sea-t08-native-diagnostic-2979089792/` (`module-http-probe.json` confirms HTTP 200;
   `vite.stdout-stderr.log` records the proxy hangup following the Go panic).

The panic source is `apps/godspeed-casework-go/internal/projection/store.go`: `At` indexes
`s.revisions[idx]` at line 119 using `byCursor` line 115; `Append` deletes the evicted cursor and
shortens `revisions` at lines 168-169 without decrementing the surviving indexes. Root preserved
the byte-identical panic output in `native-retention-panic.log`. The fix must preserve the existing
`Store.At` assertions and belongs to the separate Store correction/review. No test bypassed it.

## Resource and cleanup record

The recorded pre-run host RAM/process snapshots showed approximately 1.6 GiB, 1.8 GiB, and
2.1 GiB available across the escalated attempts; the constrained Go settings above were used
for race runs. The final process check found no Go test, Vite, agent-browser, or sea-forge-server
process. Test cleanup closed its browser session and owned Vite/kernel processes. The short log
watcher used to capture `server.log` was interrupted and exited; preserved failure cells and
diagnostic artifacts remain under `/tmp`. The unrelated sleeping Bun PID 13814 was left untouched.

The sole compile token was explicitly released to root after the Store panic. No further
compile, test, or runtime command was run after release. T08 remains **failed/incomplete**:
the Vite import failure is fixed, but the independent Store index failure prevents the full
native retry/retention proof. Independent critic review and the required Store/Go/T07 regression
gates remain pending.

Selected artifact hashes (SHA-256):

| Artifact | SHA-256 |
| --- | --- |
| `/tmp/sea-t08-native-artifacts/native-go-race-default-tmp.log` | `f1eea1d3b7faa2eff90e8073dcef9d38626526770c37576008cf64280112606f` |
| `/tmp/sea-t08-native-diagnostic-1048786612/module-http-probe.json` | `72cd554727bcf7523cb27d1fcda96024947878bd79ac11f5a01f6f6839728786` |
| `/tmp/sea-t08-native-diagnostic-1048786612/vite.stdout-stderr.log` | `e809528ffb99cd44d0f236a901b1cdabce2f1f02bed2c3af64814a381da9d6c8` |
| `/tmp/sea-t08-native-diagnostic-2979089792/module-http-probe.json` | `c0cae0ef847d5c9f6889f0bf70fe3804590ec29f674df3a249e0dee28414deae` |
| `native-retention-panic.log` | `16d60d5c57f323c4d6bae0c58bd8a13594a4dd39be8732373ab12c4f5e38134f` |
