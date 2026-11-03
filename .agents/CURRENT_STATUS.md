# Current status

**Status revision:** 209

**Stage:** Security: review follow-up on pending identity and echo check

**Summary:** Review follow-up: pending id now follows the bound request hash (fixes a concurrent duplicate-id race); the echo check compares acting_entity_id; a changed declaration after approval becomes its own pending request and leaves the approval unspent.

**Verified:**
- cargo test (authority, thoth, cli, server, capability, core; lib and tests): pass
- cargo clippy --all-targets -D warnings: clean
- cargo fmt --check: clean

**Limits:**
- sea-forge-server doctest fails on agent_probe.rs (pre-existing, unrelated)
- A claim whose resume errors after claiming stays open until the next resume settles it as failed (fail closed, stability only)

**Next:** Owner decision on the stability limitation, then merge PR 79.
