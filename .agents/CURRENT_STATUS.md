# Current status

**Status revision:** 176

**Stage:** Limit B-12: demonstration cell seeds end to end; unsafe-path refusal carries error_class; cell backup refuses links

**Summary:** scripts/seed-cell.sh now exits 0 against the server. Three causes were fixed. (1) The server's submit error path dropped error_class, so the 'unsafe workspace-relative path' refusal (already a unsafe_path_error class) was invisible to clients; commit_plan now returns forge_error_response for every typed submit refusal. (2) seed_approvals.py sent no request_id, sent absolute plan and policy paths, and sent operator_b with no uid and no delegation; every request now carries a fresh request_id, workspace-relative paths, and is sent as the gateway principal with on_behalf_of for operator_a and operator_b. The expected refusals are checked by error_class: operator_a self-approval is separation_of_duty, operator_b approval succeeds with resolved_by=operator_b. (3) seed-cell.sh no longer places sea-forge and sea-forge-cli symlinks inside the cell. The server tool link lives under the build target dir, outside the cell, and the script honours CARGO_TARGET_DIR. casework-cell-backup.sh refuses up front, before writing any archive, a cell that holds symlinks or hardlinked files, and lists them. Restore is unchanged.

**Verified:**

- cargo test -p sea-forge-server --test conformance_dispatch_remediation: 4 passed, 0 failed (new test absolute_plan_path_refusal_carries_error_class asserts error_class unsafe_path_error and no case_id)
- cargo test -p sea-forge-server with conformance_case_episode, conformance_lifecycle, conformance_identity, sfwp_delegated_identity, security_acceptance_v1, conformance_role_propagation: all pass (11, 4, 15, 10, 1, 4, 9 tests)
- cargo clippy -p sea-forge-server --all-targets -- -D warnings: exit 0; cargo fmt -p sea-forge-server --check: exit 0
- CARGO_TARGET_DIR=~/projects/_wt/sf-target scripts/seed-cell.sh ~/projects/_wt/b12-cell: exit 0; 0 symlinks in the cell; two cases seeded (one waiting, one resolved by operator_b)
- scripts/casework-cell-backup.sh on the seeded cell: exit 0, 103 files; scripts/casework-cell-restore.sh into an empty target: exit 0, 103 files verified; diff -r against the source tree is empty
- scripts/casework-cell-backup.sh on a copy with a planted symlink and a hardlinked file: exit 1, lists both, no archive written

**Limits:**

- The seeded cell's server.yaml makes the script's uid the gateway principal; operator_a and operator_b act only through on_behalf_of (the supported delegation path), not through their own uid
- The escalation policy's argv0 link lives in <target>/demo-tools/sea-forge, outside the cell; the recorded plan argv therefore names that path, which survives only while the target dir does
- The seed script's closing hint 'sea-forge case list' is stale (no such subcommand); not changed here
- The backup check runs under the cell lock after the live-server check; it does not scan the cell for other special file types (fifos, devices) beyond what restore already refuses

**Next move:** Orchestrator reviews branch fix/b12-seed-cell-and-backup; no push or PR made here. Confirm the gateway-based seeding is the intended demo identity model before merge.

**Evidence:**

- `crates/sea-forge-server/src/lib.rs`
- `crates/sea-forge-server/tests/conformance_dispatch_remediation.rs`
- `scripts/seed_approvals.py`
- `scripts/seed-cell.sh`
- `scripts/casework-cell-backup.sh`
- `docs/execution/OPERATIONS_AND_STARTUP.md`
- `apps/godspeed-casework-go/README.md`
