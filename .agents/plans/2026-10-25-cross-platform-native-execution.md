# Cross-platform native execution — implementation record

Branch `feat/cross-platform-native` from `main` 4e67dfc (checkpoint tag
`checkpoint/pre-containers-20261025`). Assignment: `.tmp/containers.md`
(operator, 2026-10-25). Evidence: `~/projects/_wt/containers/` (recon, baseline,
CI logs) and the GitHub Actions runs named below.

## Ground truth (Phase 0)

| Question | Finding | Evidence |
|---|---|---|
| Toolchain / CI | 1.99.0 pinned, MSRV 1.92; CI = ubuntu lint/test/msrv + macos `just ci`; no Windows | `rust-toolchain.toml`, `.github/workflows/ci.yml:158` |
| Release assets | linux x86_64 tarball + SHA256SUMS + build provenance only | `release-please.yml:118-149` |
| Windows compile blockers (layer 1) | `sea-forge-changeset` (unix FileTypeExt/nlink), `sea-forge-sandbox` local.rs (`CommandExt::process_group`), authority test fixtures | Portability run 40251961705 |
| Windows compile blockers (later layers) | server/CLI Unix sockets, `SO_PEERCRED`, rustix unconditional in server | recon/sea-forge.md §4 |
| `unsafe` | zero `unsafe` blocks; workspace `unsafe_code = "deny"`, most crates `forbid` | `Cargo.toml:38` |
| OpenShell pin | SEA-Forge pins CLI `openshell 0.1.3`; every flag it uses is unchanged at v0.1.5 and HEAD c4688aa | recon/openshell-compat.md |
| OpenShell drivers | docker, podman, kubernetes, vm (Linux KVM / macOS HVF, Apple Silicon), mxc (Windows only, feature `compute-driver-mxc`) | gateway `lib.rs:73-125` |
| WSLc driver upstream | none; issue #3956 is macOS (closed, "not on roadmap") | recon/openshell.md §4 |
| MXC prerequisites | Win11 ≥ 26300.8553, `IsoSessionApp.dll`, `wxc-exec.exe` (`--features isolation_session`) | driver-mxc README:107-112 |
| This host | Win11 Pro 26300.9807; `IsoSessionApp.dll` present; `wxc-exec` absent (published in npm `@microsoft/mxc-sdk` 0.8.0, sha256 pinned); WSLc 5.0.1.1 present; no Windows Rust toolchain; WSL2 has no /dev/kvm | recon/hosts.md, recon/wslc-mxc.md |
| WSLc | Docker-like CLI; no Docker API; `--network none` blocks egress+DNS; **no `--cap-drop`, `--security-opt`, `--read-only`, `--pids-limit`**; default caps a80425fb, seccomp on, NoNewPrivs 0; ~9.5 s warm run | recon/wslc-mxc.md |
| OpenShell docker isolation model | workload `network=none`, `cap_drop ALL`, `no-new-privileges`; supervisor companion read-only, cap-less, host/bridge net | driver-docker `lib.rs:5150-5190, 6128-6140` |
| macOS | CI runner `macos-latest` (Apple Silicon VM); OpenShell VM driver CI uses `macos-15-xlarge`, no macOS VM e2e upstream | recon/openshell-compat.md §5 |
| Providers | `--no-auto-providers` disables auto-creation only; explicit `--provider` exists; credentials are placeholders substituted by the supervisor proxy | cli `main.rs:1531,1589`, `l7/mod.rs:150` |

## Decisions

1. **One `unsafe` home.** Windows FFI with no safe equivalent (named-pipe client
   token SID, pipe DACL, server-SID check, Job Objects) lives in one new
   Windows-only crate `sea-forge-os`; every other crate keeps `forbid(unsafe_code)`.
   Hard-link / reparse checks use the already-locked safe `winapi-util`.
   `windows-sys` 0.61 is already in `Cargo.lock` (tokio/mio): no new crate.
2. **Windows control plane = named pipes with OS-backed identity.** Pipe name is
   derived from the configured socket path; `first_pipe_instance` refuses a
   squatted name; remote clients rejected; DACL grants only the owner and SIDs
   named in `identity.bindings`. Peer identity comes from
   `ImpersonateNamedPipeClient` → thread token → `TokenUser` SID (kernel-backed,
   immune to PID reuse). Client verifies the server's SID. Unix keeps
   `SO_PEERCRED` unchanged.
3. **Config:** `identity.bindings[].sid` and `gateway.sid` added beside `uid`;
   on each OS only the native key authenticates; one principal ↔ one actor
   holds for both keys.
4. **Refuse, never downgrade.** Linux-only backends (Isolated, workload MCP
   sockets, postgres broker) stay Linux-gated and refuse elsewhere.
   `probe_host_virtualization` must stop returning Ok on non-Linux.
5. **WSLc:** OpenShell's container hardening (cap-drop ALL, no-new-privileges,
   read-only supervisor rootfs) cannot be expressed through WSLc 5.0.1.1's CLI.
   A WSLc path is only acceptable if OpenShell's own in-container runtime drops
   privileges itself; until proven, WSLc is reported Blocked, never Passing.

## Phase status

- Phase 0: done (this record, baseline in `~/projects/_wt/containers/baseline/BASELINE.md`).
- Phase 1: in progress.

## Phase 1 Windows limitations

Recorded from inspection on branch `feat/xplat-hardening` (stacked on
`feat/cross-platform-native`). Each item is weaker on Windows than on Linux or
macOS, or is unverified there. Line numbers refer to that branch.

- **Local sandbox containment.** The `Local` class on Windows is contained by a
  kill-on-close Job Object only. There is no filesystem, network or syscall
  containment. The Landlock and Seatbelt paths are Unix-only.
  `crates/sea-forge-sandbox/src/local.rs:121-141`.
- **Job Object limits.** Only `KILL_ON_JOB_CLOSE` is set. No memory, CPU,
  process-count or breakaway limits. `crates/sea-forge-os/src/imp.rs`
  (`Job::kill_on_close`).
- **Spawn flags.** `spawn_in_job` sets `CREATE_SUSPENDED` with
  `creation_flags`, which replaces any flags set earlier on the `Command`
  (std exposes no getter). All current call sites set none.
  `crates/sea-forge-os/src/imp.rs` (`spawn_in_job`).
- **Settlement and ACP trees.** The SWE_SEED tree and the ACP agent child now
  start suspended inside a job. Residual: a child that exits before assignment
  is reported as a spawn failure, not as an error with its own status.
  `crates/sea-forge-settlement/src/declaration.rs` (`ProcessTree::spawn`),
  `crates/sea-forge-agent/src/acp.rs` (Windows spawn block).
- **MicroVM and Postgres.** MicroVM virtualization is refused on Windows, so
  the isolated backends fail closed. `crates/sea-forge-sandbox/src/openshell.rs:191-197`.
  The Postgres broker refuses startup on Windows.
  `crates/sea-forge-server/src/lib.rs:1414`.
- **Pipe identity.** Only the SID is matched. `uid` bindings are ignored on
  Windows. `crates/sea-forge-server/src/identity.rs:458-465`. The pipe DACL is
  fixed when the server starts, so a binding change needs a restart.
  `crates/sea-forge-os/src/imp.rs` (`PipeSecurity::new`).
- **CLI response deadline.** The 120 s deadline is a helper thread, not a
  socket timeout. After a timeout the reader stays blocked on the pipe until the
  process exits. Unix sets a per-read timeout instead.
  `crates/sea-forge-cli/src/commands/agent.rs` (`exchange`, Windows).
- **Owner-only files.** `write_private` checks the file's DACL but not the
  parent directory's ACL, and it does not set an owner. `CREATE_NEW` refuses an
  existing entry, including a planted link. Ledger signing keys and the
  transcript seal key use it. `crates/sea-forge-os/src/imp.rs` (`write_private`),
  `crates/sea-forge-ledger/src/signing.rs`, `crates/sea-forge-server/src/transcript_seal.rs`.
- **Safe write.** `safe_write` opens with `FILE_FLAG_OPEN_REPARSE_POINT` and
  checks the reparse attribute after the open. A swap between the open and the
  check is not reachable through the no-follow open, but the check is not
  atomic. `crates/sea-forge-sandbox/src/lib.rs:399-420`.
- **Changeset scan.** Every reparse point (symlink, junction, mount point) is
  refused on Windows, as a `reparse_point` finding. Before this branch the scan
  ignored symlinks on Windows. Unix now flags symlinks at scan time as well.
  `crates/sea-forge-changeset/src/lib.rs` (`is_link_or_reparse`). Junction and
  symlink refusal depend on the CI runner allowing symlink creation; a missing
  privilege prints `SKIPPED`.
- **Unexercised on Windows.** The Unix socket publish path
  (`crates/sea-forge-server/src/lib.rs:1454-1462`), `SO_PEERCRED`
  (`crates/sea-forge-server/src/lib.rs:2467`), and the `observation.rs` Unix
  open path are compiled out on Windows and have no Windows test.
- **Build coverage.** Server and CLI compile and test only in the Portability
  workflow (`.github/workflows/portability.yml`). `ring` cannot be built for
  the Windows target from Linux, so these crates have no local Windows check.
