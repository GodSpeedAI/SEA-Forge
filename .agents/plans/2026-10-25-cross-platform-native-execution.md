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

## Phase 2: OpenShell enforcement capability and workload identity

Branch `feat/openshell-capability`, stacked on `feat/cross-platform-native`.
Subsystem document: `docs/subsystems/openshell-execution.md` sections 3.1 and 3.2.

* **Driver and host.** `sandbox.json`'s `compute_driver` is read from the
  verified lock. `vm`: `/dev/kvm` on Linux, `aarch64` plus `kern.hv_support=1`
  on macOS, refused on Windows. `mxc`: Windows only. `docker`, `podman` and
  `kubernetes` have no enforcement table and are refused. `probe_host_virtualization`
  is replaced by `probe_local_driver`, and it still runs only when the gateway is
  local. The `local_microvm_gateway` flag now means "the gateway runs its
  compute driver on this host".
* **Enforcement capability, separate from policy proof.** A static table gives
  what each driver enforces (Landlock hard requirement, Linux process identity,
  egress proxy). The policy's demands are read from three `policy.yaml` fields.
  An unmet demand is `openshell_enforcement_unavailable` and nothing is created.
  The result is `enforcement` in `execution.json`.
* **Workload identity.** `sea_forge_core::workload_image::WorkloadImage` parses
  an OCI reference or `native:sha256:<hex>@<os>-<arch>`. Kind must match the
  driver. A native executable is hashed in `prepare` against its pin and the host
  platform, and `argv[0]` must be that executable. The wire `image` stays a string.
* **Limits recorded.** Native hashing is TOCTOU-limited (the gateway re-opens the
  file). OpenShell's binary-identity crate covers only Linux procfs. The `vm`
  row is upstream's claim and is not live-proven (live proof NOT RUN).
