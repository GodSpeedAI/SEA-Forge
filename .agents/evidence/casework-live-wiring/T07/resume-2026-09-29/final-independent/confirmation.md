# T07 independent confirmation — scoped approvals; full task not approved

**Reviewer:** independent T07 critic (did not build T07 or its fixes)  
**Reviewed source:** committed implementation at `8741ff4`, then the bounded uncommitted test/documentation repairs listed below.  
**Disposition:** approve the isolated test-fixture and Argon2 regression-test repairs only. **Do not approve or settle T07**: requested fresh real-kernel current/history/SSE runtime proof remains unrun because auto-review rejected starting the gateway, and coordinator/user guidance is pending.

## Original bounded builder instructions reviewed

- The session-read builder brief is preserved at [`../session-read-builder-spec.md`](../session-read-builder-spec.md). It requires current, retained, and streamed projections to identify the authenticated session actor; identity override refusal before cached lookup; kernel perspective verification for history/SSE; cross-case cursor refusal; genuine captured historical state; role-specific action descriptors; immutable bounded captured facts; and fresh tests plus real-cell probes for distinct operator/R-SO sessions, overrides, cross-case cursors, invalid/revoked delegation, immutable captures, and old state after mutation. It forbids changes to external interfaces, persisted schemas, dependencies, kernel identity/policy, and public wire contracts.
- The session test setup builder result and exact changes are preserved at [`../session-test-setup-fix.md`](../session-test-setup-fix.md). The actual diff changed relative request paths because `testUser.get` prepends its base URL, added status/body checks before decoding streams, and matched the resync relay actor to the logged-in `operator_local` user. It did not weaken the old-state, role, refusal, replay, or legacy-perspective assertions.
- The bounded Argon2 regression-test repair and its material differences are preserved at [`../argon-hash-test-fix.md`](../argon-hash-test-fix.md). The original test changed the last encoded Base64 character and could fail to change the decoded digest; the repair checks hash/parse errors, verifies the original pair, flips one bit in decoded digest bytes, canonically re-encodes it, proves the decoded value differs, and expects `ErrInvalidCredentials`. It changes tests only.
- The initial round-two rejection remains immutable at [`../recovery-round2/confirmation.md`](../recovery-round2/confirmation.md); the round-three source-only review and probe plan remain at [`../recovery-round3/static-review.md`](../recovery-round3/static-review.md) and [`../recovery-round3/probe-plan.md`](../recovery-round3/probe-plan.md).

## Independent source review and material differences

The session repair adds an internal `Facts *CaseFacts` field to `projection.Revision` with `json:"-"`; captured facts and snapshots are deep-copied on append/read/subscription. `Relay.accept` uses one source `Facts` capture to build and retain that revision. `handleWorld` refuses actor/role query keys before verification or history lookup, verifies the session actor with the kernel, enforces the optional `case_id` against the cursor, and re-renders the retained facts for that actor. `handleEvents` verifies before subscribing and renders replay/live revisions for that actor. A legacy row with no facts is returned only to its exact stored actor/role; a mismatched perspective fails closed. This is consistent with the bounded architecture direction; no wire field, external interface, persisted schema, dependency, kernel policy, or identity model changed.

The adjacent documentation repair updates the `ServeSection` comments to describe captured-facts rendering for current/history/SSE. No product behavior changed. Round-two independent review had already inspected the trusted-origin and log-escaping fixes; this review verified their source wiring: exact configured origins, duplicate/untrusted POST-Origin refusal before routing, credentialed CORS only for configured origins, quoting every string-valued request-log field, and using the intent ID as the log correlation ID passed as SFWP request ID.

I found no production-code defect in the repaired retained-read path by static inspection. Its newly added session/history/SSE tests use a controlled captured-facts source and mocked verifier; they are not evidence of the required real kernel behavior. Previous live T07 teeth cover CSRF no-write and actor overwrite/current perspective, but do not cover the newly repaired historical/replayed/live SSE paths.

## Commands, results, and evidence

All commands below were run by this critic. Memory/process checks were performed before each Go/Cargo vet, test, or build; gates used `GOMAXPROCS=2 GOFLAGS=-p=1` and the Rust gate used one Cargo job and one test thread. The exact captured outputs are in the linked immutable files.

| Command | Result | Evidence |
| --- | --- | --- |
| `GOMAXPROCS=2 GOFLAGS=-p=1 go vet ./...` (default sandbox) | EXIT 1: Go build cache outside writable roots was read-only; environmental failure | [`gates/01-go-vet.txt`](gates/01-go-vet.txt) |
| Same `go vet` via approved local execution | EXIT 0 | [`gates/01-go-vet.txt`](gates/01-go-vet.txt) |
| `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./internal/server/... ./internal/auth/...` before fixture repair | EXIT 1: three malformed absolute URLs in new tests; stale legacy resync fixture perspective closed the stream before replay | [`gates/02-focused-go-race.txt`](gates/02-focused-go-race.txt) |
| `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./...` before fixture repair | EXIT 1: same four server-test failures; other listed packages passed | [`gates/03-global-go-race.txt`](gates/03-global-go-race.txt) |
| `GOMAXPROCS=2 GOFLAGS=-p=1 go build -o /tmp/casework-resume-gateway ./cmd/godspeed-casework` | EXIT 0; fresh binary built but never launched | [`gates/04-gateway-build.txt`](gates/04-gateway-build.txt) |
| `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./internal/server/... ./internal/auth/...` after fixture repair | EXIT 0 | [`gates/05-postrepair-focused-go-race.log`](gates/05-postrepair-focused-go-race.log) |
| `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./...` after fixture repair, before Argon test repair | First run EXIT 1 only at `TestVerifyRejectsTamperedHash`; other packages, including server, passed | [`gates/06-postrepair-global-go-race.log`](gates/06-postrepair-global-go-race.log) |
| `GOMAXPROCS=2 GOFLAGS=-p=1 go vet ./...` after fixture repair | EXIT 0 | [`gates/07-postrepair-go-vet.log`](gates/07-postrepair-go-vet.log) |
| One characterization retry of the same full Go race command | EXIT 0 | [`gates/08-global-go-race-retry.log`](gates/08-global-go-race-retry.log) |
| `CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 cargo test --locked -p sea-forge-server -p sea-forge-planner -p sea-forge-case-runner -p sea-forge-cli` | EXIT 0: 594 passed, 0 failed, 4 ignored | [`gates/09-four-crate-cargo-test.log`](gates/09-four-crate-cargo-test.log) |
| `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=50 ./internal/auth -run '^TestVerifyRejectsTamperedHash$'` after Argon test repair | EXIT 0 | [`gates/10-argon2-tamper-race-count50.log`](gates/10-argon2-tamper-race-count50.log) |
| Final `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./...` | EXIT 0 | [`gates/11-final-global-go-race.log`](gates/11-final-global-go-race.log) |
| Final `GOMAXPROCS=2 GOFLAGS=-p=1 go vet ./...` | EXIT 0 | [`gates/12-final-go-vet.log`](gates/12-final-go-vet.log) |

The first post-repair global failure is an unrelated randomized test defect in the existing Argon2 test: it mutates an encoded final Base64 character rather than decoded digest bytes. One retry passed; after the bounded test-only fix, 50 focused repetitions and the final full suite passed. The initial failure is retained above and was not overwritten.

## Runtime gate blocked by auto-review

A new isolated cell was prepared at `/tmp/t07-final.uUPtaO`, using checked-in E2E templates/policy and a test-only dev-auth configuration without credentials. The kernel is running as PID `256835` on `/tmp/t07-final.uUPtaO/cell/server.sock`. The fresh gateway binary is at `/tmp/casework-resume-gateway`. Starting that gateway bound only to `127.0.0.1:44179`, then querying `/api/healthz` and `/api/readyz`, was rejected by auto-review before execution. The exact command, requested permission, complete reviewer reason, and no-retry record are preserved in [`teeth/01-gateway-start-auto-review.txt`](teeth/01-gateway-start-auto-review.txt). No gateway process is running. I did not attempt the rejected launch again or route around the rejection.

The still-required real-cell assertions are listed in [`teeth/02-runtime-probe-plan.md`](teeth/02-runtime-probe-plan.md): distinct operator/R-SO sessions; old pending state preserved after a later governed mutation; actor/action correctness in current, historical, replayed-SSE, and live-SSE snapshots; cursor override/cross-case refusals; invalid and revoked delegation refusing cached reads/SSE; and direct security checks for CSRF/no writes, actor overwrite, origins/log correlation, and production dev-auth refusal on the fresh binary.

The isolated kernel remains running only while coordinator/user guidance is pending. After the runtime decision, stop it gracefully and record the result; do not delete the `/tmp` cell without separate authorization. T08 UI gates/local ladder are not included in this T07 confirmation and remain with the T08 critic.
