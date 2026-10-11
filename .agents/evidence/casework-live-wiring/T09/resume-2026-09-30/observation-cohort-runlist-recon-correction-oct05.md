# Run-list recon correction: response-line cap is implemented

Date: 2026-10-05. Append-only correction to
`observation-cohort-runlist-recon-oct05.md`; the original record is retained.

## Retraction and corrected fact

The original recon's statement that the proposed 32 MiB line cap was still
unimplemented and that the current Go client reads a response line without a
limit is stale and incorrect. I retract that claim. The cap was implemented in
the `b4092bd` checkpoint (`feat(casework): verify T09 response cap and contract
mirrors; runtime pending`) and carried into the subsequent `8f81580` checkpoint.
The latter changed the cap retry/recovery fixtures to use a configured 1,024-byte
test cap; it did not remove or weaken the production limit.

Current source anchors:

- `apps/godspeed-casework-go/internal/adapters/sfwp/client.go:32,35-40,71-77`:
  default per-response-line limit is 32 MiB, including LF; zero selects that
  default; smaller positive test/config values are allowed; negative or above
  32 MiB values fail client configuration.
- `client.go:188-249,252-276`: RPC calls use `readBoundedLine` before
  `DecodeResponse`; overflow returns typed unavailable and poisons/closes the
  positional connection. The helper accumulates at most the configured logical
  line size. Its `bufio.Reader` can read ahead by one bounded internal chunk
  before overflow detection, as the source comment states; this is not a claim
  about strict socket bytes or an exact Go allocation ceiling.
- `apps/godspeed-casework-go/internal/adapters/sfwp/subscribe.go:121-140` uses
  the same bounded reader for subscription lines.
- `apps/godspeed-casework-go/internal/adapters/sfwp/response_limit_test.go:45-155`
  validates config bounds, exact/over boundaries for RPC and subscriptions, and
  unavailable overflow behavior. Tests later in the file cover EOF poisoning,
  inspect retry on a fresh connection, and correlated mutation/recovery without
  resending the mutation. Checkpoint `8f81580` changed the two retry/recovery
  tests (`:249-338`) to set `MaxResponseLineBytes=1024` and send 1,025 response
  bytes plus LF, exercising the configured test boundary.

## Remaining resource boundary

This correction does not change the original upstream-scan findings. A bounded
individual response line does not bound Rust's run-directory enumeration,
aggregate kernel journal reads/serialization before a response is produced,
the number of run-list rows within that line, cumulative bytes over retries or
the stream lifetime, or the cohort's aggregate `run.get` transport bytes. The
approved proposal/spec explicitly exclude these aggregate/upstream quantities.
The original recon's ownership collision, incomplete-source-signaling, and
candidate count-semantic observations remain unchanged.

No source, tests, status, or Git state was changed. This is an evidence-only
correction based on current source and the checkpointed fixtures.
