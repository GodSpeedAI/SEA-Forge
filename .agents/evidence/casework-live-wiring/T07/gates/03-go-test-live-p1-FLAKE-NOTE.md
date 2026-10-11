# T07 gate 3 note: first-run flake in TestLiveDiscretionaryAddProducesAnAcceptedKernelWrite

- First full run (`03-go-test-live-p1.log`): FAILED at
  `internal/intents TestLiveDiscretionaryAddProducesAnAcceptedKernelWrite` with STALE_PROJECTION
  ("the kernel has moved past this projection ...") plus teardown noise ("client is closed")
  from the same package's earlier tests.
- Re-runs: full `./internal/... -tags live -p 1` green twice consecutively
  (`03-go-test-live-p1-run2-green.log` and the FINAL `03-go-test-live-p1.log`, which is a
  complete clean run of the finished code),
  and the intents package green twice consecutively with `-p 1` (captured in the session
  transcript below).

Assessment: pre-existing timing window in T06's test, NOT a T07 regression. The test calls
`stack.Dispatcher.Handle` directly (no HTTP, no sessions, no T07 code in the path). Its
`executeFirst` intent carries the ADD's `new_cursor` (the first frame of the add's mutation
burst); when the kernel's remaining frames of that burst land before the execute, the staleness
guard honestly refuses. Under the first run's load (race detector, cold caches) the window
shifted. This is the same load-flake class recorded in decision-log `D-3-followups-2`
("parallel per-package kernel cells made restart/resume/relay tests load-flaky; -p 1 is green
twice consecutively"). The T07 builder did not modify T06's test logic; the fix (re-read the
relay cursor after a WaitRevision before `executeFirst`) is recorded as a T06-test
stabilization candidate for the next kernel-touching task.

Session transcript excerpt (intents package, -p 1, twice):
  ok  github.com/.../internal/intents 0.401s
  ok  github.com/.../internal/intents 0.458s
