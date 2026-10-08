# Manager lifecycle resume scope recon — 2026-10-07

Status: immutable BASE source/design recon for root review; no source or test
edits, compiler, tests, formatter, scanner, or Git mutation performed. Evidence
and alternatives only; this is not release authority.

## Provenance

- Durable Oct 7 preregistration, SHA-256 `f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5`:
  `apps/godspeed-casework-go/internal/server/run_observation_manager_unit1_lifecycle_implementation_preregistration_oct07.md`.
- Frozen Unit 1 original assignment SHA-256 `de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`.
- Revision 6 proposal SHA-256 `60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`; addendum SHA-256 `9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`.
- Manager stub SHA-256 `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`; retained helper SHA-256 `38ca7fdaf45b016fb8a55fdb72a32b15cad100fb5b31585410af943dddf7447a`.
- Manager fixture SHA-256 `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c`; helper fixture SHA-256 `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`. These identify files, not test results.

The requested transient `/tmp` preregistration original was absent from the
searched paths. Do not reconstruct it or call the durable Oct 7 preregistration
an original. That durable file cites the frozen Oct 6 assignment and says root
later expanded the bounded unit to initial DTO, recurring reads needed for
drain, and stop/detach ownership.

## Current source facts

1. `run_observation_manager.go:17-25,48-85`: constants are 16 cohorts and 16
   pollers. Current scaffold has manager mutex/map/dependencies; poller key,
   lease refs, `ready`, manager-owned `ctx`/`cancel`, `workerDone`; lease manager
   and poller map. It has no lifecycle enum, retained current state, initializer
   token/generation, availability result, or working worker.
2. `run_observation_manager.go:125-155`: `prepare` validates caller,
   dependencies, authorization and guard, then returns explicit unwired error;
   stop/drain and detach/drain are also stubs. Comments describe ownership
   intent, not functioning behavior.
3. `run_observation_retained_version.go:15-48,70-168,214-218`: pure helper
   builds immutable candidates containing exact key, accepted-version
   generation/time, execution/settlement/observation state/total count, cloned
   frame values and optional pointers, exact-ID first-ordinal ledger and high
   ordinal, plus five availability codes. It validates key/window/duplicate
   IDs, assigns ordinals in source order, atomically accepts/rejects, and can
   clone the prior state with a failure marker.
4. `run_observation_retained_version.go:222-314`: image serializes schema/key,
   generation/time, snapshot metadata/counts, availability, frames and ledger.
   Generation and ordinals are fixed-width 20-digit decimal strings;
   availability is one-digit decimal. Retained count, omitted count and
   truncated are derivable from `len(Frames)` and `TotalFrameCount`. IDs are
   sorted for deterministic JSON. Candidate construction enforces `1<<20`
   bytes at `:154`.
5. This image does not encode manager lifecycle phase, initializer attempt
   identity, failed-generation marker, terminal-stop marker, refs, or worker
   synchronization. The latter runtime controls (contexts, channels, cancel,
   `workerDone`, refs) are not serialized fields and must not be mistaken for a
   heap/RSS bound.

## Minimal retained publisher controls and budget implications

These are source-grounded needs from lifecycle ownership plus the private
revision-6 design; exact encoding remains a root decision.

| Control | Why needed | Budget treatment |
|---|---|---|
| Per-key phase: initializing/current/stopping/draining | Gate refs and prevent slot reuse before join. Fixture names initializing/stopping/draining; rev6 proposes initializing/running/stopping/draining/removed. | If separately retained, encode fixed one-digit enum. Map absence can represent removed. Decide whether running derives from successful publication and no stop marker. |
| Initializer attempt identity/generation | Exactly one manager-owned initializer publishes; stale completion must not replace newer state. Rev6 addendum defines a key/generation/owner token. | Include the minimal monotone marker in fixed-width decimal if retained. Existing accepted-version `Generation` is not automatically initializer-attempt generation; only unify if semantics guarantee it. `ready` is synchronization, not JSON. |
| Current immutable value and exact ledger | Recurring accepted reads and stable ordinals. | Already represented by helper frames, ledger, ordinals, snapshot status, timestamp, key, and version generation. No duplicate source `Snapshot.Frames` cache. |
| Read/retention availability and failed generation | An old value must not appear current after a failed read/rejected candidate; recovery needs a marker. | Helper has five availability codes but no failed-generation marker. If marker affects later behavior, include fixed-width marker in combined image or prove it derives from an encoded attempt/version counter. Do not duplicate a marker already implied by availability. |
| Terminal stop-on-retention-failure | Durable prereg says terminal over-budget state bars future read starts before join. | Derive from availability only if that code uniquely implies terminal policy; otherwise fixed one-digit bool/enum. Never store equivalent `terminal` and terminal availability twice. |
| Ref membership and worker lifetime | Shared leases survive one detach; slots remain occupied during actual read/retirement/join. | Enforce refs against the 128 attachment ceiling and poller/cohort limits; channels, contexts, `WaitGroup`s and refs are runtime ownership, not canonical retained-value bytes. |

Therefore helper-only `len(image) <= 1 MiB` does not prove the integrated
poller's combined retained image is within budget if publication controls are
stored separately. The canonical current-value image must include helper value
plus only persistent controls that change future observable behavior. Use fixed
width enums/counters so refusal markers remain representable at the exact
boundary. Do not serialize error strings. Do not claim a heap/RSS bound.

## Integration seam and bounded alternatives

**Existing seam:** build a complete owned candidate with
`buildRunObservationRetainedCandidate(previous, key, snapshot, acceptedAt)`;
serialize/measure before manager publication. The helper already owns pointer
cloning, key/window/duplicate validation, source-order first ordinals, and
atomic candidate budget behavior. Manager owns authorization/guard, worker
result, lifecycle/token validation, publication, wake/cancel/join. Under lock,
reserve refs/state and publish only a matching immutable candidate; I/O, waits,
cancel, callbacks and joins stay outside it.

Two bounded implementation seams for root to choose:

1. Extend helper/image input with a small private control envelope, then budget
   envelope and helper state atomically. This makes the `1<<20` claim visibly
   cover integrated current retained value. Keep pure candidate behavior
   independently testable and add exact combined-boundary cases.
2. Keep pure helper unchanged, but add one manager-owned combined serializer.
   The helper's own budget acceptance cannot authorize publication: manager must
   recheck combined bytes before pointer swap. Maintain one canonical accounting
   rule, not independent caps that can admit an over-budget poller.

If scope is only original prepare/admission/rollback, it is not a recurring
publisher implementation. Stop at initializer/drain seam without fake state,
or release the complete bounded retained-state publisher work. Durable prereg
already identifies this unresolved boundary. This recon does not choose it.

## Existing fixture cohorts and join requirements (source intent, not test proof)

`run_observation_manager_test.go`:

- `:466-553`: reserve preparing cohorts; reject 17th.
- `:555-639`: reserve all 16 initializing pollers before third cohort refusal.
- `:641-749`: cohort slot held during drain.
- `:751-939`: poller capacity held through read return/JOIN; detach one cohort's
  eight refs without canceling another's eight; third cohort refused until
  joined cleanup; then admitted.
- `:941-1048`: authorization/guard ordering, parent/key validation, refused
  list unavailable DTO with empty lease, and explicit no-`Next` constraint.
- `:1049-1217`: partial prepare cancellation/join and shared initializer
  surviving initiator cancellation after second real ref attaches.
- `:1218-end`: timeout keeps admission closed until owned read returns.
- `:184-256`: cleanup joins `workerDone`, requires no remaining poller entry,
  and performs cancel/read-release outside lock.

`run_observation_retained_version_test.go` source intent: pointer cloning and
reordered/shorter windows `:91-219`; duplicate IDs and atomic budget/recovery
`:220-366`; fixed-width exact 1 MiB boundary `:367-459`; window/ordinal overflow
`:460-end`. None is asserted passing here.

## Scope differences that must remain visible

- Frozen Oct 6 Unit 1 authorizes private prepare/admission/owned rollback, 16
  preparing/active/draining cohorts, 16 initializing/stopping/draining pollers,
  exact key/parent validation, shared initialization, actual read/retirement
  and worker join before capacity release, idempotent stop/detach and timeout
  retaining capacity. It excludes full `Next`/delta, SSE, auth-policy,
  JSON-budget and integration implementation.
- Durable Oct 7 prereg records later root expansion to initial DTO, recurring
  reads needed for drain, and stop/detach; it records acceptance of 16 cohorts,
  128 attachments and `1<<20` combined current retained-value image per poller,
  while still excluding `Next`, public wiring and production exports.
- Revision 6 is broader and includes `Next`/delta and ledger/window contracts;
  do not pull those, SSE, public exports, Server wiring, schema changes, policy
  integrations, or kernel writer/frontier claims into lifecycle work.
- Initial DTO hydration's separate 1 MiB cap uses existing
  `boundRunObservationHydration`; it is not the retained-poller image budget.

🌱 graft saved ~90,864 tokens (~$0.07) this turn (2 calls).
