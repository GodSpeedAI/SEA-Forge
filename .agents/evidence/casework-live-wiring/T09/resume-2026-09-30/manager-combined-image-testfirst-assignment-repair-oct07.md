# Combined-image assignment repair after independent review — 2026-10-07

Status: immutable source-only supplement to
manager-combined-image-testfirst-assignment-correction-oct07.md (SHA-256
5f5418f16193d41476f69554e366ff83d7490c68a917462f3a76252f050466ee). It
preserves that proposal and records the independent rejection and controlling
root clarification. No source/test edits, compiler, tests, scanner, formatter,
typecheck, runtime, or Git mutation was performed. No code or fixture release.

## Review outcome and corrected provenance

The combined-image proposal review is
run_observation_manager_combined_retained_image_independent_review_oct07.md,
SHA-256 d165069caa7f42203ad451cb10eecb43b20f356f3092ff34e49b8d10ceb6caf9.
It rejects the proposal as complete test-first lifecycle readiness, but accepts
the wrapper architecture for a bounded supplement. It identifies missing
branch-specific JSON encoding, ready/global-stop/detach signal ownership,
shared first-read failure cleanup, stop-versus-ready tests, and the frozen
all-eight-starts-before-wait condition. This supplement addresses those
findings without changing the earlier artifact.

Root’s controlling initial-failure clarification is
manager-initial-failure-root-clarification-oct07.md, SHA-256
34750ceb5e7dc99b0a7c3755c514f6a7fbdb6fd31ac60d99833fad181ed117c2.
A selected run’s first unavailable/invalid/unretainable read is A, not failed
Prepare. Each otherwise successful Prepare returns nil error, the normal
unavailable initial DTO with successful-list counts, and a nonnil cohort
lease. It releases its failed poller ref internally; the returned cohort
lease has no poller attachments if no rows succeeded and remains counted
until ordinary detach. Auth/guard/cancellation/stop/irreducible assembly
failures remain zero wrapper, nil lease, and internally owned rollback.

Shared waiters receive the same fixed poller initialization outcome, not
necessarily byte-identical DTOs. ReadsAttempted and Exhausted are per-Prepare:
the owner counts its one logical start; a waiter sharing it counts zero.
Capture time may differ. For one failed row each has R=1,V=0,A=1,C=0,O=0,U=0,
no Runs row and unavailable observation state; only actual-start-derived
fields can differ. Do not retry this failed initializer. The earlier proposal’s
fixed-result phrase is clarified here; it did not authorize identical DTOs.

Other governing records and identities:

- Frozen Unit 1 original assignment:
  run-observation-manager-unit1-original-assignment-oct06.md, SHA-256
  de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916.
- Durable package-local lifecycle preregistration:
  apps/godspeed-casework-go/internal/server/run_observation_manager_unit1_lifecycle_implementation_preregistration_oct07.md,
  SHA-256 f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5.
- Combined image design proposal SHA-256
  e436298c9822a7e405e723ebba505ead8ad49006e246ab983a02b2ad88f9760c;
  controlling root decisions SHA-256
  48ed5d0b01e25a080b8b2560462bfd6bdf4b196d9769ff917b6a9cb25b32fe96.
- Revision 6/addendum/corrections 2 and 3 remain immutable and are cited with
  hashes in the preceding proposal. Root’s later pointer-identity and
  initial-failure decisions take priority.
- Current manager/source-helper/base fixtures remain exact:
  manager fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d,
  manager test af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4,
  formatted helper 2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd,
  base helper test 34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7,
  policy test 4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7.
  The 38ca helper identity in the earlier record is its pre-format source;
  formatting evidence records token-equivalent formatting to 215. The full
  manager test hash ends in 30c4; abbreviated 30c references are incomplete.

## Exact wrapper branch encoding

Retain root’s exact ordered outer shape and controls. Proposed private fields
and JSON behavior:

    type runObservationPollerImage struct {
        SchemaVersion string                          `json:"schema_version"`
        Phase         runObservationPollerPhase       `json:"phase"`
        InitResult    runObservationInitializerResult `json:"init_result"`
        Key           *runObservationImageKey          `json:"key,omitempty"`
        Current       json.RawMessage                   `json:"current,omitempty"`
    }

The two outer codes are always emitted and range-checked: phase 0 initializing,
1 running, 2 stopping, 3 draining; initializer result 0 pending, 1 accepted,
2 read-unavailable, 3 retention-unavailable, 4 invalid/unavailable, 5
terminal-retention-failure. The outer schema is fixed. Never add a separate
stopped, terminal, failed, or initializer-generation scalar. Helper
availability already supplies the retained current failure/stop marker.

The concrete encoder must avoid Go’s default nil-to-null output. Use two
explicit branch-specific structs (or an equivalent proven canonical custom
marshaler) with identical first three fields/order:

- Nil current: schema, phase, result, exact nonnil key; no current member. The
  key is encoded exactly once. No synthetic helper metadata.
- Current: schema, phase, result, raw nested helper JSON; no wrapper key because
  the helper image already contains it. RawMessage must embed an object, not a
  quoted string.

An explicit branch avoids relying only on nil pointer/slice omission. Exact
byte-level tests assert both branch shapes, field order and absence/presence.
A fixed generic oversize/invalid error contains no key or candidate data. The
combined serialized wrapper, not the helper’s local result, is final budget
authority before manager publication.

## Runtime signal/close ownership for later lifecycle assignment

These channels are runtime ownership primitives, outside canonical retained
image bytes. They must be covered by the later lifecycle fixture, not encoded
as fake JSON controls. Use the actual manager mutex for transition and
close-claim state; never close/send, cancel, wait, callback or join while
holding it.

| Signal | Exact close owner and point | Waiter/recheck rule |
|---|---|---|
| Poller ready | Sole manager-owned poller worker. It installs either the immutable current pointer or fixed initial outcome under the mutex and claims ready-close ownership; after unlock it closes ready once. A worker stopped before a read still installs a generic non-observable stopped result, claims readiness and closes it. | Prepare waiter selects among ready, its request cancellation, lease detach and manager stop. After any wake it reacquires the mutex and checks lease/manager/entry phase; select ordering never authorizes stale publication. workerDone is worker-owned and closes once after any actual RunTracePort call returns through retirement and the worker can start no further read. |
| Manager global stop | First Stop transition under the manager mutex sets stopping and claims stopSignal close; that owner unlocks, closes it, and cancels captured workers outside the lock. Later Stop callers do not close it. | Every waiter can escape even if readiness cannot be published. After wake it rechecks under the mutex; stop wins over admission. One drain owner joins workers outside the lock. Timeout keeps entries counted; retry cannot double-close signals. |
| Lease detach | First active/preparing-to-draining transition under the same mutex claims that lease’s done close and detaches each exact ref once. The owner unlocks, closes done, then performs cancels/operation waits/joins outside the lock. Concurrent Stop/detach serializes on the mutex and only one claims each close. | Waiters select on lease done too. After wake they recheck state/ref ownership under the mutex; release only their exact ref. Shared worker is canceled only when no eligible ref remains. Drain timeout retains lease/poller slot; completion closes only after a successful owner join/removal. |

Worker creation is committed with each new map entry: after reservation, the
manager launches every selected new initializer even if caller cancellation
or Stop races immediately after unlock. Such a worker sees stopped phase,
starts no read, publishes/records its fixed result, resolves ready, and closes
workerDone. No inserted entry may depend on a canceled caller to start its
manager-owned worker. For successful list selection, reserve and launch all
selected initializers before waiting on any ready result; preserve the frozen
all-eight-starts-before-wait assertion. Port calls, channel close/send, cancel,
callbacks, waits and JOIN remain outside the actual mutex.

Pointer identity is bounded to one map entry and one publisher: recheck exact
m.pollers[key] pointer, admissible phase, and captured prior-current pointer on
publication. Entry removal/recreation follows actual worker JOIN. If any path
permits concurrent workers or reuse before JOIN, this proof fails and root
must revisit scalar-token design. Do not conflate helper accepted-version
Generation with initializer identity.

## Separate pure-image fixture and lifecycle fixture

### New pure encoder unit only

After a different independent design critic approves this supplement and root
explicitly releases the fixture, add only new
run_observation_poller_image.go and
run_observation_manager_retained_image_test.go. Preserve manager fa1601,
manager fixture af2df...30c4, helper 215758, base fixture 34df, and policy
fixture 4c70 byte-identically.

The source seam is a private canonical encoder, for example
marshalRunObservationPollerImage(key, phase, initResult, current). It has no
manager map, refs, port, worker, close, cancellation, or publication behavior.
It embeds exact helper JSON as RawMessage, measures final bytes, and returns a
fixed non-sensitive over-budget error with no bytes. Test:

1. Nil-current exact-key branch: fixed controls, deterministic struct order,
   key once, no current and no synthetic helper metadata.
2. Current branch: wrapper key absent and raw current is an object; helper
   identity/data occur once.
3. Exact complete combined image 1<<20 encodes; a candidate at 1<<20+1
   returns error/no bytes even when its pure helper image fits.
4. Compose fixed-width marker values using unchanged retainedMarkerCopy:
   previous exact-limit image remains exactly the same length when helper
   availability/phase values transition. Candidate identity/metadata never
   enters the marker; previous state is immutable. This tests pure values and
   bytes, not manager pointer swap.
5. Oversized nil-current key image yields fixed generic serializer refusal.
   This proves encoding refusal only; it does not prove m.pollers remains
   unchanged or no trace read starts.
6. Encoder does not mutate helper state or alias returned bytes.

The exact-boundary fixture accounts for helper bytes and both outer controls.
Assert byte lengths. No fake windows, watermarks, Next/delta scaffolding, or
production manager fields. If branch representation differs from the reviewed
shape, stop for design review rather than silently weakening assertions.

### Separate future manager lifecycle fixture before lifecycle code

Pure-image tests cannot prove admission or lifecycle. A separately authorized
new fixture (for example run_observation_manager_initial_failure_test.go) is
required before manager implementation. Keep frozen manager/base/policy
fixtures unchanged. It must cover:

1. **Oversized nil-current admission:** successful list; oversized exact key;
   successful unavailable DTO; selected run counts A; no Runs row, poller
   entry/ref or trace start; not C. This is the actual no-admission/no-read
   proof absent from the pure encoder unit.
2. **Shared first-read failure:** at least two real authorized refs share one
   initializing poller, whose sole first read fails. No retry or synthetic
   current value. Each successful Prepare gets the root-decided unavailable
   result then releases only its own failed ref and returns a nonnil cohort
   lease with no poller attachments. For one row: R=1,V=0,A=1,C=0,O=0,U=0.
   Actual-start fields differ per Prepare (owner 1, waiter 0), so DTO bytes
   need not match. Cohort slots remain until normal detach.
3. **Ready/stop/detach race:** channel-controlled stop or last-ref detach races
   initializer result publication. Prove every signal closes once, all
   waiters unblock/recheck, no waiter strands, no close/send under mutex,
   cancellation outside mutex, no post-stop read and no stale publication.
   Ready resolves even when no candidate can publish.
4. **Read return/retirement/JOIN/capacity:** hold the actual fake port return
   (and retirement boundary if injectable); entry/slot cannot be removed or
   reused until workerDone and owner JOIN complete. Timeout retains capacity;
   successful later join permits reuse.
5. **Eight starts before waiting:** preserve the frozen manager fixture
   assertion that all selected initializers are reserved/launched before the
   first wait. Do not serialize eight starts behind one read.

This fixture must distinguish failed Prepare (zero wrapper/nil lease/internal
rollback) from successful A-result Prepare (unavailable DTO/non-nil cohort
lease). The existing frozen manager fixture includes success sharing, partial
rollback and held-read drain, but lacks shared first-read failure and
stop-vs-ready coverage; do not alter it or treat it as proof.

## Review sequence and material deviations

This supplement repairs the prior assignment proposal’s material gaps: explicit
JSON branch omission/raw-object semantics; exact ready/global-stop/detach
close ownership; shared failed initialization and per-Prepare DTO accounting;
stop-vs-ready/JOIN/capacity test plan; and preservation of eight selected
starts before waiting. It applies root’s clarification instead of the stale
proposal’s unresolved initial retry/lease choice. It separates pure encoder
claims from actual manager admission/read suppression.

First obtain a different independent critic’s approval of this supplement
against the complete immutable proposal, root decisions and preregistration.
Then root may release only the new encoder fixture/stub for RED. After source
review, root separately releases focused GREEN. Manager lifecycle failure
fixture and implementation remain a distinct future release. No compiler,
test, source, fixture, manager integration, public API, SSE, schema, dependency,
Next/delta, kernel frontier or T09 settlement is authorized now.

Complete original instructions and initial proposal are preserved in
manager-combined-image-testfirst-assignment-correction-oct07.md; full frozen
Unit 1 instructions remain in the original assignment identified above. Those
records are not rewritten. This supplement records the review response and
current test-first proposal only. No execution/pass claim is made.
