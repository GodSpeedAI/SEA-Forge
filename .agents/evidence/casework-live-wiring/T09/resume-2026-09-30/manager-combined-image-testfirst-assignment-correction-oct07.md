# Manager combined-image test-first assignment and provenance correction — 2026-10-07

Status: immutable source-only preparation. This corrects provenance in the
prior recon without rewriting it and proposes a narrow test-first assignment.
No source/test edits, compiler, tests, scanner, formatter, typecheck, runtime,
or Git mutation was performed for this record. It is not implementation or
fixture release authority.

## Correction to prior recon provenance

The prior record
`manager-lifecycle-resume-scope-recon-oct07.md` (SHA-256
`4a8859d9341115bd3d5fa646a73afd94afe84e3311183f5c86b4cf7c3976b02e`)
incorrectly stated that the requested transient `/tmp` preregistration
original was not found. Correct record: the durable lifecycle preregistration
is present in the Go package directory at
`apps/godspeed-casework-go/internal/server/run_observation_manager_unit1_lifecycle_implementation_preregistration_oct07.md`,
SHA-256 `f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5`.
Do not repeat the missing-original allegation. It cites the separate frozen
Oct 6 Unit 1 original assignment in BASE
(`run-observation-manager-unit1-original-assignment-oct06.md`, SHA-256
`de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`).
The old note is immutable; this new note is its provenance correction.

The old recon also named the helper at its pre-format source identity
`38ca7fdaf45b016fb8a55fdb72a32b15cad100fb5b31585410af943dddf7447a`.
Current helper is `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`;
`run_observation_retained_version_format_result_oct07.md` records that the
identity change was formatting-only, with lexical Go token sequence unchanged.
The independent algorithm review
`run_observation_retained_version_algorithm_implementation_independent_review_oct07.md`
has SHA-256 `508da6b696573f8c5f36cc0de1e29c240445361aca56047f6f3fac0e9a86cfb6`;
it reviewed the pre-format helper source and frozen base test. Current frozen
base helper test is still SHA-256
`34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`.
Current manager fixture hash is the full
`af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.
The prior recon already had the complete manager fixture hash; abbreviated
references ending at `...30c` are incomplete. No existing evidence is edited.

Root reports the helper's focused top-level/nested and race runs and exact
source/test comparisons passed for the accepted helper phase. This recon did
not reproduce those runs; see root-owned captures/results. The helper remains
unchanged for this proposed unit.

## Frozen inputs read and exact identities

- Original private Unit 1 assignment SHA-256
  `de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`.
- Durable lifecycle preregistration SHA-256
  `f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5`.
- Revision 6 proposal SHA-256
  `60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`;
  addendum `9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`.
- Retention/initializer correction 2 SHA-256
  `3494e3de99b2f5d3fe725a85971dccd0d70cef226da257ce237cce7fba2d72e1`;
  correction 3 SHA-256 `b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`.
- Combined-image design proposal SHA-256
  `e436298c9822a7e405e723ebba505ead8ad49006e246ab983a02b2ad88f9760c`;
  root combined-image decisions SHA-256
  `48ed5d0b01e25a080b8b2560462bfd6bdf4b196d9769ff917b6a9cb25b32fe96`.
- Root private-design decision SHA-256
  `24324e46e4155e7e501ff0b303a5c44ceb22babe6cb1f056a6f1309e3ba60ed5`.
- Current manager source/test: `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` /
  `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.
- Current formatted helper/base test: `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` /
  `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`.

The complete original Unit 1 assignment remains the immutable BASE artifact
identified above. It allows only private test-first prepare/admission/rollback
and lifecycle within the private component; it excludes full Next/delta, SSE,
auth-policy, JSON-budget, public integration, and runtime approval. The Oct 7
preregistration records root's later expansion to initial DTO, recurring reads
needed for drain, and stop/detach, still excluding Next/public wiring/exports.
Revision 6 contains broader Next/delta design and is context, not permission to
pull that work into this unit.

## Exact current root instruction for this preparation (verbatim)

> SOURCEONLY fresh bounded implementation preparation, no source/test
> edits/compiler/Git/scanner. Root found surviving package-local lifecycle
> originalf48 and algorithmreview508 and copiedall4BASEcmp0; earliermissingclaim
> corrected. Write NEWimmutable correction recon provenance: exactpackage
> originalf48 exists, no allegationtmpmissing original. Helperreview508
> exactlyhash508 existsapps/...algorithm_implementation_independent_review_oct07.md.
> Existingreconhash managerfixture truncated af2...30c vsactual af2...30c4
> correctinNEWnote. Donotoverwriteevidence. Yourcombinedseam/proposals stand but
> rootchoosesmanagercombinedencoder keepingpurehelper unchanged. Need exactoutercontrol
> design fromlifecyclebuilderprereg plus independentdesignreview before
> implementation. Read full combined image design proposal and rootdecisions BASE
> plus full lifecycleoriginal/prereg/rev6/addenda/corrections and
> helpercurrent215/source/frozenaf. Prepare a bounded TESTFIRST next-unit
> ORIGINAL assignment proposal before code release: preferably NEW
> run_observation_poller_image.go private throwing seam + NEW
> run_observation_manager_retained_image_test.go for nilkey/nofakevalue, exact
> combined cap/+1 atomicrefusal/markerconstantwidth, nilkeyoverbudgetnoadmission/read,
> immutablecopies. Existingmanager/sourcehelper/tests untouched.
> Choose precise minimal private APIs and separate what purecombinedencoder can
> prove vs actualmanageradmission/publication requiresfutureintegration.
> Don't writeplaceholderlifecycle code or mislabel test source asruntime.
> Architectureroot chooseswrapperpointertoken rules, independentcritic
> reviewsbefore fixture release. Differentbuilderfromformatproposal, all Luna.
> Record completeoriginalinstructions and proposal result inBASEimmutable. No
> code releaseyet.

Parent follow-up clarification incorporated: a critic is checking the design;
explicitly cover the first shared-initializer read failure as a future
lifecycle test: selected row is A/no run/no retry; every waiter releases only
its own ref; ready/stop signals cannot strand waiters; actual read/retirement
and worker JOIN precede capacity release; and stop-vs-ready is race-safe.
Root's newer decisions supersede older proposal ambiguity about an initial
read/retention failure. Pure encoder tests must not claim manager admission,
read suppression, or lifecycle behavior. Oversized nil-current image refusal
by itself does not prove no manager map entry/read. Existing `manageraf2df`,
base `34df`, and policy `4c70` fixtures remain frozen.

## Root-selected outer image contract (proposal basis)

Use one manager-owned deterministic canonical wrapper around the unchanged
helper image. Keep the helper's existing serializer and local bound; wrapper
fit is the final current-value admission check before manager publication.
This is exact from the combined-image design/root decision, still pending a
different independent design critic before fixture release:

```go
type runObservationPollerImage struct {
    SchemaVersion string                   // fixed constant, outer format
    Phase         runObservationPollerPhase // uint8, always one digit
    InitResult    runObservationInitializerResult // uint8, always one digit
    Key           *runObservationImageKey  // only when Current == nil
    Current       json.RawMessage           // exact raw helper JSON, only when nonnil
}
```

Serialize fields in that struct order; never use a map. `Phase` fixed codes:
initializing=0, running=1, stopping=2, draining=3. `InitResult` fixed codes:
pending=0, accepted=1, read-unavailable=2, retention-unavailable=3,
invalid/unavailable=4, terminal-retention-failure=5. Both control fields are
always emitted, never `omitempty`; every code is one decimal digit. For
nil-current, encode exact case/run/plan-item key once and omit `Current`; do not
fabricate execution, settlement, timestamp, frames, or helper state. For a
current value, omit wrapper `Key` and embed the exact raw bytes from
`marshalRunObservationRetainedImage`; helper already has the key. Do not quote
raw JSON as a string or mirror helper fields.

There is no extra scalar initializer counter/token. The manager-owned entry
pointer and its sole worker are the identity: before publication recheck exact
`m.pollers[key]` pointer, admissible phase, and captured prior-current pointer.
Removal/recreation requires actual worker JOIN first. Keep helper accepted
`Generation` distinct from lifecycle identity. Add a scalar only if an allowed
path demonstrates pointer identity + sole worker + join-before-reuse is
insufficient, and account for it explicitly. `ready`, `workerDone`, contexts,
cancel functions, timers, refs, dependency pointers and mutex remain runtime
ownership primitives, not serialized JSON. Derive current read/retention/
terminal/stop availability from helper `Availability`; do not duplicate
stopScheduled/failed/terminal booleans. In nil-current state `InitResult`
records the initial outcome. No candidate error strings are retained.

The budget is exactly `1<<20` bytes for the complete canonical current image
per poller, not heap/RSS. Initial DTO hydration has its separate existing 1 MiB
cap. Enforce 16 pollers/16 cohorts/128 attachments as counts under the manager
lock; those counts do not account for temporary allocations.

## Proposed next bounded test-first assignment (no code release here)

**Files if later released:** create only a new private
`run_observation_poller_image.go` and new
`run_observation_manager_retained_image_test.go`. Do not edit current manager
source `fa1601...`, manager fixture `af2df...30c4`, pure helper `215758...`,
base helper fixture `34df...`, or policy fixture `4c70...`.

**Minimal new source seam:** private enums/key/wrapper type above and one
`marshalRunObservationPollerImage(key, phase, initResult, current)` function.
It validates enum codes and, when current is nonnil, exact helper-state key;
marshals the helper with `marshalRunObservationRetainedImage`; builds the
explicit ordered wrapper with `json.RawMessage`; returns no bytes for invalid
shape or combined size greater than `runObservationRetainedImageMaxBytes`;
uses a fixed generic over-budget error with no key/payload. It has no manager,
map, port, worker, reference, callback, I/O, channel, or mutation behavior.
A private pure budget decision may be added only if necessary for a testable
candidate/marker transform; it may choose encoded values, but cannot publish
manager state. Do not add generation/token/stop booleans or generic framework.

**Fixture-first cases this seam can prove:**

1. Nil-current image is deterministic; exact key appears once; current helper
   field absent; no synthetic source metadata/frame/ledger; outer schema and
   fixed single-digit controls are present in both initial and terminal codes.
2. A well-formed current helper image whose *combined* wrapper is exactly
   `1<<20` bytes encodes successfully. Use a valid opaque variable-length ID or
   key field to size the fixture; do not add production padding/pruning.
3. A helper candidate that fits its own helper cap but makes the combined
   wrapper `1<<20+1` is refused with no bytes. The candidate and prior remain
   immutable. Applying existing `retainedMarkerCopy` to the prior and
   serializing its fixed-width availability plus the appropriate same-width
   phase code preserves exact prior image length/ledger/window; the candidate
   ID/ordinal never enters that marker. Verify both recoverable and terminal
   marker code paths as source-level pure values. This tests encoder/budget
   composition and immutable marker construction, not manager pointer swap.
4. Nil-current key image one byte over the combined cap returns only the fixed
   generic size error, with no encoded image. This proves serializer refusal
   only; it does **not** prove `m.pollers` remains unchanged or that no trace
   read starts.
5. Mutating returned encoded bytes cannot alter the current helper state;
   marker copies keep frame optional pointers and ledger independent, reusing
   existing helper cloning rather than adding another copy abstraction.

The exact-boundary fixture must account for the helper's complete bytes and
both outer control fields. Test code should assert byte lengths directly. Keep
phase and initializer codes single-digit, availability fixed-width per the
helper schema. If wrapper struct tags/order or nil-vs-empty behavior differ
from the proposed exact shape, stop for design review instead of adapting the
test silently.

**Required separate manager/lifecycle fixture before lifecycle implementation
release:** no current image-only test can prove actual no-entry/no-read behavior.
Add that assertion only in a separately released new lifecycle fixture that
calls future `prepare`: an oversized nil-current key is counted A (not C), is
absent from `m.pollers`, acquires no lease ref, and starts zero
`ReadRunTrace`. Also cover the accepted initial failure policy chosen by root:
first read failure/initial retention or invalid candidate is A, omitted from
Runs, no retry; each shared waiter gets the one fixed result and releases only
its own ref; closing ready/stop signals cannot strand any waiter; actual
RunTracePort return/retirement and `workerDone` JOIN happen before poller or
cohort capacity release; stop racing with ready is safe and idempotent. Root
has decided initial failures produce unavailable initial DTO with successful
list counts present, no Run row and no retry; they do not fail the whole
Prepare. This future fixture must prove the lifecycle behavior rather than
asserting it from the pure encoder.

**Review and release sequence:** first obtain a different independent design
critic review of the combined-image proposal against revision 6/addenda,
retention corrections, frozen Unit 1 assignment, lifecycle preregistration and
root combined-image decisions. Then root explicitly releases only the new
image fixture/stub for a fresh expected semantic RED. Root's sole compiler
owner records actual joined results. After fixture acceptance, implement the
small pure wrapper in its new file, receive a separate independent source
review, then root releases focused GREEN. Manager lifecycle integration is a
separate future assignment and must preserve the exact existing fixtures.
This proposal authorizes no compiler, test, code, fixture, runtime or public
integration action now.

## Unresolved review checks (not permission to invent fields)

- Critic must confirm `json.RawMessage` embedding, `omitempty` branch shape,
  enum validation, and exact outer field order are deterministic and compatible
  with this package's Go JSON behavior.
- Critic must review outer candidate `+1` behavior: no candidate-derived data
  survives; prior safe state plus helper marker is re-encoded within the same
  combined cap; marker changes control values only, never widths.
- Critic must check outer-only over-budget candidates when helper-level state
  itself fits, and ensure a first nil-current over-budget case creates no
  retained manager record in eventual integration.
- Critic must ensure pointer identity + one worker + join-before-reuse covers
  all initializer races, especially owner Prepare cancellation, shared waiter,
  and stop-vs-ready, before scalar generation is omitted.

No source or fixture changes and no execution evidence are claimed here.

🌱 graft saved ~45,531 tokens (~$0.04) this turn (1 call).
