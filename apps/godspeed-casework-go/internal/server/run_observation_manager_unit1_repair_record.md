# Run observation manager Unit 1 fixture repair

Date: 2026-10-06. TESTONLY source/fixture correction; no test, compiler,
formatter, scanner, Git command, or runtime gate was run.

## Original assignment preserved verbatim

> Builder TESTFIRST bounded manager unit1 prepare/admission/ownedrollback
> lifecycle within existingprivate server component. Read governingplan/
> protocol/originalmanagerassignment/fullreviewed6+addendum+correction2+3+
> caperratum and NEW root-private-design-decision-resume-oct06 (supersedes ONLY
> draftextraoperatorapproval forroutineprivatecaps underuserstandingarchitectural
> delegation; no publicrelease). Design TESTONLY +minimal compile-safe private
> stub seams in NEW run_observation_manager.go/_test.go (no committing):
> unexportednames, existing RunTracePort/runListReader/requestIdentity/auth+
> checkRunObservationPresentContext contracts groundedactualsource, constructor
> private, prepare returns exact initialDTO wrapper +privatelease. Focusonly
> cohortcap16 preparing/active/draining before list/read; atmost16pollerentries
> count initializing/stopping/draining, exactkeys/runplanparent guard,
> sharedinitializer onephysical owner contextnotinitiatingwatcher; failedPrepare
> nillease ownsrollback includingpartialattach cancellation onlywatcherless/
> readactualreturn+retirement+workerJOIN beforefreeslot; Stop/detachidempotent/
> timeoutremainscounted/no lockheld listreadcancelwaitjoin; survivingwatcherwaiter
> notstrandedownerrollback. DoNOTimplement full algorithm/Nextdelta/SSE/authpolicy/
> JSONbudget yet. Existingapprovedselector/hydration/guard no modifications.
> Prefer test throughfutureprivate fullprepare/lease ratherthaninvent
> genericregistry; defineprivateinjected authorizer/guard seam groundedexisting
> authfuncs withactualreturntypes, no bearerretained. Stubminimal explicit
> testphase/unwired no completionclaim. Everytest distinguish wrongstate and
> control workerreturns viachannels/manual no race sleeps; caps correct R
> countstubstatusunavailable sufficientexpectedRED. Write originalbounded
> assignment/evidence beforefiles, nearbytests+scopedAGENTSread. NO compile/
> typecheck/tests/git; rootheavyIDLE but permissionseparatecriticbeforeRED.
> Crossfile/integrationtask heightenedscrutiny independentcritic +root.
> Nativeapplypatch only. Reportlimitations/originalinstructions/fullhashes.

The assignment's original work boundary states that the unit creates only
`run_observation_manager.go` and `run_observation_manager_test.go`, leaves all
names unexported and the manager unwired, and does not authorize runtime,
public-stream, persistence, auth-policy, or settlement changes. The separate
independent source review rejected the original fixture for the `afterAttach`
test callback, missing draining-capacity coverage, and setup waits that timed
out against the deliberately unwired stub.

## Preserved pre-edit bytes

The repository's `.agents` evidence tree is read-only in this task's filesystem
permissions. Exact pre-edit copies therefore live beside the package as `.txt`
artifacts (ignored by Go source discovery):

* `run_observation_manager_unit1_source_preedit.txt` — SHA-256
  `ee7afab1b22c011cf79b602f415336d459fda346aad1bf0c942a4464326ff878`.
* `run_observation_manager_unit1_test_preedit.txt` — SHA-256
  `2b2d1fbcd3bfb7e7c27254673ca46b2deb30d9edd4ac862fa464e2a9b18334b6`.

The same byte-identical copies were first made in `/tmp` before editing and
their hashes matched the originals. The copy operation was used because the
read-only `.agents` boundary prevented creating the requested backup there;
the source/test edits themselves used native patch application.

## Changes in this repair

* Removed the production `afterAttach` callback and its test assignment.
* Added a bounded wait helper that selects between the expected fake-read
  signal and the `Prepare` result. If the unwired stub returns first, the test
  fails on the semantic expectation of a successful attached lease, rather
  than reporting a channel timeout. Applied it to the partial-Prepare,
  shared-initializer, poller-cap, and stop/drain lifecycle setup points.
* Added cohort-draining and poller-draining fixtures using public private-unit
  behavior only: empty leases plus one exclusive held poller for cohort
  isolation, and two eight-run cohorts for poller isolation. Each waits for a
  canceled-context `Detach` result and fake-port cancellation, checks refusal
  while the held read is not retired, releases it, joins, and checks admission
  recovery. No ownership-state test field or callback was added.
* Added `sync.Once` cleanup for held channels and cancellation contexts so
  early semantic RED exits release fake work.

Root clarified that lifecycle fixtures may select both a fake-port start and
the `Prepare` result; if the stub result arrives first, assert the expected
successful attachment contract and classify that as the expected semantic
RED. The source stub does return before the controlled fake ports. Thus none
of the later drain-capacity assertions has been demonstrated against runtime
behavior here, and they must not be reported as observed RED evidence.

## Hashes and scope

Pre-edit hashes are listed above. Post-edit hashes at record creation:

* `run_observation_manager.go` — SHA-256
  `aad810b928590acc712eb5d5eeac15a5358a85e75d2a28d7acce0bd3c9fdce0f`.
* `run_observation_manager_test.go` — SHA-256
  `64dc5a074b926aed6da85f6070ad568dd61fec9b7c88d1f4e3d7265437af3f6a`.

No production algorithm, API, public contract, test-only state field, or
manager integration was added. No test/compile result is claimed. The
remaining review boundary is independent review of these frozen fixture
changes, then root/compiler-owner disposition of any authorized focused RED.
