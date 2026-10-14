# Current status

**Status revision:** 96

**Stage:** test: recovery restart diagnostics

**Summary:** The Go live-recovery harness now reports, when the restarted server's socket never appears, whether the server process exited and the tail of its log.

**Verified:**

- T00-T08 settled; T09 remains partial; T10-T12 unstarted; stop before T13 unless separately reviewed.
- Safe checkpoint 552bd655ba53b2c46d15e0c61624a57c58143b5d remains the clean publication checkpoint; unsafe 608 history and old cb90 branch remain local.
- Exact publication request for 552bd655ba53b2c46d15e0c61624a57c58143b5d remains held; no push was attempted.
- Private codec source b23cedb3e86681d06750f0012479af4c3ef124b8fdf04f81ae008c73d1de1d26 and unchanged module declaration 11302429bfdb29634c8d780546c42de2bd10e0fdc7d49ce6bc37980a7e12f876 retain independent bounded approval.
- Initial just check failed with exit 101 on clippy sliced_string_as_bytes; the sole source repair changed token[..separator].as_bytes() to &token.as_bytes()[..separator].
- Post-repair GREEN04 passed 18 tests, 0 failures; just check passed all gates; Graft02 exited 0 with 8,059 nodes, 16,073 edges, 713 cards.
- Six post-repair gate captures were archived, root-compared, and independently confirmed.
- The 449/0/2 full-server result was on pre-repair source b69338947dca0a35f4a08f96c57d37163ac282d0902cfc2454e4f776901b6bbb; it is not a post-repair server run.
- Private codec remains unwired; startup signer lifecycle, ledger reader, filter resolution, DTO/dispatch, and T09 settlement are not approved by this slice.
- Shared ledger decoder compatibility debt CW-43 remains open; private mitigation does not establish that the shared decoder is fixed.
- Scoped 18-artifact privacy correction remains unchanged; immutable local history is preserved and no universal secret-free claim is made.
- Startup key: cargo test -p sea-forge-server passed (135 unit tests plus integration suites); clippy --all-targets and fmt clean; four continuation_key_tests cover seed derivation, seed isolation, entropy failure before durable work, and config failure before entropy.
- T09 UI: bun run typecheck clean; bun test 347 pass; bun e2e/run.ts fixture ladder J0-J9 + RECOVERY all PASS (a first run showed one J8 failure that did not recur in two later runs; cause unproven).
- T10: just casework-e2e-live --only L0..L7 passed in one run (L5 9/9, L6 5/5, L7 8/8); tooth stub-gateway and tooth shared-session both PASS and the shared-session guard was mutation-checked; go test -p 1 ./... and cargo test -p sea-forge-server -p sea-forge-case-runner reported passing by the implementing agent.
- T10 phase 4: just casework-e2e-live (fresh build, all journeys) exit 0 per implementing agent; UI 362 tests pass; go test -p 1 ./... ok; tooth console-error PASS.
- T10 prereg confirmation: three consecutive green fresh-cell runs of just casework-e2e-live at 447d9a9 (11/11 journeys each); independent audit verdict CONFIRM WITH CONDITIONS; after fixes bun test 373 pass and a further full live run passed (reported by the fixing agent); new Rust SoD test fails when the check is disabled (mutation-checked) and passes restored.
- T11A: go test -p 1 ./... green (also -tags live); go vet clean in three tag modes; live correlation and backup-wipe-restore tests pass against the real kernel; just casework-e2e-live --skip-build exit 0 (L0-L9, L-RECOV); systemd-analyze verify clean only with ExecStart stubbed to /bin/true (units never run under real systemd).
- T11B: just casework-load exit 0 three times on final code; negative controls fail as intended (removing the watermark fix gives 552 violations; a 1s p95 budget fails the recipe); go test -p 1 ./... and cargo test --no-fail-fast over the kernel crates pass; just casework-e2e-live --skip-build passes after the fixes (reported by the implementing agent).
- T11C: go vet and go test -p 1 ./... green; cargo test -p sea-forge-ledger green; regression tests fail when fixes are reverted (OIDC nonce, OIDC state cap, readyz leak, symlink-safe view temp); just casework-e2e-live --skip-build and just casework-load both pass after the fixes (reported by the implementing agent). TestLiveSubscriptionResumeAcrossRestart: failed ~80% under saturated CPUs before the test fix, 15/15 with -race after.
- T11D: workflow YAML parses and defines three jobs; referenced commands exist locally (just --dry-run casework-load renders, go vet and gofmt clean, agent-browser install --with-deps exists); not run on GitHub and actionlint is unavailable.
- READMEs were checked against code, configs and reports by the editing agent; no recipe was run for the docs change.
- T12 independent confirmation (CONFIRM WITH CONDITIONS): .agents/evidence/casework-live-wiring/T12/independent-confirmation.md; UI tests now 373 (earlier 347 figure superseded).
- T12 did not run just check, just proof or just status-check, and did not reproduce systemd-analyze, the CI workflow, codec captures or the Rust SoD mutation check.
- gitleaks over T10 evidence: 19 generic-api-key hits, all idempotency_key values in harness cell ledgers.
- just status-check passes on a clean checkout of main.
- just check (status, fmt, clippy, typecheck, cargo-deny, gitleaks) run on a clean checkout of main.
- After the merge: Rust workspace 1246 pass / 0 fail; clippy -D warnings clean; Go vet and go test -p 1 ./... ok; UI typecheck clean, 373 tests pass.
- conformance_m5 passes on Linux (4 pass, 2 helper ignores); clippy clean for sea-forge-case-runner.
- sea-forge-sandbox tests pass on Linux including destination_swapped_to_symlink_between_check_and_write_is_refused; conformance_m16 16 pass. macOS result pending CI.
- self_model_cli passes on Linux; 40 repeat runs without failure.
- Rust workspace 1246 pass / 0 fail; clippy -D warnings clean.
- just check run on this branch before push
- YAML parsing goes through sea_forge_core::yaml only: duplicate keys, merge keys, unknown tags, null collections, quoted or non-canonical booleans and non-canonical integers are rejected; input, depth, node and alias limits are bounded.
- Corpus proof: all 107 real documents keep accept/reject, policy identities, hashes and probe verdicts; no adversarial document is newly accepted.
- domainforge-core pinned to =0.20.0, which no longer depends on serde_yaml: unsafe-libyaml and serde_yaml are absent from Cargo.lock and cargo deny passes (advisories, bans, licenses, sources) with no suppressions.
- Approval consumption: a mismatched action, missing assurance or mismatched committed decision is refused without an approval_grant_consumption record; eight concurrent strict callers yield exactly one grant.
- Casework live on GitHub: the go-live job passes; the ladder passes 11 of 11 journeys on re-run.
- Deflake: ACP budget test 30 of 30 plain and 10 of 10 under CPU load; sfwp_supervisor 10 of 10.
- Tauri removal: workbench typecheck and lint pass and 212 unit tests pass without the shell; the root Cargo workspace and Cargo.lock are unaffected.
- just workbench-completion-eval-inputs-check passes with the retired packaged-desktop commands and rejects an input naming a removed desktop-shell recipe
- Rust 1.99.0: fmt, clippy -D warnings and per-crate tests pass on rustc 1.99.0; cargo +1.92.0 check --workspace --all-targets --locked passes.
- Deflake: t13_3_cancel_one_of_three_siblings_settle_normally passes 30 of 30; conformance_m13 passes 25 of 25; clippy -D warnings and fmt clean for sea-forge-server.
- sea-forge-changeset: 15 tests pass against real git (export without Git metadata, explicit changeset, deterministic proposal commit, compare-and-swap promotion, stale-base refusal).
- sea-forge-sandbox OpenShell adapter: openshell_stub tests pass against stub openshell and openshell-prover executables (control flow and failure handling only; not isolation).
- ExecuteSandboxCommand guest-command action added (policy kind execute_sandbox_command); host ExecuteCommand gate and untrusted_executable check unchanged.
- PromoteChangeset is approval-only: its rule must require approval, the only grant path is grant_after_approval, and the compare-and-swap is performed by the trusted promotion path.
- Real OPA 1.21.1 subprocess candidate (OpaCandidate::evaluate) replaces the placeholder engine; failures become Deny candidates. The 7 real-OPA authority tests and 6 real-OPA cell-episode tests pass locally with --ignored against opa 1.21.1.
- Projection lock (openshell.lock) covers exactly three artifacts: policy.yaml, sandbox.json and mapping-report.json.
- Semantic-cell integration on Rust 1.99.0 and the strict YAML boundary: clippy --workspace --all-targets -D warnings and fmt clean; authority, changeset, sandbox, runtime and case-runner tests pass (297 passed, 0 failed); cell_episode 24 pass by default.
- Test isolation: with GIT_DIR, GIT_INDEX_FILE and GIT_WORK_TREE pointing at a decoy repository, the changeset and case-runner tests pass (50 passed, 0 failed) and leave the decoy's HEAD, refs, config, index and object count unchanged.
- OPA candidate binding: a candidate evaluated over another policy's Rego bundle is denied whether or not opa is required (opa_candidate::candidate_from_another_bundle_is_denied); opa_candidate passes 29 of 29 and cell_episode 30 of 30 with --include-ignored against opa 1.21.1.
- authority-opa CI job added and required by gate; the pinned digest 668506eb...5031 equals the GitHub asset digest and the release's .sha256 for opa v1.21.1 linux amd64 static.
- Recovery diagnostics: go vet -tags live clean; TestLiveKillMidCommitRecovery passes 25 of 25 locally against a real server build (the CI failure was not reproduced).

**Limits:**

- Unsafe 608 history and old private casework/live-wiring branch cb90 remain local; do not publish them.
- Exact 552bd655ba53b2c46d15e0c61624a57c58143b5d publication request remains held; no push or push capture is claimed.
- Codec remains private and unwired. Startup, ledger reader, filter/ACK/frontier, public DTO/dispatch, and supported-writer integration remain outside this slice.
- Serialize compiler and heavy gates under root's process, RAM, and swap guard.
- Preserve user .jolli deletions and unrelated .gemini changes; do not alter old private history or scoped privacy correction.
- Startup key unit skipped the grant's RED-first step and ran gates outside root serialization; independent critic review is still required.
- Kernel does not resume an escalated item after approval over SFWP; L5 asserts the approval record and ledger, not downstream effects. The operator's denial is the absence of Approve plus a typed UNAUTHORIZED_ROLE refusal, not a panel-shown refusal.
- Live gateway emits no execution_progress and no typed settlement object; pill progress is never shown.
- Independent critic review of the startup key unit found no failures (Haiku critic); T09/T10 independent confirmation not done.
- After a gateway restart the live page stays Reconnecting until reload (no re-login prompt); ArtifactService caches an error per ref until reload; Thoth policy grant and self-model rebuild are applied in the e2e harness, not in casework-cell-init.
- T10 independent re-confirmation of the post-audit fixes has not been run; the three-run proof predates them.
- Kernel gap CW-45: no resume after approval over SFWP; live harness uses dev auth, hand-built server.yaml, harness-applied policy grant and self-model rebuild (T10-DEV-5); no video/trace evidence (T10-DEV-6).
- T11 not complete: load test (just casework-load), /security-review with high findings fixed, and CI jobs still pending.
- serve.production defaults to false so a production config omitting it is treated as dev auth; kernel socket is 0600 so gateway and kernel must share a uid; metrics endpoint unauthenticated (loopback-only); backup is offline only; continuation key is memory-only; no kernel-side metrics.
- Budgets calibrated on one shared 6-CPU WSL2 host with no scale above 50 clients and no long soak; kernel throughput is bounded (~3-4 intents/s) because pre-action assurance verifies every ledger stream.
- TestLiveSubscriptionResumeAcrossRestart failed once in about seven runs under load and passed on re-run; cause unproven.
- Open operator decisions: CW-49 (OIDC state binding + PKCE), CW-51 (kernel-side bound on reopen/terminate reasons is protocol-visible, needs ask-first), CW-47 (login lockout keying behind a proxy). Lows CW-47..CW-52 are logged, not fixed.
- TestT11IntentInputBounds is a weak regression proof (fails only by not compiling without the fix); restore script hardening F9 has no automated test.
- casework-live.yml is unverified on GitHub (CW-53 asks for one manual workflow_dispatch before relying on it; CW-54 nightly load job may be flaky on shared runners).
- UI dev recipe casework-ui-up shows the local adapter unless VITE_CASEWORK_SOURCE=live; configs/live-serve.json has no static_root, so the gateway serves the UI only with a config that sets it (documented, not changed).
- Open operator decisions: CW-47, CW-49, CW-51; DEBT CW-43..CW-54 remain open.
- just check must be run before any merge or publication; casework-live GitHub workflow has not been dispatched (CW-53).
- T13 not started; nothing published or merged.
- DEBT M-53 still lists two failures on the old base: macOS conformance_m5 tests assume the Landlock jail, and workbench packaged_stack records_survive_the_kernel_being_stopped_and_started_again; PR #11 CI will show whether they persist.
- The symlink guard fix is verified on Linux only; macOS is verified by CI.
- The macOS self_model_cli failure is not yet explained.
- The macOS fix is verified by CI only.
- Stricter than before: an explicit null collection (for example a bare `rules:`) and True/TRUE booleans no longer load.
- Fields under serde flatten or internally tagged enums (planner AgentTask max_turns and token_budget) skip the spelling check; explicit YAML core tags are honoured.
- Approval consumption residual: a grant window that closes between validation and the ledger append still spends the approval.
- Casework live ladder journey L-RECOV (kill -9 recovery) timed out once on GitHub and passed on re-run; treat as flaky until it has a longer record.
- The agent-browser ladder journey L-RECOV (15 s wait for a panel to close, e2e/journeys-live/helpers.ts) timed out once on GitHub and was not changed.
- Workbench renderer: twelve runtime modules still call the Tauri bridge (invoke/listen) with no other transport, so the workbench has no backend at runtime until that bridge is replaced. The workbench completion evaluation input therefore records the integrated evaluation as BLOCKED.
- Toolchain: devbox run could not be verified locally (Nix cache timeout); the MSRV (1.92) CI job has not run on GitHub before this change.
- OpenShell backend is verified only against stub executables; no live gateway, MicroVM, or real enforcement has been exercised.
- Live MicroVM proof is BLOCKED / UNPROVEN: the WSL2 host has no /dev/kvm (nestedVirtualization=false). The 6 tests in crates/sea-forge-case-runner/tests/cell_episode_live.rs are ignored and have never executed.
- Changeset promotion requires the target ref not to be checked out in a working tree; a promotion that loses a race after its pre-check can leave unreferenced objects in the authoritative repository (refs unchanged).
- The OpenShell 0.1.3 MicroVM driver has no per-sandbox CPU or memory limits (gateway-level vcpus and mem_mib only).
- The RepairAgent model's single projected policy is constant true; real-OPA deny and escalate outcomes are exercised with the separate fixture fixtures/policies/gate.
- The real-OPA tests are ignored by default locally; the authority-opa CI job runs them. Its opa digest pin shows the binary is the published release asset, not that it matches a source build. The job has not run on main before this change.
- An OPA candidate is bound to its Rego bundle but not to the action under evaluation; one candidate can be passed to several evaluate calls.
- TestLiveKillMidCommitRecovery has failed intermittently on GitHub (restarted server's socket never appeared within 20 s) and the cause is not known; it may be a real restart defect after SIGKILL rather than a slow runner. The harness now prints process state and log tail on that failure.

**Next:** Run the live MicroVM proof on a KVM host (BLOCKED on this WSL2 host); keep the approval-only promotion path unchanged; diagnose the intermittent TestLiveKillMidCommitRecovery restart failure.

**Evidence:**

- .agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml
- .agents/specs/godspeed.casework-cognitive-environment-spec.yaml
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-clean-publication-final-independent-review-oct09.md
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-private-continuation-codec-root-grant-oct09.md
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/private-codec-green03-exit-oct09.raw.json
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/private-codec-green04-exit-oct09.raw.json
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/private-codec-check01-exit-oct09.raw.json
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/private-codec-check02-exit-oct09.raw.json
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/private-codec-server-full01-exit-oct09.raw.json
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/private-codec-fmt02-exit-oct09.raw.json
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/private-codec-graft01-exit-oct09.raw.json
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/private-codec-graft02-exit-oct09.raw.json
- .agents/DEBT.md
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-startup-continuation-key-root-grant-oct09.md
- .agents/reports/casework-live-wiring/decision-log.yaml
- .agents/evidence/casework-live-wiring/T10/latest
- deploy/systemd
- scripts/casework-cell-backup.sh
- .agents/reports/casework-live-wiring/config-reference.md
- .agents/reports/casework-live-wiring/runbook.md
- .agents/reports/casework-live-wiring/load-budgets.md
- .agents/evidence/casework-live-wiring/T11
- .agents/reports/casework-live-wiring/security-review.md
- .github/workflows/casework-live.yml
- apps/godspeed-casework-go/README.md
- apps/godspeed-cognitive-ui/README.md
- .agents/evidence/casework-live-wiring/T12/independent-confirmation.md
- .agents/evidence/casework-live-wiring/T10
- .agents/reports/casework-live-wiring/delivery-summary.md
- crates/sea-forge-ledger/src/types.rs
- .gitleaks.toml
- https://github.com/GodSpeedAI/SEA-Forge/pull/11
- crates/sea-forge-sandbox/src/lib.rs
- crates/sea-forge-cli/tests/self_model_cli.rs
- .gitignore
- .agents/WORKBENCH-SLICE-PROMPT.md
- .agents/specs/cognitive-environment/RUN_PLAN.md

**Spec:** .agents/specs/godspeed.casework-cognitive-environment-spec.yaml

**Ledger:** .agents/DEBT.md
