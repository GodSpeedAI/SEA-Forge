# Protected Ask integration supplement

Root scope clarification, 2026-10-01. Read together with the unchanged original
ask-runtime-implementation-assignment.md and the independent fixture review.

The independently approved fixtures expose missing route behavior and unsafe ambiguous Ask
resend on EOF/overflow. Their two test files remain frozen throughout implementation.

Builder ownership: new internal/server/ask.go and internal/adapters/sfwp/ask.go; narrow edits
to SFWP frame.go/client.go, server server.go/http.go/ratelimit.go, ports/ask.go if essential,
and cmd/godspeed-casework/main.go. This clarification additionally permits the narrow Ask
injection in internal/livestack/stack.go, plus new ask_adapter_test.go and a live-tagged
server/ask_live_test.go. These are necessary to test actual configured adapter translation
and delegated kernel integration; manually constructed transport fixtures do not prove either.
No edits to frozen Ask fixtures, earlier cap tests, contract mirrors, Rust, UI or goldens.

Add adapter checks for all semantic request fields, service/effective actor pair, optional
case omission, no request_id, complete answer mapping and honest malformed/unsupported
upstream disclosure failure. Empty arrays are legitimate; missing/null required arrays and
unknown enum values must not become fabricated defaults. Match actual kernel view types.

Live proof should reuse livetest.NewCell and livestack.AssembleStack over a disposable owned
temporary cell, never the operator cell. Use existing target/debug/sea-forge and
target/debug/sea-forge-server; preflight their presence so Go tests do not spawn a Rust build.
Self-model rebuild and fixture-only disclosure policy seeding must be explicit and confined
to that cell. Select a real bundled-model subject from repository/CLI-supported vocabulary.
Do not fabricate a source concept or manufacture a successful disclosure.

Assert the actual HTTP/session/CSRF path, real kernel denied and granted answers, complete
disclosures, and exact linked question/plan/decision/answer ledger records. Verify persisted
question actor and writer attribution against the effective end user, API answer equality
with its committed answer payload, and ledger integrity using the existing CLI. Authentication,
CSRF and invalid-body refusals must add no disclosure records. Preserve actual fixture evidence
without credentials. Independent critic validates the live harness and reruns it.

Builder makes native apply_patch edits only, does not compile or run tests. Root retains the
sole compiler token until source freeze and explicit independent critic transfer. The critic
receives both original assignment and this scope adjustment, explaining every material difference.
This is a bounded T09 unit; remaining run observation and UI work still require confirmation.
