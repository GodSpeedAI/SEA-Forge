# sea-forge-sandbox

## Overview

SEA Forge sandbox backends and workspace path safety. This crate is the single
choke point through which every governed command execution reaches a process:
it defines the `ExecutionSandbox` abstraction, the sandbox classes (isolated
Linux jail, macOS App Sandbox jail, OpenShell projection containers, local
test backend), the network posture derived from authority grants, and the
path-safety primitives that keep client-supplied relative paths from escaping
the workspace and artifacts trees.

## Interface

- Crate contract lives in [./src/README.md](./src/README.md) (module tree,
  backends, isolation machinery, helper binaries).
- Key entry points, cited from source:
  - `ExecutionSandbox` trait and `SandboxSpec`/`SandboxHandle` —
    [src/lib.rs:ExecutionSandbox](./src/lib.rs)
  - `select_sandbox(class)` backend factory — `src/lib.rs:select_sandbox`
  - `SandboxClass` / `NetworkPosture` — `src/lib.rs:SandboxClass`,
    `src/lib.rs:NetworkPosture`
  - Path safety: `safe_join`, `safe_existing`, `safe_write`,
    `safe_write_within`, `validate_relative_path` — `src/lib.rs`
  - `OpenShellSandbox` and `OpenShellBinding` — `src/openshell.rs`
  - `IsolatedSandbox` — `src/isolated.rs:IsolatedSandbox`
  - `JailSandbox` (macOS) — `src/jail.rs:JailSandbox`
- Helper binaries: `sea-forge-jail-launcher`, `sea-forge-mcp-connect`,
  `sea-forge-workload-exec` — see [./src/bin/README.md](./src/bin/README.md).

## Invariants

- No command may be spawned outside a sandbox prepared by this crate; sandbox
  class and network posture come from the authority grant, never from
  child-controlled request input (consumers derive `NetworkPosture::from_granted_ports`
  from the grant — `src/lib.rs:NetworkPosture`).
- Unavailable isolation primitives fail closed with
  `workload_isolation_unavailable` instead of degrading to weaker isolation
  (`src/workload.rs:UNAVAILABLE`).
- Client-relative paths become host paths only through the `safe_*` functions,
  which reject traversal and symlinks that escape the root (`src/lib.rs`).

## Side Effects

- Spawns and reaps workload processes in local, jailed, or container
  environments (`src/local.rs`, `src/jail.rs`, `src/isolated.rs`,
  `src/openshell.rs`).
- Applies kernel isolation state: user namespaces, Landlock rulesets, seccomp
  filters, and macOS App Sandbox entitlements (`src/socket_policy.rs`,
  `src/syscall_filter.rs`, `src/workload.rs`).
- Reads and writes the run's workspace/artifacts trees and workload identity
  state on the host (`src/workload.rs`, `src/environment.rs`).

## Verification

- Run: `cargo test -p sea-forge-sandbox` (repo-wide: `just test`)
