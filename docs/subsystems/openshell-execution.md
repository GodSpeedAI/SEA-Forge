# Subsystem: OpenShell Execution

> **Authorized work in an NVIDIA OpenShell sandbox defined by a DomainForge OpenShell projection.**  
> Governed crates: `sea-forge-authority` (`sandbox_command`, `promote_changeset`, `opa`), `sea-forge-sandbox` (`openshell.rs`), `sea-forge-runtime`, `sea-forge-changeset`  
> Decision record: [ADR-009](../decisions/ADR-009-semantic-cell-execution.md)

---

## 1. What this is

SEA-Forge can run authorized work in an NVIDIA OpenShell sandbox. The sandbox is defined by a DomainForge OpenShell projection, a directory with these files:

* `policy.yaml`: the sandbox policy (the Cell boundary).
* `sandbox.json`: sandbox creation parameters that OpenShell keeps outside the policy, including `vm_shape`, the workspace mount and the execution timeout.
* `mapping-report.json`: the source element of each policy rule.
* `openshell.lock`: the identities the projection binds (`cell_id`, `model.hash`, `policy_digest`, `openshell_version`) and the sha256 of its files.

`SandboxClass::OpenShell` (written `openshell` in policy files) is implemented in `crates/sea-forge-sandbox/src/openshell.rs` as a synchronous adapter over the `openshell` CLI and `openshell-prover`. The adapter does not use OpenShell's Rust SDK (see ADR-009, decision 5). `sea_forge_runtime::execute_openshell` is the entry point. A guest command is authorized as the typed action `ExecuteSandboxCommand`, never as a host `ExecuteCommand` (ADR-009, decision 11). `sea-forge-changeset` provides the transactional workspace, and promotion is authorized as the typed action `PromoteChangeset` (ADR-009, decision 12).

The OpenShell backend is never selected implicitly. `select_sandbox(SandboxClass::OpenShell)` returns `openshell_binding_missing`; a projection binding must be supplied through `select_openshell`. The backend never falls back to `local` or `jail`.

---

## 2. Operational dependencies

| Component | Version | Required for | Notes |
|---|---|---|---|
| `openshell` CLI | 0.1.3 (release asset `openshell-x86_64-unknown-linux-musl.tar.gz`) | Sandbox lifecycle, policy reads, file transfer, logs | Required. The adapter refuses any version other than the one the projection's `openshell.lock` names (`openshell_version_mismatch`). |
| `openshell-prover` | 0.1.3 | Policy containment check, before and after sandbox creation | Required. Its JSON result must be `within_boundary` with all of `filesystem`, `landlock`, `network_l4`, `network_rest` and `process` covered. |
| `openshell-gateway` | 0.1.3 | The control plane the CLI talks to | Required and reachable. For the primary path it runs natively on the host as a normal process or user service. |
| `openshell-driver-vm` | 0.1.3 | The libkrun MicroVM driver, which is the primary compute driver | Required for the primary path. It is a single binary that embeds libkrun, libkrunfw, the guest init and the supervisor. The gateway looks for it in `driver_dir` if configured, else `~/.local/libexec/openshell`, `/usr/libexec/openshell`, `/usr/local/libexec/openshell`, `/usr/local/libexec`, `<gateway prefix>/libexec`, or next to the gateway binary. It is not found via `PATH`. |
| `mke2fs`, `debugfs`, `e2fsck` | e2fsprogs 1.43 or newer | The VM driver's root filesystem preparation | Required by the VM driver. `openshell-gateway config preflight` checks these. |
| `ssh` client | system | `openshell sandbox upload` and `download` | OpenShell's transfer commands use the system `ssh`. |
| `git` | system | Changeset staging and promotion | Used by `sea-forge-changeset`. |
| OPA | 1.21.1 | The authority's `opa` policy engine | Required whenever a policy configures the `opa` engine: `OpaCandidate::evaluate` runs `opa eval` for each decision. The sandbox adapter does not use it. |

Not required: Docker Engine, Docker Desktop, Podman, Kubernetes. They are optional gateway compute drivers. The VM driver pulls OCI images directly from the registry (`oci-client`) and prepares an ext4 root filesystem itself. If a local Docker or Podman daemon is reachable and already holds the image, the driver may export from it instead, so a host with no daemon exercises the registry path.

---

## 3. Platform support

| Platform | Virtualization backend | Status for this integration |
|---|---|---|
| Linux x86_64 / aarch64 | KVM. `/dev/kvm` must be openable by the user running the gateway. | Supported by OpenShell. This integration's live proof has NOT been run yet (see section 9). |
| macOS Apple Silicon | Hypervisor.framework. The driver binary must carry the `com.apple.security.hypervisor` entitlement (upstream's Homebrew formula ad-hoc signs it). | Not tested by this integration. |
| Windows | The gateway's compute drivers are unsupported natively. Under WSL2 the Linux build runs, but MicroVMs need nested virtualization: `/dev/kvm` exists only when `nestedVirtualization=true` is set under `[wsl2]` in `%UserProfile%\.wslconfig` and WSL is restarted (`wsl --shutdown`), and the CPU and Windows build must support it. | OpenShell's own support matrix lists only "WSL 2 + Docker Desktop" as experimental and makes no MicroVM-on-WSL2 claim. `probe_host_virtualization()` detects a missing `/dev/kvm`, reports error class `openshell_virtualization_unavailable`, and names the WSL2 setting when it sees a Microsoft kernel. There is no fallback. The development host is WSL2 with `nestedVirtualization=false`, so the live proof is BLOCKED there. |

---

## 4. Install (Docker-free, Linux)

Download the five release assets and the three checksum files with the GitHub CLI:

```bash
DL="$(mktemp -d)"
cd "$DL"
for asset in \
    openshell-x86_64-unknown-linux-musl.tar.gz \
    openshell-prover-x86_64-unknown-linux-musl.tar.gz \
    openshell-gateway-x86_64-unknown-linux-gnu.tar.gz \
    openshell-driver-vm-x86_64-unknown-linux-gnu.tar.gz \
    openshell-checksums-sha256.txt \
    openshell-prover-checksums-sha256.txt \
    openshell-gateway-checksums-sha256.txt; do
    gh release download v0.1.3 -R NVIDIA/OpenShell -p "$asset"
done
```

Verify the downloads. Each checksum file lists more assets than were downloaded, so `--ignore-missing` is needed:

```bash
for sums in *-checksums-sha256.txt; do
    sha256sum -c --ignore-missing "$sums"
done
```

Extract each tarball into its own directory. Each one holds a single binary at its root:

```bash
mkdir -p cli prover gw vm
tar -xzf openshell-x86_64-unknown-linux-musl.tar.gz -C cli
tar -xzf openshell-prover-x86_64-unknown-linux-musl.tar.gz -C prover
tar -xzf openshell-gateway-x86_64-unknown-linux-gnu.tar.gz -C gw
tar -xzf openshell-driver-vm-x86_64-unknown-linux-gnu.tar.gz -C vm
```

Place the binaries. `openshell`, `openshell-prover` and `openshell-gateway` go in `~/.local/bin`; `openshell-driver-vm` goes in `~/.local/libexec/openshell/`:

```bash
mkdir -p ~/.local/bin ~/.local/libexec/openshell
install -m 0755 cli/openshell ~/.local/bin/openshell
install -m 0755 prover/openshell-prover ~/.local/bin/openshell-prover
install -m 0755 gw/openshell-gateway ~/.local/bin/openshell-gateway
install -m 0755 vm/openshell-driver-vm ~/.local/libexec/openshell/openshell-driver-vm
```

Confirm the versions. Each should print its name followed by `0.1.3`:

```bash
openshell --version
openshell-prover --version
openshell-gateway --version
~/.local/libexec/openshell/openshell-driver-vm --version
```

Upstream's `install.sh` also works. It enables a systemd user service and may select a container driver when it detects one, so this sequence is preferred when a container-free host is required.

---

## 5. Gateway configuration for a Cell

In OpenShell 0.1.3 the MicroVM driver has no per-sandbox CPU or memory limits; only the gateway-level `vcpus` and `mem_mib` apply. A Cell's resources are therefore the VM shape in the gateway configuration. The values come from the projection's `sandbox.json` `vm_shape` (`vcpus`, `mem_mib`, `overlay_disk_mib`). For the RepairAgent fixture these are 4, 8192 and 20480.

The gateway file below is the one that passed `openshell-gateway config preflight` on the development host. Replace `<user>` with the account name. Schema `version = 2` is required by OpenShell 0.1.3.

```toml
[openshell]
version = 2

[openshell.gateway]
compute_driver = "vm"
disable_tls = true

[openshell.drivers.vm]
grpc_endpoint = "http://127.0.0.1:18081"
driver_dir = "/home/<user>/.local/libexec/openshell"
state_dir = "/home/<user>/.local/state/openshell/vm-driver-godspeed"
vcpus = 4
mem_mib = 8192
overlay_disk_mib = 20480
```

Commands:

```bash
# Validate the configuration without starting the gateway.
openshell-gateway config preflight --path gateway.toml

# Start the gateway in the foreground.
openshell-gateway --config gateway.toml --port 18081 \
    --db-url "sqlite:$HOME/.local/state/openshell/gateway-godspeed/openshell.db?mode=rwc"

# In another shell: register it and check it.
openshell gateway add http://127.0.0.1:18081 --local --name godspeed
openshell --gateway godspeed status
```

Notes:

* `disable_tls` with a loopback bind is for a single-user development host only.
* `compute_driver` must be set explicitly. The VM driver is never auto-detected.
* `openshell gateway add` with an `http://` endpoint and `--local` stores a local plaintext registration.
* Preflight does not check KVM. A missing `/dev/kvm` surfaces when a sandbox starts.

---

## 6. How a run is governed

A governed run passes through these steps in order. Each step either passes or ends the run.

1. **Projection identity, OPA, authority.** `run_cell_episode` first reads the projection identity and requires an image. If the policy configures `opa`, the OPA candidate is recorded as ledger kind `opa_evaluation`. The authority engine then evaluates an `execute_sandbox_command` action (`ExecuteSandboxCommand`), and the decision is committed as `authority_decision` before any repository access or sandbox call. Only a rule with `sandbox_class: openshell` grants it, and that rule must pin exactly one `cell_id`, one `model_hash`, one `openshell_policy_digest` and one `sandbox_image`, and must name an `argv0`. A host `ExecuteCommand` is unchanged and never reaches this class: the `openshell` class is refused on any rule that is not `execute_sandbox_command`.
2. **Binding.** `sea_forge_runtime::execute_openshell` reads the projection's identity from `openshell.lock` and compares the `cell_id`, `model.hash` and policy digest with the grant. The image is compared with the grant's `sandbox_image`. A mismatch is refused with `openshell_binding_mismatch`. The lock must cover exactly `policy.yaml`, `sandbox.json` and `mapping-report.json`. Each file must match its sha256 in the lock, and the projection directory must hold exactly those three files. A missing, extra or edited file is refused with `openshell_projection_tampered`.
3. **Host checks.** The adapter checks the CLI version against the lock, host virtualization (when the gateway is local), and gateway reachability (`openshell status`).
4. **Projected policy.** `openshell-prover` must report the projected `policy.yaml` as `within_boundary` with all five required domains covered. A failure here is `openshell_policy_unverified`, and no sandbox is created.
5. **Creation and effective policy.** The sandbox is created with `--policy policy.yaml`, `--detach`, `--no-auto-providers` and `--no-tty`, from the configured image when one is set. The policy the gateway actually composed is then read with `openshell sandbox get --policy-only`, including provider rules. It must also be within the projected policy, checked by the prover. Otherwise the sandbox is deleted and the run fails with `openshell_effective_policy_exceeds_boundary`.
6. **Workspace.** The private workspace is uploaded with `openshell sandbox upload --no-git-ignore`. It is an export of the base Git tree, with no `.git`. The authoritative repository is never mounted or uploaded.
7. **Execution.** The command runs with `openshell sandbox exec` in the workspace directory, with only the request's environment variables passed through `--env` and the Cell's timeout. Exit code 124 is reported as `TimedOut`. If the CLI itself does not finish within the timeout plus 30 seconds, the adapter kills it and reports a timeout. A non-zero exit with logged denials is reported as a sandbox violation. Stdout and stderr are captured in `artifacts/stdout.txt` and `artifacts/stderr.txt`.
8. **Collection and cleanup.** The workspace is downloaded to the result directory with `openshell sandbox download`. Denials are written to `denials.log`. The sandbox is deleted on every path once it exists. An explicit changeset is then built in a separate staging repository by `sea-forge-changeset`.
9. **Settlement.** Settlement decides the outcome (`sea-forge-settlement`). This adapter does not decide it.
10. **Promotion request.** The episode runs `PromoteChangeset` only when settlement accepted and validation accepted a non-empty changeset. The rule must require approval, so the evaluation escalates and nothing is allowed outright. The evaluation is committed as a second `authority_decision`, and the episode returns a `PromotionRequest`. The episode never moves the authoritative ref. The same OPA candidate is passed to this evaluation. It can only deny or escalate; the promotion rule still escalates.
11. **Promotion.** `promote_cell_changeset` is a separate call. It checks that the request describes exactly its changeset and this repository, then consumes an approval through `grant_after_approval` (`approval_grant_consumption`). The requester must differ from the resolver, and the resolver must hold the approver role. The grant is bound to the exact proposal (repository, reference, expected base, proposed commit and tree, changeset digest and paths). A changed proposal needs a new decision and a new approval. The compare-and-swap, `git update-ref <ref> <new> <expected-old>`, is performed by the trusted promotion path (`sea_forge_changeset::promote`), never by a guest command. Each attempt that reaches the swap is recorded as `workspace_promotion`. A moved ref is a conflict (`stale_base`), and nothing is merged.

For the happy path the ledger kinds are, in order: `authority_decision`, `openshell_execution`, `workspace_changeset`, `settlement_event`, `authority_decision`, `settlement_criteria`, `approval_resolution`, `approval_grant_consumption`, `workspace_promotion`. Details are in ADR-009, "Episode flow".

### Error classes

Classes raised by the OpenShell adapter and its runtime entry point. Each class is a string in the `class` field of the returned error.

| Class | Raised when |
|---|---|
| `sandbox_command_requires_openshell` | A sandbox operation reaches the host runtime `execute`. Local and jail backends also refuse sandbox operations. |
| `sandbox_command_required` | A host operation reaches `execute_openshell`. |
| `openshell_binding_missing` | The `openshell` class is selected through `select_sandbox` without a projection binding. Use `select_openshell`. |
| `openshell_binding_mismatch` | The grant's `cell_id`, `model_hash`, `openshell_policy_digest` or `sandbox_image` does not equal the projection's value (`openshell.lock` identity, or the image). |
| `openshell_projection_tampered` | `openshell.lock` does not cover exactly `policy.yaml`, `sandbox.json` and `mapping-report.json`; a file is missing, extra, or does not match its sha256 in the lock; or the lock's policy digest disagrees with `policy.yaml`. |
| `openshell_projection_error` | A projection file cannot be read or parsed, a required field is missing, or the projection does not declare exactly one workspace mount. |
| `openshell_unavailable` | The `openshell` or `openshell-prover` binary cannot be started. |
| `openshell_version_mismatch` | `openshell --version` does not report the version the projection's lock names. |
| `openshell_virtualization_unavailable` | The gateway is local and `/dev/kvm` cannot be opened. On a Microsoft kernel the message names the WSL2 setting. |
| `openshell_gateway_unavailable` | `openshell status` does not succeed. |
| `openshell_policy_unverified` | The projected `policy.yaml` is not `within_boundary`, or the prover did not cover a required domain, or the prover printed no JSON result. |
| `openshell_create_failed` | `openshell sandbox create` fails. |
| `openshell_policy_unavailable` | The effective policy or the policy status cannot be read, or the status is not JSON. |
| `openshell_effective_policy_exceeds_boundary` | The effective policy the gateway composed is not `within_boundary`, or the prover did not cover a required domain. The sandbox is deleted. |
| `openshell_transfer_failed` | `openshell sandbox upload` or `download` fails. |
| `openshell_logs_unavailable` | `openshell logs --source sandbox` fails when denials are collected. |
| `openshell_timeout` | A CLI command does not finish within its deadline: 60 s for control commands (`status`, `get`, `policy get`, `logs`, `delete`), 1800 s for `sandbox create`, and 600 s for `upload` and `download`. |
| `openshell_cleanup_failed` | `openshell sandbox delete` fails and the sandbox still exists. |

---

## 7. Evidence

Files are written under `<run>/artifacts/openshell/`:

* `execution.json`: written when the sandbox is created and verified. Fields: `schema` (`sea-forge.openshell.execution/v1`), `sandbox` (the sandbox name), `gateway`, `image`, `cell_id`, `model_hash`, `projected_policy_digest`, `openshell_version`, `effective_policy` (`sha256`, `hash`, `version`, `config_revision`, `source`, as reported by `openshell policy get`), and `prover` (`projected` and `effective`, the two prover JSON envelopes).
* `effective-policy.yaml`: the policy the gateway composed, as returned by `openshell sandbox get --policy-only`.
* `denials.log`: the lines of `openshell logs --source sandbox -n 2000` that contain ` DENIED `, written after the command finishes. The log format is not verified in this change.

The command's stdout and stderr are written to `<run>/artifacts/stdout.txt` and `<run>/artifacts/stderr.txt`, outside the `openshell/` directory.

Ledger record kinds for a governed run are added by `sea-forge-case-runner` and are documented there. They are not listed here.

---

## 8. Limits

* The prover checks policy containment, not runtime isolation.
* Filesystem widening is reported by the prover as `unsupported` (unresolved path), which the adapter treats as failure.
* External effects of a command, for example an HTTP request that reached an allowed host, are not rolled back by rejecting a changeset.
* Credential-bound network flows and read-only mounts are refused by the DomainForge projection, so no credential is delivered into a sandbox yet.
* The sandbox name is derived from the run's artifacts directory: `sf-` followed by the first 16 hex characters of the sha256 of that path.
* Changeset promotion needs the target ref not to be checked out in a working tree.
* The OPA engine is a subprocess evaluation with a timeout. It is not a server and holds no state between decisions.
* A grant attempt with a mismatched action consumes the approval. It fails closed, and a fresh approval is then required.
* The governed episode (`crates/sea-forge-case-runner/src/cell_episode.rs`) does not move the authoritative ref. It returns a `PromotionRequest`, and a separate `promote_cell_changeset` call performs the approved promotion (section 6). See ADR-009, Known limitations.
* BoxLite (`boxlite-ai/boxlite`, Apache-2.0, an embeddable micro-VM library with copy-on-write QCOW2 disks and clone/export) was reviewed as a possible future donor for snapshots and forks. It is not a dependency.

---

## 9. Verification status

| Claim | How verified | Status |
|---|---|---|
| Adapter control flow and failure handling | `cargo test -p sea-forge-sandbox --test openshell_stub` against stub executables (21 tests in the file) | Not re-run for this document. Last recorded result: passed (16 tests at that time; the file now has 21 `#[test]` functions). |
| Changeset guarantees | `cargo test -p sea-forge-changeset` against real git (15 tests in the file) | Not re-run for this document. Last recorded result: passed. |
| Authority: sandbox-command rules, promotion rules, OPA stub failures | `cargo test -p sea-forge-authority --test sandbox_command --test promote_changeset --test opa_candidate` | Passes. The tests marked `requires opa 1.21.1 on PATH` (`real_opa_*`) are ignored by default; run with `--ignored` against a local `opa` 1.21.1, all 7 pass. The `authority-opa` CI job runs them (see below). |
| Projected policy within prover coverage; weakened variants rejected | DomainForge gate `scripts/verify/projection-targets/openshell.sh` with the real `openshell-prover` 0.1.3 | Passes. The gate script is not in this worktree, so it was not re-run here. |
| Gateway configuration for the VM driver | `openshell-gateway config preflight` | Passes on the development host. |
| Live MicroVM execution; real filesystem, network and timeout enforcement; non-bootstrap image preparation (upstream issue NVIDIA/OpenShell#2358, fixed upstream in merge `cb6e88ac`, an ancestor of v0.1.3); cancellation and cleanup | `scripts/openshell/live-proof.sh` | NOT RUN. BLOCKED: the development host is WSL2 with `nestedVirtualization=false`, so `/dev/kvm` does not exist. The script runs `crates/sea-forge-case-runner/tests/cell_episode_live.rs`: six tests that exist and compile, are ignored by default, and have never executed. |

CI: the required `authority-opa` job installs `opa` 1.21.1, verifies it against a SHA-256 digest pinned in the workflow, and runs `cargo test -p sea-forge-authority --test opa_candidate -- --include-ignored` and `cargo test -p sea-forge-case-runner --test cell_episode -- --include-ignored`. The pin shows the binary is the published release asset; it does not prove the binary matches a source build.

---

## 10. Security invariants

Each invariant names the code that enforces it and the test that shows it. "UNPROVEN" means no test in the repository shows it.

Test names are given as `file::test_name`, with the file under `crates/`. The case-runner episode tests are in `crates/sea-forge-case-runner/tests/cell_episode.rs`.

| Invariant | Enforced in | Proven by |
|---|---|---|
| Host `ExecuteCommand` gate is unchanged | `sea-forge-authority/src/lib.rs` (host `execute_command` rules, `untrusted_executable`) | `sea-forge-authority/tests/sandbox_command.rs::host_command_restriction_is_unchanged` |
| A sandbox declaration alone cannot bypass the host gate | `sea-forge-authority/src/lib.rs` (`sandbox_class: openshell` accepted only on `execute_sandbox_command`) | `sea-forge-authority/tests/sandbox_command.rs::sandbox_class_on_a_host_command_rule_does_not_load`; `sea-forge-authority/tests/sandbox_command.rs::host_command_never_matches_a_sandbox_rule` |
| Every sandbox-command rule pins cell, model, policy digest, image and argv0 | `sea-forge-authority/src/lib.rs` (rule validation) | `sea-forge-authority/tests/sandbox_command.rs::sandbox_command_rule_requires_every_binding`; `sea-forge-authority/tests/sandbox_command.rs::malformed_sandbox_command_is_denied` |
| A guest command cannot run on the host runtime | `sea-forge-runtime/src/lib.rs` (`execute` refuses with `sandbox_command_requires_openshell`) | `sea-forge-runtime/tests/sandbox_command_routing.rs::guest_command_cannot_run_on_the_host` |
| A guest command cannot run on the local or jail backend | `sea-forge-sandbox` local and jail backends | `sea-forge-sandbox/tests/sandbox_command_host_refusal.rs::local_backend_refuses_sandbox_commands`; `sea-forge-sandbox/tests/sandbox_command_host_refusal.rs::jail_backend_refuses_sandbox_commands` |
| A host command is refused by the OpenShell backend | `sea-forge-sandbox/src/openshell.rs`, `sea-forge-runtime/src/lib.rs` (`execute_openshell` refuses with `sandbox_command_required`) | `sea-forge-sandbox/tests/openshell_stub.rs::host_command_operation_is_refused_by_the_openshell_backend` |
| A grant is bound to cell, model, policy digest and image | `sea-forge-runtime/src/lib.rs` (`execute_openshell` compares the request with the projection and image) | `sea-forge-runtime/tests/sandbox_command_routing.rs::openshell_execution_requires_the_authorized_projection`; `sea-forge-authority/tests/sandbox_command.rs::mismatched_sandbox_command_is_denied`. Policy-digest mismatch in the episode: `sea-forge-case-runner/tests/cell_episode.rs::grant_for_a_different_policy_digest_is_refused` |
| A tampered policy or sandbox file is refused before anything runs | `sea-forge-sandbox/src/openshell.rs` (per-file sha256 against `openshell.lock`) | `sea-forge-sandbox/tests/openshell_stub.rs::tampered_policy_is_refused_before_anything_runs`; `sea-forge-sandbox/tests/openshell_stub.rs::tampered_mapping_report_is_refused` |
| A missing or extra projection file is refused | `sea-forge-sandbox/src/openshell.rs` (lock must cover exactly the three artifacts; directory must hold exactly them) | `sea-forge-sandbox/tests/openshell_stub.rs::projection_with_missing_mapping_report_is_refused`; `sea-forge-sandbox/tests/openshell_stub.rs::projection_with_an_unexpected_extra_file_is_refused` |
| The projected policy is checked against the boundary before creation | `sea-forge-sandbox/src/openshell.rs` (prover before `sandbox create`) | `sea-forge-sandbox/tests/openshell_stub.rs::policy_outside_prover_coverage_is_refused`; `sea-forge-sandbox/tests/openshell_stub.rs::incomplete_prover_coverage_is_refused` |
| The effective policy is checked against the boundary before execution, and the sandbox is deleted on failure | `sea-forge-sandbox/src/openshell.rs` (`openshell_effective_policy_exceeds_boundary`) | `sea-forge-sandbox/tests/openshell_stub.rs::provider_expanded_effective_policy_is_refused_and_cleaned_up`; `sea-forge-case-runner/tests/cell_episode.rs::provider_expanded_effective_policy_is_refused` |
| No fallback to a weaker sandbox class | `sea-forge-sandbox/src/lib.rs` (`select_sandbox` refuses `openshell` without a binding); `openshell.rs` (no fallback path) | `sea-forge-sandbox/tests/openshell_stub.rs::select_sandbox_never_downgrades_openshell`; `sea-forge-sandbox/tests/openshell_stub.rs::unavailable_gateway_is_an_explicit_failure` |
| The sandbox is deleted on every path once created | `sea-forge-sandbox/src/openshell.rs` (cleanup on each exit) | `sea-forge-sandbox/tests/openshell_stub.rs::create_failure_is_reported_and_cleaned_up`; `sea-forge-sandbox/tests/openshell_stub.rs::destroy_is_idempotent` |
| The host environment is not forwarded to the sandbox | `sea-forge-sandbox/src/openshell.rs` (`--env` from the request only) | `sea-forge-sandbox/tests/openshell_stub.rs::host_environment_is_not_forwarded` |
| The authority decision is committed before sandbox creation | `sea-forge-case-runner/src/cell_episode.rs` (`authority_decision` is committed before the repository is touched and before `execute_openshell`) | `sea-forge-case-runner/tests/cell_episode.rs::authorized_repair_is_executed_settled_and_promoted` (asserts ledger order: `authority_decision` before `openshell_execution`). No test sequences the commit against the sandbox create call itself. |
| A promotion cannot be allowed outright | `sea-forge-authority/src/lib.rs` (`promote_changeset` without `requires_approval: true` fails bundle load; grant guard) | `sea-forge-authority/tests/promote_changeset.rs::a_well_formed_promotion_escalates_through_the_rule_and_never_allows`; `sea-forge-authority/tests/promote_changeset.rs::an_allow_rule_without_requires_approval_does_not_load`; `sea-forge-authority/tests/promote_changeset.rs::engine_grant_cannot_mint_a_promotion_without_approval` |
| A promotion needs an approved grant | `sea-forge-authority/src/lib.rs` (`grant_after_approval` only) | `sea-forge-authority/tests/promote_changeset.rs::rejected_and_pending_approvals_never_grant`; `sea-forge-case-runner/tests/cell_episode.rs::promotion_without_commit_authority_is_not_performed`; `sea-forge-case-runner/tests/cell_episode.rs::rejected_approval_does_not_promote` |
| Self-approval is refused | `sea-forge-authority/src/lib.rs` (`grant_after_approval`: requester != resolver) | `sea-forge-authority/tests/promote_changeset.rs::the_requester_cannot_approve_their_own_promotion` |
| An approval is single-use | `sea-forge-authority/src/lib.rs` (`approval_grant_consumption`, ledger-backed) | `sea-forge-authority/tests/promote_changeset.rs::an_approval_by_another_security_officer_grants_exactly_once` |
| A changed proposal, tree, base or reference is refused | `sea-forge-authority/src/lib.rs` (grant bound to the exact action hash) | `sea-forge-authority/tests/promote_changeset.rs::an_approval_is_bound_to_the_exact_changeset_base_tree_and_reference` |
| `GitCommit` is never a grant path for promotion | `sea-forge-authority/src/lib.rs` (`GitCommit` escalates as unsupported, not grantable) | `sea-forge-authority/tests/promote_changeset.rs::git_commit_still_escalates_as_unsupported_and_is_not_grantable` |
| A stale base is refused and does not merge | `sea-forge-changeset/src/lib.rs` (`promote`: `git update-ref <ref> <new> <expected-old>`, class `stale_base`) | `sea-forge-changeset/tests/changeset.rs::stale_base_is_a_conflict_not_a_merge`; `sea-forge-changeset/tests/changeset.rs::concurrent_changesets_from_one_base_conflict` |
| A rejected changeset never mutates the authoritative repository | `sea-forge-changeset/src/lib.rs`; `sea-forge-case-runner/src/cell_episode.rs` (promotion only on accepted settlement) | `sea-forge-changeset/tests/changeset.rs::rejected_changeset_leaves_the_authoritative_repository_byte_identical`; `sea-forge-case-runner/tests/cell_episode.rs::failed_command_is_rejected_and_not_promoted`; `sea-forge-case-runner/tests/cell_episode.rs::symlink_in_result_is_rejected_and_not_promoted`; `sea-forge-case-runner/tests/cell_episode.rs::rejected_settlement_and_escalated_validation_request_nothing` |
| A tampered changeset is refused | `sea-forge-changeset/src/lib.rs` | `sea-forge-changeset/tests/changeset.rs::tampered_changeset_is_refused` |
| Unsafe file types and `.git` metadata are refused | `sea-forge-changeset/src/lib.rs` | `sea-forge-changeset/tests/changeset.rs::unsafe_file_types_and_git_metadata_are_refused` |
| Promotion into a checked-out branch is refused | `sea-forge-changeset/src/lib.rs` | `sea-forge-changeset/tests/changeset.rs::promotion_refuses_a_checked_out_branch` |
| An OPA candidate evaluated over another policy's Rego bundle is refused | `sea-forge-authority/src/opa.rs` (`is_bound_to`); `sea-forge-authority/src/lib.rs` (unbound candidate: Deny) | `sea-forge-authority/tests/opa_candidate.rs::candidate_from_another_bundle_is_denied` |
| OPA failure never allows | `sea-forge-authority/src/opa.rs` (every failure becomes a Deny candidate); `sea-forge-authority/src/lib.rs` (no candidate: Deny if required, else Escalate) | `sea-forge-authority/tests/opa_candidate.rs::opa_not_configured_or_unavailable_never_allows`; `sea-forge-authority/tests/opa_candidate.rs::stub_nonzero_exit_fails_closed`; `sea-forge-authority/tests/opa_candidate.rs::stub_timeout_fails_closed`; `sea-forge-authority/tests/opa_candidate.rs::stub_garbage_stdout_fails_closed`; `sea-forge-authority/tests/opa_candidate.rs::stub_missing_binary_fails_closed` |
| A local OPA deny cannot be overridden by an allow | `sea-forge-authority/src/lib.rs` (`resolve_candidates`: Deny wins) | `sea-forge-authority/tests/opa_candidate.rs::real_opa_allow_cannot_override_local_deny` (requires `opa` 1.21.1; ignored by default, run by the `authority-opa` CI job with `--include-ignored`) |
| An edited or extended Rego bundle is refused | `sea-forge-authority/src/opa.rs` (per-file hash against `provenance.json`; exact file set) | `sea-forge-authority/tests/opa_candidate.rs::edited_rego_is_tampered`; `sea-forge-authority/tests/opa_candidate.rs::extra_file_under_bundle_is_tampered` |
| Real `opa` 1.21.1 agrees with the fixture outcomes | `sea-forge-authority/src/opa.rs` with the pinned binary | `sea-forge-authority/tests/opa_candidate.rs::real_opa_matches_each_fixture_outcome` (requires `opa` 1.21.1; ignored by default, run by the `authority-opa` CI job with `--include-ignored`) |
| Real isolation: network, filesystem and process containment inside a MicroVM | OpenShell and its MicroVM driver | UNPROVEN: needs live MicroVM |
| Live timeout enforcement, cancellation and cleanup inside a MicroVM | OpenShell adapter and gateway | UNPROVEN: needs live MicroVM |

---

## 11. Live proof

`scripts/openshell/live-proof.sh` is the only script that starts a real gateway and sandboxes for the live proof. Its exit codes are:

| Exit code | Meaning |
|---|---|
| `0` | The live tests ran and passed. This is the only success. |
| `1` | A check or test failed. The script prints `FAIL: <reason>`. This includes a non-zero `cargo test` status. |
| `3` | A prerequisite is missing. The script prints `BLOCKED: <reason>` and starts nothing further. Examples: `openshell`, `openshell-prover` or `openshell-gateway` is not the pinned version; `openshell-driver-vm` is not found; `/dev/kvm` is not readable; a required host tool is missing; the live test file is not present. |

On exit the script stops the gateway it started, removes its registration, and keeps the original exit code. On a host without `/dev/kvm` it stops at the preflight and starts nothing.

Operator actions on the development host, before a live run. Tooling in this repository does not perform them:

1. **Enable nested virtualization for WSL2.** In `%UserProfile%\.wslconfig`, add `[wsl2]` and `nestedVirtualization=true`. Run `wsl --shutdown` and start WSL again. Confirm that `/dev/kvm` exists and is readable and writable by the account that runs the gateway. The CPU and the Windows build must support nested virtualization.
2. **Install the pinned OpenShell 0.1.3 binaries** as described in section 4, and confirm each `--version` output.
3. **Install e2fsprogs 1.43 or newer, the system `ssh` client and `git`.** Check with `openshell-gateway config preflight` (section 5).
4. **Stop any container engine** that the script would detect, or unset `DOCKER_HOST`. Otherwise the script refuses unless `ALLOW_CONTAINER_SOCKET=1` is set.
5. **Provide the image registry access** that the default images need (`IMAGE`, `SECOND_IMAGE`).
6. The live tests are in `crates/sea-forge-case-runner/tests/cell_episode_live.rs` (six tests, all ignored unless `SEA_FORGE_OPENSHELL_LIVE=1`; each panics if the gateway environment is missing). They compile and have never executed. The timeout test drives the sandbox adapter directly with a 5 second limit, because the projection's `exec_timeout_seconds` is 1800 and the episode has no per-request override; it expects `ExecutionStatus::TimedOut`.

Until steps 1 to 6 are done, the live proof stays BLOCKED or UNPROVEN, and no claim about real isolation, real timeouts or real image preparation is made.

---

## 12. Related

* [ADR-009: Semantic Cell execution](../decisions/ADR-009-semantic-cell-execution.md): the decisions this subsystem implements.
* [Sandboxing & Runtime Execution](sandbox-runtime.md): the local and jail backends and the general runtime seam.
