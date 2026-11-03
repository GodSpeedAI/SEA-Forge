# Current status

**Status revision:** 186

**Stage:** Security: requester cannot resolve their own escalation, and the attempt is audited

**Summary:** resolve_approval refuses when the resolving actor is the escalation requester, before state checks and policy evaluation. The attempt is recorded as an approval_self_resolution_refused ledger record that references the escalation.

**Verified:**
- cargo test -p sea-forge-authority: 238 pass, 0 fail, 7 ignored
- cargo clippy -p sea-forge-authority --all-targets -D warnings: clean
- self-resolution test asserts the refusal, the pending state, and exactly one audit record referencing the escalation

**Limits:**
- Live SEA-Forge run of the self-approval path not yet performed
- Approval transition for settlement_declaration not yet in the production policy

**Next:** Add the settlement approval gate through a versioned spec amendment, then prove approve and reject against live authority records.

**Recorded at:** 2026-11-03T01:00:00Z
