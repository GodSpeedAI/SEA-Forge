# Server request admission and timeout containment specification

Status: approved v0.1 — owner-approved for implementation — 2026-08-31
Scope: proposed `sea-forge-server` Unix-socket request admission and timeout behavior.
Normative language: RFC 2119.

## Purpose

The server's 10-second request-response deadline must not permit unbounded
queued work, detached execution from a request that was not lawfully admitted,
or side effects from a request that was never admitted. A lawfully admitted,
durable operation may outlive its client response under the existing governed
lifecycle; this proposal makes overload explicit and fail-closed.

## Problem statement

The current request wrapper returns a timeout response while its spawned
handler can continue. Some handlers acquire the run semaphore only inside that
detached work and may create case material before acquiring it. Under a hung
endpoint or a flood of requests, clients can therefore receive timeouts while
unbounded waiting tasks accumulate and later perform work.

Connection admission is separately bounded. This spec governs **request-work
admission**, not the maximum number of connected sockets.

## Requirements

- **REQ-ADMISSION-001 — bounded work admission.** Before a request can create
  a case, append a request/correlation record, acquire a run permit, initiate
  an agent operation, or cause any other governed side effect, it MUST hold a
  bounded admission permit. The capacity and queue policy are
  implementation-defined but MUST be finite and documented.
- **REQ-ADMISSION-002 — overload refusal.** When no admission permit is
  available, the server MUST return a typed `server_busy` response promptly.
  It MUST NOT queue an unbounded task or silently block the Tokio scheduler.
- **REQ-ADMISSION-003 — no-effect refusal.** A request refused under
  REQ-ADMISSION-002 MUST create no case, run, correlation record, ledger
  record, child process, network call, or agent invocation.
- **REQ-ADMISSION-004 — timeout containment.** A request that reaches the
  10-second request deadline before lawful admission MUST be cancelled and
  MUST have the same no-effect property as REQ-ADMISSION-003. For an
  id-bearing governed mutation, a timeout after admission MUST report its
  durable request or run locator and MUST NOT create a duplicate operation on
  retry. Read-only and no-ID requests need not mint a durable locator.
- **REQ-ADMISSION-005 — durable admitted work.** A lawfully admitted durable
  mutation MUST carry a caller request ID or a server-generated durable request
  locator before it crosses its durable governance boundary. It MAY outlive the
  client response, but its continuation MUST be bounded by the applicable
  run/agent timeout, discoverable through `request.get_status`, and settle
  through the normal evidence-based lifecycle. A no-ID mutation that cannot be
  assigned such a locator MUST fail closed before admission.
- **REQ-ADMISSION-006 — authority before effects.** Admission is not
  authorization. Existing authority, identity, policy, and settlement rules
  remain mandatory and must be evaluated in their existing order.
- **REQ-ADMISSION-007 — observability.** The server MUST emit structured,
  redacted diagnostics for admission, busy refusal, cancellation before
  admission, and post-admission timeout. Diagnostics MUST include a request
  identifier when supplied and MUST NOT include secrets or request payloads.

## Non-goals

- Changing the 10-second request-response contract in `spec-full.md` §11.1.
- Cancelling or rewriting an already committed durable operation merely because
  its client disconnects.
- Adding a network API, background queue service, async runtime to a kernel
  crate, or a fallback that bypasses authority.

## Required proof

Conformance must include real Unix-socket teeth tests that:

1. saturate a hung endpoint and submit more unique requests than admission
   capacity; every excess request receives `server_busy` and has no
   correlation, case, child, or network effect;
2. hold requests until the request deadline and prove pre-admission timeout
   creates no durable state;
3. prove an admitted operation remains discoverable and settles exactly once
   after its client times out;
4. retry the same request identifier across busy and timeout paths and prove
   no duplicate authority or side effect;
5. submit a no-ID durable mutation and prove the server assigns a durable
   locator before admission or fails closed with no effect; and
6. run variation with concurrent clients, disconnects, and recovery/restart.

A passing happy-path request, an exit code, or an in-memory task-count claim is
not sufficient evidence. The proof must inspect the durable records and the
absence of effects for refused requests.

## Acceptance and handoff

Implementation is not complete until all requirements map to focused tests and
evidence, the normal server conformance suite remains green, and an independent
review confirms the no-effect claims. Any selected queue capacity, request
classification, or cancellation point is implementation-defined and must be
documented beside the implementation before it is relied on operationally.
