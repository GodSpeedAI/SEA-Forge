# Combined retained-image lifecycle design proposal

Date: 2026-10-07  
Status: source-only design proposal for independent architecture review; no lifecycle implementation or fixture change is released.

## Purpose and authority

This proposal resolves the lifecycle-unit boundary left open by the immutable Oct 7 preregistration. It chooses a manager-owned combined encoder around the existing pure retained helper. Keep `run_observation_retained_version.go` behavior unchanged; its current source differs from the frozen algorithm identity only by the exact `gofmt` formatting record in this directory.

Reviewed source/design inputs include:

- `manager-lifecycle-resume-scope-recon-oct07.md` (preregistration provenance f48df1c8), which inventories current manager/helper fields and the missing combined budget boundary.
- `run-observation-manager-concrete-proposal-revision6-oct06.md`, its revision-6 addendum and independent reviews, and retention corrections 2 and 3 with their reviews.
- `run-observation-manager-root-private-design-decision-resume-oct06.md`, which accepts private limits of 16 cohort slots, 128 attachments, and one combined `1<<20` serialized current-value image per poller under the operator's standing instruction.
- `run_observation_manager_unit1_lifecycle_implementation_preregistration_oct07.md` and frozen `run-observation-manager-unit1-original-assignment-oct06.md` (SHA-256 `de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`).
- Current manager `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`, current manager fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`, formatted retained helper `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`, and frozen helper fixture `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`.

This is a design record only. It does not claim existing fixtures passed, does not add a lifecycle behavior claim, and does not release code. A different critic must review it against the cited records. Root retains all implementation, test, and semantic release decisions.

## Combined image and exact field inventory

Use one manager-owned canonical serializer per poller. It composes lifecycle controls with the existing helper image; the manager layer is the final admission check before publishing a candidate. Keep the helper's own serializer and bound unchanged as a local first check.

Conceptual private DTO, with exact Go spelling left to the implementation reviewer:

```go
type runObservationPollerImage struct {
    SchemaVersion string                    // constant outer format identity
    Phase         uint8                     // one decimal digit
    InitResult    uint8                     // one decimal digit
    Key           *runObservationImageKey   // only while Current is nil
    Current       json.RawMessage           // raw helper JSON object, only when nonnil
}
```

Fields are emitted in that order by an explicit struct, never a map.

| Retained fact | Canonical encoding | Ownership and duplication rule |
|---|---|---|
| Outer image format | Fixed constant string | Manager image only; distinct from the helper's inner schema version. |
| Poller phase | One-digit enum: initializing, running, stopping, draining | The only generic manager stop/drain phase. No separate stop boolean. Removed means no manager entry. |
| Initializer result | One-digit enum: pending, accepted, read-unavailable, retention-unavailable, invalid/unavailable, terminal-retention-failure | Always present at fixed width; never preserve an error string. When `Current` is nil, this distinguishes the initializer result without inventing an accepted snapshot. |
| Exact key | In the `Key` branch only when `Current == nil`; otherwise it appears in the helper image | Serialize once. The empty branch contains only the exact case/run/plan key and controls, never fake trace metadata, frames, or a fabricated helper state. |
| Current retained helper value | Raw nested JSON from `marshalRunObservationRetainedImage`, only when current state exists | Serialize once as `json.RawMessage`; it already contains exact key, accepted-version generation, timestamp, safe snapshot metadata, current frames, full exact-ID ledger, ordinal high-water, and helper availability. Do not mirror any of those fields in the wrapper. |
| Lease/ref membership | Runtime map/reference ownership, capped at 128 attachments | Not serialized into the per-poller current-value image. Enforce actual count under the manager mutex; it is synchronization/ownership state, not retained trace data. |
| Worker/readiness/cancellation | Runtime worker, ready/workerDone channels, context/cancel, timer | Not serialized. There is one manager-owned worker and no second physical-read path. |
| Dependencies and lock | Manager mutex, ports, authorizer, guard, verifier, clock | Runtime dependencies; never serialized. |
| Last-read time, raw snapshot, candidate, temporary DTO/delta, error | Not retained | Do not add a timestamp duplicate, `ports.RunTraceSnapshot` cache, duplicate frame slice, or arbitrary auxiliary error string. The worker waits at least one second after a completed read before its next start, so no retained poll timestamp is needed. |

For a nonnil current value, `Current` is the exact byte slice returned by the helper marshaler embedded as a raw object, not a quoted JSON string. For a nil current value, `Current` is absent and the exact key is encoded once in `Key`. Never put both a wrapper key and helper key into the same image. Do not retain `Snapshot.Frames` beside helper frames.

The fixed-width outer `Phase` and `InitResult` stay present in every image. Helper availability is already a one-digit field. Thus a marker-only transition changes codes, not width. There is no new manager `stopScheduled`, `failed`, or `readUnavailable` flag: once a helper value exists, derive read/retention/terminal/stop availability from its existing `Availability`; use `Phase` for generic lifecycle admission/drain. When no helper value exists yet, `InitResult` supplies the fixed-width initial result.

### Initializer result codes proposed for review

Reserve fixed one-digit codes: 0 pending; 1 accepted; 2 read unavailable; 3 retention unavailable; 4 invalid/unavailable; 5 terminal retention failure. These codes describe only initialization while `Current` is nil. After an accepted initial publication the code remains 1; later read/retention availability comes solely from the helper state. No raw adapter error or candidate-derived value crosses the initializer result.

The original revision-6 addendum illustrates an explicit scalar initializer generation/token. This proposal uses a stricter ownership invariant instead: each map entry owns exactly one worker, and only that worker publishes. Publication rechecks both that `m.pollers[key]` is the exact same poller pointer and that the entry remains in an admissible phase with the same captured prior-current pointer. The manager cannot remove/recreate a key until the old worker has returned and been joined, so pointer identity plus single-worker ownership prevents an old completion from publishing into a replacement entry without a second counter. This intentionally does not reuse the helper's accepted-version generation as a lifecycle token. The critic/root should reject this simplification if any allowed path can overlap initializer workers or replace an entry before join; in that case use a separate checked-overflow entry/initializer generation and account for it explicitly. Never conflate it with the helper's accepted-version generation.

## Admission and publication algorithm

1. Under the manager mutex, reserve a cohort slot before list work as already required. Validate the selected exact key/parent through the existing guard/selection contracts outside the mutex.
2. For every selected key, decide attach/current/shared-initializer/new-initializer under the mutex. For new entries, first construct and measure the nil-current image containing only the key, `Phase=initializing`, and `InitResult=pending`. If it exceeds `1<<20`, retain no poller entry, attach no ref, and start no trace read. Treat the selected item as unretainable (`A`) rather than capacity-limited (`C`); confirm this count mapping with root before implementation.
3. Reserve all selected poller entries and refs before waiting for any initializer. Launch their manager-owned workers after unlocking; do not serialize eight selected starts behind the first read. The existing `TestRunObservationManagerPollerCapCountsInitializingEntries` is frozen and already covers poller admission while initializers are outstanding; preserve its channel-controlled assertion that all reservations/starts precede waiting. Never call a port, wait, cancel, send a callback, or join while holding the manager mutex.
4. Each poller has one worker/read at a time. Before a read start, under the mutex verify exact map-pointer identity, phase is initializing/running, and no retained helper availability forbids another start. Release the lock before `ReadRunTrace`. After a completed read, wait at least one second before the next start. Port retries, admission, cancellation, and retirement remain owned by the existing `RunTracePort` path.
5. Capture the immutable prior pointer, call the unchanged pure candidate builder, and build the complete outer image before publication. For an accepted helper result, encode controls plus the candidate helper JSON. Reacquire the mutex and publish only if the map still contains that exact poller pointer, phase remains admissible, and the prior pointer is still current. The worker is the only value publisher; attach/detach can change phase but not publish a competing value.
6. If helper encoding or the outer image is too large, discard the entire candidate. With a prior value, prepare only a copy of the prior safe value carrying the fixed marker appropriate to the validated candidate (recoverable nonterminal retention-unavailable, terminal retention failure, or helper stop-scheduling). Encode that old value plus controls before publication. The complete old ledger and prior metadata remain unchanged; a marker cannot evict or partially add an ID. If the marker value itself cannot fit, stop and report the contradiction; do not reset the ledger or publish a partial image.
7. For an initial candidate rejected before any prior value exists, publish no helper value. Update only the fixed `InitResult` code and lifecycle phase. A first terminal over-budget result moves to stopping before the worker can start another read. A nonterminal retention/read failure can remain retryable only if root confirms the intended Prepare/lease result while no current state exists.
8. Close the runtime ready channel exactly once only after the fixed initializer result or full canonical current pointer has been installed under the lock. A canceled initiating `Prepare` releases only its own reference; it cannot cancel publication needed by surviving authorized refs. If the final authorized reference leaves, mark stopping under lock, cancel outside it, join the actual worker outside it, drain operations/notifiers, and only then remove the entry/release capacity. Timeout leaves it counted and stopping/draining.
9. For a terminal over-budget candidate with a prior state, atomically publish only its prior safe copy with terminal-retention availability and move to stopping under the same lock; no candidate terminal standing/frame/count/time is retained or exposed. For first-terminal-over-budget with no prior value, record only the fixed initial result and stopping phase. In both cases set the no-future-read state before join, let an already-started read/retirement finish, then join and drain. This follows correction 3 and avoids a circular “join before stop” order.

The 16 cohort slots, 128 attachment ceiling, and exact combined per-poller byte cap are current private root decisions. They do not imply any bound on temporary allocations, old versions held by readers, transport/decoded buffers, Go heap, or RSS. The initial DTO still uses `boundRunObservationHydration` once and its separate 1 MiB limit.

## Test delta and frozen fixtures

Do not edit `run_observation_manager_test.go`, `run_observation_retained_version_test.go`, or `run_observation_retained_policy_test.go`. Existing manager tests cover cohort/poller admission, 16 initializing entries, in-progress joins, shared initializer survival, rollback, and timeout retention. Existing helper/policy tests remain the semantic authority for the pure candidate and its marker copies. No test result is claimed by this proposal.

The existing suites do not test manager controls composed with the helper image or the nil-current image. If root approves the fixture delta, add only the new file `run_observation_manager_retained_image_test.go`. Its focused cases:

1. Encode a nil-current poller and assert the exact key appears once, `Current` is absent, the image has no synthetic execution/settlement/frame/ledger, controls use the fixed code widths, and the canonical bytes are deterministic.
2. Build a complete current helper value whose combined outer image is exactly `1<<20`; assert it is accepted. Transition only availability/phase markers and assert byte length is unchanged and still exactly at the boundary.
3. Build a complete candidate whose combined image is exactly `1<<20 + 1`; assert no candidate state or candidate ID/ordinal is published, the prior full ledger and all safe fields remain byte/deep-equal except the fixed marker, and the marker image has the same length as the prior exact-limit image.
4. Give a nil-current key whose minimal image exceeds `1<<20`; assert it is not retained in `m.pollers` and causes no trace read.
5. Exercise stale publication with two distinct poller pointers for the same key: completion from the old pointer is ignored after the map names the newer entry. The permitted removal protocol must still join the old worker first.

Use well-formed safe fixture values and direct byte-size assertions. Tune only an opaque valid field in test setup; do not add production padding/pruning. A reviewer must determine whether the new focused file is needed or these cases can be added to some already authorized *new* fixture without changing any frozen file. If current manager behavior cannot support the assertions, report the scope dependency instead of weakening them.

## Decisions required before a source assignment

1. **Initial no-current outcome:** Choose how a valid selected row is represented in the initial DTO and lease when its first read fails or a first nonterminal candidate cannot fit. The fixed `InitResult` safely records the outcome, but the Unit 1 contract must say whether Prepare returns an unavailable row with a lease that can recover, or fails/rolls back. Do not synthesize a retained trace snapshot.
2. **Oversized nil-key image:** Confirm that the selected row counts as unretainable (`A`) and that no poller entry/read is created. Confirm error/DTO behavior without exposing key content.
3. **Single-worker token proof:** Confirm pointer identity + one manager-owned publisher is sufficient in place of a scalar generation token. If not, choose the separate lifecycle generation, checked overflow action, and its byte accounting. The helper's `Generation` remains only the accepted candidate version.
4. **Runtime reference accounting:** Confirm lease/ref maps and cohort counts remain runtime ownership outside the per-poller current-value image, while the admitted 128 ceiling is enforced exactly. Do not call that a whole-manager memory bound.
5. **Outer-bound refusal:** Confirm policy for an initial candidate whose helper image fits its local cap but whose combined image does not. No earlier candidate bytes may be published; no old value exists to mark. For a later candidate, keep the complete old ledger, mark atomically, and recover only on a later complete candidate that fits.
6. **Boundary test authority:** Approve the narrowly scoped new focused test file only after source review. Keep all three existing fixtures byte-identical. No tests, compiler, build, scanner, or runtime command was run for this proposal.

## No source release

This document requests independent design review, then root decision. It grants no implementation or fixture-write authority. After any future release, the implementation must first receive a fresh independent source review and root approval before the sole compiler owner runs focused RED/GREEN or broader gates. No `Next`, delta/SSE, public V4/server wiring, schema, auth-policy, kernel writer/frontier, dependency, or lifecycle-independent refactor is within this proposal.

No tests, compiler, build, scanner, Git operation, runtime check, or external write was run for this document.

