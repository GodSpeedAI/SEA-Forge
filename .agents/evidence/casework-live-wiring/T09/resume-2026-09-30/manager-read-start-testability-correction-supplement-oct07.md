# Read-start lifecycle proposal correction supplement

Date: 2026-10-07  
Status: proposal-only correction after independent rejection; no lifecycle
source, fixture, or implementation release.

This supplement corrects the three findings in
`manager-read-start-testability-corrected-proposal-independent-review-oct07.md`
against corrected proposal SHA-256
`979fbdf521377c58f2615a2b3111c089fa9e0869fd350a0e4c973ae3c9166f8b` and its
archived assignment SHA-256
`28313e82372adfc7ce55cb008594da796c3c43cb06ce9682b1e5bd0fd45de25f`. The
proposal work was interrupted by separate key-extraction, primitive snapshot,
and local mise-help tasks; those tasks did not implement this lifecycle.
The complete repair request is archived in
`manager-read-start-three-finding-repair-assignment-oct07.md`.

## Corrected three-clause matrix

| Finding | Required amendment to the proposal and test-first matrix |
|---|---|
| Cohort admission | Make `beginPrepareOperation` the first manager transition after caller/auth/guard checks and before list/read. Under `m.mu`, reject if stopping or if the admitted preparing/active/draining cohort count is already 16; otherwise reserve one exact cohort lease, record that lease in the manager's cohort membership, and register the Prepare creator operation atomically. Return that owned lease/operation to this Prepare. Keep the reservation counted while preparing, active, and draining; release only the exact lease owner after its work/ref cleanup and any actual worker JOIN required by that lease. Operation WaitGroup registration alone is not a cohort slot. The existing whole-Prepare test requires a 17th preparing cohort to make zero additional list/trace calls and a 17th active cohort to make zero calls (`run_observation_manager_test.go:488-553`); preregistration requires atomic lease reservation before list/read and counts all three cohort phases (`run_observation_manager_unit1_lifecycle_implementation_preregistration_oct07.md:92-102`). Auth, guard, list/port calls and callbacks stay outside the mutex. Stop closes admission under that same mutex before waiting outside it, preventing a new positive operation registration after Wait begins. |
| Shared first-read error and oversized key | Extend the combined manager fixture with both missing successful-A cases. For a single real initial `ReadRunTrace` failure shared by two distinct successful Prepare leases: both return nil error and nonnil, distinct cohort leases; each gets the same fixed-poller failure outcome, but DTOs are per-Prepare snapshots, not byte-identical (`ReadsAttempted=1` for the invocation owner and `0` for the shared waiter, with each capture time/counts produced for that Prepare). Assert one poller/worker and one read total; A counts `R=1,V=0,A=1,C=0,O=0,U=0`, no Runs; each Prepare releases only its exact failed ref once; worker return/retirement and `workerDone` JOIN precede capacity reuse. This follows the root initial-failure clarification and root ref-owner decision. Also select a valid run whose `planItemID` is long enough that its *combined nil-current image* exceeds 1 MiB. Preserve a valid bounded `caseID`/guard identity. The manager must return successful A with nonnil cohort lease, `R=1,V=0,A=1,C=0,O=0,U=0`, `ReadsAttempted=0`, no poller entry/ref/worker and no port call. This is image/retained-state refusal after a valid selected candidate, not capacity C. `selectObservationRuns` checks supported execution ordering and retains at most eight rows (`run_observation_selection.go:14-49`) and does not cap `planItemID` length; still verify the constructed fixture passes upstream validation. A pure encoder refusal test proves only encoder refusal, not manager classification or absence of an actual manager read. |
| Stop lock discipline | Replace the ambiguous `stopAndDrain` sentence with this sequence: under `m.mu`, set `stopping`, claim the single stop/drain owner, and snapshot owned cancel functions and the work to join. Unlock. Then invoke the snapshots' cancel functions and close any owned signals outside the mutex; wait for registered Prepare operations outside it; only after creator launch operations complete, join actual workers and release reservations according to exact lease ownership. Never invoke a cancel function while holding `m.mu`. Keep the existing `claimPollerReadStart` eligibility linearization, mandatory runtime `runPollerFromClaim` continuation, immutable launch list, and launch-every-owned-worker-once-before-any-wait behavior from the root clarifications. No callback/test hook, new result code, serialized field, or budget-control field is added. |

## Implementation boundary, source facts, and remaining proof

The current unimplemented scaffold has only the manager mutex and poller map;
`prepare` runs caller construction, authorization, and guard before returning
the intentional unwired error (`run_observation_manager.go:48-58,116-141`). The
current lease has per-lease poller membership but no manager cohort registry
(`run_observation_manager.go:75-78`). Therefore the revised proposal must
name a manager-owned exact-lease registry (or an equivalent single source of
cohort cardinality) and an atomic admission transition; it must not count the
creator WaitGroup as the cohort limit. Proposed method names/signatures remain
private implementation choices for root review. No separate scalar count is
needed if registry length is the sole cardinality source.

The actual manager test already exercises 16 preparing cohorts blocked in list,
rejects the 17th before another list/trace call, and rejects a 17th active
cohort (`run_observation_manager_test.go:488-553`). Preserve it. The existing
poller-cap fixture starts all 16 initial reads across two eight-run cohorts
before releasing any, rejects the next initializer, and then joins/drains
(`run_observation_manager_test.go:555-639`). Preserve that all-eight-starts
before-wait requirement unchanged. These existing cases do not replace the two
missing A cases specified above.

The source basis includes Unit 1 original assignment SHA-256
`de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`, lifecycle
preregistration `f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5`,
revision 6 `60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`,
its addendum `9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`,
and retention corrections 1/2/3
(`449c73301de85794e9aec3093e82a88fdba8c60e46f207cd30454a15f1ce0524`,
`3494e3de99b2f5d3fe725a85971dccd0d70cef226da257ce237cce7fba2d72e1`,
`b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`). For
the ownership and worker ordering details, retain root ref-owner decision
`5b9888e5a7212b8d0696f790822f08e718dd03a71f51b28c2bec2ea0098f8df4`, initial
failure clarification `manager-initial-failure-root-clarification-oct07.md`,
worker launch clarification
`manager-worker-launch-root-clarification-oct07.md`, and read-start decision
`e1b1db0e7155ff162869c54845d0251783c72e4ed2837f3961ac91e58df602ac` with its
review `0e6f3385f703aaf3eeb5832b7b66414dec2d85984afafea3f2f1e4b9a830612c`.

Current private source identities carried forward: key
`a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b`, manager
scaffold `ab9f1c35757aecd8c8c02de300a95205a1bd07aa5e39ebe8ae056dbc8878fe62`,
manager fixture
`af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`, image
encoder `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`,
image fixture
`cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`, retained
helper `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`, and
base helper fixture
`34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`. The
policy fixture's previous `4c70...` hash is pre-format; root observed its
whitespace-only formatted version in both copies and will provide the new full
identity. It is pending here, not asserted as current.

## Deviations and release status

Relative to corrected proposal `979...`, this amendment requires cohort lease
reservation and cap refusal in the pre-list/pre-read admission transition;
adds explicit shared real first-read failure and oversized nil-current key A
tests with their exact counts, ownership, and JOIN conditions; and prohibits
cancel invocation under the mutex. It retains the approved eligibility and
launch model, the existing code/phase ranges, all-eight start ordering, and
all prior scope exclusions. It adds no lifecycle implementation, fake seam,
new enum, persisted/serialized field, or unrelated field. It does not claim
that the encoder-only fixture proves manager admission behavior.

This is a proposal correction only. A different independent source reviewer
must review the complete original assignment, corrected proposal, rejection,
root decisions, and this supplement before any lifecycle fixture/source
release. No lifecycle tests, compiler, formatter, scanner, Git operation, or
runtime claim was made for this record.
