# Private poller image encoder — test-first authoring assignment

Date: 2026-10-07  
Status: immutable bounded assignment proposal for an upcoming builder. No
source or fixture edits, compiler, tests, scanner, formatter, typecheck, runtime,
or Git operation was performed. No source/fixture release is granted here.

## Complete current root instructions

The following task instruction is preserved in full before any future source
work:

> SOURCEONLY preparation for upcoming pureencoder TESTFIRST builder task; no
> source/test edits until separateexplicitrelease. Fresh otherbuilder fixes
> your2protocoldefects now; critic prelimrootdecisionssound finalsupplementpending.
> Prepare NEW immutable bounded authoring assignment with concrete minimal
> private API and test matrix, only NEW run_observation_poller_image.go throwing
> seam + NEW run_observation_manager_retained_image_test.go. Use exact
> branch-specific explicitorderedstructs schema/phase/initresult/key OR
> rawcurrentJSON, fixedcodesrangephase0..3/init0..5, helperimageunchanged/pure.
> Encoderreturnsownedbytes<=1MiB orfixedgenericerror/nilbytes, nolifecycleMap/ref/
> ports/cancels/publish/JOIN. Design synthetictests6matrix accepted:
> nil/no fake; nonnil rawobject/keyonce; combinedexact1MiB and +1eveninnerfits;
> phase/markerconstantwidth priorunchanged; oversizednilkeygenericrefusalONLY
> encoder notnoRead; no source mutation/bytealias. Include invalidcode and
> currentkeymismatch expectationsifminimalprivatecontract demands, identify
> beforewritesratherthansilentexpand. Existingmanagerfa/af/helper215/base34/
> policy4 frozen. Completeoriginalrootinstructionsrecord BEFORE edits/
> sourcefuture, nearbyhelper/tests/Graft anchors required. Explicittestprefix
> new '^TestRunObservationPollerImage' independent from intentionalmanagerstub
> tests. No compiler/scanner/Git. You willauthorstub+fixtures onlyafter
> rootfinaldesignapproval/release; differentcritic source+expectedRED before
> algorithmbuilder. AllLuna no nested.

This assignment is a proposal derived from that instruction. It grants no
builder source write; root must provide separate explicit design/fixture
release after review. No worker or sub-agent is requested.

## Governing records, source state, and authority

Root-selected wrapper decisions are in
manager-combined-image-root-decisions-oct07.md (SHA-256
48ed5d0b01e25a080b8b2560462bfd6bdf4b196d9769ff917b6a9cb25b32fe96) and
combined design proposal run_observation_manager_combined_retained_image_design_proposal_oct07.md
(SHA-256 e436298c9822a7e405e723ebba505ead8ad49006e246ab983a02b2ad88f9760c).
The full Unit 1 scope and frozen original assignment are preserved in the
package-local lifecycle preregistration (SHA-256
f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5) and
BASE run-observation-manager-unit1-original-assignment-oct06.md (SHA-256
de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916).
The previous combined-image assignment and its review repair are immutable:
manager-combined-image-testfirst-assignment-correction-oct07.md (SHA-256
5f5418f16193d41476f69554e366ff83d7490c68a917462f3a76252f050466ee) and
manager-combined-image-testfirst-assignment-repair-oct07.md (SHA-256
477ee9aa26d3b68d6cc2a01ef72d0bed3a4cf358345950129c3068f40b937fdc).
The combined-image independent design review and root initial-failure
clarification govern lifecycle follow-on scope; they do not expand this pure
encoder unit.

Current frozen source identities:

| File | SHA-256 | Scope |
|---|---|---|
| internal/server/run_observation_manager.go | fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d | frozen unwired manager |
| internal/server/run_observation_manager_test.go | af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4 | frozen manager fixture; complete hash ends in 30c4 |
| internal/server/run_observation_retained_version.go | 2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd | formatted pure helper; unchanged |
| internal/server/run_observation_retained_version_test.go | 34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7 | frozen base helper fixture |
| internal/server/run_observation_retained_policy_test.go | 4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7 | frozen policy fixture |

The helper’s pre-format identity is 38ca7fdaf45b016fb8a55fdb72a32b15cad100fb5b31585410af943dddf7447a; the immutable formatting result says the only difference to 215758 is formatting whitespace, with token sequence unchanged. Its independent algorithm review is
run_observation_retained_version_algorithm_implementation_independent_review_oct07.md
(SHA-256 508da6b696573f8c5f36cc0de1e29c240445361aca56047f6f3fac0e9a86cfb6).
Root reports the accepted helper runtime captures; this assignment performs no
re-run and claims no new test result. Preserve all five current files above
byte-identically.

## Repo guidance and retrieval anchors

Root AGENTS.md, .agents/AGENTS.md, current status, and graft skill were
consulted earlier in this task chain; no applicable apps/godspeed-casework-go
AGENTS.md was found. Graft retrieval for this assignment located
marshalRunObservationRetainedImage at
run_observation_retained_version.go:263-311 and the helper’s state/type
surface. Graft skeleton anchors:

- run_observation_retained_version.go:38-50 contains exact key, accepted
  generation/time, status/counts, retained frames, full ID ledger/high ordinal
  and availability in runObservationRetainedState.
- run_observation_retained_version.go:222-256 is the explicit inner image
  layout; lines 258-311 perform fixed-width control formatting, deterministic
  ID order and JSON serialization.
- run_observation_manager.go:73-85 has the current runtime poller/lease
  scaffold; lines 125-155 leave prepare, stopAndDrain and detachAndDrain
  unwired. It is not an image encoder and remains frozen.
- run_observation_retained_version_test.go:367-459 already tests helper-side
  fixed-width controls and exact inner 1 MiB boundary. It pads the serialized
  caseID field to hit the exact byte limit, then verifies +1 refusal. Preserve
  this test and use the same source-grounded technique to size the outer image.
- run_observation_manager_test.go:555 onward contains intentional manager stub
  tests, including initializing capacity; do not run the whole package as the
  pure image RED because those tests are out of this unit.

The package module is apps/godspeed-casework-go (Go 1.27). No scoped Go-app
AGENTS.md was found. Root's just/go ownership and fresh-resource rules still
govern any later test execution.

## Only proposed files and exact API

After root’s separate design approval, the builder may create only:

1. New internal/server/run_observation_poller_image.go.
2. New internal/server/run_observation_manager_retained_image_test.go.

No edits to manager, helper, any existing tests, ports/adapters, contract,
schema, public surface, docs outside the new immutable assignment, or
dependencies.

The new file is a pure private serializer. It has no manager map, lease refs,
ports, worker/context/cancel, channels, callbacks, notification, lifecycle
publication, admission, or JOIN behavior. Do not add a manager field.

Use this minimal function contract:

    marshalRunObservationPollerImage(
        key runObservationPollerKey,
        phase runObservationPollerPhase,
        initResult runObservationInitializerResult,
        current *runObservationRetainedState,
    ) ([]byte, error)

The returned slice is newly owned by the caller. On every error it returns
nil bytes. On success it returns the complete canonical wrapper bytes with
length at most runObservationRetainedImageMaxBytes (1<<20). Use a fixed
generic, non-sensitive error; never interpolate key, state, candidate, or
marshal error text. Keep the existing helper and its own serializer/bound
unchanged. Call marshalRunObservationRetainedImage for a nonnil current state,
then embed its bytes as json.RawMessage and measure the final wrapper bytes.

Private encoding types are branch-specific explicit ordered structs, not maps
and not a single struct relying on Go’s nil-to-null behavior:

- Nil-current branch field order and JSON names: schema_version, phase,
  init_result, key. Schema value is the fixed private format constant
  run-observation-poller-v1. Key is an object with case_id, run_id,
  plan_item_id in that order. There is no current member and no synthetic
  helper value.
- Current branch field order/names: schema_version, phase, init_result,
  current. Current is the raw helper JSON object (not a quoted string); there
  is no wrapper key because the inner helper image already includes it.

Both branch types always serialize schema_version, phase, and init_result.
Phase is a private uint8 enum with only 0 initializing, 1 running, 2 stopping,
3 draining. Init result is a private uint8 enum with only 0 pending, 1 accepted,
2 read-unavailable, 3 retention-unavailable, 4 invalid/unavailable, 5
terminal-retention-failure. Validate both ranges before marshaling. For a
non-nil current value, require current.Key == key; reject mismatch with the
same fixed generic error and nil bytes. Do not validate/normalize key contents
beyond that exact identity seam; selection owns source identity validation.
Map any inner marshal error to the fixed generic error. No error may reveal key
or retained payload.

No auxiliary marker, phase boolean, generation, or failed-read field is added.
For an accepted current value the wrapper’s InitResult is accepted; the helper
Availability remains the one current read/retention/terminal marker. The helper
accepted-version Generation is not lifecycle generation. This serializer does
not choose a marker or publish one; manager orchestration later uses the
unchanged helper marker-copy function and must measure before pointer install.

## New fixture matrix and claims

Every new test name begins exactly with TestRunObservationPollerImage. The
future focused command is:

    cd apps/godspeed-casework-go
    go test ./internal/server -run '^TestRunObservationPollerImage' -count=1

This command is specified for the future sole compiler owner only; it was not
run here. The prefix prevents intentional manager-stub tests from entering the
pure encoder RED/GREEN.

Required test cases:

1. **Nil current, no fake data.** Use a concrete key and controls; assert exact
   branch field order and names, key appears once, no current field, and no
   execution/settlement/timestamp/frame/ledger/helper metadata exists. Assert
   fixed schema and one-digit controls.
2. **Non-nil current, raw object and key once.** Build a well-formed helper
   state. Assert wrapper key absent, current decodes as an object rather than
   quoted JSON, exact key exists once inside the inner value, and a second
   encode yields identical owned bytes.
3. **Combined exact and +1.** Construct a helper state whose inner image fits
   its own bound but whose complete wrapper is exactly 1<<20; encoder succeeds
   with exactly that many bytes. Increase a valid single-occurrence inner key
   field by one byte (the existing helper exact-limit test uses caseID padding)
   so the inner image remains below 1<<20 and the outer image is exactly
   1<<20+1; encoder returns nil bytes and the fixed generic error. No production
   padding, truncation, pruning, or fake trace row.
4. **Fixed-width marker composition and prior immutability.** Begin with a
   prior state whose complete wrapper is exactly 1<<20. Clone only via
   retainedMarkerCopy; change only helper Availability and, where needed,
   phase code. Encoded length remains exactly 1<<20 for current/retention/
   terminal marker values; the prior state/map/frame pointers remain unchanged.
   This exercises pure encoder + already-existing marker copy only; it does
   not assert that manager publication occurred or that a candidate was
   rejected atomically in a live poller.
5. **Oversized nil-current key.** Use a long but valid key that makes the
   nil-current wrapper exceed the cap. Assert nil bytes and fixed generic
   refusal without key text. Label this strictly as serializer refusal. It
   does not prove no map entry, reference, or trace read; that requires the
   separately authorized future manager lifecycle fixture.
6. **No source mutation / returned-byte alias.** For both branches, snapshot
   input state deeply, encode, mutate bytes returned by one call, and show the
   state and a separately encoded result are unchanged.
7. **Minimal shape validation.** An out-of-range phase/init code and a
   current key mismatch each return nil bytes plus the fixed generic error.
   Keep these checks narrow; do not add unrelated payload validation.

Use assertions over exact raw output/order and byte lengths, not Go map
iteration order. The exact combined candidate case must demonstrate
inner-helper bytes < 1<<20 while wrapper bytes = 1<<20+1. Marker tests may
assert helper key, full ledger, window, and metadata are unchanged. The
candidate-overbudget marker-selection/publication algorithm is not this
encoder’s API and is not proved by these tests.

## Test-first and review sequence

The current authorization is preparation only. After a different independent
design critic accepts the final combined-image supplement, root must explicitly
release the two new files. The builder first writes the new prefixed fixture
and the minimal compile-safe throwing seam (fixed generic error/nil bytes), no
other source; all tests must compile and fail at encoder assertions rather than
unrelated manager tests. The source/test identities and error output are
captured by root’s protocol. An independent critic reviews both exact files
and the expected focused RED before root accepts the RED.

Only then may a different algorithm builder implement the encoder in the same
new file. A fresh source critic reviews it against this assignment and accepted
RED; root explicitly releases the focused GREEN command to its sole compiler
owner. No broad package test is implied because current manager tests target
the unwired scaffold. No test, compiler, formatter, scanner, or runtime action
is authorized now.

## Explicit exclusions and next boundary

A nil-current overbudget result proves only that the serializer refuses bytes.
It does not prove manager admission, map insertion, ref acquisition, or
ReadRunTrace suppression. The future lifecycle failure fixture is separate:
it must test actual Prepare behavior, count oversized selected key as A (not C),
and observe no entry/ref/read. First shared read failure, successful-A DTO
with a nonnil empty cohort lease, per-Prepare ReadsAttempted/Exhausted/capture
time, each waiter releasing only its own ref, ready/stop/detach signaling,
actual port return/retirement and JOIN-before-capacity-release, stop-vs-ready,
and all selected starts before any wait all belong to that later lifecycle
fixture and implementation assignment.

No Next/delta/watermark/SSE, source continuity, public V4/server integration,
schema change, auth-policy change, kernel writer/frontier, dependency, heap/RSS
claim, fake marker field, or lifecycle implementation is part of this encoder
unit. Existing manager fa1601/af2df, helper 215758/base34, and policy4c70 remain
frozen.

No execution or passing-test claim is made.

🌱 graft saved ~45,735 tokens (about $0.03 plus two calls each under $0.01) this turn.
