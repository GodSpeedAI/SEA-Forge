# Safe-trace socket sandbox review erratum

Date: 2026-10-05. This immutable erratum corrects the cleanup statement in
`safe-trace-fixture-independent-review-socket-sandbox-oct05.md`; that earlier
artifact remains unchanged.

The earlier supplement incorrectly said that a failed `net.Listen` leaks the
new temporary directory. Direct inspection of
`apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go:56-60`
shows that the error branch calls `os.RemoveAll(dir)` before `t.Fatal(err)`.
The setup failure therefore does clean its newly created temporary directory.
There is no source-based cleanup gap for that failure path, and the earlier
leak concern should be disregarded. The observed `setsockopt: operation not
permitted` failure still prevented the intended assertion and runtime peer
cleanup from being exercised.

No source or prior evidence file was modified.
