# Run observation manager unit 1 — fresh repair 2 record

Date: 2026-10-07. TESTONLY source/test scaffold repair; no compiler, test, formatter, scanner, Git command, or runtime gate is authorized or claimed.

## Original assignment

The immutable original assignment is `run-observation-manager-unit1-original-assignment-oct06.md` (SHA-256 `de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`). It authorizes only unexported, private prepare/admission/rollback test seams, with no manager integration, full algorithm, public streaming/API/schema, auth-policy changes, or runtime claim. It requires deterministic controlled worker returns and an independent critic before any separately authorized focused RED.

The previously saved pre-edit package-local copies are verified unchanged:

* `run_observation_manager_unit1_source_preedit.txt`: `ee7afab1b22c011cf79b602f415336d459fda346aad1bf0c942a4464326ff878`.
* `run_observation_manager_unit1_test_preedit.txt`: `2b2d1fbcd3bfb7e7c27254673ca46b2deb30d9edd4ac862fa464e2a9b18334b6`.

## Fresh bounded instructions

> Fresh DIFFERENT builder repair Go manager unit1 aad810/64dc by manager6_addendum, after independent cursor_bounds REJECT. SOURCEONLY/no compiler/gates/Git. Read ORIGINAL unit1 assignment/fullapprovedprivatebundle+rootauthority, package-local repairrecord and full run_observation_manager_unit1_repair_independent_source_review_oct07.md. Address ALL fourfindings: retry separate signal for actualretry before release; guarantee secondrealref attached before initiatingcancel; wait ALL16 controlledlaterread signals (8A+8B, exactunique runIDs) before pollercapdetach/admission; cleanup mustcancelmanager/contexts ANDreleaseheldreads+boundedJOIN everyexit (notreleaseonlyafterctxblocked). Originalbuilderno test-onlyprodcallbacks/counters/hooks. Root architecture authorizes minimal REAL private ownership representation scaffold needed eventualproduction: manager mutex + pollerkey/entry map + real perentrylease-reference membership/ready/cancellation/workerDone contracts grounded fullproposal. Keep alwaysunavailable prepare/unwiredstub; no fullstatemachine/Next/budget/integration. These are actual futureproductionownership fields, nottestingduplicatestate; onlyaddminimum neededtoobservesecondref underactualmutex. White-box test can boundedly inspect actualrefmembership with runtime.Gosched and resultselect (no sleeps toorder) untiltwoattached; stub must fail early semantic resultassertion, nottimeout. If existingobservable contract givescleanerbarrier preferit; deterministiclaunchalone insufficient. Preservepreedit source/test exactNEW nativepatch backup (if .agentsreadonly report packageartifact rootwillcopy); everypersistentwrite nativepatchONLY (no cp). NEW immutable record FULLoriginal+repairinstructions+allfindings+deviations/hashes, firstwritefreeze neveredit afterward. Frozenexistingselector/hydration/guard untouched; no publicschema/security/deps. Independentdifferentcritic+root beforeONEactualRED.

## Independent review and all four findings

Read `run_observation_manager_unit1_repair_independent_source_review_oct07.md` in full. It rejects focused-RED readiness for four fixture defects:

1. Retry deadlock: the second `run-second` call waits for `allowRetry`, but the test waits for a signal emitted only by the first call before releasing it. Add a distinct retry-start signal before the retry wait and select it against the Prepare result.
2. Shared initializer race: launching the second Prepare goroutine does not prove its watcher ref is attached. Observe the manager's actual mutex-protected ref membership before canceling the first caller; do not add callback/counter hooks.
3. Poller cap premise incomplete: wait for all sixteen controlled later reads, eight unique A run IDs and eight unique B run IDs, before detaching A or probing C. Verify A cancellation and B remains active while all 16 slots are occupied.
4. Fake-worker leaks: early failures can leave reads blocked on `ctx.Done()` because cleanup only releases the later-read channel. Every affected test must cancel caller contexts, initiate manager stop/drain, release held fake reads, and boundedly join cleanup on every exit.

The review approves no manager implementation or runtime verification. Keep `prepare`, `stopAndDrain`, and `detachAndDrain` always unavailable. Add only the smallest real private ownership representation authorized by root: manager mutex, keyed poller/entry map, actual lease refs under the mutex, and entry readiness/cancel/workerDone contract fields. These fields represent future manager ownership; they are not test-only duplicate state. Keep the existing selector, hydration, guard, public types, auth/security policy, and dependencies unchanged.

## Deviations and evidence boundary

No deviation is intended. The `.agents/evidence` tree is read-only for this worker; this package-local immutable record is the exact pre-edit/repair provenance artifact for root to archive. Existing pre-edit package copies and root's archived byte-matching `.raw` copies remain untouched. This task does not run `gofmt`, tests, typecheck, compiler, Git, or any runtime. No expected RED or passing behavior is claimed. Record post-edit source/test SHA-256 identities here only after the patch is complete; never revise this record afterward.
