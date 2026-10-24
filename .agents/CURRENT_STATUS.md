# Current status

**Status revision:** 162

**Stage:** obligation-settlement (Horizon HORIZON-001 1.1.0 IS7)

**Summary:** SEA-Forge records an authority request's claims with its decision and settles an obligation from RealityTrace's own ledger: the new authority_closure verb reads RealityTrace's head (C) itself and stamps the closure with SEA-Forge's clock; settle routes an operation whose decision asserted a modality to settle_obligation, which checks the threshold against RealityTrace's record, admits runs in (B, C] started after the first execution evidence, and applies the supported/contradicted/inconsistent/unresolved table.

**Verified:**

- cargo test -p sea-forge-authority: all suites pass, including 18 new cep_obligation tests
- dropping the C bound from admissibility fails c_is_read_at_closure_so_a_run_appended_later_is_excluded
- cargo clippy -p sea-forge-authority -p sea-forge-server --tests -D warnings: clean; cargo fmt applied

**Limits:**

- Only the obligation modality settles; prohibition and permission are refused as unsupported
- The live end-to-end check (Horizon J02 revision 2 with a real RealityTrace) runs from Horizon after Cognate #65 lands
- sea-forge-server test binaries were not run locally (RAM); CI runs them

**Next move:** Merge after CI passes; then Horizon stage C3 drives question/threshold declare, claims, discrimination, verification, closure and settlement against a live cell. (Rebased onto main with the docs refresh and the L5 Design-click fix; status renumbered 159 -> 162.)

**Evidence:**

- `crates/sea-forge-authority/src/cep/obligation.rs`
- `crates/sea-forge-authority/tests/cep_obligation.rs`
