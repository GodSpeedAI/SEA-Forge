# PhaseA scoped-test repair scope

Fresh builder t09_run_children_fixture_builder owns ONLY scoped_runs_test.go and
scoped_live_test.go after immutable independent REJECTc7590c52ff30ce8fdc829c55fac3755fceff5755a93d95e18e954401101d910d.
Original runtime assignment and test-phase supplement remain binding; all other files frozen.

Root verified Config.Dial is injectable (client.go Config). Count synchronous dial attempts
and return a test error on any unexpected dial; assert zero attempts for blank refs. Also
assert fake-server accepted connections and parsed request lines are zero. Accepted connection
count alone can race the asynchronous listener goroutine, so it cannot prove pre-dial refusal.
Keep the existing typed-invalid assertions. No production dial-hook change is needed.

The source fake must record the scoped CaseRef and the positive LiveSource test must assert
exactly case_1, alongside one scoped and zero legacy calls. Preserve all malformed mapping,
ref/ownership/clone coverage and existing tests/declarations. No compiler token; freeze repair
hashes for independent re-review and compiling scoped assertion RED.

Before re-review, root additionally authorized gofmt whitespace-only correction on the
same two new uncommitted fixtures, via native apply_patch. scoped_live_test.go needed the
correction inherited from its original builder; scoped_runs_test.go already had no diff.
No logic/literal or other-file changes. Final hashes: scoped_runs0013b05f69def466e8434182642b4543e68c5b4ad7fd1a3f3f14608337fcc2b5;
scoped_live361fc0328217bf78d4ac3d7c92dad6f8641e931225e3c78cd58685a7bb4a7800.
