# Current status

**Status revision:** 415

**Stage:** CognitiveOS #108 integration: #116 and #117 integrated

**Summary:** Integrated #116 (execution-profile approval standing, CEP validators in hosted CI) and #117 (owner-bound replay, status audit, gate-first settlement, minimal settlement receipt) into the SEA integration branch; merged-tree gates pass locally, hosted CI pending.

**Verified:**

- #116 merged with no code conflicts (approval standing carries execution_profile_digest; governed requests fail closed). #117 merged with one test-file conflict, resolved by keeping both appended test groups; crates/sea-forge-authority/src/cep.rs auto-merged and both behaviours hold (authority tests 33 in cep_authority incl. the execution-profile validator, 6 execution-profile approval tests, 85 settlement tests).
- Merged-tree gates: workspace build, authority suite with CEP_REPO=gsr-heads/cep and REQUIRE_CEP=1 (every binary green, 21 test results ok, none skipped), server lib 208 plus 1 ignored, conformance_cep_authority 27, sfwp 26, case_authoring 21, lifecycle 10, FDE scenarios 49 with FDE_WP21A_FINAL=1, ledger tests, clippy --workspace --all-targets -D warnings, fmt, 12 contract digests, workbench contracts gate.
- ADR-016 (minimal settlement receipt) and ADR-017 (execution profile boundary) do not collide.

**Limits:**

- Hosted CI has not yet run on the integration branch; no claim of full CI or proof gates.
- Endpoint identity remains unverified: the SEA trusted-launcher API authenticates the caller to SEA and is not wired into the Cognate receipt transport; deployed reverse endpoint trust and authenticated receipt acceptance need an owner decision (#117 deployment authority).
- Independent hardening review history: the external review of the internal diff was declined by the owner; later reviews were fresh non-interactive Sonnet sessions without builder context, and a Sol pass was refused by the provider safeguard, so no fully independent reviewer has qualified the receipt, ownership or replay work.
- Power-loss durability of the new claim ledger and non-Unix behaviour (Windows named pipes) are unverified; the new socket tests are Unix-only. Ledger chain and MMR are unsigned local files and the symlink check is check-then-open.
- Request ids are cell-global (open debt in OBSERVED_DEBT.md).
- Cognate receipt qualification (9dafdf6) and the Workbench checkpoints from revision 413 (7f1a726 payment preview needs an upstream runtime and authority contract, the ticket 110 Team restoration conflict, the unqualified cleanup checkpoint, the failed full Workbench gate and capacity-timeout shard, opt-in sandbox and live-settlement fixtures) remain outstanding; Horizon full gate passed on 14be523 but cross-repository acceptance is outstanding.
- A cold build after the host crash corrupted incremental state for the authority test binaries (mold segfault, dep-graph memory-map error); cargo clean -p sea-forge-authority fixed it and nothing else was cleaned.

**Next:** Observe hosted CI on the pushed SHA and prove the cep_* tests ran under REQUIRE_CEP=1; then resolve the owner decisions above.

**Evidence:**

- crates/sea-forge-authority/src/cep.rs
- crates/sea-forge-server/tests/conformance_cep_authority.rs
- docs/decisions/ADR-016-minimal-settlement-receipt.md
- docs/decisions/ADR-017-execution-profile-boundary.md
- .agents/OBSERVED_DEBT.md
