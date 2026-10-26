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
| `unsafe` | workspace `unsafe_code = "deny"`; pre-existing Linux exceptions: `sea-forge-sandbox/src/socket_policy.rs` (seccomp notify) and `src/bin/sea-forge-workload-exec.rs` (raw syscalls). (Corrected after review F25; first written as "zero".) | `Cargo.toml:38` |
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

1. **One home for new `unsafe`.** Windows FFI with no safe equivalent (named-pipe client
   token SID, pipe DACL, server-SID check, Job Objects) lives in one new
   Windows-only crate `sea-forge-os`; no other new `unsafe` is added (the two
   pre-existing Linux sandbox exceptions are unchanged).
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

## Capability matrix (2026-10-26, corrected after review pass 1)

| Backend | Host | Status | Evidence |
|---|---|---|---|
| Local | all | **Trusted host execution: no FS, network or syscall containment on any OS** (env cleared; process group / Job Object for lifecycle only) | `local.rs` applies no Landlock, seccomp or Seatbelt |
| Isolated (per-run UID, Landlock, seccomp, namespaces) | Linux | Passing | `workload_boundary.rs`, `workload_isolation.rs`, `unix_socket_reach.rs`, `state_tree.rs` in ci.yml; kernel-security socket gates |
| Jail (Landlock FS + **TCP-only** network rules) | Linux | Passing for FS and TCP; **UDP/DNS not restricted** | `conformance_m1.rs` |
| OpenShell `vm` (KVM microVM) | Linux | Passing, live | openshell-live runs 40268910844, 40269436237, 40269901711, 40269528859 (6/6), 40275738307 (9/9); Docker stopped |
| Jail (App Sandbox) | macOS | Passing for FS and TCP in the suite; UDP/DNS blocking seen in the probe run only; `setsid` descendants outlive the run (F08) | portability.yml `macos`, `macos_app_sandbox_jail.rs` |
| OpenShell `vm` (HVF) | macOS | **Blocked** | hosted runners report `kern.hv_support=0`; needs bare-metal Apple Silicon |
| Control plane (named pipe, SID identity) | Windows | Passing (see review fixes) | `scripts/windows/identity-e2e.ps1` |
| OpenShell `mxc` (AppContainer) | Windows | Upstream driver works; **SEA-Forge Blocked** | Phase 4 section |
| WSLc | Windows | **Blocked** | Phase 5 section |

## Conformance matrix (Phase 8 scenarios, corrected after review pass 1)

`L` = live on the real mechanism with the named test, `S` = stub CLI (sequence
and refusal logic only), `—` = the backend claims no such guarantee, `GAP` =
applicable, no test that exercises it, `BLK` = backend blocked. Only `L` counts.

| # | Scenario | Linux Isolated | Linux Jail | Linux OpenShell vm | macOS Jail | Windows Local (lifecycle only) | Windows pipe CP |
|---|---|---|---|---|---|---|---|
| 1 | authorized workspace r/w | L | L | L `live_authorized_episode_runs_in_a_microvm` | L | L exit code + stdout | — |
| 2 | unauthorized FS write blocked | L | L | GAP (`live_host_filesystem_is_not_visible` only reads) | L | — | — |
| 3 | symlink/reparse/junction escape | L | GAP | L `live_workspace_symlink_cannot_reach_host_files` | L | — | L `changeset_windows.rs` |
| 4 | unauthorized network blocked | L | L TCP only; UDP/DNS GAP | BLK no reachable controlled endpoint, no denial record (F17) | L TCP (`nc -z`); UDP/DNS seen only in probe run 40254192454 | — | — |
| 5 | authorized network restricted | GAP | L TCP ports | GAP | — (AllowTcpPorts refused) | — | — |
| 6 | modified executable rejected | GAP | GAP | GAP (native pin is MXC-only; stub) | GAP | — | — |
| 7 | provider credentials inaccessible | GAP | GAP | S host env not forwarded | L `$HOME` secret unreadable | L env cleared | — |
| 8 | exfiltration to unauthorized endpoint | via 4 | TCP only | GAP | via 4 | — | — |
| 9 | projection / lock tampering | — | — | L `live_tampered_projection_is_refused_before_create` + S | — | — | — |
| 10 | frozen-run authority widening | GAP | GAP | S (projection snapshot + approved identity, review F03) | GAP | — | — |
| 11 | process-tree cancellation | L | GAP | L `live_timeout_cancels_the_command` | L (same pgid only; `setsid` escapes) | L cooperative tree only; WMI-created processes are outside the job | — |
| 12 | no host fallback on failure | L | L | S | L `jail_unavailable` | — | L second server, squatting |
| 13 | concurrent sandboxes isolated | L | GAP | L `live_concurrent_sandboxes_are_isolated` (control token, overlap checked) | GAP | — | L second-user DACL denial |
| 14 | restart leaves no orphans | L (launcher death) | GAP | GAP (gateway/driver restart untested; normal cleanup only) | GAP | L kill-on-close | — |
| 15 | evidence attributable | — | — | L execution.json, effective-policy.yaml, denials.log | — | — | — |
| 16 | transfer cannot escape paths | L | L | GAP (no malicious-archive test) | — | — | L |

Windows Local claims lifecycle and environment only, so most rows are `—`.
The stretch bar (12 exercised scenarios per applicable backend) is met by
Linux Isolated only; it is not met by Linux OpenShell vm (6 L: 1, 3, 9, 11, 13, 15), Linux Jail or
macOS Jail.

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

## Independent review

### Pass 1 (2026-10-26)

- Reviewer: `codex exec -s read-only` (codex-cli 0.163.0), model `gpt-6.1-sol`
  (OpenAI; the builder is Claude Opus 5.5), session 01a176c5-403d-7462-bfd5-1edc26929d9c,
  exit 0. Frozen SHA c78e7d5f, read-only worktree. Packet: spec excerpt, full
  diff from 4e67dfca, this record, baseline, live log, MXC summary, test map, skills.
  Output: `~/projects/_wt/containers/review/review-findings-c78e7d5.md`.
- Counts: critical 0, high 10, medium 14, low 1.
- Resolutions (fix commits listed as they land):
  - F01 confirmed (pre-existing backend, wrong attribution in this record): matrices corrected.
  - F07 confirmed as a claim error: Windows Local lifecycle is cooperative only; matrix corrected.
  - F10 confirmed (pre-existing, documented TCP-only): matrix marks Linux Jail TCP-only, UDP/DNS GAP.
  - F18 confirmed: cells limited to tests that attempt the scenario.
  - F19 confirmed: package.yml uses inputs directly; publish verifies built commit = tag.
  - F20 confirmed (pre-existing in the tarball): helpers shipped and install-checked.
  - F22, F23, F24 confirmed: skills corrected.
  - F25 confirmed: unsafe inventory corrected.
  - Sandbox branch `fix/review1-sandbox` (merged; live 9/9 run 40286201426, Portability 40283796428):
    - F02 confirmed, fixed: `prepare` reads `openshell gateway info -o json` `compute_drivers[].name` and
      refuses `openshell_driver_mismatch` / `openshell_driver_unverified` before create; execution.json
      records `compute_driver.source = "gateway_info"`.
    - F03 confirmed, fixed: the projection is snapshotted into `artifacts/openshell/projection/` before any
      read, the lock is verified on the snapshot, every later read and CLI/prover argument uses it, and the
      runtime-approved identity (`OpenShellSandbox::approving`) must match (`openshell_projection_tampered`).
    - F06 confirmed, fixed: `artifact_output.rs` removes a planted entry and opens `create_new` + `O_NOFOLLOW`
      for local, Linux/macOS jail and OpenShell outputs; test `artifact_output_no_follow.rs`.
    - F08 confirmed, partly fixed: the process group is killed on completion too (local, Linux jail, macOS
      jail). A `setsid()` descendant still escapes; documented in `docs/subsystems/sandbox-runtime.md`. OPEN.
    - F16 confirmed, fixed: B plants and must find its own control, grep must exit 0, A must outlive B; B
      searches the policy's writable trees `/sandbox` and `/tmp`.
    - F17 confirmed, claim narrowed: OpenShell 0.1.3 logs no denial for the probe (denials.log empty); the
      test asserts no connection only and says so. Scenario 4 on OpenShell vm stays "no egress observed",
      without a positive control. OPEN (needs a reachable controlled endpoint).
    - F21 confirmed, fixed: OCI repository-component separator grammar.
  - Windows branch `fix/review1-windows` (merged; Portability 40285346128 green incl. identity E2E):
    - F04, F05 confirmed, fixed: `safe_write_within` opens without truncation (`OPEN_ALWAYS`, no-follow),
      refuses link count ≠ 1, reparse points and a handle final path outside the canonical root, and deletes
      a file it created on refusal; tests in `safe_write_windows.rs`.
    - F09 confirmed, fixed: CLI opens with identification-only QoS; the server accepts identification level.
    - F11 confirmed, fixed: client ACE is `0x12018B` (no `FILE_CREATE_PIPE_INSTANCE`); E2E shows an admitted
      user cannot create an instance.
    - F12 confirmed, fixed: identity read bounded by the request deadline.
    - F13 confirmed, fixed: optional `SEA_FORGE_SERVER_SID` pins the expected server; not exercised by a
      CLI-driven E2E (limitation).
    - F14 confirmed, fixed: `Job::terminate_and_wait` (5 s) used by local and settlement.
    - F15 confirmed, fixed: `SEA_FORGE_REQUIRE_LINK_PRIVILEGE=1` in CI turns missing link privilege into a failure.
