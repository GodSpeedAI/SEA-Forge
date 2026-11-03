# Current status

**Status revision:** 191

**Stage:** Security: durable pending state for an approval-gated external settlement

**Summary:** An approval-gated settlement declaration now writes a durable settlement_pending record to the authority ledger before returning a pending outcome. The record is idempotent by the SHA-256 of the canonical request, so an identical request returns the same record. The CLI resume path and the SWE_SEED delegation path treat a pending outcome as no declaration: nothing is sent to the authority and no declaration is appended.

**Verified:**
- cargo test -p sea-forge-authority: all suites pass (cep_settlement: 31 pass, including durable reload, duplicate request, and no declaration while pending)
- cargo test -p sea-forge-server: 550 pass, 0 fail
- cargo test -p sea-forge-cli --no-fail-fast: 87 pass, 2 fail (the two expected conformance_m8 CLI tests, not rewritten)
- cargo clippy -p sea-forge-authority -p sea-forge-artifact-ip -p sea-forge-server -p sea-forge-cli --all-targets -- -D warnings: clean
- cargo fmt --all -- --check: clean

**Limits:**
- Nothing resolves a settlement_pending record yet: no approval, rejection, expiry, or use transition is recorded, so a pending settlement cannot be completed
- conformance_m8_cli_capitalize_commits_one_pending_and_approval_then_exits_5 is refused by policy for the Operator identity (no external settlement rule for that role), before any pending record is written
- conformance_m8_cli_parks_external_gate_profiles_without_fabricating_records exits 1 where it expects 5
- Two identical requests racing before either record is committed can return a ledger integrity error instead of the pending id
- The resume state machine, approval validation, and reconciliation are not implemented

**Next:** Implement approval resolution and use for settlement_pending records (approve, reject, expire, consume once), then bind the resumed declaration to a validated approval.
