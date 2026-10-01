# Independent runtime Phase A source review: REJECT

Review date: 2026-10-01

## Frozen source identity and scope

Reviewed the original runtime assignment, runtime test phases, root fixture clarification, the repaired pure-child fixture, and the five frozen Phase A files. No tests/compiler were run and no source was edited by the reviewer.

- `apps/godspeed-casework-go/internal/ports/ports.go` — `703bf0d94a5d235d4c69450838171a64bfddc999cc97cba6bfa30938b2cb9ec6`
- `apps/godspeed-casework-go/internal/projection/builder.go` — `69d9ba3cc7ef7788ed673059451ede73209f50673f6e184c0c283f189393203c`
- `apps/godspeed-casework-go/internal/adapters/sfwp/scoped_runs_test.go` — `00cff41b79d5d8ea19303d8c34df5e558764da03fbb64b10e431109d44eb3681`
- `apps/godspeed-casework-go/internal/projection/scoped_live_test.go` — `108e757ca2fb2fea485a519ea1cab2a486907660b77458f701db108771a70661`
- `apps/godspeed-casework-go/internal/projection/unreadable_runs_test.go` — `de85a2edb9bd05dc5542c7c9850a9cf9caea69dbb491eb21f95fe93913bfcebb`
- Preserved pure fixture: `apps/godspeed-casework-go/internal/projection/run_children_test.go` — `4ddb2445a7ff6726c3b6fd93832ac4122b53da4d26e5b54180f9a1cafb802741`

The two production changes are limited to the approved `RunListResult` semantic declaration and `CaseFacts.UnreadableRunIDs` field. The three new tests cover broad adapter mapping/refusal/invalid-row cases, source scoped-vs-legacy counts and ownership validation, and retained unreadable-ID cloning. The following two test gaps are material against the explicit original runtime contract.

## Blocking findings

1. **Blank request is not proven to fail before dial.** In `scoped_runs_test.go` lines 77–93, the blank-case test asserts a typed invalid error and `fs.requestCount("run_list") == 0`. The original assignment requires invalid before dial. The existing fake server tracks accepted connections separately (`internal/adapters/sfwp/client_test.go`, `fakeServer.conns`), but the new test never checks it. An implementation could connect and then reject without sending a `run_list` line (or send another verb) and this test would still pass. Assert no accepted connection for empty and whitespace-only refs, or add equivalent evidence that no dial occurred.

2. **Live source does not prove it passes the selected case to the scoped port.** In `scoped_live_test.go` lines 44–46, `RunsListForCase` discards its `CaseRef`; the successful-source test checks only scoped call count and legacy call count. A LiveSource that calls the scoped method for the wrong case still passes this fake and can attach data from a different scope. Record the received `CaseRef` and assert it exactly equals the requested `case_1` in the valid flow.

The scoped adapter test does prove that a supplied `case_actual` is encoded as the wire `case_id`; it does not close the separate LiveSource-to-port argument gap. These requirements are explicitly about pre-dial validation and selected-case scoping, so the fixture is rejected before compilation. No expected-RED result is claimed and the compiler token was not used.

Disposition: fresh builder must make only the bounded test corrections; keep the approved declarations, pure fixture and other assertions intact. After freeze, independent review must restart against the new hashes before the scoped expected-RED gate.
