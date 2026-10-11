# Initial lease bookkeeping TDD result correction

This additive correction preserves
`run-observation-initial-lease-bookkeeping-tdd-result-oct08.md` unchanged.
It supersedes that document's manager-test after-hash and its statement that
there were six frozen files. The final source identities are:

| File | Final SHA-256 | Result |
| --- | --- | --- |
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `e2b7709410ff4766503a52580e0cdec8b9eaf189f695bbb75ae87b2dbbdf3285` | unchanged from initial result |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go` | `eb14a7b572ca782595d0207b66b2edd80b1961ccb6189b5dd8be3db1e648177c` | includes final trace-call assertions |

The manager test changed from the first result's recorded hash
`5e8f2728278590d6364d51b464146068baebc92b41f602d79c84eb7410c481eb` by adding
an atomic trace-call count assertion: the first real Prepare must perform one
accepted terminal read per run, and the second must add exactly one read. This
supports the deterministic generation-1 expectation without inspecting a later
current pointer. No other source changed after the table above was captured.

The directly frozen delta source, delta test, and poller worker remain
byte-identical at the hashes recorded in the original result. The earlier
phrase “six frozen implementation/test inputs” was inaccurate; that result
lists three such files.

Root's static source review rejected this test fixture with two compile defects:

1. `longEventID` is declared with `const` despite calling `strings.Repeat`.
2. The first frame-construction loop declares a local `key` that is unused.

These are disclosed, not repaired here; source is frozen for handoff to a fresh
builder. The independent semantic review is also pending. No compiler, test,
formatter, scanner, build, or Git command was run. The original full assignment
and complete scope/result details are preserved in the preceding result file.
