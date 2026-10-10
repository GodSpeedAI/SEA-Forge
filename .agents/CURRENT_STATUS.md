# Current status

**Status revision:** 62

**Stage:** Casework live wiring: T09 UI gate green; T10 live ladder L0-L7 implemented and green (single run); L8, L9, L-RECOV and 3-run confirmation pending

**Summary:** Plan T09 UI gate passes (typecheck, 347 UI tests, 11-journey fixture ladder). T10 live agent-browser ladder L0-L7 plus both teeth now run via just casework-e2e-live against fresh cells, with durable-file assertions; seven product defects found by the live ladder were fixed (gateway discretionary/evidence/lifecycle actions, kernel run-view lifecycle overlay, reopen reason, approval lifetime, stale justification, pill states). Not settled: independent confirmation, L8/L9/L-RECOV, three consecutive green fresh-cell runs.

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

**Limits:**

- Unsafe 608 history and old private casework/live-wiring branch cb90 remain local; do not publish them.
- Exact 552bd655ba53b2c46d15e0c61624a57c58143b5d publication request remains held; no push or push capture is claimed.
- Codec remains private and unwired. Startup, ledger reader, filter/ACK/frontier, public DTO/dispatch, and supported-writer integration remain outside this slice.
- Serialize compiler and heavy gates under root's process, RAM, and swap guard.
- Preserve user .jolli deletions and unrelated .gemini changes; do not alter old private history or scoped privacy correction.
- Startup key unit skipped the grant's RED-first step and ran gates outside root serialization; independent critic review is still required.
- T10 is one green run, not the preregistered three consecutive fresh-cell runs with independent confirmation; L8, L9, L-RECOV not implemented.
- Kernel does not resume an escalated item after approval over SFWP; L5 asserts the approval record and ledger, not downstream effects. The operator's denial is the absence of Approve plus a typed UNAUTHORIZED_ROLE refusal, not a panel-shown refusal.
- Live gateway emits no execution_progress and no typed settlement object; pill progress is never shown.
- Independent critic review of the startup key unit found no failures (Haiku critic); T09/T10 independent confirmation not done.

**Next:** Implement T10 phase 4 (L8 thoth.ask, L9 single-session run with empty errors/console, L-RECOV), then three consecutive green runs on fresh cells and independent confirmation; then T11 (packaging, load, security review, CI) and T12. Cursor v4 spec server work remains unimplemented and is not on the T10 path so far.

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

**Spec:** .agents/specs/godspeed.casework-cognitive-environment-spec.yaml

**Ledger:** .agents/DEBT.md
