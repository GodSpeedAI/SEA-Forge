# T12 independent confirmation

Verdict: CONFIRM WITH CONDITIONS (target_settlement met as worded for the dev-auth live harness; conditions below).

Clone: /home/sprime01/projects/sea-rs-t12-fresh (git clone --local, HEAD e284e26, status clean before gates; ~19 GB incl. 18 GB target/).
Host: 6 CPU, 7.9 GB, WSL2. Logs: this directory (gate-*.log). Clone run evidence: <clone>/.agents/evidence/casework-live-wiring/T10/run-2026-10-10T23-34-05-442Z, run-...23-49-08-874Z, run-...2026-10-11T00-04-45-225Z (each with results.json, durable-delta.jsonl, cell-durable/ snapshot of the cell files) and T11/run-20261011T002224Z.

## Gate table

| Gate | Command (in clone) | Exit | Duration | Counts |
|---|---|---|---|---|
| UI | bun install --frozen-lockfile; bun run typecheck; bun test | 0 | 38s | 373 pass, 0 fail, 35 files |
| Fixture ladder | just casework-ui-up; bun e2e/run.ts; just casework-ui-down | 0 | 1410s | J0-J9 + RECOVERY, 11/11 PASS |
| Go default | go vet ./...; go test -p 1 -count=1 ./... | 0 | 83s | 12 pkgs ok |
| Go live | go test -p 1 -count=1 -tags live ./... | 0 | 344s | 12 pkgs ok; only skip is TestGoldenCapture (env-gated recorder) |
| Rust | cargo test --workspace --all-features --locked --no-fail-fast (cold, CARGO_BUILD_JOBS=3) = `just test` | 0 | 454s | 125 suites, 1158 pass, 0 fail, 4 ignored (self_invoke_noop_pass/fail; t16_1 and t16_6 real-ACP release gates needing env) |
| Live ladder run 1 | just casework-e2e-live | 0 | 892s | 11/11 journeys PASS, 70 steps; cell /tmp/t10-live-eukSX5 |
| Live ladder run 2 | same | 0 | 937s | 11/11, 70 steps; cell /tmp/t10-live-y0xujn |
| Live ladder run 3 | same | 0 | 911s | 11/11, 70 steps; cell /tmp/t10-live-H0uvoE |
| Tooth stub-gateway | --tooth stub-gateway --skip-build | 0 | 60s | TOOTH PASS: L1 failed on durable assertion, no new case dir |
| Tooth shared-session | --tooth shared-session --skip-build | 0 | 4s | TOOTH PASS: L5 guard "same agent-browser session" |
| Tooth shared-cookie | --tooth shared-cookie --skip-build | 0 | 58s | TOOTH PASS: same casework_session cookie and both operator_local |
| Tooth console-error | --tooth console-error --skip-build | 0 | 6s | TOOTH PASS: strict console/errors check caught injected errors |
| Load | just casework-load | 0 | 112s | intent p95 8.0 s (budget 20 s), event lag p95 7.75 s (budget 40 s), 0 errors; kernel kill -9 and gateway kill -9 under 50 SSE: 50/50 resynced, bounded RSS (30/47 MB) |

Not run: `just check` (fmt, clippy -D warnings, deny, gitleaks), `just proof`, `just status-check`, workbench gates, CI on GitHub, real systemd. These are outside the T12 gate but several CURRENT_STATUS claims rest on them.

## Criteria

1. No production path loads LocalContractAdapter/fakeauthority/Northstar: PASS.
   - Fresh `NODE_ENV=production vite build` into a separate outDir: 0 hits for LocalContractAdapter, fakeauthority, Northstar (any case), createLocalAgent, failArtifact, corruptArtifact, adapters/local. Each live run also records bundle-scan (13 JS assets, disk + HTTP) with violations [].
   - src/main.tsx imports the local adapter only via dynamic import under `import.meta.env.DEV && requestedSource !== 'live'`; production builds always take the HttpCaseworkAdapter branch.
   - Go build with no tag: internal/coordinator and internal/artifactstore are fully excluded, binary has 0 "northstar" strings, and `-serve` with adapter=fixture is refused at config validation ("fixture adapter selection is refused by this build"), exit 2.
   - Caveat: fixtures and the local adapter still exist in the repo for tests/dev (as guardrails require).
2. L0-L9 + L-RECOV pass 3x on fresh cells with durable proof: PASS (with the CW-44 deviation).
   - Three consecutive runs, distinct mkdtemp cells and distinct case ids, all 11 journeys PASS, 70 steps each. Every journey has 2-8 durable-delta entries (41 per run).
   - Sampled and independently re-derived from the retained cell-durable snapshots (all 3 runs): approvals.jsonl has pending then approved with resolved_by rso_local; ledger has approval_request, approval_resolution_authority_request, authority_decision, approval_resolution; case-events.jsonl has case_closed (case_engine), case_reopened / case_terminated / human_task_completed (operator_local); L6 artifact bytes in the cell hash to the sha256 the dock displayed (run 1 verified directly: 4fed4f84...); operator denial left approvals.jsonl unchanged; L9 and L-RECOV strict console/errors counts 0.
   - Harness cells are deleted after each run; the retained evidence is the harness-copied cell-durable snapshot, so I could not reread live cell files after the fact.
   - Not met as worded in the plan: no video, no trace (CW-44 / T10-DEV-6, host ffmpeg and trace-stop failures). HAR exists for L6 only.
3. Two users with distinct roles ledgered distinctly, SoD holds: PARTIAL-to-PASS.
   - operator_local (operator) and rso_local (R-SO) in separate agent-browser sessions with different cookie digests; approvals.jsonl and ledger attribute the resolution to rso_local, case actions to operator_local. Kernel SoD is covered by Rust tests (conformance_identity.rs separation_of_duty, reconnect-laundering test; case_ops requester tests), which passed in the Rust gate.
   - Caveats: authentication in the live ladder is auth.mode=dev, production:false (any password), not OIDC/argon2; the operator's "denial" in the UI is a gateway-level typed UNAUTHORIZED_ROLE refusal (action not offered), not a kernel refusal (T10-DEV-3); the e2e policy grant (`allow-e2e-approval-resolution`) is applied by the harness; L7 then has operator_local complete the sign-off human task by hand because of kernel gap CW-45 (an approval never resumes the case). Production auth is covered by Go unit/live tests only, not by a browser ladder.
4. Errata section 0 / decision log consistent with code: PASS with notes.
   - Journeys use only real TraceKind names (item_enabled, item_activated, item_completed, item_failed, item_terminated, human_task_completed, settlement_recorded, case_created, case_closed, case_reopened, case_terminated, plan_mutated), all present in crates/sea-forge-core/src/types.rs TraceKind; grep finds no item_started or approval_decided in journeys-live or live.
   - Decision log T10-DEV-1..7 match observed behavior (no PlanCreated on commit, write-only episodes, gateway-level denial, no resume, dev harness, no video, no self-heal).
   - Gap: the decision log has no T11 entry (one mention of T11), though T11 added units, metrics, security fixes and CI.

## Discrepancies, claimed vs observed (.agents/CURRENT_STATUS.md)

- Stale/self-contradicting: "T10-T12 unstarted" and "T11 not complete: load test, /security-review, CI pending" sit beside "T11 complete in code"; "bun test 347 pass" (now 373). The file should be rewritten.
- Reproduced: UI 373 pass; fixture ladder 11/11; 3 green live runs (now at e284e26 rather than 447d9a9); all 4 teeth; load budgets; go test -p 1 default and -tags live; cargo test all green.
- Not reproduced / not attempted: `just check` and clippy sliced_string_as_bytes repair claims; GREEN03/GREEN04 codec and Graft02 captures; the 449/0/2 full-server count (I ran the whole workspace, 1158/0/4, no per-crate breakdown); systemd-analyze verify; CW-53 GitHub workflow; "negative controls" for the load test (not rerun); mutation checks of the Rust SoD test (not rerun).
- "T09 remains partial" and "T09/T10 independent confirmation not done": T10 now has this third-run reconfirmation at e284e26; T09 and T11 have no independent confirmation that I performed. I did not assess T09 contract-extension scope.

## Open items (independent list)

- DEBT CW-43 (shared signature decoder aliases; private codec unwired), CW-44 (harness is not a production cell, no video/trace), CW-45 (kernel does not resume a case after approval; L7 completes by hand), CW-46 (UI stays Reconnecting after gateway restart, artifact errors cached until reload; asserted as-is by L-RECOV), CW-47 (login lockout surface), CW-48 (502 bodies echo kernel text), CW-49 (OIDC state not browser-bound, no PKCE; low/medium), CW-50 (header/log hygiene), CW-51 (kernel does not bound reopen/terminate reasons), CW-52 (kernel unit less hardened), CW-53 (CI workflow never run on GitHub), CW-54 (load budgets uncalibrated for runners).
- Live gateway emits no execution_progress; no typed settlement object; pill progress never shown.
- Security review: F1 high was fixed; no open high findings per the document (I did not repeat the review; regression tests ran in the Go gate).
- Load budgets are single-host, <=50 clients, no soak (admitted in load-budgets.md); kernel throughput ~4 intents/s.

## Conditions for CONFIRM

1. Record CW-44/CW-45/CW-46 and dev-auth as accepted limits of T12, and do not describe the result as production-auth or video/trace evidence.
2. Fix .agents/CURRENT_STATUS.md contradictions (T10-T12 "unstarted", T11 "not complete", stale counts) and add a T11 section to the decision log.
3. T09 and T11 still lack their own independent confirmation; run `just check` before any merge or publication (not run here).
4. CW-53: one manual dispatch of casework-live.yml before relying on CI.

## Housekeeping

All gates run inside the clone; no repo file outside this directory was changed; no commits, pushes or git config changes. `agent-browser close --all` reported no active sessions; stack processes exited with their runs. 14 harness temp cells remain under /tmp/t10-live-* (includes teeth runs and earlier agents' runs). Unrelated chrome-devtools-mcp processes belong to the host, not this run. Run-time variance: L-RECOV ~340-357 s each; whole live run ~15 min.
