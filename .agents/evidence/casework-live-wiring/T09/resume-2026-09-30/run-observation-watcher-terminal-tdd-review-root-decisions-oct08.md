# Source-review boundary decisions

Date: 2026-10-08
Applies to source grant bfcfa5cd and frozen result b1705384.

The independent critic identified two proof gaps. Preserve the frozen result
and await its immutable verdict before a fresh different builder repairs them.

1. The pre-read independent-watcher contract requires a group-6 subcase in
   addition to existing invalidation during a held read. Gate an actual
   existing injected Current/authorization call before the trace port, attach
   an independently valid watcher, invalidate the initiating watcher, release
   the dependency, and prove the valid survivor authorizes the shared read.
   The invalid watcher fails with zero DTO/nil lease; survivor remains eligible
   with correct shared read accounting. Use real dependency state and bounded
   channels, not a production lifecycle hook or private-field transition.
   Current source may call the trace port without that required check; report
   that actual missing boundary as expected failure and release held callbacks.

2. The bounded canonical outer encoder deliberately discards oversized bytes
   and returns errRunObservationPollerImageUnavailable (poller_image.go:90-94).
   Requiring its error to be nil for oversize would contradict that contract.
   Prove the cap cause independently: validate candidate key/accepted inner,
   marshal the existing runObservationPollerImageWithCurrent type with exact
   schema/phase/result and inner json.RawMessage, require raw marshal success
   and actual complete raw length above the unchanged cap. Also require the
   canonical bounded encoder's exact refusal sentinel and nil bytes. A generic
   marshal error alone is insufficient proof. Do not change the encoder or cap.

These decisions clarify existing required behaviors; they grant no source
edits or execution by themselves. Independent review remains evidence-based.
