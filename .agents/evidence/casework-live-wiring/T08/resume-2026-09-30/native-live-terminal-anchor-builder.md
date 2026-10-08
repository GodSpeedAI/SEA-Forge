# Native harness terminal cursor repair: original instructions and result

Fresh Luna builder t09_wiring_scope was assigned only native-events-live.ts and narrowly
native_events_live_test.go after the takeover critic's static rejection. Root retained
the compile token until stable source; builder ran no compile/test/runtime command.

Original bounded requirements: replace receipt/head equality with actual terminal published
cursor identity; keep durable activation/settlement/completion checks and retained K/L
resync/replay/disposal assertions; validate EXECUTE_ITEM on task_prepare itself; remove
impossible ADD offer assertions and document direct protected T06 ADD as actual feed setup,
not UI action exposure. No production projection change, fake native source/frame/cursor,
dependency, contract amendment or assertion weakening.

Root explicitly required an authoritative event-tail identity, not only250ms quiet time.
Builder verified run_case_mutation awaits publication before returning, publish_event appends
the persisted EventFrame before broadcast, and detail contains the exact TraceEvent ID.
Case cev_* IDs and event-frame opaque cursors are distinct and must never be conflated.

Result: Go test-only hook reads the last actual durable case trace ID/kind, finds its same-case
persisted SFWP frame via EventsGetRange after C (limit500), fails if absent/mismatched, and
waits for exact Store.At(K), case/summary, Head=Oldest=K, retention1 and observed case cursor
at least K. It matches the actual tail kind, allowing legitimate milestone_achieved after
item_completed. TS completion counters remain separate, target offer is local, and protected
ADD feeds remain real. Ret1 C-to-K/offlineL/resync+same-L snapshot and ret2 retained-C replay
and pending-retry disposal assertions remain.

Builder reports gofmt and whitespace checks only; no runtime approval. Root transferred sole
compile ownership to independent critic t07_live_confirmation for actual native/shared gates.
Direct fixture trace reads are explicit test verification; production gateway authority still
uses SFWP. Original static rejection remains immutable alongside this result.
