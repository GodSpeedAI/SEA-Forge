# Supervisor verification report erratum

Date: 2026-10-05. This new immutable erratum corrects two evidence-description issues in `supervisor-clippy-independent-verification-oct05.md` (SHA-256 `78f51525631c74411fe30f82d35802119bb61cd29d2f1e2df759f731d650679d`). The original report is not edited.

## Test command feature flag

The root assignment requested the raw `sfwp_supervisor` integration test binary with locked dependencies, all features, and one test thread. The captured gate 2 command was `CARGO_BUILD_JOBS=1 cargo test -p sea-forge-server --test sfwp_supervisor --locked -- --test-threads=1`; it omitted the literal `--all-features` flag. Inspection of `crates/sea-forge-server/Cargo.toml` found no `[features]` section, so this package currently defines no optional features to activate. The omission is a syntactic deviation from the requested command; based on the manifest it does not change feature selection for this target. The captured command still ran the complete named integration test binary, all four tests passed, and exit was zero. Do not describe the command as an exact textual match to the assignment.

## Exit capture size

Each of `supervisor-clippy-gate1.exit`, `supervisor-clippy-gate2.exit`, and `supervisor-clippy-gate3.exit` contains `0\n`, which is **two bytes**, not one. Each has SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`; `wc -c` reported 2 bytes for each. The original report's “one-byte record” phrase is incorrect; it accurately describes the textual exit value and hash but not its byte length.

No command was rerun, no compiler token was used, and no source or existing evidence file was changed.
