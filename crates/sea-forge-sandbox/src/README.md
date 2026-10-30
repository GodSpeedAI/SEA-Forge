# sea-forge-sandbox — src

## Overview

Implementation of the `sea-forge-sandbox` crate: SEA Forge sandbox backends and
workspace path safety. This module tree defines the sandbox abstraction that every
command execution flows through (backend selection per sandbox class, network
posture, path-safety primitives), plus the concrete backends: local (test
shortcut), Linux isolated jail (user namespace + Landlock + seccomp), and
OpenShell (projection-defined container sandbox).

## Interface

- Core abstraction (`lib.rs`):
  - `SandboxClass` enum (`isolated` / `openshell` / `local` parseable via
    `FromStr`) — `lib.rs:SandboxClass`.
  - `NetworkPosture` enum with `from_granted_ports` / `granted_ports` —
    `lib.rs:NetworkPosture`.
  - `SandboxSpec`, `SandboxHandle`, `RelPath`, `SandboxError` — `lib.rs`.
  - `ExecutionSandbox` trait (`prepare` / `execute` / `destroy`) —
    `lib.rs:ExecutionSandbox`.
  - Backend selection: `select_sandbox(class)` — `lib.rs:select_sandbox`;
    `select_openshell(binding)` — `lib.rs:select_openshell`.
  - Path safety: `validate_relative_path`, `safe_lexical_join`, `safe_join`,
    `safe_existing`, `safe_write`, `safe_write_within`, `materialize` — `lib.rs`.
- Concrete backends:
  - `LocalSandbox` — `local.rs:LocalSandbox` (test/local-shortcut backend;
    timeout kill via `kill_timed_out_job` — `local.rs:kill_timed_out_job`).
  - `IsolatedSandbox` — `isolated.rs:IsolatedSandbox` (Linux-only jail backend,
    drives the helper binaries in [./bin/README.md](./bin/README.md)).
  - `JailSandbox`, `InteractiveJailChild`, `spawn_interactive`,
    `restrict_current_thread`, `WORKLOAD_SYSTEM_READ` — `jail.rs`
    (macOS App Sandbox jail).
  - `OpenShellSandbox` (`openshell.rs:OpenShellSandbox`) with `OpenShellBinding`,
    `ApprovedProjection`, `ComputeDriver`, `Enforcement`, `ProjectionIdentity`,
    and `projection_identity(dir)` — `openshell.rs`.
- Isolation machinery:
  - `workload.rs`: workload identity pool and confinement —
    `WorkloadIdentityPool`, `WorkloadIdentity`, `WorkloadConfinement`,
    `WORKLOAD_INNER_ID`, `isolated_command`, `spawn_interactive`,
    `validate_read_paths`, `validate_socket_grants`, `harden_state_tree`,
    `prepare_traversal`, `grant_exclusive_access`, `revoke_exclusive_access`,
    `Lifeline`, `sibling_binary`, `ensure_helper_reachable` — `workload.rs`.
  - `socket_policy.rs`: Landlock rulesets and a seccomp user-notify socket
    broker — `landlock_abi_version`, `apply` (returns `Applied`),
    `install_supervisor_filter`, `supervisor_installable`, `supervise` —
    `socket_policy.rs`.
  - `syscall_filter.rs`: `restrict_current_thread` (seccomp filter applied to
    the current thread) — `syscall_filter.rs:restrict_current_thread`.
  - `observation.rs`: symlink-safe opening of observation roots/files —
    `open_observation_root`, `open_observation_file` — `observation.rs`.
- Environment: `EnvironmentSpec`, `load_pinned_environment`,
  `store_builtin_environment`, `demo_environment`, `materialize_base`,
  `parse_env_ref`, `parse_evaluator_score` — `environment.rs`.
- Artifact output: `artifact_output.rs` (stdout/stderr capture layout).
- Helper binaries live in [./bin/README.md](./bin/README.md).

## Invariants

- Path-safety functions reject path traversal and unsafe components before any
  filesystem access: `validate_relative_path` / `safe_join` /
  `safe_lexical_join` / `safe_existing` in `lib.rs` are the only sanctioned way
  to turn a client-relative path into a host path.
- Isolated execution may read only the bound executable's own directory tree
  beyond the system set, and never one containing the run (`lib.rs:execute`
  caller contract in `sea-forge-runtime`; enforced via
  `workload.rs:validate_read_paths`).
- OpenShell sandboxes bind network access to binary identity inside the sandbox,
  so no host executable identity is checked for `execute_sandbox_command`
  (`openshell.rs` module contract, see `sea-forge-runtime:execute_openshell`).
- `OpenShellSandbox` must run only a projection matching its approved identity
  (cell/model/policy digest), enforced with `ApprovedProjection`
  (`openshell.rs:ApprovedProjection`).
- Workload isolation is fail-closed: unavailable kernel facilities surface
  `SandboxError` with class `workload_isolation_unavailable`
  (`workload.rs:UNAVAILABLE`, `socket_policy.rs:UNAVAILABLE`) rather than a
  fallback to weaker isolation.
- The sandbox lifetime is caller-owned: every `prepare` must be matched by
  `destroy` on every path (`lib.rs:ExecutionSandbox`).

## Side Effects

- Spawns child processes: local commands (`local.rs`), unshare/jail helper
  chains (`workload.rs:isolated_command`, `jail.rs:spawn_interactive`), and
  OpenShell container runtime invocations (`openshell.rs:run`).
- Kernel security state: applies Landlock rulesets and seccomp filters to
  threads/processes (`socket_policy.rs:apply`,
  `socket_policy.rs:install_supervisor_filter`,
  `socket_policy.rs:supervise`, `syscall_filter.rs:restrict_current_thread`,
  `jail.rs:restrict_current_thread`).
- Disk access: writes sandbox workspace/artifacts trees, copies projection
  snapshots, and reads workload state (`/proc`, subuid/subgid delegation files)
  (`workload.rs`, `openshell.rs:snapshot_projection`).
- UID/GID management: allocates subordinate workload identities via
  newuidmap/newgidmap and reaps them (`workload.rs:WorkloadIdentityPool`,
  `workload.rs:delegation`).
- Socket mediation: the seccomp-notify broker intercepts and authorizes socket
  syscalls of the workload, denying anything not covered by grants
  (`socket_policy.rs:supervise`, broker_* functions).

## Verification

- Run: `cargo test -p sea-forge-sandbox` (repo-wide: `just test`)
