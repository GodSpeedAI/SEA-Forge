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
- Phase 1: done (Windows control plane; portability run 40260946741).
- Phase 2: done (enforcement capability, workload identity).
- Phase 3: Linux live proof passing (6/6, four runs); macOS Blocked (no nested virt on hosted runners).
- Phase 4: MXC probed; see Phase 4 section.
- Phase 5: WSLc Blocked.
- Phase 6: done.
- H4 packaging: done (package.yml, run 40257119484).

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
  row is upstream's claim and is not live-proven (live proof: see Phase 3).

## Phase 6 — macOS App Sandbox jail

- Scope: implement the existing `Jail` sandbox class on macOS with the App Sandbox, not a new class. No `sandbox-exec`, SBPL or `sandbox_init`.
- Mechanism: `sea-forge-jail-launcher` (a new bin in `sea-forge-sandbox`, which only execs its argv) is copied per execution into a fresh directory beside the artifacts root, ad-hoc signed with workspace/artifacts read-write temporary exceptions and no network entitlement, verified for `com.apple.security.app-sandbox`, then run with `env_clear`, the same timeout and process-group kill as the local path.
- Refused with `jail_unavailable`: `NetworkPosture::AllowTcpPorts` (no per-port control), a missing `codesign` or launcher, an unsigned or unverified launcher, and a launcher copy inside the workspace.
- Evidence: probe run 40254192454 on macOS 26.6.2 arm64 (`probes/macos-app-sandbox` on `ci/macos-app-sandbox-probe`).
- Verification: `.github/workflows/portability.yml` job `macos` runs `tests/macos_app_sandbox_jail.rs` and `--lib`. Linux tests are unchanged; `conformance_m1` now treats macOS like Linux for the construction check.
- Packaging: the macOS pkg/dmg ship `sea-forge-jail-launcher` beside `sea-forge` (package.yml). Not done: read-only system-tree exceptions beyond the App Sandbox defaults.

## Phase 5 — WSLc (BLOCKED, evidence `~/projects/_wt/containers/recon/wslc-deep.md`, `~/projects/_wt/containers/recon/wslc-mxc.md`, 2026-10-25)
wslc.exe reports 3.0.3.0 (WSL 3.0.3.0). Findings on Win11 26300.9807:
- No container→Windows-host route: loopback listener 127.0.0.1:18555 unreachable via host.docker.internal, 172.17.0.1, 10.255.255.254, LAN IPs; `--network host` → "host mode networking is not supported". OpenShell supervisors dial the gateway, so a WSLc-hosted supervisor cannot reach a Windows gateway.
- Containers cannot create network namespaces (`unshare -n`, `unshare -Urn` EPERM) and the CLI has no --cap-drop/--security-opt/--read-only/--pids-limit; in-container NNP works, bounding-set drop untested with util-linux.
- Bridge egress (DNS, HTTP to example.com/1.1.1.1) failed on this host; `--network none` blocks all.
- Works: reverse publish (-p 127.0.0.1:port), rw/ro bind mounts (virtiofs), lifecycle verbs, image digests via `image inspect`, `events` JSON stream.
Decision: no WSLc driver is written. It would be a new container driver whose supervisor cannot reach the gateway and whose hardening flags do not exist (spec: no competing sandbox framework). Least-cost remedy: WSLc gaining a host-gateway route (or host networking) plus cap-drop/no-new-privileges/read-only flags or a Docker-compatible API (OpenShell's docker driver would then apply unchanged).

## Phase 4 — MXC probe (`~/projects/_wt/containers/recon/mxc-probe.md`)
wxc-exec.exe from npm @microsoft/mxc-sdk 0.8.0 (tgz sha256 06bb2399…, exe sha256 6049c647…): --probe tier base-container, isolationSessionAvailable true. processcontainer run: write outside grant "Access is denied" + absent on host; granted workspace write present; network block → DNS failure.

## Phase 6 — macOS App Sandbox probe (branch ci/macos-app-sandbox-probe, run 40254192454, macOS 26.6.2 arm64)
Ad-hoc-signed launcher with app-sandbox + temporary-exception absolute-path rw (workspace): writes outside, symlink-through, hard-link creation, $HOME secret read all denied (control without entitlement: all succeed); inherited by sh and perl grandchildren; network.client=false blocks TCP+UDP loopback, DNS, HTTPS; network.client=true allows all (no per-host/port control). Env is not filtered (launcher's caller must clear it — SEA-Forge already env_clears). /usr/bin/python3 xcrun shim refuses to run in App Sandbox.
Decision: implement SandboxClass::Jail on macOS via App Sandbox (Denied network only; AllowTcpPorts refused). OpenShell remains the class for governed egress/credentials; no macOS-native OpenShell driver exists upstream (#3956 closed, not on roadmap).

## Phase 3 — Linux/macOS OpenShell MicroVM (branch ci/openshell-live)
- Linux ubuntu-24.04 + KVM, OpenShell 0.1.3 release binaries (sha256 vs asset digest + checksums files; driver-vm digest only), Docker stopped: gateway mTLS up, `Using compute driver driver=vm`, supervisor sessions accepted (runs 40255566950, 40256113550). SEA-Forge refused every sandbox: effective policy ⊄ boundary because OpenShell 0.1.3 supervisor enriches proxy-mode policies with baseline RO paths incl. /var/log, /app (supervisor lib.rs:1566-1590) which DomainForge's BASELINE_READ_ONLY lacked. Fail-closed behaviour confirmed live. Fix routed to DomainForge (projection must describe runtime enforcement), fixture regenerated.
- live-proof.sh had never run (no KVM locally): needed gateway launch-signing keys and mTLS client identity (upstream e2e pattern); unauthenticated gateway escape hatch deliberately NOT used. NGC default image needs auth on hosted runners → docker.io/library/ubuntu:24.04.
- macOS hosted runner: kern.hv_support=0 (nested virt unavailable) → `krun_start_enter failed: -22`. BLOCKED (external prerequisite: Apple Silicon host with Hypervisor.framework, e.g. self-hosted or larger runner).

## Phase 4 — MXC live (Windows 11 Pro 26300.9807, 2026-10-26)

- Build: OpenShell v0.1.3 (source e1f3c82) built for x86_64-pc-windows-msvc by
  `.github/workflows/openshell-windows.yml` (branch `ci/openshell-windows`, run
  40262602873; SHA256SUMS verified on download). The gateway imports
  `libz3.dll`; without it it exits `0xC0000135`. `wxc-exec.exe` and
  `winhttp-proxy-shim.exe` come from npm `@microsoft/mxc-sdk` 0.8.0 (pinned).
- Upstream `run-mxc-e2e.ps1`, backend `process_container` (AppContainer):
  **PASS 4/4**: fs-rw (in-policy write lands), fs-readonly (control write lands,
  denied write blocked), fs-default-deny (agent launched, write absent),
  network-reject (create refused by policy). Evidence
  `~/projects/_wt/containers/evidence/mxc-e2e-host-20261026/`.
- Local harness/config changes needed to reach that result (diff in the
  evidence bundle, `harness-local-changes.diff`), each an upstream v0.1.3 gap:
  1. the example `mxc-gateway.toml` lacks the required `[openshell] version = 2`;
  2. caller `driver_config` is refused unless `allow_driver_config = true`;
  3. MXC host filesystem grants are refused while resource admission is enabled
     ("no trusted label resolver"), so the run sets
     `[openshell.drivers.mxc.resource_admission] enabled = false`;
  4. `Get-NetTCPConnection` cannot see listeners on this host (CIM error), so the
     readiness probe is a TCP connect;
  5. the policy fixtures hardcode `C:/work/openshell-mxc-e2e`; a custom
     `-DemoDir` is not substituted.
- Upstream `wxc_exec_real` (`--ignored`, real wxc-exec): 6 dry-run config tests
  pass; `pc_oneshot_in_policy_write_succeeds` and
  `pc_oneshot_out_of_policy_write_denied` pass (evidence
  `wxc_exec_real-oneshot-20261026.log`). `iso_lifecycle_round_trip` reports ok
  but isolation_session provisioning on this host fails `policy_validation`
  ("the network is unrestricted and cannot be filtered or denied"), so it is
  counted as unverified. `pc_https_egress_reads_injected_ca_bundle` hung past
  15 min and was killed: NOT RUN to completion.
- SEA-Forge on MXC: **Blocked**. The v0.1.3 MXC driver runs one fixed command at
  create (`driver_config.command`), has no in-sandbox supervisor, no
  `sandbox exec`/upload/download, and `supports_sandbox_authentication = false`.
  SEA-Forge's OpenShell backend uploads a workspace, runs `sandbox exec`, and
  downloads artifacts; none of that exists on MXC. Host grants also require
  turning resource admission off (item 3), which SEA-Forge would have to refuse
  as an authority downgrade. DomainForge projects only `vm`; an `mxc` projection
  needs host paths. Phase 2's capability table already refuses what MXC cannot
  enforce (Landlock hard requirement, Linux process identity). Least-cost
  remedy upstream: an MXC exec/transfer surface plus a label resolver.

## Capability matrix (2026-10-26)

| Backend | Host | Status | Evidence |
|---|---|---|---|
| Local (Landlock + seccomp) | Linux | Passing | ci.yml `test (ubuntu)`, kernel-security socket gates (22.04/24.04 VMs) |
| Jail (Landlock, no net / port allowlist) | Linux | Passing | `tests/conformance_m1.rs` in ci.yml |
| OpenShell `vm` (KVM microVM) | Linux | Passing, live | openshell-live runs 40268910844 + repetitions; 6/6 `cell_episode_live` with Docker stopped |
| Local (Seatbelt) | macOS | Built, untested by scenario | no scenario tests (gap) |
| Jail (App Sandbox) | macOS | Passing | portability.yml `macos`, `tests/macos_app_sandbox_jail.rs` 7/7 |
| OpenShell `vm` (HVF) | macOS | **Blocked** | hosted runners report `kern.hv_support=0`; `krun_start_enter failed: -22`. Needs a bare-metal Apple Silicon runner |
| Local (Job Object only) | Windows | Passing for lifecycle only; **no FS/network containment** | portability.yml `windows`: `windows_local_job.rs` 5/5 |
| Control plane (named pipe, SID identity) | Windows | Passing | `scripts/windows/identity-e2e.ps1` 10/10 |
| OpenShell `mxc` (AppContainer) | Windows | Upstream driver works; **SEA-Forge Blocked** | Phase 4 section |
| WSLc | Windows | **Blocked** | Phase 5 section |
| Isolated (Firecracker-class microVM, no OpenShell) | Linux | Unchanged; refuses off Linux | — |

## Conformance matrix (Phase 8 scenarios)

`L` = live on the real mechanism, `S` = stub CLI (sequence and refusal logic only),
`—` = not applicable to the backend's claimed guarantees, `GAP` = applicable, no test,
`BLK` = backend blocked. A `GAP` or `BLK` is never a pass.

| # | Scenario | Linux Local | Linux Jail | Linux OpenShell vm | macOS Jail | Windows Local | Windows pipe CP |
|---|---|---|---|---|---|---|---|
| 1 | authorized workspace r/w | L | L | L (`live_authorized_episode_runs_in_a_microvm`) | L | L (exit/stdout) | — |
| 2 | unauthorized FS write blocked | L | L | L (`live_host_filesystem_is_not_visible`) | L | — (not contained) | — |
| 3 | symlink/reparse/junction escape | L | GAP | L (`live_workspace_symlink_cannot_reach_host_files`: link dangles in the VM) | L | — | L (`changeset_windows.rs`) |
| 4 | unauthorized network blocked | L | L | L (`live_network_egress_is_denied`) | L | — (not contained) | — |
| 5 | authorized network restricted | L(?) | L | GAP (no live allowed-endpoint test) | — (AllowTcpPorts refused) | — | — |
| 6 | modified executable rejected | GAP | GAP | S (native hash pin) | GAP | GAP | — |
| 7 | provider credentials inaccessible | GAP | GAP | S (host env not forwarded) | L (`$HOME` secret read denied) | L (env cleared) | — |
| 8 | exfiltration to unauthorized endpoint | via 4 | via 4 | via 4 | via 4 | — | — |
| 9 | projection / lock tampering | — | — | L (`live_tampered_projection_is_refused_before_create`: `openshell_projection_tampered`, no sandbox) + S | — | — | — |
| 10 | frozen-run authority widening | GAP (sandbox side) | GAP | GAP | GAP | GAP | — |
| 11 | process-tree cancellation | L | GAP | L (`live_timeout_cancels_the_command`) | L | L (tree + WMI) | — |
| 12 | no host fallback on failure | L | L | S | L (`jail_unavailable`) | — | L (second server, squatting) |
| 13 | concurrent sandboxes isolated | L | GAP | L (`live_concurrent_sandboxes_are_isolated`; search partial, timing-based overlap) | GAP | GAP | L (second-user DACL denial) |
| 14 | restart leaves no orphans | L (partial) | GAP | L (cleanup) + S (destroy idempotent); gateway restart GAP | GAP | L (kill-on-close) | — |
| 15 | evidence attributable | — | — | L (execution.json, effective-policy.yaml, denials.log) | — | — | — |
| 16 | transfer cannot escape paths | L | L | L (flat upload / download into run dir) | — | — | L |

Per applicable backend, the stretch bar of 12 exercised scenarios is met by
Linux Local only. Linux OpenShell vm has 12 live (run 40275738307, 9/9 tests), 2 stub-only, 2 GAP (5, 10); macOS Jail 9;
Windows Local covers lifecycle and environment only, because it claims no
filesystem or network containment.

## Phase 3 — Linux live proof result (2026-10-26)

- DomainForge v0.21.2 (PR GodSpeedAI/DomainForge#162) adds the OpenShell 0.1.3
  supervisor baseline (`/app`, `/var/log`) to the projected read-only set; the
  RepairAgent fixture was regenerated with that CLI (only `policy.yaml`,
  `mapping-report.json`, `openshell.lock` changed). SEA-Forge's
  `domainforge-core = "=0.21.1"` crate pin is unchanged (follow-up), and the
  fixture's `rego/` projection still comes from 0.21.1.
- Bug found live and fixed: `openshell sandbox upload <dir> <dest>` nests a
  named directory as `<dest>/<dirname>/`; only `.` extracts flat, while
  download is flat. SEA-Forge now runs the upload inside the workspace with
  source `.` (`crates/sea-forge-sandbox/src/openshell.rs`, stub test asserts
  source and cwd).
- Workload image contract: the projection keeps `/sandbox` read-only and
  grants `/sandbox/workspace`, so the image must contain that directory (the
  VM driver creates only `/sandbox`). The live job builds public base +
  `/sandbox/workspace`, pushes OCI-manifest images to GHCR (the guest image
  prep rejects Docker v2 manifests), pins them by digest, then stops Docker.
- Result: `cell_episode_live` 6/6 on four runs with Docker stopped and the
  `vm` driver on KVM: 40268910844, 40269436237, 40269901711
  (`ci/openshell-live`) and 40269528859 (integration branch, includes the
  Phase 2 enforcement checks). Variation across the six tests: two images,
  write / read-back / timeout / egress / host-FS probes, fresh workspaces and
  run ids per test and per run.

## Footprint (after, 2026-10-26; baseline in `~/projects/_wt/containers/baseline/BASELINE.md`)

| Measure | Baseline v0.2.0 | After | Change |
|---|---|---|---|
| Production LOC (non-blank, non-comment, excl. `#[cfg(test)]`) | 55,801 | 58,104 (after consolidation) | +2,303 (+4.1%) |
| `sea-forge` (linux x86_64) | 26,864,824 B | 26,758,344 B | -0.4% |
| `sea-forge-server` (linux x86_64) | 31,751,208 B | 31,649,504 B | -0.3% |
| Linux tarball | 17,697,486 B | 17,647,409 B | -0.3% |
| `sea-forge --version` | 0.00 s, 5.8-6.0 MB RSS | 0.00-0.02 s, 5.7-6.0 MB RSS | none |
| New runtime dependencies | — | none (Windows: `windows-sys` already locked) | — |

LOC growth is the Windows control plane (`sea-forge-os` +402, server identity
and pipe transport), the OpenShell enforcement capability, and the macOS jail.
The consolidation pass removed 68 lines; further cuts would merge Unix and
Windows paths whose security semantics differ, which the spec forbids. Package
sizes: `.deb` 12.4 MB, Windows setup `.exe` 11.9 MB, macOS universal `.pkg`
31.8 MB / `.dmg` 32.2 MB.

## Packaging defect found (pre-existing in v0.2.0)

The Devbox (Nix) toolchain linked `/nix/store/...-glibc-2.42/lib/ld-linux-x86-64.so.2`
as the program interpreter and a `/nix/store` RUNPATH into the Linux binaries,
and Nix's `libiconv.2.dylib` into the macOS binaries. The released v0.2.0
tarball cannot start on a host without Nix. Install tests passed because the
runners have Nix. `package.yml` now patches the Linux interpreter and RUNPATH,
rewrites the macOS libiconv path to `/usr/lib/libiconv.2.dylib` and re-signs,
refuses any remaining `/nix/store` reference, and installs the `.deb` in clean
`ubuntu:22.04` and `debian:12` containers (run 40270037897: both pass).
