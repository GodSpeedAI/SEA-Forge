# Initial run hydration payload cap — original bounded assignment

Root decomposes one private prerequisite from the approved T09 proposal:
RunTraceObservation payload JSON must fit1MiB by deterministic oldest-frame
omission with accurate counts. This does not release manager/SSE/public wiring.
Read applicable instructions, Graft/Neatcode, approved proposal39/43, normative
spec/contract and actual Go RunTraceObservation/RunTraceFrame declarations.

## Phase1 declarations and fixtures only

Native apply_patch writes ONLY two NEW files in
apps/godspeed-casework-go/internal/server/: run_observation_hydration_cap.go
and run_observation_hydration_cap_test.go. Private signature:
boundRunObservationHydration(contract.RunTraceObservation)
(contract.RunTraceObservation,error). Source body is ONLY typed KindUnavailable
test-first stub. No algorithm, source reads, wiring or public type changes.
No compiler/tests/scans/Graft build/Git/status/debt/evidence operations. Read-only
gofmt diagnostics allowed; native fixes only. Return two hashes, full actual
diff, deviations and freeze. Independent critic gets this ORIGINAL and full
result; source approval precedes separately root-granted actual assertion RED.

## Eventual behavior pinned by fixtures

Cap exactly1<<20 bytes of encoding/json serialization of the returned
RunTraceObservation payload, excluding outer StreamEvent and SSE framing.
Trusted caller owns scope/auth/identity/safe-kind validation. At most8 runs and
1024 retained frames/run are supported; reject an over-bound input with typed
KindUnavailable/no partial result rather than unbounded processing. Frame count
metadata must be consistent: retained equals length, total>=retained, omitted
equals total-retained, truncated equals omitted>0. Reject inconsistent counts
or an unparseable frame timestamp with typed unavailable/no partial result.
Do not add ID grammar conversions or infer execution/settlement.

If the input fits, preserve every field and order. Otherwise remove globally
oldest frames by parsed RFC3339Nano instant, then complete unmodified RunID bytes,
then complete EventID bytes ascending for instant ties. Keep surviving frames
in their original per-run order; don't reorder runs. For each affected run retain
original TotalFrameCount, update RetainedFrameCount/omitted/truncated exactly,
including omissions already made by the trace-port ring. All cohort count
pointers, identities, observations, standings, states and hydration budget stay
unchanged. Remove only as many oldest frames as needed for the cap. Do not drop
required run metadata. If metadata alone cannot fit after all frames are removed,
return typed unavailable and no partial envelope; emit no oversized payload.

Never mutate caller data. Successful output must have independent run/frame
backing arrays and independent mutable metadata pointers, including cohort count
pointers and optional frame execution_status/exit_code. Strings are value fields.
Bounds apply to the projected payload only, not aggregate transport/journals or
upstream enumeration. Implementation must avoid O(frame_count) repeated full
payload marshals; a bounded search over oldest-removal prefixes is acceptable.
No implementation is released until accepted actual assertion RED.

## Required explicit tests

- Empty and short well-formed payloads fit with complete field preservation.
- Exactly-cap fits unchanged; cap+1 removes the explicitly oldest frame and
  final actual marshaled bytes fit. Fixture byte sizing may use independent
  encoding/json measurement, not the production helper/comparator.
- Multiple runs exceed cap; explicit survivors distinguish global timestamps,
  equal instants with differing offsets, complete run-ID/event-ID tie ordering,
  and original per-run order. Include a deliberately unsorted per-run trace.
- Existing trace-ring omissions remain in final exact total/retained/omitted/
  truncated values. All cohort counts/budget/standing/state fields preserved.
- Caller unchanged after success/failure; mutating returned arrays and count/
  optional frame metadata pointers must leave the original unchanged.
- Irreducible metadata >cap fails typed unavailable with no partial result.
- More than8 runs, more than1024 frames/run, inconsistent frame counts and
  invalid timestamp are independently rejected. Distinguish expected error
  classes, while positive cases expose the throw-only stub.

Use explicit expected IDs/values and targeted assertions. No sleeps/network or
new dependencies. Reject/rebuild with a DIFFERENT builder for any critic defect.
Later production source and fresh focused/server/module/canonical gates require
separate root release. T09 remains partial.
