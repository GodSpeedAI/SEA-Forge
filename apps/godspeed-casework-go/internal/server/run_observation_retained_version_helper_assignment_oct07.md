# Retained-version helper — immutable original assignment

Date: 2026-10-07. Record written before source/test files. Source-only test
scaffold task; no tests, compiler, scanner, typecheck, formatter, or Git.

## Full original root instructions

> Fresh bounded Go manager Unit1 lifecycle implementation ORIGINAL assignment/
> preregistration SOURCEONLY while UIcompilerbusy. NO production/source edits
> until root later verifies fresh current af2df fixture RED and explicitly
> releases. Read currentfa1601 manager +af2df fixture, complete approved
> revision6/addenda/corrections/responsecaperratum/rootcaps, existing selector/
> hydration/presentguard/traceports/pacing (Graftfirst). Root architecture:
> this milestone implements private prepare/shared-initializer/poller lifecycle,
> exactinitialDTO, recurring>=1s reads needed draining tests, idempotentdetach/
> stop, allpreparing/active/draining caps and actualworkerJOIN; no Server/HTTP/
> SSE integration/Next/publiccontracts. Any retainedframe/everseenledger
> implementation MUST obey approved 1MiB combinedcanonicalimage and 1024window,
> clone/lifetime/overflow rules rather than omitbounds. Any function notyetunit
> implemented cannot masqueradeasproduction; publicintegrationheld.
> Sharedinitialization manager-ownedctx, canceledinitiator doesnotcancelotherref,
> registrationrefownership underactualmu, callbacks/I/O/cancel/JOIN offlock,
> canceledworkholdsentry/cohortcapacityuntilactualJOIN, timeoutdraining nofree,
> stopclosesadmission. Initialcounts/readbudgetexactrevision6; canceled/auth/
> guard/stop/assemblyerrors zeroDTO/nillease+ownedrollback;
> sourceRunListUnavailable => DTOcompletedemptyLease, Nextfutureobligation. Do
> not make16entrycaptestpassby serializingselectedinitialreads: reserve/spawn8
> initializers beforewaiting. Existingphysicalportadmissiononlyretryowner no
> newlimiter. Return sourcegroundedboundedfileplan, invariants, unresolved
> contradictions, necessaryunitfixtures/RED selection; writeNEW immutable
> originalinstructionsrecord/prereg only nativepatch with explicitpendingrelease.
> Root retainssemantics/architecture andcriticallyreviewsplan. No tests/compile/
> Git/deps/scanners.

> Root architecture resolvespreregscope: implement bounded retained-version
> publisher as separate PURE private helper unit BEFORE lifecycle implementation,
> then lifecycle integrates it. NoNext needed to construct/store immutableexact
> ledger/window/highwater/acceptedtimestamp; futureNextconsumes samepublisher.
> Root DOES authorize retentionmodel needed for recurringpollers, notunbounded
> stub. NextboundedTASK now SOURCEONLY test-first helper scaffold+focusedfixtures
> ONLY, no lifecyclealgorithm yet. WriteNEW immutable ORIGINAL full root
> instructions/prereg beforeediting, preserve anyread existingfiles, onlyNEW
> internal/server/run_observation_retained_version.go and *_test.go. Designhelper
> againstrevision6+allcorrections actualporttypes; 1MiB combineddeterministic
> canonicalimage ofkey/safemetadata/window/exacteverseenEventID→firstordinal,
> 1024window, firstordinalneverreset/evict, deepoptionalpointerclones.
> Atomiccandidatebuild no priormutation; knownIDfirstordinalunchanged across
> reorder/shorter/sourcechanges; new IDs increasingstableordinal; sourceaccepted
> timestampforcurrentversion; highwatercapturedindependentpruning. Controlmarker/
> state/u64decimal20digitempty/alwayspresent accounting fixedwidth so safeerror/
> draining/terminal marker fits exactbudget. Distinguish nonterminaloverbudget
> preserveoldversion/ledger transient recovery withfullledger vs terminalover
> budget retainoldsafevalues+terminalmarker nooverbudgetpayload, barsreadbefore
> actualportJOIN; helperreturns typedoutcome data, lifecycleownsreadstop/JOIN.
> Counts/sourcecompleteness notinvented. No leases/Next/auth/HTTP integration/
> newdeps. Scaffoldcompiledeclarationsand deliberateunavailable stub; test-first
> focused matrix mustdiscriminatecorrectalgorithm and exposeinitialRED without
> setuptimeouts, include 1MiBexact/oneover irreduciblemetadata, duplicates/
> reorder/shorter/ledgergrowth/refusalatomicrecovery/terminalmarker/fixedwidth
> maxu64/pointerclones/window1024/time/highwater. If exactproposalgap mechanically
> unsettled, documentandaskROOT(notuser) beforeinventingrule. Rootretains
> semantics; DIFFERENTguardcritic fulloriginalinstructions/sourcebeforeanytest.
> Compiler remainsmanager_revision UI, no compile/test/scanner/Git. Avoid broad
> source rereadingneededanchorsonly/Graftfirst. Reportboundedtypes+fixtures/hash
> readyforcritic.

## Governing records and inspected anchors

* `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-manager-concrete-proposal-revision6-oct06.md`
* Revision 6 addendum, retention/initializer correction, correction revision 2,
  terminal/control-image correction revision 3, response-line-cap erratum and
  their independent reviews in the same evidence directory.
* `run-observation-manager-root-private-design-decision-resume-oct06.md`.
* `internal/ports/run_trace.go` types; `internal/contract/contract.go` frame and
  observation DTO types; `internal/server/run_observation_hydration_cap.go`.
* Original manager Unit 1 assignment and this helper task's full root messages
  above. The full original lifecycle assignment remains at
  `run_observation_manager_unit1_original_assignment_oct06.md`.

Root resolves the retained state model as a separate pure helper before
lifecycle. This task adds only the new retained-version helper Go file and its
new focused test file, plus this package-local immutable evidence record. It
does not edit manager, hydration, selector, contract, ports, adapters, or any
existing test/source file. The helper has no manager locks, lease/wake state,
I/O, auth, drain, or `Next` behavior. Its successful result is immutable
candidate current state plus exact ledger/high-water and a timestamp captured
for the accepted candidate. Its failure returns safe typed outcome information
without leaking candidate metadata.

## Planned private contract and test matrix

Represent one immutable retained version as the exact poller key and safe
scalar metadata, generation, source total and retained window, accepted
timestamp, high-water ordinal, deep-copied source-order frames paired with
first ordinals, exact `EventID -> first ordinal` ledger, and bounded control
state for availability/terminal refusal. Build against a copied ledger and
candidate only; do not mutate the input state. Validate run/case/plan identity,
frame count consistency, the 1,024 retained-frame bound, and ordinal overflow
before accepting a candidate. Existing IDs never get a new ordinal even if
reordered, payload-rewritten, or absent from a later shorter source window.
New exact opaque IDs are assigned increasing ordinals in source order. Never
parse, normalize, sort, or reset IDs/ordinals. High-water is recorded before
any initial DTO pruning; the helper does not prune the initial DTO.

The canonical image must deterministically encode every retained safe value
once: schema/version tag; exact key and safe metadata; current generation,
accepted timestamp, standings, source total and frame-window counts; each
source-order retained frame with optional pointer values and its ordinal; exact
ledger entries in deterministic unsigned UTF-8 byte order; high-water/next
ordinal; availability/terminal/scheduling controls. The 1 MiB check is on
canonical JSON bytes. Per correction revision 3, enums and booleans use
fixed-width single-digit codes, and every uint64 lifecycle number uses exactly
20 decimal digits; controls are always encoded, not omitted. This budget is
only the canonical serialized current retained-value image, not heap/RSS or
transient candidate memory.

Focused future algorithm assertions are: exact image limit accepted and one
byte over rejected; metadata-only irreducibility; duplicate/reorder/shorter
window preserve first ordinals and exact source order; new IDs get increasing
ordinals; nonterminal over-budget rejection leaves prior committed version and
ledger byte-for-byte equivalent and permits recovery with the full ledger;
terminal refusal reveals no candidate data, retains prior safe values plus a
fixed-width internal terminal marker, and declares no further reads allowed;
fixed-width controls preserve image length at max counters; optional frame
pointers never alias input or output; exactly 1,024 frames are accepted;
accepted timestamp and high-water remain version-coherent before DTO pruning.
The current helper is intentionally an unavailable stub, so focused tests
should be behavioral REDs when a later authorized test run occurs, not setup
timeouts or undefined-name compile errors. This task itself runs no tests.

## Unresolved exact-model question held for root

The correction requires fixed-width encoding of lifecycle enums/booleans and
20-digit counters/generations but does not enumerate every accounting control
field. Before defining the canonical wire-image struct, root must confirm the
minimal field list: exact case/run/plan key, safe metadata/current version,
frame window and ordinals, sorted exact ledger, availability, terminal
retention failure, stop/scheduling state, and all generation/next-ordinal
counters. No speculative control fields should be included. This record
captures the question; it does not silently decide it.

## Evidence limits

No source/test files existed for this helper at the time this immutable record
was written; no existing source was modified or overwritten. No compile, test,
scanner, formatter, typecheck, runtime, or Git action is authorized or claimed.
The source/test fixture identities and the code/test hashes will be recorded
in a separate append-only result receipt after the independent source critic.
