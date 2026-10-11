# Run observation manager Unit 1 repair 3 — cleanup-only source record and hash receipt

Date: 2026-10-07. Source-only repair of the TESTONLY fixture cleanup helper. No tests, compiler, formatter, typecheck, scanner, build, Git, or runtime command was run or authorized by this task.

## Original Unit 1 assignment

The immutable original assignment is `run-observation-manager-unit1-original-assignment-oct06.md` (SHA-256 `de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`). Its relevant full text:

> Builder TESTFIRST bounded manager unit1 prepare/admission/ownedrollback lifecycle within existingprivate server component. Read governingplan/protocol/originalmanagerassignment/fullreviewed6+addendum+correction2+3+caperratum and NEW root-private-design-decision-resume-oct06 (supersedes ONLY draftextraoperatorapproval forroutineprivatecaps underuserstandingarchitectural delegation; no publicrelease). Design TESTONLY +minimal compile-safe private stub seams in NEW run_observation_manager.go/_test.go (no committing): unexportednames, existing RunTracePort/runListReader/requestIdentity/auth+checkRunObservationPresentContext contracts groundedactualsource, constructor private, prepare returns exact initialDTO wrapper +privatelease. Focusonly cohortcap16 preparing/active/draining before list/read; atmost16pollerentries count initializing/stopping/draining, exactkeys/runplanparent guard, sharedinitializer onephysical owner contextnotinitiatingwatcher; failedPrepare nillease ownsrollback includingpartialattach cancellation onlywatcherless/readactualreturn+retirement+workerJOIN beforefreeslot; Stop/detachidempotent/timeoutremainscounted/no lockheld listreadcancelwaitjoin; survivingwatcherwaiter notstrandedownerrollback. DoNOTimplement full algorithm/Nextdelta/SSE/authpolicy/JSONbudget yet. Existingapprovedselector/hydration/guard no modifications. Prefer test throughfutureprivate fullprepare/lease ratherthaninvent genericregistry; defineprivateinjected authorizer/guard seam groundedexisting authfuncs withactualreturntypes, no bearerretained. Stubminimal explicit testphase/unwired no completionclaim. Everytest distinguish wrongstate and control workerreturns viachannels/manual no race sleeps toorder; caps correct R countstubstatusunavailable sufficientexpectedRED. Write originalbounded assignment/evidence beforefiles, nearbytests+scopedAGENTSread. NO compile/typecheck/tests/git; rootheavyIDLE but permissionseparatecriticbeforeRED. Crossfile/integrationtask heightenedscrutiny independentcritic +root. Nativeapplypatch only. Reportlimitations/originalinstructions/fullhashes.

## Repair 3 instruction and disposition

> Fresh DIFFERENT builder Go managerunit1 tiny repair3 (currentfa160/6bc builtbootstrap), SOURCEONLY/no gates/Git/compiler. Read originalfullunit1assignment/fullapprovedprivatebundle+rootownershipscaffold and package repair2 record/hashreceipt/new independentREJECT run_observation_manager_unit1_repair2_independent_source_review_oct07.md ada3eb. All4priorissuesfixed; remaining cleanup helper currentlyaccepts ANY KindUnavailable stopAndDrain without proving mapempty/allownedworkersJOIN. FixONLYcleanup testhelper/assertions: allowtypedunavailable solelydeliberatelyunwired no-work manager with emptyrealpollerstate/no ownedworkers; otherwise captureactualworkerDone underactualmanager.mu beforedrain, cancel/releasealreadycontrolledworkerworkoutside lock, boundedJOIN/stopreturn+assertentrymapempty and ALLcapturedworkersclosed (includingifstopreturnedunavailable). Don't silentlymaskfailed drain/liveentries. Preservecurrentfa/6bc before ANY edit in NEW nativepatchpreeditbackups, exactknownhashes (previousbuildermissed immediatepreimage; doNOTrepeat). NEW originalassignment+repairinstruction/findingrecord + posthashreceipt immutable writeonce; no editswrittenrecords. DoNOTtouch UI source828 or Go productionalgorithm/guard/hydration/selectors/contracts/deps. New sourceonly testhelper shouldstillallowearlysemanticstubRED with no livework. Independentdifferentcritic/root beforeactualRED. If .agentsreadonly packagebackup rootwillarchive, allwritesnativepatchONLY.

The cleanup-only boundary was followed. No production algorithm or private manager source was changed. No manager lifecycle state was added. The retained `refs >= want` shared-initializer assertion was outside this repair’s scope and had been marked nonblocking by the cited review; it was not changed.

## Review findings addressed

The immutable repair 2 review `run_observation_manager_unit1_repair2_independent_source_review_oct07.md` has SHA-256 `ada3eb9955f8a9912a365cb11b5a61920e15b31f9ffa59d2d3f84f3a682cd417`. It accepted the repair of four earlier fixture defects: distinct retry-start synchronization; observing actual mutex-protected ref membership before initiator cancellation; waiting for all sixteen exact controlled later reads and checking A/B cancellation/ownership; and cleanup that cancels callers, releases controlled reads, and bounds manager drain. Its remaining rejection was that cleanup treated every `KindUnavailable` from `stopAndDrain` as clean without checking actual entries or `workerDone` channels.

The revised `cleanupRunObservationManager` now:

1. Takes a snapshot of the manager's actual `pollers` entries, `cancel` functions, and `workerDone` channels under `manager.mu` before drain.
2. Calls captured worker cancels and releases controlled reads only after unlocking.
3. Starts `stopAndDrain` with a two-second context and waits on its result.
4. Independently waits, under the same bounded context, for every captured non-nil `workerDone` channel to close, even if `stopAndDrain` returned typed unavailable.
5. Re-locks the actual map and asserts it is empty after stop returns.
6. Accepts `KindUnavailable` only when the pre-drain ownership snapshot was empty and the post-drain map is empty. Any unavailable result with captured work, any remaining entry, missing done channel, non-unavailable error, or timeout is reported as cleanup failure.

This keeps expected early semantic RED cleanup valid for the deliberately unwired stub: the stub owns no pollers and `stopAndDrain` returns typed unavailable. A manager that reports unavailable while retaining actual poller/worker ownership is surfaced as an error rather than silently treated as joined. Captured worker cancellation and controlled read release run outside the manager mutex.

## Root-approved private boundary reviewed

Read `run-observation-manager-root-private-design-decision-resume-oct06.md` and its independent review. Root approves only the private, unexported, unwired/test-first scope; existing port/DTO/auth/present-context contracts stay fixed. The approved lifecycle representation consists of actual manager-owned keyed entries, real lease refs, readiness/cancellation state, and `workerDone` for real retirement/join. The cleanup helper inspects that actual private ownership state without introducing hooks or duplicate test state. It preserves the existing no-lock-held cancel/wait/join principle. This repair changes no production architecture, selector, hydration, guard, public contract, auth/security policy, dependency, or caller wiring.

## Pre-edit preservation and post-edit identities

Before editing, the current candidate bytes were copied through native `apply_patch` into new package-local backups. The backup and pre-edit source/test hashes match exactly:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` pre-edit and unchanged post-edit | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| `run_observation_manager_repair3_preedit.raw` | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| `run_observation_manager_test.go` pre-edit | `6bc6666ee176a56e233636bca0d72fcda439e430a03c0642e4fd2ad695af826a` |
| `run_observation_manager_test_repair3_preedit.raw` | `6bc6666ee176a56e233636bca0d72fcda439e430a03c0642e4fd2ad695af826a` |
| `run_observation_manager_test.go` post-edit | `b1b585320ef65ed1cb27a2b618bcca4a8b2e368c9646e96a6229e6855e91c72a` |
| repair 2 rejection reviewed | `ada3eb9955f8a9912a365cb11b5a61920e15b31f9ffa59d2d3f84f3a682cd417` |
| repair 2 source/test record | `2cb8d19a1ee3597744d1e4effbc3b44f5e504d7343135620e50022a8ba1f285b` |
| repair 2 hash receipt | `a0250cdafaf5fde5bcb042448dfce070096897500e9de95c6ba4e227b486689d` |

Direct unified diff against the exact test preimage shows only the cleanup helper changed. The private manager source remains byte-identical. Package-local backups are new and preserve the immediate repair 2 preimage; prior historical records and backups were not overwritten.

## Verification limits / next review

Read the full current helper, all call sites of `cleanupRunObservationManager`, the manager ownership fields, original assignment, root private decision and its review, repair 2 record/receipt, and the full repair 2 rejection. Graft was used first; exact source was verified directly. No claim is made that this test helper compiles or passes. The different independent critic and root must review this frozen candidate before any separately authorized focused RED. No runtime/compiler/test permission is conveyed by this record.
