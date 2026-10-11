# T08 native live EventSource builder handoff

Root recorded this note from the builder's delivered instructions/results after its
evidence write was refused by the read-only `.agents` mount. No runtime evidence is claimed.

## Original bounded assignment

Source-only changes to the test stack retention helper, a live-tagged Go server test,
and an optional UI E2E browser helper. Preserve production retention, Vite configuration,
adapters, dependencies and the global ladder. Use actual kernel-backed authority/relay/server
and Chromium native EventSource through agent-browser; no fake feed/world/polyfill or
fetch-only substitution. Preserve existing listeners; own only isolated test processes.

Retention-one phase must prove eviction, ordered resync_required(K) and authoritative
snapshot(K) with the same cursor, real execution/durable trace, actual stream interruption,
bounded reconnect/resync after another real mutation, and disposal during pending retry.
Retention-two phase must prove native replay from retained C to real L without resync,
then disposal. All listeners/sessions/processes need bounded waits and owned cleanup.
No compile/test/build/vet/Bun/browser/runtime command was authorized to this builder.

## Delivered source and claims

- `internal/livestack/stack.go`: original AssembleStack delegates to a test-only retention helper.
- `internal/server/native_events_live_test.go`: new live-tagged test with real isolated cell,
  acquired loopback listeners, private Vite proxy configuration, and isolated browser session.
- `e2e/native-events-live.ts`: real HTTP adapter and native EventSource session login.

Builder reports phase one executes a real item, checks durable activation/settlement/completion,
retention-one bounds and the exact resync/snapshot pair, and compares streamed/retained/current
standing. It then force-closes the stream, observes error and disposes during retry, waiting
1.75 seconds beyond the configured 1.5-second cap before checking no requests/subscribers.

Phase two uses a genuine ADD_DISCRETIONARY_WORK rather than EXECUTE_ITEM, because execution
emits multiple real frames and can evict C from a two-revision store. This choice was authorized
by root. Builder asserts one durable plan_mutated delta, retained C and new L, a reconnect
request with last=C, exactly one snapshot L without resync, and disposal bounds.

## Limitations and independent review

Builder ran only gofmt and diff whitespace checks. Source files are stable; Go/TypeScript
correctness, browser CLI compatibility and all live assertions remain unverified. The delivered
phase-one description disposes before retry; root flags the original requested post-interruption
resync-after-new-mutation coverage for independent completeness review. No task approval,
passing runtime result or settlement follows from this source handoff.
