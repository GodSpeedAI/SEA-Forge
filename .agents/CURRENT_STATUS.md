# Current status

**Status revision:** 173

**Stage:** settlement-pending error class and offline ledger crash recovery (ledger repair)

**Summary:** Settlement that is not yet decidable now returns its own error class cep_settlement_pending (CepAuthorityError::SettlementPending) at the two named sites (window still open; closure not yet recorded), instead of cep_evidence_refused. Nothing is written for either. Ledger crash recovery is tightened: quarantine only removes the bytes after the last complete newline-terminated record, at most one record; a corrupt record followed by more data, or a broken chain, is refused with 'damaged beyond an interrupted append' and nothing is truncated. New sea-forge ledger repair (offline, exclusive lock) quarantines the torn tail, rebuilds the MMR cache only from verified entries, then verifies. Verify stays read-only and prints the repair hint for crash residue only.

**Verified:**

- cargo test -p sea-forge-authority --lib --test cep_approvals --test cep_authority --test cep_compiler_upgrade --test cep_obligation --test cep_settlement --test cep_transition: 43+14+18+9+21+17+17 passed, 0 failed
- cargo test -p sea-forge-ledger --lib --test crash_residue --test conformance_m0_ledger --test projection_boundary: 37+23+6+3 passed, 0 failed (crash_residue 6 new; m0 incl. crash_recovery_quarantines_incomplete_tail and rebuilds_mmr_ahead_of_entries unchanged)
- cargo test -p sea-forge-cli --test ledger_repair --test conformance_m0_migrate --test conformance_m13 --test conformance_m3: 2+3+8+2 passed, 0 failed (ledger_repair 2 new; conformance_m3 tampered-ledger resume still fails closed)
- cargo test -p sea-forge-server --lib cep_authority; --test crash_recovery_startup --test conformance_cep_authority: 1+3+13 passed, 0 failed (startup_refuses_corruption_before_the_tail_and_cuts_no_history new)
- cargo clippy -p sea-forge-authority -p sea-forge-ledger -p sea-forge-cli -p sea-forge-server --all-targets -- -D warnings: exit 0
- cargo fmt --check -p sea-forge-authority -p sea-forge-ledger -p sea-forge-cli -p sea-forge-server: exit 0
- Existing startup and ledger-tamper tests reviewed: none restarts a server over a tampered middle entry; the conformance_m3 tamper case expects a failing resume and still gets one

**Limits:**

- The live release PR content is produced by the Release run on main after this merges; it has been checked here by the updater run and the locked resolution, not yet by CI on the regenerated PR
- Until RELEASE_PLEASE_TOKEN exists, a release PR opened by GITHUB_TOKEN gets its pull_request runs in an approval-required state; a maintainer approves them (carried from 167)
- Creating RELEASE_PLEASE_TOKEN is a maintainer credential action, not done here (carried from 167)
- Only the obligation modality settles; prohibition and permission are refused as unsupported (carried from 164)
- The DomainForge write-authority engine is still a filename-stem approximation, not model-semantic authority (SUP-09h); readiness B-08 records it as a documented limitation (carried from 164)
- A crate added later with versioned path dependencies must be added to the extra-files list, or the next release PR fails the same way
- The release-binaries job has not run on GitHub; it is unverified until a release run or a workflow_dispatch with tag v0.2.0 completes
- Binaries are built for x86_64-unknown-linux-gnu only; no other platform is shipped
- SHA256SUMS is produced in the same job as the tarball and proves integrity against corruption, not against a compromised release; no signing is in place
- Only sea-forge and sea-forge-server ship; sandbox helper binaries sea-forge-workload-exec and sea-forge-mcp-connect are internal
- obligation.rs record_closure 'window is still open; a closure is recorded once it closes' still returns cep_evidence_refused by decision (it concerns recording a closure, not settlement)
- ledger repair is offline by contract but does not check that the server is stopped; the operator must stop the server first
- startup now refuses a ledger whose chain is broken as well as mid-ledger corruption (fail closed); a crash-torn final record is still repaired silently at startup
- a complete final record missing only its newline is quarantined as one torn record

**Next move:** Merge the branch after CI is green (maintainer decision). Horizon should key retry on error_class cep_settlement_pending, and operators should use sea-forge ledger repair (with --root before the action word) for crash residue.

**Evidence:**

- `crates/sea-forge-authority/src/cep.rs`
- `crates/sea-forge-authority/src/cep/obligation.rs`
- `crates/sea-forge-authority/tests/cep_obligation.rs`
- `crates/sea-forge-server/src/cep_authority.rs`
- `crates/sea-forge-ledger/src/types.rs`
- `crates/sea-forge-ledger/tests/crash_residue.rs`
- `crates/sea-forge-cli/src/main.rs`
- `crates/sea-forge-cli/src/commands/ledger.rs`
- `crates/sea-forge-cli/tests/ledger_repair.rs`
- `crates/sea-forge-server/src/lib.rs`
- `crates/sea-forge-server/tests/crash_recovery_startup.rs`
- `docs/reference/cli-command-reference.md`
- `docs/operations/troubleshooting.md`
