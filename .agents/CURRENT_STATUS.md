**Status revision:** 381

**Stage:** CognitiveOS convergence #108 | implementation session 1, integration bases and #109

**Summary:** Created one integration branch per affected repository, aligned Cognate Workbench and Horizon on 0.10.0, and verified the standalone Workbench and Horizon baselines for ticket #109; #110, #111 and #117 are the ready frontier.

**Verified:**

- Cognate Workbench integration/cognitiveos-108 merges capability-runtime t4a 9824735 and WP-23 668f686 plus the vocabulary commit; every @godspeedai/cognate-* pin and installed copy resolves to 0.10.0; bun run typecheck passes.
- Workbench expert, team, independent-review and plugin suites passed 107 tests with 1053 assertions; basic session, tool execution and contract suites passed 76 tests; SXR was absent from PATH.
- Horizon integration/cognitiveos-108 at 18b27d1 passed check-single-cognate (28 packages, one copy each) and bun run check.
- cognate-runtime-api 0.10.0 exports CaseContext and caseContextIssues, which satisfies the I5 compatibility gate the convergence design requires.
- Cognate integration/cognitiveos-108 is created from the 0.10.0 main line; SEA-Forge integration/cognitiveos-108 is created from fde/integration plus the design package.

**Limits:**

- No CognitiveOS ticket is implemented yet beyond the #109 alignment; #110, #111 and #117 are next.
- Workbench consumes published @godspeedai/cognate-* packages only. The #112 rlm contract needs a published Cognate version and there is no npm credential on this host; no alias or tarball substitute is permitted.
- The full Workbench suite, browser journeys, PostgreSQL suites and Horizon verify were not run in this session.
- Integration branches are local-only; none are pushed and no protected-main merge is proposed.

**Next:** Commit and push the #109 version gate, implement #110, #111 and #117 in isolated worktrees, then implement the Cognate half of #112 and resolve the Cognate publication gate with the owner.

**Evidence:**

- .agents/reports/2026-11-19-cognitiveos-convergence/IMPLEMENTATION-HANDOFF.md
- https://github.com/GodSpeedAI/SEA-Forge/issues/108

---

# Historical status

**Status revision:** 380

**Stage:** FDE open work | WP-23 assignment binding landed; guest-reset control green

**Summary:** The WP-23 lane (fde/wp23-delegate-verifier de8f76d1) now binds the full approved assignment to the delegated run per assignment-binding.v1, contracts-first, with a fail-closed trusted read; the Portability guest-reset control at fde/wp21b-evidence-f9eaf164 passed all three jobs; the missing fde-sxr-integration RealityTrace worktree was recreated at da47df5.

**Verified:**

- SEA WP-23 de8f76d1: 173 conformance scenarios (159 passing, 14 pending wp-31) with FDE_WP21A_FINAL=1; server+case-runner 806 passed / 0 failed / 22 ignored; 13 contract digests match (assignment-binding.v1.json added, execution-episode-ref.v1.json deliberately re-pinned per OD-3); S2 slice matched all 17 frozen rows vs the 637cbece baseline; graft build refreshed; hosted CI 44804758916 running
- Portability guest-reset control run 44780753549 (fde/wp21b-evidence-f9eaf164): Field Case guest reset Linux/ext4 3m9s, verify windows 18m55s, verify macos app sandbox jail 2m57s, field-case-linux-guest-reset artifact uploaded
- _wt/fde-sxr-integration recreated as a detached worktree at da47df591cb6bd846f5537266c4ea3d8a1802504 with sxr-cli/examples/nt07_persist.rs present (joined NT-07 fixture reference)

**Limits:**

- APFS and fixed-NTFS guest resets remain open (no approved Windows guest image or disposable Apple host registered); only application-restart-level evidence exists
- The WP-23 binding has no SFWP read verb or Cognate/Workbench wiring yet; the trusted read is internal
- Hosted CI 44804758916 on de8f76d1 was still running when this entry was recorded

**Next:** Wire Cognate's assigned-start Connect providers against the recorded binding and the durable idempotency regression (handoff section 2 items 3-4), then Workbench PR #40 through the operator journey; dispatch the guest-reset workflow at the next combined storage/promotion candidate.

**Evidence:**

- https://github.com/GodSpeedAI/SEA-Forge/actions/runs/44804758916
- https://github.com/GodSpeedAI/SEA-Forge/actions/runs/44780753549
- contracts/fde/assignment-binding.v1.json (de8f76d1)


**Verified:**

- Cognate 3aae0d8 passed all 193 focused tests and hosted verify 44753752380. A real PostgreSQL record lock forced expiry before publication and the gated write left no event, record update, outbox, or idempotency result. The isolated mutation without the final check failed the same assertion.
- SEA 637cbece passed all 49 FDE scenario tests, all 12 frozen digests, and the exact S2 comparison with 17 matching rows, including AT-09 partial. The regression failed before the assignment transition stored its exact delegation link.
- Workbench ccf4462 passed 16 focused tests and a real Chromium journey with a server-issued operator session. The live-server test reproduced the missing policy-refusal audit event before the fix. The final route preserves the existing operator-only policy and records the denied direct request.
- SEA hosted CI 44754422337 passed all ten jobs at 637cbece.
- Workbench 9824735 passed all seven jobs in hosted verify 44757271146, 16 focused local tests, and a real Chromium journey that displayed a live activation refusal with the recorded actor before the operator cleared safe mode.
- Workbench safe-mode properties passed at FC_RUNS=2000 with the existing cap on A5. The earlier safe-mode-only run 44755662049 failed to launch the preview browser on its hosted runner. The exact preview test passed locally.
- Exact checkpoint fc2da542 matched all 17 frozen S2 rows except duration, including the expected AT-09 partial result. Joined NT-07 passed again against the real RealityTrace SQLite ledger, and both native Windows and macOS jobs passed in Portability run 44665538414.
- Hosted Workbench verify run 44663728747 passed all seven jobs at 8fed14e on retry. Its initial attempt missed a same-tenant settings row, and the exact test passed locally before the retry.
- The local proof runner returned exit 2 for slice s3 because only slices s1 and s2 are registered.
- Cognate 3f01dc5 passed the hosted full verify job in run 44663363590. The local focused Connect and assignment suite passed all 98 tests, and protobuf regeneration matched both checked-in files.
- Workbench safe boot at 05d50c8 passed all seven jobs in 44662429211. The live capability detail checkpoint 8fed14e passed type checking, 11 focused tests, and a Chromium journey that registered and activated a real sample bundle. The browser showed the new activation without a page reload.
- The WP-22 fixture correction at 9879de2b passed all ten hosted CI jobs and the macOS native restart and Linux/ext4 guest reset jobs. The platform-neutral fixture follow-up fc2da542 passed 112 case-runner tests with 20 ignored.
- WP-22 fixture-only follow-up 9879de2b passed all 112 case-runner tests, including the unchanged writer scan. The 20 ignored tests include mode-specific and externally controlled fixtures.
- Workbench T4a checkpoint 05d50c8 is published. The restart regression failed on bbe6c73 because a bundle activation succeeded with a durable safe-mode violation. The corrected gate passed the same regression and six additional safe-mode tests. Type checking passed.
- The existing S1 anchor digest verified locally. Its same-user, unsigned receipt and missing server push time keep independent anchoring partial.
- Clean checkpoint 7bfe543c matched all 17 frozen S2 baseline rows except duration, including the AT-09 partial result and all 12 contract digests.
- The joined NT-07 test passed at 7bfe543c using the RealityTrace SQLite proof at da47df5. The settled evidence retained its world reference after promotion and a new case episode froze under the new pin.
- WP-22 is published at 7bfe543c on fde/wp22-snapshot-pin. The promotion reader passed 11 staged tests and all 12 saga tests, plus case-runner Clippy. The selected pin survives a cell repin, and a corrupt selector cannot fall back to a stale flat pin.
- At 9cbc3e16, separate deliberate lock and recovery mutations failed each of their targeted W3, W4, and W2 tests in the observed runs.
- The WP-23 Workbench binding passed its 4 focused tests and type checking. A broader run was stopped after failures in stream cancellation, directory-picker selection, and the tool-execution journey.
- Hosted CI run 44599724625 passed all jobs at a33c8147, including both Ubuntu and macOS workspace shards, MSRV 1.92, live dependency and secret gates, OPA, and Workbench.
- The first macOS server run saw the close event before the legacy case projection completed. The follow-up is test-only and observes both records within the existing deadlines. All seven local supervisor tests and the hosted macOS server shard passed.
- Portability run 44597378183 passed all jobs at 48f393c4, including native Windows and macOS staged publication and application restart and seven Linux/ext4 guest resets.
- The redacted scan across all nine Chrono migration commits and the follow-up test commit found zero leaks. All staged-secret canaries passed.
- SEA Forge candidate 48f393c4 is published on fde/chrono-integration. The live advisory, ban, license, and source checks passed, and the locked all-feature all-platform graph contains no Chrono. The final kernel dependency gate passed on the host and Windows.
- DomainForge 1.0.0-rc.2 at 0226b50 passed 1343 native, 23 WASM, and the Python wheel and Node addon identity checks. RealityTrace at 7be5d8c passed 213 workspace tests with 3 ignored.
- The candidate passed 42 DomainForge adapter, 10 authority compiler-upgrade, and 13 real socket conformance tests. Both historical and linked compiler upgrades retain explicit producer provenance. The historical 0.22.0 AT-01 replay remains pinned to its own binary.
- Clean checkpoint 48f393c4 matched all 17 baseline S2 rows except duration. Fifteen scenarios passed, AT-09 remained at the expected partial result, and all 12 frozen contract digests matched.
- On isolated WP-22 checkpoint 9cbc3e16, each focused baseline passed. Removing each of the two locks separately exposed a lost settled evidence winner and a mismatched case pin. Reverting orphan-effect classification returned PreSaga instead of Uncertain.
- SEA Forge checkpoint 44828457 passed workspace lint and typecheck, 349 authority tests, and 4 PostgreSQL deadline tests. The follow-up b71946d9 passed all 6 timestamp regressions and focused Clippy.
- RealityTrace compatibility checkpoint 7be5d8c passed 75 core tests and 39 CLI tests. SEA Forge pin checkpoint 6bffd5c5 passed 45 RealityTrace adapter tests with 1 ignored and the 20-crate host and Windows no-async-kernel gate.
- Exact fd843472 hosted CI run 44425451223 passed both Ubuntu and both macOS workspace shards, MSRV 1.92, authority, and Workbench. The only failed prerequisite job was lint at the live Chrono advisory.
- Exact fd843472 Portability run 44423954384 passed all jobs, including native Windows and macOS staged tests and the seven Linux/ext4 guest resets. The evidence artifact and review record are linked in ADR-015.
- Exact fd843472 local workspace tests recorded 2064 passed, 30 ignored, and the existing WSL CLI lifecycle failure. The hosted Ubuntu shards passed for the same commit.
- The second independent Codex Terra review at fd843472 exited zero with zero findings at each severity. The first pass and the two resolved test findings are recorded in .agents/plans/2026-11-18-wp21b-selector-review.md.
- Exact fd843472 S2 results match all 17 baseline rows except durations. Fifteen scenarios passed, AT-09 is the expected partial, and all 12 frozen contract digests matched. Focused Clippy and the 20-crate no-async-kernel gate passed.
- DomainForge checkpoint 6167787 had 1345 native tests pass, 13 ignored and 23 WASM tests pass. SEA Forge timestamp checkpoints 58856ed3 and 992f0a6a cover current-time records and deterministic release timestamps.
- Portability run 44421054364 passed all three jobs. The Linux artifact shows six guest resets with a fresh recovery boot on the same ext4 raw virtio disk, QEMU 8.2.2, Linux 6.8.0-149-generic. Every valid recovery committed a retry.
- Native Windows 10.0.26300 fixed NTFS passed the OS helper boundary test and all 14 staged case-runner library tests. The actual paused reader verified the prior generation and a fresh reader selected the new one. The error-variant test distinguished refusal from publication uncertainty.
- Joined NT-07 passed against RealityTrace checkpoint da47df5. The real SEA emitter, RealityTrace append, CLI ledger verify, and CLI export exercised the same E8 before promotion, after promotion, and after re-pin.
- The new guest probe compiled with 12 passing staged library tests and one externally controlled test ignored by design. The Portability workflow passed actionlint.
- The held-reader regression failed before the change on native Windows 10.0.26300 fixed NTFS and passed afterward. All 12 staged library tests and 9 C1–C7 model tests passed in the same native environment.
- Both hosted Ubuntu test shards passed at 085e6c6e in CI run 44402751703. S2 matched the comparison baseline with 15 passes and the expected AT-09 partial. Both hosted macOS jobs were cancelled.
- The lineage checkpoint passed all 12 case-runner library tests and all 3 writer and lock scan tests. The historical digest test retrieved a committed predecessor, excluded a complete unselected generation, and rejected a damaged lineage. Existing selected and flat lookup precedence passed.
- The targeted macOS rest retry for run 44365325837 ended cancelled. The existing fixed NTFS and Linux application restart evidence remains available.
- On Linux the streamed capture passed 11 case-runner unit tests, all 3 writer and lock scans, all 9 C1–C7 model tests, all 23 settlement unit tests, and the server Field Case write-only sealing regression.
- The staged tests exercised a real partial scratch write followed by StorageFull, refusal of a modified spool with the prior generation still selected, and a healthy duplicate-path retry with identical published bytes.
- RealityTrace checkpoint 55e496b persists the emitted world pin in envelope scope. All 6 focused mapper tests and all 3 CLI ledger tests passed, including a real SQLite append and export with hash verification.
- The offline locked cargo-deny license gate reproduced the MIT-0 failure and passed with the approved package and version exception. The global allow list and advisory gate retain their existing configuration.
- Both hosted Ubuntu test steps passed at 387ce96a in CI run 44365325837, including the cgroup-v2 delegation preflight and the workspace test shard that covers the WSL lifecycle failure.
- The corrected legacy migration fixture passed all 9 case-runner library tests on Linux and native fixed NTFS Windows. The native Windows C1–C7 model suite also passed all 9 tests.
- S2 at 387ce96a matched all 17 baseline rows except duration. Sixteen rows passed, AT-09 remained partial for WP-31 human A3, and all 12 contract digests matched.
- The full WSL workspace suite at 387ce96a recorded 2058 passed, 1 failed, and 28 ignored. Only the known undelegated cgroup-v2 lifecycle diagnostic failed. The raw-writer scan, staged restart test, and selected-artifact regression all passed.
- Hosted CI run 44365325837 was started on isolated checkpoint 387ce96a. The lint report confirmed the chrono advisory RUSTSEC-2026-0543 and the MIT-0 license rejection for borrow-or-share. Workbench, MSRV, and authority checks passed.
- The staged case-runner interruption test passed on Linux and native Windows on a fixed NTFS volume. It covered both sides of first publication and selector replacement. The observed snapshot matched the interrupted boundary and a subsequent commit succeeded in every case.
- DomainForge checked in its generated timestamp migration at 2f59e44. The native compatibility test, all-target all-feature Clippy, formatting, and commit lint passed. The earlier native CLI suite and WASM check passed.
- The server selected-snapshot regression passed on Linux. Artifact retrieval returned the published bytes, digest, run ID, evidence ID, and URI.
- At 89bc705a all 104 case-runner tests passed with 19 ignored. The source scan also passed with the interruption and artifact regressions applied.
- In Portability run 44320285578 at 13b5ec10, native Windows job 134857115151 and macOS App Sandbox job 134857114371 both passed.
- The earlier pinned-read checkpoint 1cbc292e passed native Windows in job 134850336686.
- The full workspace run at 13b5ec10 recorded 2056 passed, 2 failed, and 28 ignored. The failures were the raw-writer source scan on the Windows test helper and the known WSL lifecycle stderr diagnostic.
- All 8 case-runner library tests passed locally, including the staged publication and durability-boundary cases.
- All affected Windows targets cross-checked, including the new native NTFS publication test in the existing Portability test target.
- Clippy with -D warnings passed for all targets of sandbox, case-runner, server, evidence, and settlement.
- The Field Case sealing regression passed after the Unix hardening. The macOS sandbox target and Windows os and sandbox targets cross-checked.
- An independent Linux syscall trace observed scratch writes for the trace, evidence, and report before their canonical run files appeared.
- S2 at bcafa556 matched the baseline across all 17 rows except duration. It recorded 15 scenario passes, AT-09 partial, and 12 contract digests verified.
- Native Windows job 134841303184 exposed error 87 in the initial pinned read. A native local NTFS fixture reproduced the old generic-access status and read the exact expected bytes with the corrected concrete rights.
- The settlement package passed 21 unit tests and the four focused server conformance targets passed 101 tests. A real Field Case write-only handler sealed the exact bytes used for its evidence hash and settlement.
- At bc7cd8ca, the full workspace suite recorded 2055 passed and 28 ignored. The sole failure remains the confirmed local WSL cgroup diagnostic before the CLI JSON frame.
- A standard-user Windows NTFS fixture succeeded at directory FlushFileBuffers, first sibling installation, and selector replacement through SetFileInformationByHandle with write-through, including with a non-deletable parent handle held.
- All 90 server conformance tests passed after the freeze batch change. The focused legacy replay regression also passed and checks exact restored outbox bytes.
- S2 proof at baa4f109 matched the baseline in all 17 rows except durations. It recorded 15 scenario passes, AT-09 partial, and all 12 frozen contract digests verified.
- All 90 tests passed in the interaction, FDE scenario, and Field Case conformance targets.
- On isolated checkpoint 8c0fce88, CI run 44306807062 passed Ubuntu and macOS test shards, MSRV, authority, and Workbench. Portability run 44306811678 passed native Windows and macOS App Sandbox.
- The full local server suite passed 689 tests with 2 ignored.
- The isolated native macOS server and App Sandbox jobs passed on 8c0fce88.
- Server conformance passed all 91 tests across conformance_case_episode, conformance_fde_scenarios, and conformance_field_case.
- Checkpoint 8c0fce88 is on fde/wp21b-verify-5f67ae95. Native CI run 44306807062 and Portability run 44306811678 were dispatched for that checkpoint.
- All 29 focused guard tests passed, including a regression proving the first write can remain after a later filesystem failure.
- All 80 server conformance tests passed across the FDE scenarios and Field Case targets with typed admission present.
- The case-runner library passed all 8 tests locally. The two server conformance targets passed 80 tests locally with the fixture repairs present.
- The Linux S1 driver example compiled. Windows compilation stopped in the cross-toolchain before reaching the driver because the local ring build lacks MSVC tools.
- Native verification at 5f67ae95 passed Ubuntu server and rest and the macOS App Sandbox job. The native macOS server and rest jobs found fixture failures, and Windows found the missing example entry point.
- Focused server conformance passed 80 tests across conformance_fde_scenarios and conformance_field_case.
- The Linux model suite passed all nine tests, including its existing exact inotify event comparison.
- All case-runner targets cross-checked for aarch64-apple-darwin and x86_64-pc-windows-msvc.
- The case-runner suite passed 103 tests with 19 ignored, including five staged-batch tests.
- The staged tests verify first and subsequent publication, pinned concurrent readers, legacy preservation including a nested run artifact, invalid staged identity, and both sides of the selector publication boundary.
- The legacy migration path refuses an incomplete file set before publication. The write-boundary scan still checks raw writes and permits the two reviewed private staging routines.
- The server suite passed 689 tests with 2 ignored. WSL workspace and S2 row results are recorded at revision 341.
- The workspace rerun passed 2048 tests with 28 ignored. The only failure was the WSL CLI lifecycle diagnostic test.
- The final reader S2 proof matched the recovered checkpoint row for row. Fifteen scenarios passed and all 12 frozen contract digests matched.
- A private fixture confirmed that the WSL sandbox cgroup warning precedes the JSON internal_error diagnostic on stderr.
- The owner approved an internal uncertainty outcome for a published selector with an unconfirmed durability flush. Prepublication refusals retain the frozen FDE response.
- The local t13_1 fixture returned a missing case_id request error before the fix. T13.1 passes with the existing case_id field populated.
- The focused CLI M13 and M5 suites passed all five tests. M5 checks all eight case-owned runtime workspaces and retains the top-level side-effect restriction.
- Case-runner suite passed 97 tests with 19 ignored. The reviewed server suite passed 689 tests with 2 ignored.
- The new server tests verify selected run data, flat run precedence, and integrity errors for corrupt data and invalid discovery.
- The S2 proof before the final reader review matched the prior checkpoint row by row. Fifteen scenarios passed, AT-09 remained partial, and all 12 frozen contract digests matched.
- The standalone staging and publisher prototype passed five tests on this host, including a round trip through the production reader, concurrent readers, and injected flush failures.
- just crate-test sea-forge-case-runner passed 96 tests with 19 ignored.
- The focused server case views suite passed 3 tests and the CLI ledger suite passed 4.
- S2 proof on 9745a729 matched 9cbc3e16 row for row, with 15 scenario passes, AT-09 partial, and all 12 contract digests verified.
- On checkpoint 9cbc3e16, S2 proof recorded 15 scenario passes and AT-09 partial. All 12 frozen contract digests matched.
- With the terminal lifecycle change, just crate-test sea-forge-case-runner passed 94 tests with 19 ignored, including the new positive termination test.
- just crate-test sea-forge-case-runner passed 93 tests with 19 ignored, including 9 model and 12 promotion saga tests.
- just crate-test sea-forge-server passed 687 tests with 2 ignored.
- git diff --check passed for the recovered worktree changes.
- cargo update -p domainforge-core --precise 0.22.0: only domainforge-core changed in Cargo.lock
- cargo test -p sea-forge-domainforge: all suites ok (22, 4, 5, 11 passed; 0 failed)
- cargo test -p sea-forge-authority --test cep_compiler_upgrade: 9 passed; opa_candidate 22 passed + 7 ignored passed with opa
- cargo test -p sea-forge-server --test conformance_cep_authority: 13 passed
- cargo test -p sea-forge-case-runner --test cell_episode: 24 passed + 6 ignored passed with opa
- fixtures regenerated with the 0.22.0 CLI; cargo fmt --check on touched crates: clean
- scripts/check-agent-context.sh: ok
- 2026-11-13 readiness harness (godspeed-readiness/scripts/readiness.sh, local_operator profile, heads synced to origin main) reported GO 30/30 after the DomainForge 0.21.2 lockstep; evidence in the S0 baseline files under the report directory
- 2026-11-13 scripts/e2e-world-loop.sh against pinned origin heads: final line 'e2e-world-loop: OK' (revision 233)
- gitleaks scan of the report directory: no leaks found
- PR #71 checks: 20+ green incl. gate/test/windows/macos/kernel-22.04; run 40614104830 all four legs success; independent artifact verification
- sea-forge-server and sea-forge-sandbox tests pass on this WSL2 host (workload isolation and cgroup-delegated lanes report NOT_RUN); fmt and clippy -D warnings clean
- conformance_case_episode 11 passed (cgroup host verdict is an object; trace and evidence compared for equality)
- Merged origin/main (be63b711, DomainForge 0.22.0 lockstep); branch assertions compare against EXPECTED_DOMAINFORGE_VERSION
- scripts/fde-proof.sh --slice s1 on a clean tree: 0 FAIL; fde-s1-verify.sh: anchors, manifest, replay bytes and classes agree

**Limits:**

- Horizon Hand off owns the assignment Brief and Budget. The current SEA delegate contract carries the episode and assignment ID but has no binding for those approved values. Production needs a versioned binding and an authenticated handoff tied to the existing SEA run.
- Workbench has authenticated safe-mode status and operator clear. The bundle details route still needs authored panel descriptors and views from the immutable version directory plus host fault recording. Effective grant display depends on the T4b projection replay fix.
- T4b restart verification must show that a rebuilt authority index retains the same grant principal, permissions, expiry, and activation sequence as the live index. The separate T4b checkout has existing uncommitted work in that area.
- The older promotion-store initializer failed Windows directory fsync in the original reader fixture setup. The final Windows reader tests use prepared case data and pass. Production promotion-store writes on Windows require separate durability support and qualification.
- S3 has no registered proof slice or confirmed live phone enrollment route. The owner runbook and WP-35 independent anchoring require those implementation and provisioning prerequisites.
- SEA assignment events do not identify their exact issued or resumed delegation link. The legacy writer appends the assignment state and link separately, so the latest rows cannot establish a current unambiguous delegation after a partial write.
- SEA stores the delegation and assignment state but does not store the Brief and Budget. The durable Cognate assignment reader requires an existing run. Delegate-only admission needs a trusted assignment source before start and a server-authenticated agent identity bound to the SEA delegate.
- The local just check reached the Git history scan after passing status, format, Clippy, typecheck and live cargo-deny. The workstation full-history scan reported 166 findings outside the clean Chrono commit range.
- GitHub reports no registered self-hosted runners for this repository. An approved bootable Windows image and a disposable remote Apple host with guest reset control are still missing.
- Hosted native staged tests establish application restart behavior. The Apple guest host and approved Windows guest image or reset-enabled runner remain missing for cross-platform OS-reset qualification.
- The Linux proof establishes guest OS-reset recovery for the tested ext4 configuration. macOS/APFS and Windows/fixed-NTFS still need disposable persistent disks and external reset controllers. The versioned writer is still test-gated.
- Ordinary C7 delivery to RealityTrace needs an approved producer and write entrypoint. The joined proof uses the already-admitted security acceptance producer.
- The fixed NTFS tests establish replacement visibility and application restart. They do not establish host-reset durability.
- Production activation requires deploying the v1/v2 reader first and retaining it as a rollback target. The writer remains staged.
- The production Field Case dispatch uses streamed capture with the legacy writer. The versioned writer exercises the same spool operation under staged tests.
- The macOS server and rest jobs in CI run 44365325837 were queued when this snapshot was recorded. The native restart evidence on Linux and fixed NTFS concerns application process exits.
- The Windows C1–C7 model compares final filesystem differences to the journal. Transient runtime observation and host-crash durability remain open on Windows.
- The local WSL host lacks delegated cgroup-v2. Its CLI lifecycle failure leaves a native Linux verification gap on this checkpoint.
- Settlement batch grouping still uses the legacy writer. The declaration prepares it for versioned publication.
- Execution admission uses the typed legacy outcome. Versioned publication and the case-record, outbox, and recovery outcome consumers still need integration.
- The staged writer runs in the test build. Runtime foreground, recovery, and background writes need typed outcome handling.
- Non-Linux model tests compare final-byte changes to the journal. Native publication durability and independent transient C7 observation need platform-specific evidence.
- The WSL lifecycle gate needs a writable delegated cgroup-v2 directory. AT-09 awaits the WP-31 human A3 settlement proof.
- Cognate/Horizon live suites not re-run
- 0.21.0 prior-identity fixtures kept deliberately: they model the older producer
- docs/how-to/upgrade-domainforge-producer.md still records a 0.21.1 check and its a71ff6 digest as history
- A detached grandchild that survives the process-group kill could append after the operator attests NotDeclared; lease safety assumes the authority call is bounded
- --declared <ref> is not checked against a ledger declaration
- A failure inside declare before the transport spawns is held for an operator, which is conservative
- The CLI builds an Operator from --actor, the same local trust model as approve
- The Horizon policy renderer and the sea-rs fixture are not tied by a test (follow-up)
- The server test covers a nonzero exit only; the timeout variant is covered at the CLI
- The world-load log is INFO level; SFWP has no way to list loaded worlds
- The liveness guard sees the recorded process group only: a process that left it (setsid/setpgid), a resolver in another pid namespace on the same boot, and the instant between fork and the ledger write are not seen
- An Unknown liveness result or a malformed launch record can be overridden only by the operator's audited --attest-command-gone; a group that is positively alive is always refused
- Exactly-once across an operator NotDeclared and a late declaration still relies on the external authority treating authorization_request_sha256 as an idempotency key (contract documented in sea-forge-settlement declaration.rs)
- Server-crash containment comes from the Horizon systemd unit (KillMode=control-group, Horizon#26); it takes effect once the owner reinstalls the unit
- An attempt with no launch record keeps the lease-only rule: NotDeclared is accepted after the lease lapses without a liveness check or --attest-command-gone. This covers claims made before launch recording existed, a transport without launch evidence, and a request with no authorization_ref. A failed launch-record write stops the command, so it does not produce such an attempt.
- Only sea-forge-capability was tested (the sole rusqlite user); full workspace tests, CI and MSRV 1.92 job not run locally; rusqlite 0.41.1 declares rust-version 1.89
- The FDE planning baseline on main is documents only; S1 implementation is on the fde/integration branches, not on main
- Owner decisions OD-1 to OD-6 are open; see .agents/reports/fde-convergence-2026-11-12/H-egm-review-record.md
- The planning baseline is draft 3 and the SOP is a v0.4.1 draft; neither is approved
- cargo test --workspace and just ci not run (host memory); Cognate/Horizon live suites not re-run
- FDE planning baseline is documents only (S1 not started); the hardening work changes product code
- Open: 24.04 fixture defect (non-gating); admission control; F17 allow-control (upstream); Milestone C on-device
- cgroup pidfd sweep cannot reach child-cgroup members (fails closed as survivors); macOS ProcessGroup claim rests on a best-effort census; live egress allow-control policy shape: owner decisions pending on PR #71
- Stack ran on the DomainForge 3960326 build (SEA-Forge embeds 0.21.2); I-1 execution gate unwired; anchor local-only (OD-6); ep-3 NO_GAIN; AT-14/NT-03/NT-06 PARTIAL

**Next:** Carry the Horizon Hand off Brief and Budget through a versioned SEA admission binding and authenticate the same admitted run to Cognate. Wire authored version-directory fragments and host fault recording into T4a. Qualify APFS and fixed-NTFS guest resets on disposable hosts.

**Evidence:**

- .agents/specs/fde-field-case-storage.spec.md
- docs/decisions/ADR-015-versioned-field-case-storage.md
- .agents/reports/fde-convergence-2026-11-12/STATUS.md
- .agents/reports/fde-convergence-2026-11-12/H-egm-review-record.md
- .agents/reports/fde-convergence-2026-11-12/F-implementation-plan.md
- .agents/plans/2026-10-28-consolidated-final-report.md
- .agents/plans/2026-10-27-arm64-jetson-packaging.md
- .agents/plans/2026-10-26-capability-harvest.md
- .agents/reports/fde-convergence-2026-11-12/evidence/s1/wp-19-gate.md
- .agents/evidence/fde-field-case/s1-20261114T234200Z/claim-ledger.md
