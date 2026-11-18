# Current status

**Status revision:** 351

**Stage:** FDE S2: capture and seal Field Case run evidence

**Summary:** Run the Field Case executor in scratch space, settle against captured bytes, and declare its sealed evidence through the case guard.

**Verified:**

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

- The staged versioned writer is still inactive in production. The NTFS fixture establishes native API behavior on one host, and remains short of publisher integration or crash-recovery qualification.
- Native Windows verifies the Windows build and control plane. WP-21B still needs a durable selector publication mechanism qualified on its supported Windows storage environment.
- The CI gate remains red on the chrono advisory and MIT-0 license rejection. The Windows native job does not qualify Field Case selector durability.
- The native lint rerun also rejects MIT-0 for transitive borrow-or-share 0.2.4 through fluent-uri and jsonschema. This requires a license policy or dependency decision.
- Settlement batch grouping still uses the legacy writer. The declaration prepares it for versioned publication.
- Execution admission uses the typed legacy outcome. Versioned publication and the case-record, outbox, and recovery outcome consumers still need integration.
- Native verification for the fixture repairs is pending. RustSec RUSTSEC-2026-0543 blocks the lint gate through both direct chrono use and domainforge-core.
- The staged writer runs in the test build. Runtime foreground, recovery, and background writes need typed outcome handling.
- Non-Linux model tests compare final-byte changes to the journal. Native publication durability and independent transient C7 observation need platform-specific evidence.
- Runtime evidence sealing and historical artifact resolution remain open. NT-07 needs a RealityTrace world-reference persistence contract and a cross-repo re-pin fixture.
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

**Next:** Cross-check the sealed reader on Windows, rerun S2, then integrate and qualify the staged publication backend and remaining runtime writers.

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
