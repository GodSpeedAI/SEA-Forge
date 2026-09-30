# Prepared T08 live verification sequence

Independent critic `t08_final_static` prepared this source-only invocation plan; no runtime
gate ran for this preparation. Transfer the sole compile token only after T07 verification
finishes. Check available host RAM and actual compiler processes before every command.

1. Verify the existing `target/debug/sea-forge-server` executable and `agent-browser` PATH.
   The helper auto-builds Cargo if the binary is absent: never let that happen concurrently.
   From the Go module:

   ```sh
   GOMAXPROCS=2 GOFLAGS=-p=1 go test -p=1 -parallel=1 -race -tags=live -run '^TestNativeEventSourceResyncRetryAndRetentionReplay$' -count=1 ./internal/server
   ```

   The test owns fresh retention-one/two cells, unique browser session and random loopback
   Go/private Vite ports. It must prove actual native EventSource and cookie transport,
   ret1 disconnect/offline real mutation K-to-L/resync+same-L snapshot, pending retry disposal,
   and ret2 retained-C replay delivering only snapshot-L. Preserve failed cells copied by
   the harness; do not touch the pre-existing Vite listener or prior independent test gateways.
2. From the UI app, run the shared local conformance test in `localAdapter.test.ts`.
   This checks the same suite body against the authored local fixture, not live authority.
3. Confirm host port 4179 is free. Run `CASEWORK_LIVE=1 bun run e2e/live-conformance.ts`
   with a unique existing TMPDIR and serial Go environment. The script internally builds Go
   and launches its own real gateway/kernel; this is fetch-streaming proof, separate from
   native browser proof. Preserve temporary cell and output. Verify its owned processes
   actually exit after cleanup; it currently kills without awaiting both process exits.

The stale refusal assertion proves typed STALE_PROJECTION and bytewise durable no-write.
Independent source inspection found one rawPOST with no dispatch retry. The existing unit
request counter covers 429; stale-specific request counting is not present. Do not describe
the durable-state assertion as a measured transport-request count.

These prepared commands are not passing evidence and do not settle T08.
