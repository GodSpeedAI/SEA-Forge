# Current status

**Status revision:** 210

**Stage:** Security: failed resume attempts settle their claim from the ledger

**Summary:** A resume that errors after claiming no longer leaves the claim open. The claim is released when no settlement declaration names it, completed when an authorised declaration does, and otherwise left open for reconciliation. Never declares.

**Verified:**
- cargo test -p sea-forge-authority --test cep_settlement: 52 pass (3 new: released, left open, completed)
- cargo test -p sea-forge-cli --test resume_recovery (6 pass) and --test conformance_m8_artifact (5 pass)
- cargo clippy -p sea-forge-authority -p sea-forge-cli --all-targets -D warnings: clean
- cargo fmt --all --check: clean

**Limits:**
- Server path (sea-forge-server delegation.rs settle_unrecorded_claim) still records declaration_not_made on non-unavailable errors without checking for an appended declaration; not changed here
- The completed branch is tested at authority level with a hand-built declaration, not through a CLI run
- sea-forge-server doctest fails on agent_probe.rs (pre-existing, unrelated)

**Next:** Commit and open PR for review; decide whether the server path gets the same fix.
