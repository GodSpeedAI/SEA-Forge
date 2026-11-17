# sea-forge-runtime

## Overview

SEA Forge command execution runtime. The crate owns the single seam where an
authorized `ActionGrant` becomes a real, sandboxed process: it consumes the
grant, re-verifies executable identity at spawn time, selects and prepares the
sandbox from `sea-forge-sandbox`, and reports an `ExecutionResult`. Two
entrypoints exist: host-command execution (`execute`) and OpenShell
projection-container execution with workspace download (`execute_openshell`).

## Interface

- Crate contract lives in [./src/README.md](./src/README.md).
- `execute(grant, request, run_id, workspace, artifacts)` —
  `src/lib.rs:execute` (host command execution in the grant's sandbox class).
- `execute_openshell(grant, request, run_id, workspace, artifacts, binding,
  result_dir)` — `src/lib.rs:execute_openshell` (OpenShell-only sandbox
  command execution with workspace copy-back).

## Invariants

- Process exit is not success: execution results carry
  `ExecutionStatus::Completed` / `TimedOut` / `SpawnFailed` and the caller
  settles on them; timeouts kill and reap the child
  (`src/lib.rs` tests, `sea-forge-core::types::ExecutionStatus`).
- `execute_sandbox_command` and openshell-class grants never reach a host
  spawn through `execute` — refused before grant consumption
  (`src/lib.rs:execute`).
- Sandbox class, network posture, and executable identity all come from the
  authority grant, never from child-controlled request input
  (`src/lib.rs:execute`).

## Side Effects

- Spawns and reaps child processes inside sandboxes
  (`src/lib.rs:execute`, `src/lib.rs:execute_openshell`).
- Writes execution stdout/stderr into the artifacts tree (via the sandbox
  backend, `src/lib.rs:execute`) and copies the OpenShell workspace back to
  `result_dir` (`src/lib.rs:execute_openshell`).

## Verification

- Run: `cargo test -p sea-forge-runtime` (repo-wide: `just test`)
