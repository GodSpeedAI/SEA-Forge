# sea-forge-runtime — src

## Overview

Implementation of the `sea-forge-runtime` crate: SEA Forge command execution
runtime. This is the bridge that turns an authorized `ActionGrant` plus an
`ExecutionRequest` into an actual sandboxed process run: it derives the
sandbox class and network posture from the grant, authorizes the execution,
verifies executable identity, prepares/destroys the sandbox selected from
`sea-forge-sandbox`, and returns the `ExecutionResult`.

## Interface

- `execute(grant, request, run_id, workspace, artifacts) -> Result<ExecutionResult, ForgeError>`
  (`src/lib.rs:execute`): authorized execution of an `ExecuteCommand` /
  `ExecutePlanItem` request in the sandbox class named by the grant. Refuses
  `execute_sandbox_command` and openshell-class grants before the grant is
  inspected or consumed (`src/lib.rs:execute`).
- `execute_openshell(grant, request, run_id, workspace, artifacts, binding, result_dir)
  -> Result<ExecutionResult, ForgeError>` (`src/lib.rs:execute_openshell`):
  executes an authorized `ExecuteSandboxCommand` in an OpenShell sandbox defined
  by a DomainForge projection and copies the sandbox workspace back to
  `result_dir` for the changeset flow (`src/lib.rs:execute_openshell`).

## Invariants

- A sandbox command (`execute_sandbox_command`) is authorized only for an
  OpenShell sandbox, and the host path has no OpenShell backend; both are
  refused before the grant is inspected or consumed, so nothing is spawned on
  the host (`src/lib.rs:execute`).
- The network posture is derived from authority (the grant's `network`
  boundary via `NetworkPosture::from_granted_ports`) before the grant is
  consumed — never from child-controlled request input
  (`src/lib.rs:execute`).
- Decision-to-spawn executable identity: the decision-time executable identity
  captured from the grant is re-resolved (canonicalized) immediately before the
  spawn; a mismatch is refused, shrinking the decision→spawn window to a
  same-instant race (F-25.h, `src/lib.rs:execute`).
- OpenShell binding match: the request's cell, model hash, policy digest, and
  image must equal the projection identity in `binding`, and the grant's
  boundaries must each name exactly that single value; anything else is refused
  with `openshell_binding_mismatch` before a sandbox is created
  (`src/lib.rs:execute_openshell`).
- The OpenShell sandbox is destroyed on every path once it exists, and a failed
  destroy is never hidden behind the execution outcome — both failures are
  reported ("sandbox lifetime unverified"); there is no fallback to another
  sandbox class (`src/lib.rs:execute_openshell`).
- OpenShell execution always runs with `NetworkPosture::Denied`
  (`src/lib.rs:execute_openshell`).

## Side Effects

- Spawns the workload process inside a prepared sandbox (local, isolated, or
  OpenShell backend from `sea-forge-sandbox`) and destroys the sandbox after
  the run (`src/lib.rs:execute`, `src/lib.rs:execute_openshell`).
- Canonicalizes the requested executable path on the host filesystem before
  spawn (`src/lib.rs:execute`).
- Downloads (copies) the OpenShell sandbox workspace back to `result_dir`
  after a successful execution (`src/lib.rs:execute_openshell`).

## Verification

- Run: `cargo test -p sea-forge-runtime` (repo-wide: `just test`)
