# Current status

**Status revision:** 164

**Stage:** obligation-settlement review hardening (Horizon HORIZON-001 1.1.0 IS7)

**Summary:** The discrimination run must also have completed no later than SEA-Forge's first execution evidence for the operation, at closure and at settlement. Review follow-up on obligation settlement: the discrimination run named in authority_closure must now be a completed, uncrashed run of exactly the pinned verifier argv, after the threshold, with a known non-zero exit, checked both before the closure is frozen and again at settlement; any other failing observation is refused. The RealityTrace adapter refuses a ledger export in which a row of the ledger lacks a numeric append_ordinal or a string record_kind instead of silently dropping it. SEA-Forge records an authority request's claims with its decision and settles an obligation from RealityTrace's own ledger: the new authority_closure verb reads RealityTrace's head (C) itself and stamps the closure with SEA-Forge's clock; settle routes an operation whose decision asserted a modality to settle_obligation, which checks the threshold against RealityTrace's record, admits runs in (B, C] started after the first execution evidence, and applies the supported/contradicted/inconsistent/unresolved table.

**Verified:**

- cep_obligation: 20 pass locally (a discrimination run completed after execution evidence is refused at closure); clippy -D warnings clean for sea-forge-authority.
- cep_obligation: discrimination runs of another command, crashed, or without a completion are refused at closure and nothing settles; an already-observing run is refused at closure (was: at settlement).
- realitytrace_source: a malformed row of the requested ledger fails the read; malformed rows of other ledgers are ignored.
- cargo test -p sea-forge-authority: all suites pass, including 18 new cep_obligation tests
- dropping the C bound from admissibility fails c_is_read_at_closure_so_a_run_appended_later_is_excluded
- cargo clippy -p sea-forge-authority -p sea-forge-server --tests -D warnings: clean; cargo fmt applied

**Limits:**

- Only the obligation modality settles; prohibition and permission are refused as unsupported
- The live end-to-end check (Horizon J02 revision 2 with a real RealityTrace) runs from Horizon after Cognate #65 lands
- sea-forge-server test binaries were not run locally (RAM); CI runs them
- The DomainForge write-authority engine is still a filename-stem approximation, not model-semantic authority (SUP-09h); readiness B-08 records it as a documented limitation

**Next move:** Merge once CI is green; then Horizon #5 (needs authority_closure and obligation settlement).

**Evidence:**

- `crates/sea-forge-authority/src/cep/obligation.rs`
- `crates/sea-forge-authority/tests/cep_obligation.rs`
