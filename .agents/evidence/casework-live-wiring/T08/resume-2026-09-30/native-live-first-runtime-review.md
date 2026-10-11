# T08 native test first actual gate rejection

Independent critic `t08_final_static` ran the prepared live-tag command in approved host
context after checking available RAM (2.4 GiB), existing kernel binary, agent-browser and
free port4179. Existing Vite4178, gateways44279/44280 and kernels were preserved.

```sh
GOMAXPROCS=2 GOFLAGS=-p=1 go test -p=1 -parallel=1 -race -tags=live -run '^TestNativeEventSourceResyncRetryAndRetentionReplay$' -count=1 ./internal/server
```

**REJECT:** exit1 at compilation: unused imports internal/intents and internal/projection
in native_events_live_test.go. No native test/browser/cell started. Exact output is copied
alongside this note as native-live-first-compile-reject.log; it is not runtime evidence.
The critic subsequently hit a provider usage limit before further gates. No source edit.

Root assigned fresh builder t09_wiring_scope to remove only unused imports, preserving
all assertions, then independent critic t07_live_confirmation to take over verification.
Root temporarily owns the compile token until stable builder result/explicit transfer.
No task settlement; T07 remains settled and T08 remains in flight.
