# Unit5A cancellation prerequisite: test-first assignment

Root architecture proposal pending independent critique. HOLD all Go writes/compilers
until unit4 approval/checkpoint and explicit release. Original T09 bounds and cancellation
source facts govern. No dependency, public interface, kernel operation or identity change.

Observable requirement: manual context cancellation must end the caller's active run_get
read promptly, poison/discard that positional connection, release its occupied pool/read
slot, and allow owner cancellation AND drain. It must not close other checked-out or idle
connections or later reuse of a completed request's connection. Preserving a valid received
mutation/Ask result and existing ambiguity/retry/correlation semantics remains mandatory.
Do not call Client.Close globally to simulate canceling a single observation read.

Fresh builder phase1 owns ONLY new internal/adapters/sfwp/client_cancellation_test.go.
Use current Client API and deterministic fake/net.Pipe helpers, no new declarations needed.
Read source/tests before writing. No production changes until independent assertion RED.

Required tests:
- Actual run_get request reaches fake peer, then caller manually cancels while response
  is withheld. Synchronize on request receipt, not guessed sleeps. Use long5s request
  timeout versus bounded1s cancellation completion assertion so deadline expiry cannot
  masquerade as manual cancellation. Require typed unavailable wrapping context.Canceled,
  exactly one admitted run_get/send, no fresh retry and owned peer sees connection close.
  Baseline expected RED must cleanly unblock its owned peer/call after assertion without
  leaving goroutines. No hidden socket/compile/infrastructure error is assertion RED.
- With pool2, another actual checked-out request remains valid and succeeds after first
  caller cancels; prove request identities/replies, no global Close and no unexpected dial.
- Canceled read's partial bytes/connection are never reused for a later inspect. Later
  inspect acquires a clean positional connection, returns correct response, capacity drains.
- Cancel a former request context AFTER successful completed call. Next request can reuse
  its idle connection; no late callback closes a returned/reassigned pool connection.
- Completion/cancellation race exercises callback retirement and pool return under race
  detector. Assert bounded joins/owned cleanup, not just lack of a panic.

Freeze fixture/hash/coverage/deviations. Independent critic receives these ORIGINAL
instructions plus actual fixture; source review and compiling intended assertion RED
required before production repair. Sole compiler token and actual-host RAM/process check
for EVERY command. Go256MiB/GOGC50/GOMAXPROCS2/p1/race/count1/parallel1; socket tests host
escalation. Native apply_patch persistent writes; all raw outputs/exits/preflights use NEW
immutable paths and exact byte/hash checks. Fresh builder after any rejection.

Phase2 repair scope initially only client.go. Cancellation watcher must affect only the
checked-out network connection; retire/join callback before releasing that connection,
avoid racing conn.dead/pool bookkeeping, and never reset an already canceled read to a
future deadline. This proposal deliberately does not prescribe a particular goroutine/
AfterFunc implementation. Existing ask no-resend and mutation correlation are immutable.
Focused/full SFWP race and canonical/full Go proof needed for approval, plus actual blocked
read cancel/drain evidence. Unit5A does not approve cohort/poller/SSE or fullT09 by itself.
