# How-To: Troubleshoot Cell Failures & Operational Errors

> **A diagnostic guide for operators and developers resolving common runtime errors, lock conflicts, and sandbox denials in SEA Forge.**

---

## 1. Cell Socket Lock Conflicts (`AddrInUse`)

### Symptom
When starting `sea-forge-server`, the process immediately crashes with:
```text
[ERROR] failed to bind cell socket: Address already in use (os error 98)
[ERROR] another server owns lock: /tmp/cell/server.sock.lock
```

### Root Cause
SEA Forge strictly enforces that **only one server process may own a cell root at a time**. The lockfile `<root>/server.sock.lock` is held using an exclusive advisory `flock(2)` to prevent dual-server ledger corruption.

### Diagnosis
Check whether another server process is running on this cell:
```sh
# Linux / macOS
lsof /tmp/cell/server.sock.lock
# Or check running processes
pgrep -fl sea-forge-server
```

### Remediation
1. If an active server is running, use the existing server or specify a different cell root via `export SEA_FORGE_ROOT=/tmp/another-cell`.
2. If the previous server crashed and left a stale socket file, verify that no process holds the lock. If no process owns it, the server's staging-rename logic will automatically recover on next start. Never manually delete the lockfile while a process is actively running.

---

## 2. Landlock Sandbox Denials (`jail_violation`)

### Symptom
A run completes with settlement status `Rejected`, and `settlement.json` reports:
```json
{
  "status": "rejected",
  "basis": ["authority_allow", "jail_violation"]
}
```

### Root Cause
The workload process attempted an operation forbidden by the Linux Landlock LSM sandbox:
* Attempted to write outside `<run_dir>/workspace` or `<run_dir>/artifacts`.
* Attempted to open an outbound TCP connection while `NetworkPosture::Denied` was active.

### Diagnosis
Inspect the captured standard error stream:
```sh
cat .sea-forge/runs/<run_id>/artifacts/stderr.txt
```
Look for `Permission denied (os error 13)` or `Operation not permitted (os error 1)`.

### Remediation
1. **If the action was malicious or buggy:** The sandbox functioned correctly; investigate why the child attempted unauthorized mutations.
2. **If the action was legitimate:**
   * If writing files: Ensure the task writes relative to `.` or into its designated workspace.
   * If network access is required: Update the active policy bundle to explicitly grant the necessary TCP ports under the rule's `boundaries.network.ports` list.

---

## 3. Server Busy Overload (`server_busy`)

### Symptom
An SFWP command (such as `case.commit`) returns the error:
```json
{
  "error": "server_busy",
  "message": "request admission queue capacity exceeded (8 waiters)"
}
```

### Root Cause
The server's request admission semaphore protects cell execution from unbounded thread spawning. If `max_concurrent_runs` tasks are executing and 8 additional protected requests are already queued in the waiting room, all further mutation requests fail closed immediately.

### Diagnosis
Check how many cases are currently running:
```sh
./target/debug/sea-forge runs --unsettled --root $SEA_FORGE_ROOT
```

### Remediation
* Wait for in-flight tasks to complete.
* If tasks are deadlocked or hanging, check if a child process exceeded its timeout or is waiting for interactive stdin (child processes must be non-interactive).
* If your workload legitimately requires higher concurrency, increase `max_concurrent_runs` in `$SEA_FORGE_ROOT/server.yaml`.

---

## 4. Corrupted or Desynchronized SQLite Memory

### Symptom
Executing `sea-forge recall` returns:
```text
[ERROR] sqlite error: database disk image is malformed
```
Or search returns outdated results after manually inspecting `capabilities.jsonl`.

### Root Cause
The SQLite database (`.sea-forge/memory/sqlite.db`) is a disposable projection, not source truth (Invariant **DATA-01**). An unclean power interruption or disk full event can corrupt the SQLite FTS index.

### Remediation
Run the rebuild command to replay the append-only journals into a fresh SQLite database:
```sh
./target/debug/sea-forge recall --rebuild --root $SEA_FORGE_ROOT
```
The command scans `.sea-forge/capabilities.jsonl` and `.sea-forge/memory/items.jsonl`, regenerating the SQLite FTS index to byte-stable accuracy.

---

## 5. False Success Rejection (`required_artifact_missing`)

### Symptom
A command exited with code 0, but the run failed with exit code 1:
```text
Settlement: Rejected
Basis: authority_allow, exit_zero, required_artifact_missing:model.sea
```

### Root Cause
The child process terminated with exit code 0, but failed to create the file promised by the plan item's `SettlementCriteria` (Invariant **DOM-01**).

### Diagnosis
Inspect the workspace to see what the child actually produced:
```sh
ls -la .sea-forge/runs/<run_id>/workspace/
```

### Remediation
* Check if the child wrote the file with an incorrect name or to an unexpected subpath (e.g. `Model.sea` instead of `model.sea`).
* Fix the child script to ensure the required artifact is materialized at the expected relative path before exiting 0.
