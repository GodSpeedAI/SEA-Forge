# Creator/list drain proposal correction assignment — 2026-10-07

## Full root instruction

> Fresh DOCONLY builder correction after independentREJECT27b7295c10ab22a4d5abb3fab5fd051cf9b225057ce3af1f2817367d0135be5c of creator/drainproposal3a5e5d70a7adfd177e951dd0118f6f7f32ed2f906306695687fd8f7b7a293119. Read fulloriginal run-observation-creator-drain-boundary-proposal-assignment-oct07.md/fullproposal +actualindependentreview (locate exactcreator-drainreview filename). Preservealloldimmutablefiles. Write NEW immutable fullthisassignment + shortcorrectionsupplement nativeapply_patchONLY. EXACT amendments: (1) creator MUST on EVERY Preparepath afterlastrequest-scopedlist/initialwaituse cancel ONLY its LOCAL derivedpreparectx (including normal success/empty/realinitialA/refused-list), thenJOINbridge, then finishexactleasecreatorDone/globalWG, BEFOREownrollbackdrainwait; leaseDone canremainopen forsuccessfulactivelease; localcancel MUSTNOTcancelsharedworkerctx ordropeligiblelease. Stop/detachcancellation via leaseDone stillsignalsbeforewait. (2) newdetach-heldlisttest explicitlytypedcancel/stop/unavailableerror+zeroDTO+nilleasenotgenericfailure; untouchedBsuccesswhileBheldcontextlive. (3) reservePollerBatch rechecksmanagerstopping/lease.draining/exactcohortmembershipunderμ AFTERlist beforeanyattach/reserve, latecanceledPrepare returns typedzero/nil withownedrollback/noactualtracecall; allownedreservedbatchstillunconditionallylaunchbeforecreatorfinish evenStopwinslater. (4) Stop timeout/testdistinguisherrorreturnfromsuccessfulactualdrain; timeoutleavescohort/workerreverseownership counted, creatorDone/drainDone/stopDonecompletionnotfabricated; lateractuallistreturn/bridgeJOIN/creatorfinish+workerJOINrequiredforrelease. Add TDDsuccessbridge-exit observable at successfulempty/unavailable-listPrepare creatorDone closed/bridgeownedgone viaactualcompletion no newhook/field; can extendnewheldlistBsuccessfulreturn andreadcreatorDoneclosed toprovefinish, notalter9existingassertions. Preserveminimaltwoaddedchannels/exactleasefinish/fourqualifiedfinishOnce closures +2newcases, originalaf/6primitivesfrozen, no newenum/counter/token/callback/privatefield or scopepublic. Rootalreadyapprovedreverse-ref7a43. This is ONLYdoccorrection, no Go/testwrites/compiler/formatter/scanner/explicitGraftbuild/Git. Read-onlyGraftqueryallowed. Report exactsupplementhash/fullmaterialdeviations and limits; differentcriticreviewbeforeTDDsourcerelease. Keepboundedquickly, no redundantbroadhistoryreread.

## Rejected proposal basis

The original complete assignment is
`run-observation-creator-drain-boundary-proposal-assignment-oct07.md`,
SHA-256
`cacbd074f25446d155c57c1c1960a7839faad4158ee22168de501eed435d91f7`.
The complete proposal is
`run-observation-creator-drain-boundary-proposal-oct07.md`, SHA-256
`3a5e5d70a7adfd177e951dd0118f6f7f32ed2f906306695687fd8f7b7a293119`.
The actual independent review is
`run-observation-creator-drain-boundary-independent-review-oct07.md`.
It rejected the proposal because successful Prepare did not cancel its local
derived context, leaving the bridge goroutine and creator completion stuck.
It also required typed held-list cancellation assertions, post-list admission
recheck, and explicit Stop timeout retention/retry behavior.

## Exact correction boundary

Write only a new immutable document supplement. Preserve all prior files.
Amend the proposal's lifecycle sequence and fixture additions to incorporate
all four clauses in the full instruction above. Keep its exact-lease
`finishPrepareOperation(lease)`, per-lease `creatorDone`/`leaseDone`, existing
`drainDone`, local cancellation bridge and four function-qualified fixture
finish call-shape changes. Preserve all nine existing assertions and the two
held-list test cases; the new bridge-exit observation may be incorporated into
the held-list B-success case, without adding fields/hooks or changing existing
assertions. Keep manager fixture `af2df...` and all six primitive files frozen.

This is documentation-only. No Go or test writes, compiler, formatter,
scanner, explicit Graft build, or Git action is authorized. A different critic
must review the corrected proposal before any TDD source release.
