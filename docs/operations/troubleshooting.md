# Operations: Troubleshooting & Diagnostic Matrix

> **A structured symptom-cause-remediation guide for operators, developers, and on-call engineers diagnosing SEA Forge.**

---

## Diagnostic Matrix

| Symptom | Probable Root Cause | Investigation Steps | Immediate Remediation | Governed Invariant |
|---|---|---|---|---|
| **Server fails on startup: `AddrInUse`** | Another `sea-forge-server` is running or holds `<root>/server.sock.lock`. | Run `pgrep -fl sea-forge-server` or check `lsof server.sock.lock`. | Stop the existing server or specify a new cell root via `SEA_FORGE_ROOT`. Never delete lockfiles while processes run. | **OPS-01** |
| **Server fails on startup: `InvalidConfig`** | `server.yaml` syntax error or unparseable field. | Check `server.log` or stderr for exact line/column failure. | Fix YAML syntax. Server refuses to start with invalid configuration. | **OPS-01** |
| **Run exits with code 2** | Raw intent string did not match any recognized `IntentPattern`. | Inspect CLI stdout; verify `--intent` string against known patterns. | Use standard intent patterns (`"generate demo model"`) or supply an explicit `--plan <path>`. | Kernel Input |
| **Run exits with code 3** | Plan item attempted to write directly to a protected generated zone (`src/gen/`). | Inspect `plan.json` operations; look for paths beginning with `src/gen/`. | Remove direct manual write; generate code via spec pipeline instead. | **GEN-01** |
| **Settlement Rejected: `exit_zero` + `required_artifact_missing`** | Child exited 0 but failed to create declared output file (False Success). | Inspect `.sea-forge/runs/<id>/artifacts/` and child script logic. | Fix child script to ensure the required artifact is created at the exact relative path. | **DOM-01** |
| **Settlement Rejected: `jail_violation`** | Child attempted unauthorized filesystem write or network socket connection. | Read `.sea-forge/runs/<id>/artifacts/stderr.txt` for `EACCES` or `EPERM`. | Ensure workload writes relative to `workspace/`; grant TCP ports in policy if network is required. | **AUTH-01** |
| **SFWP command returns `server_busy`** | Admission waiting queue (8 slots) full; server under heavy load or deadlocked. | Check active runs via `sea-forge runs --unsettled` and inspect `server.log`. | Wait for in-flight tasks; kill hanging non-interactive children; increase `max_concurrent_runs` in `server.yaml`. | Server Admission |
| **Approval resolution rejected: `sod_violation`** | Proposer attempted to resolve an approval on its own item. | Check `plan_item.proposed_by` against the calling actor ID in `approvals.jsonl`. | The approval must be resolved by a distinct, authorized human actor. | **AUTH-04** (SoD) |
| **`sea-forge recall` fails with SQLite error** | Derived SQLite index (`sqlite.db`) is corrupted or desynchronized. | Check `.sea-forge/memory/sqlite.db` file size and permissions. | Run `sea-forge recall --rebuild --root <cell>` to replay append-only source records. | **DATA-01** |
| **Workbench displays `GovernedDenialSurface`** | One or more G1–G9 route guards failed (e.g. socket disconnected or jail missing). | Check the specific guard error message displayed on the denial card. | Remediate the specific environmental issue (e.g. launch cell server or bind identity). | **API-02** |
| **`just no-async-kernel` fails in CI** | A dependency in one of the 19 kernel crates transitively added `tokio` or `reqwest`. | Run `cargo tree -p <crate> -i tokio` to identify which crate introduced the dependency. | Remove async crate dependency or move the integration to `sea-forge-server` or `sea-forge-agent`. | **BUILD-01** |
| **`just workbench-contracts-gate` fails** | SFWP Rust types were changed without regenerating TS contracts or JSON schemas. | Run `git diff workbench/packages/contracts/`. | Run `cargo run -p sea-forge-server --bin gen_sfwp_schema` followed by `cd workbench && bun run generate:contracts`. | **API-01** |

---

## Diagnostic Checklists

### 1. The 30-Second Cell Health Check
When diagnosing an unresponsive or failing cell:
```sh
# 1. Verify socket existence and permissions (must be 0600)
ls -l $SEA_FORGE_ROOT/server.sock

# 2. Query system hello over SFWP
./target/debug/sea-forge ask concept "cell" --root $SEA_FORGE_ROOT

# 3. Check for unsettled or hanging runs
./target/debug/sea-forge runs --unsettled --root $SEA_FORGE_ROOT

# 4. Verify integrity ledger hash chains
./target/debug/sea-forge ledger verify main --root $SEA_FORGE_ROOT
```

### 2. Forensic Investigation of a Failed Run
To reconstruct what happened during a rejected or halted run:
1. Open `.sea-forge/runs/<run_id>/trace.jsonl` to find the exact timestamp and event where execution failed or halted.
2. Open `authority.json` to inspect the evaluated verdict, matched rule ID, and reason string.
3. Open `settlement.json` to view the non-empty `basis` tokens.
4. Read `artifacts/stderr.txt` to inspect runtime child process diagnostics.
