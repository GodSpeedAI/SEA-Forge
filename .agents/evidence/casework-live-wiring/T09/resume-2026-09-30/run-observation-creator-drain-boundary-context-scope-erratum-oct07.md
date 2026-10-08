# Creator-context scope erratum — 2026-10-07

This immutable erratum clarifies the scope of
`run-observation-creator-drain-boundary-correction-supplement-oct07.md`,
SHA-256 `37d943adc9756af9e8c6e75fbdb32b62805811b4920165b1edc863c494dd7c26`.
The original supplement remains unchanged.

The phrase “Every Prepare path” in its “Corrected creator and cancellation
order” section means every **admitted Prepare creator that has created its
local derived request context**. A request rejected during identity,
authorization, guard, dependency, stop, or capacity checks before that local
context is created has no context bridge to cancel or join. Once an admitted
creator creates the context/bridge, every subsequent return path must cancel
that local context after its last list/initial-wait use, join the bridge, then
close the exact `creatorDone` and balance the global operation registration.
This remains before any wait for that creator's own rollback drain. The local
cancel remains isolated from the shared worker context and does not itself
drain a successful lease.

This wording clarification adds no source state, test hook, lifecycle behavior,
or implementation authority. It changes no source, fixture, earlier immutable
assignment, proposal, or review. The independent critic should read the
correction supplement and this erratum together.
