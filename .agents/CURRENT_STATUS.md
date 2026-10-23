# Current status

**Status revision:** 155

**Stage:** domainforge-authority-path

**Summary:** The DomainForge write-authority approximation (evaluate_authority) now matches a model name only for a plain workspace-relative path: an absolute path, a `.` or `..` component, a backslash or an empty path is rejected before the stem match, so `../elsewhere/Sample.sea` no longer matches the `Sample` entity. Every remaining allow was already allowed, so this only narrows. Candidate resolution is deny-wins, so the DomainForge candidate never widened what local policy allows; the heuristic stays an approximation (SUP-09h) until domainforge-core's authority evaluator is wired, and no shipped policy bundle enables the engine.

**Verified:**

- cargo test -p sea-forge-authority --test conformance_m0_authority domainforge: escaping paths (../, absolute, ./, dir/../, backslash, empty) are denied and models/Sample.sea is still allowed; the test fails with the guard disabled (mutation-checked)
- cargo test -p sea-forge-authority (all targets), -p sea-forge-domainforge, -p sea-forge-cli --test lifecycle: all pass
- cargo clippy -p sea-forge-domainforge -p sea-forge-authority --all-targets -D warnings: clean

**Limits:**

- still a filename-stem approximation, not model-semantic authority (SUP-09h); readiness B-08 records it as a documented limitation

**Next move:** Replace the approximation with domainforge-core's authority evaluator when it exists; until then, do not enable engine: domainforge for production policy.

**Evidence:**

- crates/sea-forge-domainforge/src/lib.rs
- crates/sea-forge-authority/tests/conformance_m0_authority.rs

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
