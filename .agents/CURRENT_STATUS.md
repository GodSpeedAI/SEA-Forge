# Current status

**Status revision:** 65

**Stage:** Casework live wiring: T10 confirmed with conditions; T11 part A (packaging, metrics, correlation, backup/restore) done; load test, security review, CI pending

**Summary:** T11 part A adds systemd units and env examples, a config reference, a Prometheus text metrics package on a loopback-only listener (latency, SFWP errors, SSE clients, subscription lag), a live test tracing one correlation id across browser, gateway, kernel request and ledger, and tested offline cell backup/restore scripts with a runbook.

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

**Next:** T11 part B load test with budgets (N concurrent SSE clients plus intent burst; restart server under 50 SSE clients tooth), then /security-review of gateway and identity delegation with fixes, then CI wiring; then T12.

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

**Spec:** .agents/specs/godspeed.casework-cognitive-environment-spec.yaml

**Ledger:** .agents/DEBT.md
