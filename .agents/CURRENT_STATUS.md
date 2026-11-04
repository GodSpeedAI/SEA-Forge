# Current status

**Status revision:** 214

**Stage:** Security: uncertain-claim recovery and operator resolution (PR 84); Horizon approval-gate engine verification (PR 15)

**Summary:** A claim whose lease has lapsed with no ledger declaration is no longer failed automatically. The authority call may have appended a declaration before the worker died, so the claim is held open and blocks every resume. An operator resolves it with resolve_uncertain_settlement: either completing it on a declaration found in the authority record, or attesting that none exists and releasing the claim for a fenced retry. Resolution is audited, requires the operator role, and is refused before the lease lapses. The Horizon approval-gate policy is checked against the SEA-Forge engine with the rendered fixture.

**Verified:**
- cargo test -p sea-forge-authority -p sea-forge-cli -p sea-forge-server -p sea-forge-artifact-ip --lib --tests: 83 suites ok, 0 failed
- cep_settlement: 73 pass including post-append completion, crash-then-operator resolution, concurrent reconcilers, concurrent operator resolutions, stale-worker fencing
- horizon_gateway_policy: 9 pass against the rendered Horizon policy fixture
- clippy -D warnings on sea-forge-authority and sea-forge-cli: clean
- origin/main 22bf66a3 workspace doctests: sea-forge-server failed under the multi-package selection (deterministic); passes alone

**Limits:**
- No end-to-end server test drives submit_swe_seed_declaration through a transport; the ambiguous case is tested at the authority layer
- No independent review yet; PR 84 must not merge until one is obtained
- CLI approve takes the actor id from --actor and hard-codes the operator role; self-approval is enforced on that string, so the CLI is not an authenticated identity path
- Operator resolution trusts the declaration reference the operator supplies; it is not checked against the external authority record

**Next:** Independent review of PR 84; server end-to-end test; CLI identity fix; then SPEC-CR-003.
