# Current status

**Status revision:** 213

**Stage:** Security: keep the claim lease after an ambiguous SWE_SEED declare failure (server path)

**Summary:** A declare failure after the SWE_SEED authority call has started no longer fails the attempt claim. The external authority may still append its declaration after an error, so the claim keeps its lease and reconciliation settles it after the lease lapses. Only failures before the call starts are settled at once, and an unavailable authority still releases the claim. Baseline: origin/main 22bf66a3 passes all unit and integration suites; the three doctest targets failed once under --no-fail-fast and passed on rerun.

**Verified:**
- origin/main 22bf66a3 cargo test --workspace: all unit and integration suites pass (0 failures); doctests pass on rerun
- sea-forge-server lib: 184 pass, 0 fail, including ambiguous_declare_tests (2)
- cargo fmt --check clean; cargo clippy -p sea-forge-server --all-targets -D warnings clean

**Limits:**
- The post-start path is verified by the decision helper's unit tests, not by an end-to-end server test with a fake authority transport; that test is still needed
- Operator recovery for a declaration that lands after its lease lapsed is unchanged (quarantine)

**Next:** Independent review of the branch, then merge; then SEA-Forge PR #15 policy dependency and the gateway batch.
