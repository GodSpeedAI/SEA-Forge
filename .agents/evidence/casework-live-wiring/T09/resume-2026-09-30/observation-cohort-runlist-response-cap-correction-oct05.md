# Append-only correction: scoped run-list response cap is implemented

Date: 2026-10-05. This supplements
`observation-cohort-runlist-recon-oct05.md` and its first correction. The
original recon remains unchanged.

The earlier recon's claim that the 32 MiB response-line cap was unimplemented
is retracted. Production Go SFWP client source has a validated default
`MaxResponseLineBytes` of 32 MiB including LF (`client.go:32,35-40,71-77`).
`readBoundedLine` is used before RPC JSON decode and for subscription events
(`client.go:188-249,252-276`; `subscribe.go:121-140`). It accumulates at most
the logical configured line cap; its bounded `bufio.Reader` may read ahead by
one chunk before overflow detection. This does not assert an exact Go heap
allocation ceiling or strict socket-byte consumption.

Checkpoint `b4092bd` added the implementation and response-limit suite.
Checkpoint `8f81580` updates the scoped retry/recovery fixture cases to set a
1,024-byte limit and exercise oversized responses against that limit; it does
not defer or remove the 32 MiB production default. `response_limit_test.go`
validates invalid configuration before dial, the default and hard upper bound,
exact and over-limit RPC/subscription lines, overflow poisoning, inspect retry
on a fresh connection, and correlated mutation recovery without resending.

The correction leaves intact the resource bounds excluded by the approved
contract: the upstream server still enumerates run directories for `run.list`,
there is no pagination, and the per-line cap does not bound aggregate kernel
reads/serialization, list row count below the line cap, cohort-wide transport
bytes, retries in aggregate, or stream-lifetime bytes. See
`client.go:252-276`, `response_limit_test.go:45-155,249-338`,
`.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md:39-43`,
and `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:82-106`.

This correction is source/evidence-only; no code, test, status, or Git file was
changed.
