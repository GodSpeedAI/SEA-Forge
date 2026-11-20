# Current status

**Status revision:** 381

**Stage:** FDE WP-23 | Versioned assignment binding on the delegated run

**Summary:** A Field Case delegate now binds the full approved assignment (explicit domain_meaning and five SOP currency ceilings) to the admitted run, recorded as a versioned binding that joins link and run, with a fail-closed trusted read.

**Verified:**

- Contracts first (OD-3): new contracts/fde/assignment-binding.v1.json (AssignmentApproval, Binding, Error) with valid/invalid/semantic fixtures and an 8-scenario family; DelegateRequest gained the optional assignment member; DIGESTS re-pinned in the same change (13 entries match via scripts/check-fde-contract-digests.sh); README section 11 and CHANGELOG updated; the other 12 schemas' bytes are unchanged
- TDD: the new assignment-binding family failed red for the expected reasons before implementation (five admissions where refusal was required; two accepted delegations with no recorded binding); after implementation all 49 conformance targets passed with FDE_WP21A_FINAL=1 (173 scenarios: 159 passing, 14 pending wp-31)
- sea-forge-server and sea-forge-case-runner full suites: 806 passed, 0 failed, 22 ignored (FDE_WP21A_FINAL=1); cargo fmt --check and clippy clean; graft build refreshed
- S2 slice at this tree: all 17 rows match the frozen baseline at 637cbece ignoring duration (16 PASS, AT-09 expected PARTIAL, digest preflight PASS)
- Portability guest-reset control run 44780753549 at fde/wp21b-evidence-f9eaf164 passed all three jobs (Linux/ext4 guest reset, Windows, macOS sandbox) and uploaded field-case-linux-guest-reset

**Limits:**

- The binding is recorded and readable in the case store; no SFWP read verb exposes the trusted read yet (Cognate/Workbench wiring is the next WP-23 step)
- execution-episode-ref.v1.json was re-pinned (deliberate, contracts-first, same commit); consumers vendoring by digest must pick up the new line
- APFS and fixed-NTFS guest resets remain open: no approved Windows guest image or disposable Apple host is registered; application-restart-level evidence only

**Next:** Wire Cognate's published assigned-start Connect providers against this binding (handoff section 2 items 3-4), then Workbench PR #40 through the operator journey; keep the trusted read internal until that contract lands.

**Evidence:**

- contracts/fde/assignment-binding.v1.json
- contracts/fde/fixtures/assignment-binding/
- .tmp/fde-proof/s2/results.json
- https://github.com/GodSpeedAI/SEA-Forge/actions/runs/44780753549

---

# Historical status

**Verified:**

- The existing socket scenario reproduced the missing link ID. The same conformance harness passed all 49 tests after the fix with FDE_WP21A_FINAL enabled. All 12 frozen contract digests matched.
- CI run 44597369464 passed both Ubuntu shards, MSRV 1.92, the live security job, Workbench, and OPA. Its initial macOS server shard observed the legacy close event while the case projection was still active.
- The supervisor tests now wait for both durable close indicators within the existing deadlines. All seven supervisor tests passed locally. Existing state, event ordering, attribution, and quiescence assertions remain in the suite.
- At exact 48f393c4, all 17 S2 rows matched the baseline except duration. Native macOS Portability and all seven Linux/ext4 guest resets passed in run 44597378183.
- The redacted range scan found zero leaks in the nine Chrono migration commits and all staged-secret canaries passed.
- The linked DomainForge 1.0.0-rc.2 producer is pinned to 0226b504ebda3dcb64ccd8859af7de8c9f0ad7ba. At that commit 1343 native and 23 WASM tests passed, 13 native tests were ignored, and the Python wheel and Node addon passed their focused identity tests.
- SEA Forge preserved the historical 0.21.0 to 0.22.0 compiler-upgrade tests and verified an authorized 0.22.0 to 1.0.0-rc.2 upgrade with equivalent semantic closure and separately recorded producer identities.
- The locked all-feature all-target dependency graph contains no Chrono. Live cargo-deny passed advisories, bans, licenses, and sources. Only the official DomainForge Git repository was added to the source allow list for the full-revision pin.
- With the final producer pin applied, all 17 S2 rows matched fd843472 except durations. Fifteen scenarios passed, AT-09 remained at the expected partial result, and the preflight verified all 12 frozen contract digests.
- RealityTrace checkpoint 7be5d8c passed 75 core tests and 39 CLI tests, including the fixed envelope hash and historical timestamp boundaries. The sxr-core normal, build, and development dependency tree contains no Chrono.
- SEA Forge passed 45 RealityTrace adapter tests with 1 live-environment test ignored on the updated pin. The locked all-feature all-platform inverse graph now identifies DomainForge 0.22.0 as the only Chrono caller.
- At checkpoint 44828457, just lint and just typecheck passed across the workspace.
- The follow-up passed all 6 focused timestamp tests and all-target sea-forge-core Clippy. It retains the separate typed RFC 3339 validation path and compatible serialized UTC spellings.
- The direct migration passed 5 focused timestamp tests, 4 PostgreSQL deadline tests, and 349 authority tests with 7 OPA-dependent tests ignored.
- The direct migration compiled across workspace targets. The 20-crate synchronous-kernel dependency gate passed on the host and x86_64-pc-windows-msvc graphs.
- The locked all-feature all-platform dependency graph now reaches Chrono only through the DomainForge 0.22.0 registry dependency and RealityTrace sxr-core 0.3.0 Git pin.
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

- The legacy writer may leave a partial prefix. The new field enables an exact join when all three authoritative records are present. Production assigned starts still need the authenticated cross-service handoff and authoritative Brief and Budget.
- Local just check completed status, formatting, Clippy, typecheck, and the live dependency gate, then stopped at 166 findings in the workstation's full Git history. All new Chrono migration commits scanned clean.
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

**Next:** Verify the published checkpoint with the frozen S2 baseline and hosted CI. Continue using the admitted SEA run as the execution authority for the Cognate bridge.

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
