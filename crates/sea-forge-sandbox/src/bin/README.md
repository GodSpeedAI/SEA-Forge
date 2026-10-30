# sea-forge-sandbox helper binaries

## Overview

Trusted helper executables shipped by the `sea-forge-sandbox` crate. Each one is a
small, single-purpose process that a sandbox backend copies next to the workspace
and launches inside (or at the boundary of) an isolated workload. They are part of
the trusted computing base: each does exactly one thing and refuses to run where
its platform contract does not hold.

## Interface

- `sea-forge-jail-launcher` (`sea-forge-jail-launcher.rs:main`): macOS-only trusted
  launcher for the App Sandbox jail. Replaces itself with `argv[1..]` via
  `Command::exec`, keeping the process id so the caller's timeout and process-group
  handling act on the workload directly (`sea-forge-jail-launcher.rs:main`). Exits
  2 on usage error, 127 on exec failure, and 1 when built for a non-macOS target.
- `sea-forge-mcp-connect` (`sea-forge-mcp-connect.rs:main`): workload-side stdio
  relay to a SEA Forge MCP broker Unix socket. Moves bytes between stdin/stdout and
  the socket in two directions (one thread per direction, `UnixStream::connect`
  with the socket path from `argv[1]`). Untrusted by design: the broker
  authenticates the peer UID and authorizes every call (`sea-forge-mcp-connect.rs`
  module doc).
- `sea-forge-workload-exec` (`sea-forge-workload-exec.rs:main`): trusted in-namespace
  step of an isolated Linux workload launch. Invoked after `unshare` has switched to
  the workload's subordinate identity and made this helper pid 1 of a fresh pid
  namespace. Parses `--workspace`, `--artifacts`, `--read`, `--tcp-port`,
  `--socket-grant`, `--lifeline` flags and a trailing `-- argv...`
  (`sea-forge-workload-exec.rs:main`); applies the Landlock posture and the seccomp
  filter to itself before spawning the workload as its child.

## Invariants

- `sea-forge-jail-launcher` does exactly one thing: exec the target argv and keep
  the process id; it performs no other work (`sea-forge-jail-launcher.rs` module doc).
- `sea-forge-workload-exec` applies Landlock and the syscall filter to itself
  *before* running the workload, so both are in force before any workload code runs
  (`sea-forge-workload-exec.rs` module doc).
- With `--lifeline FD`, the helper stays as the namespace's init: it reaps every
  orphan and exits when the launcher's end of the lifeline pipe closes, after which
  the kernel kills every process in the pid namespace — no workload outlives its
  launcher (`sea-forge-workload-exec.rs` module doc).
- SEC-06 inherited-descriptor hygiene: on entry the helper issues one raw
  `close_range(3, u32::MAX, CLOSE_RANGE_CLOEXEC)` syscall so no descriptor opened
  before the restriction applied can leak into the workload across exec
  (`sea-forge-workload-exec.rs:main`).
- `--read` arguments must be absolute paths, else the helper exits 126
  (`sea-forge-workload-exec.rs:main`).

## Side Effects

- `sea-forge-workload-exec` performs one raw `SYS_close_range` syscall with
  `CLOSE_RANGE_CLOEXEC` (`#![allow(unsafe_code)]` is a stated choice;
  `sea-forge-workload-exec.rs:main`).
- `sea-forge-mcp-connect` opens a Unix domain socket connection to the path given
  as `argv[1]` and relays stdin/stdout over it (`sea-forge-mcp-connect.rs:main`).
- All three binaries write diagnostics to stderr and exit with distinct status
  codes on failure (2 usage / 126 flag or setup failure / 127 exec failure).

## Verification

- Run: `cargo test -p sea-forge-sandbox` (repo-wide: `just test`)
