# Windows Sandbox Backend Architecture Review

Date: 2026-07-31
Scope: repository and Microsoft-source analysis only; no implementation or native WSLC execution
Microsoft source baseline: `microsoft/WSL` tag `2.9.3`, commit `9c8821651512f4d25516935d792a4ac91b5cc19f`

## Executive decision

Use a **worker-declared, grant-bound backend**, not one universal Windows sandbox:

| Worker class | Backend | Decision |
|---|---|---|
| Declared Linux/OCI worker | WSLC session plus Linux container | Preferred Windows backend after WSLC reaches GA and passes SEA Forge native conformance. During the public preview it is evaluation-only, not production-ready. |
| Declared native Windows worker | Standard AppContainer plus Job Object and staged workspace | Required native backend. Implement after the WSLC evaluation slice unless a native worker is needed earlier. |
| Hostile, undeclared, or backend-incompatible workload | No implemented backend; future dedicated MicroVM | Refuse today. A MicroVM is the recommended direction, but no implementation or stronger-boundary claim exists until its hypervisor, guest, devices, sharing, lifecycle, and tests are defined. |
| Trusted local maintenance | Existing `local` class | Non-isolating and never an automatic fallback from `jail` or `microvm`. |

**Recommendation:** Package each governed worker as either an immutable OCI image pinned by digest or a declared native Windows bundle with a dependency/resource manifest. Bind the package identity, exact backend, entrypoint, argv, working directory, environment, isolation configuration, mounts, network policy, limits, and output contract into the authority grant. Refuse execution if the selected backend cannot establish that exact contract.

**Verified from Microsoft source or documentation:** WSLC 2.9.3 is a public preview. Its public header says the API may break and must not be relied on for production; the API reference says to evaluate feasibility now and deploy production code only after GA, then targeted for fall 2026 (`src/windows/WslcSDK/wslcsdk.h:9-16`).[^wslc-api][^wslc-announcement]

**Recommendation:** Do not advertise production WSLC support while that statement remains. A Microsoft employee's stated intent to support production later does not supersede the current preview contract.

**Uncertainty requiring native testing:** No WSLC command or SDK test was run. Host-version behavior, generated OCI defaults, crash cleanup, network isolation, filesystem edge cases, and worker compatibility remain unverified on a real Windows host.

## Evidence labels

| Label | Meaning |
|---|---|
| **Verified from repository** | Directly established by SEA Forge source, tests, or normative specification. |
| **Verified from Microsoft source or documentation** | Directly established by Microsoft documentation or the pinned WSL 2.9.3 source snapshot. |
| **Inference** | A conclusion derived from verified evidence but not directly proved by a relevant native test. |
| **Recommendation** | Proposed SEA Forge architecture or policy. It is not implemented. |
| **Uncertainty requiring native testing** | A material runtime property that source inspection alone cannot establish. |

## 1. Repository architecture and constraints

### 1.1 Authority precedes execution

**Verified from repository:** SEA Forge's core invariant is that authority decides before side effects and remains separate from sandbox enforcement (`AGENTS.md`; `.agents/specs/spec-minimum.md:720-743`). The runtime receives a move-only `ActionGrant`, derives sandbox class and network posture from it, consumes authorization, and only then prepares and executes the backend (`crates/sea-forge-authority/src/lib.rs:80-273`; `crates/sea-forge-runtime/src/lib.rs:7-66`).

```text
typed operation
  -> evaluate every authority request
  -> commit every authority decision
  -> require all Allow
  -> mint exact ActionGrant
  -> select the grant-bound backend
  -> prepare isolation
  -> execute and capture evidence
  -> settlement accepts or rejects the declared outcome
```

**Verified from repository:** Runtime backend selection comes from `ActionGrant`, not from a worker's preference. The planner also carries `PlanItem.sandbox_class`, but that value is not independently bound into `AuthorityEvaluation` (`crates/sea-forge-planner/src/case_engine.rs:235-247`; `crates/sea-forge-authority/src/lib.rs:1711-1842`).

**Recommendation:** Remove this ambiguity before Windows implementation. Authority must bind one canonical backend requirement and its configuration. A worker declaration proposes compatibility; it never authorizes itself.

### 1.2 Current sandbox contract

**Verified from repository:** `SandboxClass` is `local`, `jail`, or `microvm`. `SandboxSpec` currently carries only workspace root, artifacts root, and `NetworkPosture::{Denied, AllowTcpPorts}` (`crates/sea-forge-sandbox/src/lib.rs:48-105`). It does not represent read-only inputs, package identity, child policy, inherited resources, memory/CPU/process limits, output quota, or destination-aware egress.

**Verified from repository:** Policy validation rejects `network` as an unsupported boundary dimension while `ActionGrant::network_tcp_ports` attempts to consume it (`crates/sea-forge-authority/src/lib.rs:110-129,1264-1280`). Network grants therefore are not presently a coherent public contract.

**Recommendation:** Do not expose Windows networking until a typed policy is validated, hashed, granted, and compared at execution.

### 1.3 Current platform behavior

**Verified from repository:** Linux `JailSandbox` applies Landlock immediately before direct argv spawn and fails closed if enforcement is unavailable (`crates/sea-forge-sandbox/src/jail.rs:38-121,297-413`). It permits read access beneath `/` while restricting writes to workspace/artifacts, so it is not default-deny read isolation.

**Verified from repository:** No macOS Seatbelt backend exists. Non-Linux jail creation and execution are rejected (`crates/sea-forge-sandbox/src/jail.rs:191-245`; `crates/sea-forge-sandbox/tests/conformance_m1.rs:476-501`).

**Verified from repository:** Generic commands and ACP agents use direct tokenized argv with explicit environments; ACP shell executables are rejected (`crates/sea-forge-sandbox/src/local.rs:24-78`; `crates/sea-forge-agent/src/config.rs:63-95,175-235`; `crates/sea-forge-agent/src/acp.rs:291-339`).

**Verified from repository:** Unix process groups provide partial lifecycle containment. On non-Unix ACP cleanup kills only the direct child. Normal direct-child exit does not prove that all descendants exited (`crates/sea-forge-sandbox/src/local.rs:59-107`; `crates/sea-forge-agent/src/acp.rs:163-210`).

**Verified from repository:** Full Windows product support also requires replacing Unix socket transport and peer credentials and applying a private Windows DACL to transcript keys (`crates/sea-forge-server/src/lib.rs:924-1005`; `crates/sea-forge-server/src/identity.rs:48-83`; `crates/sea-forge-server/src/transcript_seal.rs:129-146`). A sandbox prototype alone does not make the product Windows-ready.

## 2. Worker compatibility contract

### 2.1 Required declaration

**Recommendation:** Registration must reject an undeclared worker. The minimum declaration is:

| Field | OCI/Linux worker | Native Windows worker |
|---|---|---|
| Package identity | Registry plus immutable manifest digest | Signed/hashed bundle identity and executable-relative path |
| Backend | `wslc` | `appcontainer` |
| Entrypoint | OCI argv and image working directory | Bundle-relative executable and argv |
| Runtime user | Required non-root UID/GID unless explicitly justified | Per-run AppContainer identity |
| Files | Read-only declared inputs; writable workspace/artifacts/temp | Read-only staged inputs; writable staged workspace/artifacts/temp |
| Children | Denied or bounded count | Denied or bounded Job membership |
| Network | Denied or typed broker/direct policy | Denied or typed broker/direct policy |
| Limits | Wall time, VM/container memory/CPU, process/ulimit policy, total session-storage bytes, writable-volume bytes, stream/output bytes | Wall time, Job memory/CPU/process policy, writable-volume bytes, stream/output bytes, and no other writable profile path |
| Outputs | Relative paths, type/count/size constraints | Relative paths, type/count/size constraints |
| Compatibility | WSLC version/image/platform requirements | DLL, registry, COM, font, JIT, plugin, mitigation requirements |
| Protocol/outcome | ACP limits/permissions, evidence requirements, settlement criteria, required cleanup disposition | ACP limits/permissions, evidence requirements, settlement criteria, required cleanup disposition |

**Recommendation:** A mutable tag such as `latest`, arbitrary host `argv[0]`, or recursive access to an installed runtime tree is not a worker package.

### 2.2 Why declaration is necessary

**Verified from repository:** Current `ExecuteCommand` accepts arbitrary authorized argv and cwd. Configurable ACP workers may be native binaries, interpreters, JIT runtimes, or tools that spawn helpers (`crates/sea-forge-core/src/types.rs:145-155`; `crates/sea-forge-agent/src/config.rs:63-95`).

**Inference:** AppContainer compatibility cannot be guaranteed for arbitrary installed Win32 programs because executable/DLL trees, registry, COM, fonts, certificate stores, plugins, and profile files vary. WSLC cannot safely infer an OCI image, runtime user, capabilities, or output contract from arbitrary argv either.

**Recommendation:** Keep arbitrary argv only for trusted `local` execution. Governed Windows `jail` execution must reference a registered package.

## 3. WSLC architecture and controls

### 3.1 Session and VM boundary

**Verified from Microsoft source or documentation:** Each newly created WSLC session receives a single-use per-user `wslcsession.exe`, a newly constructed `HcsVirtualMachine`, a fresh VM GUID/HCS compute system, and a distinct Linux kernel instance (`src/windows/service/exe/WSLCSessionManager.cpp:279-292`; `src/windows/wslcsession/main.cpp:74-89`; `src/windows/service/exe/HcsVirtualMachine.cpp:78-99,313-332`).

**Verified from Microsoft source or documentation:** Each session starts its own containerd and dockerd inside that VM (`src/windows/wslcsession/WSLCSession.cpp:389-405,621-667`). The packaged kernel, initrd, modules, base system VHD, privileged SYSTEM service, plugin manager, and host networking authorities remain shared host infrastructure.

**Verified from Microsoft source or documentation:** The base system VHD is read-only with an ephemeral writable overlay. Session Docker state is either tmpfs or a caller-selected `storage.vhdx` mounted at `/var/lib/docker` (`src/windows/wslcsession/WSLCSession.cpp:437-520`; `src/linux/init/WSLCInit.cpp:678-771`). A persistent VHD intentionally retains images, containers, networks, and volumes across sessions.

**Inference:** A per-run, non-persistent SDK session provides a materially stronger boundary than a Linux namespace container alone because each session owns a separate utility VM/kernel. It is not a separate physical-host or administrative trust domain: SYSTEM, administrators, HCS, WSL service code, shared host networking, and shared kernel/base artifacts remain trusted.

**Recommendation:** SEA Forge should create one non-persistent WSLC session per run with a unique supervisor-owned storage directory. Never use the CLI default persistent session and never share a writable storage path between runs.

### 3.2 Public API shape and gaps

**Verified from Microsoft source or documentation:** The public contract is layered `Session -> Container -> Process` and supports C, C++, and C# projections.[^wslc-api] Session settings expose VM CPU, memory, timeout, VHD, and feature flags. Container settings expose image, process, network none/bridged, ports, bind volumes, named VHD volumes, auto-remove, and GPU.

**Verified from Microsoft source or documentation:** The public SDK does **not** expose container user selection, capability add/drop, seccomp profile, `no-new-privileges`, PID/IPC/user namespaces, full rootfs read-only, generic devices, per-container memory/CPU/ulimits, or arbitrary file copy (`src/windows/WslcSDK/wslcsdk.h:148-260,327-336`). Some CPU/memory/ulimit and user controls exist in the internal COM/CLI surface only (`src/windows/service/inc/wslc.idl:153-160,262-266`).

**Verified from Microsoft source or documentation:** The public `Privileged` flag is accepted but stripped by the public-to-internal conversion mask; the internal container flags have no corresponding bit (`src/windows/WslcSDK/wslcsdk.cpp:50-58,76-81,938-946`; `src/windows/service/inc/WSLCShared.idl:94-105`). It is a no-op in 2.9.3.

**Recommendation:** Integrate through the public SDK, not internal COM or CLI parsing. Missing security controls are release blockers unless the immutable image and native tests prove acceptable effective defaults. Do not depend on undocumented internal interfaces.

### 3.3 OCI security profile

**Verified from Microsoft source or documentation:** WSLC sends a Docker Engine API 1.44 `CreateContainer` request to bundled dockerd; Docker/containerd/runc generate the OCI bundle (`src/windows/inc/docker_schema.h:5-14,220-244,285-311`; `src/windows/wslcsession/DockerHTTPClient.cpp:274-283`). WSLC source does not define the exact generated capability set, seccomp profile, namespace set, device rules, masked/read-only paths, or `noNewPrivileges` value.

**Verified from Microsoft source or documentation:** If no internal user is set, the image's configured user applies; absent an image user, the conventional Docker result is UID 0. The public SDK cannot override it (`src/windows/wslcsession/WSLCContainer.cpp:1470-1473`; `src/windows/WslcSDK/wslcsdk.cpp:212-228`). Container rootfs is writable by default because no `ReadonlyRootfs` control is projected.

**Recommendation:** Require the image configuration to declare a numeric non-root user and verify the effective UID/GID before accepting the backend. Reject images that require root unless a separately reviewed worker profile explicitly permits it. Do not claim a fixed seccomp/capability/no-new-privileges profile until native inspection records the generated OCI spec for the supported WSL version.

### 3.4 Files and volumes

**Verified from Microsoft source or documentation:** Public bind mounts use VirtioFS. Windows paths are canonicalized; file mounts share the parent directory and then bind the file inside Docker. Read-only is propagated through the host share, VirtioFS device, guest mount, and Docker bind (`src/windows/wslcsession/WSLCContainer.cpp:1523-1565`; `src/windows/wslcsession/WSLCVirtualMachine.cpp:1044-1133`; `src/windows/service/exe/HcsVirtualMachine.cpp:594-650`).

**Verified from Microsoft source or documentation:** VirtioFS devices cannot be removed dynamically, so shares remain attached to the session after guest unmount (`src/windows/wslcsession/WSLCVirtualMachine.cpp:1141-1164`). Public named volumes are separate ext4 VHDs and may be attached read-only (`src/windows/wslcsession/WSLCVhdVolume.cpp:122-200`).

**Recommendation:** Copy approved inputs once into a fresh supervisor-owned staging tree, then mount only that tree read-only. Mount fresh workspace, artifacts, and temp trees read-write. Never mount the repository root, `.sea-forge`, user profile, arbitrary source parent, Docker/WSLC storage root, or governance records.

**Recommendation:** The trusted copy-in algorithm must open and validate the final source object once, retain that handle, and copy bytes from that same handle. It must not validate one object and reopen the path. Reject reparse points, named streams, device/UNC namespaces, unsupported remote filesystems, case-fold collisions, and ambiguous hard-link situations. Trusted staging parents must be non-writable by the worker.

**Recommendation:** Run Windows sandbox preparation and launch under one dedicated, non-interactive SEA Forge service identity whose Windows profile and `%TEMP%` live on a hard-bounded service volume. This is required because WSLC always collects Linux crash dumps into the calling user's `%TEMP%\wslc-crashes`; the public API exposes notification, not a disable or byte-limit control (`src/windows/wslcsession/WSLCVirtualMachine.cpp:277-289,1296-1384`). Account for that shared crash-dump budget in admission and reject new runs when capacity cannot be reserved.

Bound every other writable store while the worker runs, not only during collection. For WSLC, grant and reserve a hard maximum for the session storage VHD, which contains writable image/rootfs/container state, plus a separate size-limited VHD-backed workspace/artifact/temp volume. For AppContainer, make the generated profile tree non-writable by the worker, redirect required writable home/temp/cache locations to a dedicated per-run bounded Windows volume, and reject the backend if native tests find any other writable profile/system path. Stream stdout/stderr through supervisor writers that stop at their granted byte limits. Reserve the total worst-case allocation before launch. The exact Windows volume/profile mechanism is a native-test gate. Neither WSLC nor a Windows Job Object supplies a host output-disk quota.

**Recommendation:** After the entire container/session process tree is stopped, enumerate from trusted directory handles. Open each candidate output once without following reparse points, validate its file identity, link count, stream/type/size, and containment from that handle, retain the handle, and copy bytes from that same handle. Never validate an output object and reopen its path. Enforce file count and aggregate byte limits while copying, then publish only declared outputs.

### 3.5 Networking

**Verified from Microsoft source or documentation:** WSLC has a session VM network mode (`none`, NAT, or Consomme) and a separate Docker container mode (`none`, bridge, host, container, or custom bridge). The public SDK hard-codes session Consomme with DNS tunneling and exposes only container `none` or bridged (`src/windows/WslcSDK/wslcsdk.cpp:420-444,750-759`; `src/windows/WslcSDK/wslcsdk.h:79-83`).

**Verified from Microsoft source or documentation:** Consomme translates guest traffic into ordinary Windows host sockets and therefore follows host routing, firewall, VPN, proxy, and enterprise integration. Published ports can bind loopback, wildcard, or a host-adapter address through internal/CLI controls (`doc/docs/technical-documentation/localhost.md:14-20`; `src/windows/common/ConsommeNetworking.cpp:91-194`).[^consomme]

**Verified from Microsoft source or documentation:** Container network `none` has no `eth0` and rejects port publishing in Microsoft tests (`src/windows/wslcsession/WSLCContainer.cpp:193-197`; `test/windows/WslcSdkWinRTTests.cpp:698-706`). Session mode `none` also avoids GNS/port relay, but it is not selectable through the public SDK.

**Verified from repository:** Current Landlock `NetworkPosture::Denied` denies handled TCP connect and bind operations; it does not define a portable ban on all local socket creation (`crates/sea-forge-sandbox/src/lib.rs:48-66`; `crates/sea-forge-sandbox/src/jail.rs:80-97`). WSLC container network `none` removes external container networking but does not prove that a process cannot bind or listen on Linux loopback.

**Recommendation:** Resolve that semantic mismatch before granting WSLC execution. Define a typed `ExternalNetworkDenied` posture for no host/LAN/Internet connectivity and a separate `NoTcpEndpoints` posture when TCP bind/connect themselves must fail. Initial WSLC support may accept only `ExternalNetworkDenied`, use public container network `none`, publish no ports, expose no broker, and enable no GPU. It must verify absence of `eth0`, failed TCP/UDP/IPv4/IPv6/DNS/host/LAN access, and zero external observations. A grant requiring `NoTcpEndpoints` is incompatible with WSLC unless an additional proved control enforces it.

**Recommendation:** Destination-specific access should use a narrow trusted protocol broker outside the worker. Do not claim exact host/port enforcement from bridged mode, Consomme, Windows Firewall defaults, or broad WFP guidance. A privileged WFP component is not recommended for the initial design; consider it only after a concrete direct-socket requirement, separate threat model, service lifecycle design, and native proof that filters bind to the exact per-run identity without widening authority.

### 3.6 Images and provenance

**Verified from Microsoft source or documentation:** Pull accepts tag or digest and defaults an unqualified reference to `latest`. WSLC forwards it to Docker. A machine registry allowlist can restrict registry names, but an absent or empty allowlist permits all registries (`src/windows/wslcsession/WSLCSession.cpp:832-858`; `src/windows/inc/wslpolicies.h:159-190`).

**Verified from Microsoft source or documentation:** WSLC implements no signature, publisher, cosign/Notary, attestation, SBOM, mandatory digest, or resolved-manifest binding policy. Public `WslcImageInfo.sha256` is populated from the local image ID, not the repository digest (`src/windows/WslcSDK/wslcsdk.cpp:1563-1601`).

**Recommendation:** SEA Forge must resolve and approve an immutable manifest digest before granting execution, pull only from an approved registry, inspect the local image after pull, and bind the resolved image identity into the grant and evidence. Signature/provenance verification belongs above WSLC; the registry allowlist alone is insufficient.

**Uncertainty requiring native testing:** Tag-to-manifest races, multi-architecture selection, local image-ID/reporting behavior, and any downstream Microsoft image verification not represented in source.

### 3.7 Lifecycle and cleanup

**Verified from Microsoft source or documentation:** Releasing the last strong reference to a non-persistent session triggers `Terminate`; a persistent/default CLI session survives reference release (`src/windows/wslcsession/WSLCSession.cpp:420-430`; `test/windows/WslcSdkTests.cpp:296-318`; `test/windows/WSLCTests.cpp:9424-9445`).

**Verified from Microsoft source or documentation:** Orderly termination stops dockerd/containerd, unmounts storage, releases the VM, and signals completion. Daemon stops use bounded TERM/KILL waits; HCS VM teardown also has a force path (`src/windows/wslcsession/WSLCSession.cpp:2403-2428,2769-2914`; `src/windows/service/exe/HcsVirtualMachine.cpp:352-375`).

**Verified from Microsoft source or documentation:** The service places each session process in a kill-on-close Job and requests HCS termination on last compute-system handle close (`src/windows/service/exe/WSLCSessionManager.cpp:471-483`; `src/windows/service/exe/HcsVirtualMachine.cpp:95-99`). These mechanisms terminate processes/VMs; they do not prove graceful cleanup, data deletion, or secure erasure.

**Verified from Microsoft source or documentation:** Container launcher delete-on-close is a C++ destructor attempt whose failure is logged. Process crashes can bypass it. Auto-remove depends on daemon event processing. Persistent VHD data, some swap/VHD cleanup failures, and crash dumps can survive (`src/windows/common/WSLCContainerLauncher.cpp:27-45`; `src/windows/wslcsession/WSLCSession.cpp:452-466,2887-2909`; `src/windows/wslcsession/WSLCVirtualMachine.cpp:1296-1385`).

**Recommendation:** SEA Forge must explicitly stop/delete the container, terminate the non-persistent session, wait for the termination event, close all handles, and remove/quarantine staging and unique storage. Persist cleanup intent before creation. On startup, sweep incomplete runs and refuse reuse of any storage identity whose prior cleanup is uncertain.

**Uncertainty requiring native testing:** Bounded teardown after hard supervisor crash; absence of surviving HCS systems, sockets, endpoints, VHD handles, shares, and containers after service crash/restart; behavior under power loss; and secure deletion of residual sparse-VHD blocks or crash dumps.

## 4. AppContainer architecture

### 4.1 Role

**Verified from Microsoft source or documentation:** AppContainer is a capability-based Low Integrity boundary. Resource access requires both ordinary principal access and package/capability SID access. Without a network capability, network access is denied.[^appcontainer]

**Recommendation:** Use standard AppContainer, not LPAC, for the first native Windows compatibility profile. Pair it with a fresh per-run identity, staged workspace, explicit inherited-handle list, conservative process mitigations, and a preconfigured Job Object. The fixed SEA Forge service identity owns every profile, so the service can reacquire its own primary token after restart without storing user credentials or depending on an interactive login. Persist that owner SID and cleanup intent before profile creation; delete the profile under that identity only after the Job is empty and all profile/storage handles close. Quarantine the SID/profile and every associated resource grant if deletion is incomplete.

**Verified from Microsoft source or documentation:** Standard AppContainer retains ambient access granted through broad application-package SIDs to some system files/directories, registry keys, and COM objects. LPAC opts out of that broad surface and therefore needs explicit capabilities such as `registryRead` and `lpacCom`.[^appcontainer]

**Recommendation:** Treat that ambient standard-AppContainer surface as part of the worker compatibility/threat profile, enumerate it in native tests, and accept it explicitly per registered bundle. Defer LPAC implementation until a concrete worker needs the narrower surface and its complete resource manifest passes compatibility tests. LPAC is an alternative strict profile, never a fallback or an extra layer around standard AppContainer.

### 4.2 Process and tree containment

**Recommendation:** Obtain the fixed SEA Forge service identity's primary token and call `CreateProcessAsUserW`; impersonation alone is insufficient because `CreateProcessW` uses the caller process's primary token. Supply a fully qualified, non-null `lpApplicationName` inside the immutable bundle/staging root; encode the exact grant-bound argv into a separate mutable Windows command-line buffer; set an explicit current directory and Unicode environment; and pass `EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT | CREATE_SUSPENDED`. Never ask Windows to infer the executable from command-line text.

Use exactly these policy-derived attributes:

| Attribute/control | Requirement |
|---|---|
| `SECURITY_CAPABILITIES` | Exact fresh AppContainer SID; no network capability initially. |
| `JOB_LIST` | Assign the preconfigured Job during process creation, never launch then assign. |
| `HANDLE_LIST` | Only intended stdin/stdout/stderr and broker handles. Mark each listed handle inheritable, pass `bInheritHandles=TRUE`, and keep every unlisted handle non-inheritable. |
| Child-process policy | Deny only for workers declared childless; otherwise bound by Job process count. |
| Mitigation policy | A declared compatibility profile; no silent weakening on launch failure. |

Before resume, verify the exact AppContainer token/capabilities, Job membership, required mitigations, and suspended state. Any mismatch terminates the Job and fails closed.[^process-attributes]

**Recommendation:** Configure kill-on-close, no breakaway, process-count, memory, and CPU controls. A supervisor timer enforces wall time; a completion port tracks the process tree. Wait until the Job is empty before output collection.[^job-objects]

**Verified from Microsoft source or documentation:** Job Objects provide process-tree lifecycle and selected process/memory/CPU controls. They do not provide a filesystem access boundary or a general output/disk-byte quota.[^job-objects]

### 4.3 Files and compatibility

**Recommendation:** Use the same bounded-volume, open-once copy-in/copy-out staging contract as WSLC. Never add AppContainer ACLs to arbitrary user-selected source files. Create staging with explicit DACLs that grant the fixed service principal and exact per-run package SID only the required access. Grant read/execute to a supervisor-owned immutable registered bundle or copy the bundle into per-run staging; never launch from an arbitrary installed path. Make the generated AppContainer profile non-writable after setup and redirect every required writable path to the bounded volume. Record every resource and ACE associated with the per-run SID; remove per-run ACEs and delete the profile using the reacquired service primary token after all handles close. Quarantine and retry on any cleanup ambiguity.

**Inference:** Standard AppContainer is more compatible than LPAC but still cannot promise arbitrary installed runtime compatibility. Native worker registration must declare DLL trees, registry/COM needs, fonts, JIT/plugins, child processes, temporary storage, and mitigations.

### 4.4 Networking

**Recommendation:** Supply no network capability initially. Broad `internetClient`, `internetClientServer`, and `privateNetworkClientServer` capabilities cannot represent an exact destination allowlist.[^app-capabilities]

**Recommendation:** Use the same authority-bound protocol broker as WSLC for supported destination-specific operations. Do not install loopback exemptions. Direct arbitrary sockets remain unsupported until a separately approved enforcement design exists.

## 5. Threat and control comparison

| Property | AppContainer + Job | WSLC per-run VM + container | Hybrid selection | Dedicated MicroVM |
|---|---|---|---|---|
| Workload type | Native Windows | Declared Linux/OCI | Both, selected per package | Hostile/incompatible |
| Kernel boundary | Shared Windows kernel | Separate Linux kernel in HCS utility VM | Depends on selected backend | Separate guest kernel |
| Inner process boundary | AppContainer token | Docker/runc container inside VM | Depends on selected backend | Guest policy/runtime |
| Host resource exposure | Explicit staging plus ambient standard-AppContainer system file/registry/COM surface accepted per worker | Explicit VirtioFS staging mounts plus WSLC/HCS host services | Common staging contract; backend-specific ambient surface | Not designed |
| Default initial network | No capability; exact local-socket semantics need tests | Container network `none`; external denial only | Grant must use backend-supported semantics | Not designed |
| Process-tree control | Windows Job | Container/session/VM lifecycle | Backend-specific guard | Not designed |
| Resource controls | Job memory/CPU/process plus no writable profile path and bounded writable volume | VM CPU/memory plus bounded total session VHD and bounded writable VHD; internal container limits not all public | Grant-bound normalized limits | Not designed |
| Arbitrary native argv | No; declared bundle only | No | No | Possible after explicit policy |
| OCI image support | No | Native fit | Yes | Yes if guest supports it |
| Exact OCI security profile | N/A | Not exposed by WSLC; generated by Docker/runc | Must be evidenced | Controlled by SEA Forge guest image |
| API maturity | Stable Win32 APIs | Public preview in 2.9.3 | Limited by selected backend | Depends on chosen platform |
| Cleanup certainty | Requires durable profile/staging recovery | Several best-effort paths; requires durable storage/staging recovery | Common recovery state machine | Requires durable VM/disk recovery |
| Strong hostile-code claim | Insufficient without narrower threat model | Insufficient while preview/defaults are unverified | No | Unverified recommendation only |

**Inference:** WSLC offers a stronger kernel boundary than AppContainer for Linux workers but a less mature and less configurable public API. AppContainer offers stable Windows-native controls but shares the host kernel and has substantial native compatibility constraints. They are complementary, not interchangeable security levels.

## 6. A/B/C/D decision

| Option | Decision | Reason |
|---|---|---|
| A. AppContainer for every worker | Reject | Cannot run Linux OCI images directly and cannot honestly support arbitrary installed native argv. |
| B. WSLC for every worker | Reject | Runs Linux containers, not native Windows workers; public API is preview and omits material OCI controls. |
| C. Hybrid, package-declared backend | Select | Smallest design that supports both declared OCI/Linux and native Windows workers without weakening either boundary by fallback. |
| D. MicroVM for every worker | Reject as default; retain as an unimplemented recommendation | The repository has no backend. Hypervisor, guest image, devices, sharing, lifecycle, and proof must be designed before any stronger-boundary claim. |

**Recommendation:** `SandboxClass::Jail` remains the policy-visible native isolation requirement. The grant additionally binds `backend_kind` (`wslc` or `appcontainer`) because backend choice changes the trusted computing base and must not be an unrecorded host preference. `Microvm` remains an explicit unavailable class until a concrete implementation and proof define its strength. No class may fall back to `local`.

## 7. Proposed abstraction

**Recommendation:** Keep the public model small and hash the canonical execution contract:

```rust
struct SandboxContract {
    backend: BackendKind,
    package: PackageIdentity,
    entrypoint: String,
    argv: Vec<String>,
    working_directory: RelPath,
    environment: Vec<(String, String)>,
    inputs: Vec<ApprovedInput>,
    outputs: Vec<ApprovedOutput>,
    network: NetworkPolicy,
    children: ChildPolicy,
    limits: ResourceLimits,
    compatibility_profile: String,
    protocol: ProtocolPolicy,
    evidence: EvidenceRequirements,
    settlement: SettlementCriteria,
    cleanup: CleanupDisposition,
}

enum BackendKind { Wslc, AppContainer, Microvm }
enum PackageIdentity { OciDigest(String), NativeBundleDigest(String) }
enum NetworkPolicy {
    ExternalNetworkDenied,
    NoTcpEndpoints,
    Brokered(Vec<Destination>),
}
enum ChildPolicy { Denied, Bounded(u32) }
```

The literal Rust API may differ. The required properties are:

1. Authority hashes and grants the complete canonical contract.
2. Runtime compares the exact hash before consuming the grant.
3. Prepared state owns non-cloneable backend resources and cleanup state.
4. ACP and ordinary commands use the same owned lifecycle guard.
5. ACP limits, mediated permissions, evidence requirements, settlement criteria, and expected cleanup disposition are part of the canonical grant, not mutable downstream choices.
6. Evidence records requested/effective backend, package identity, invocation, limits, mounts, network mode, process-tree completion, output validation, and cleanup status.
7. Settlement unconditionally rejects a run when containment, required evidence, output validation, or cleanup disposition cannot be established. A grant may choose delete versus an explicitly acceptable quarantine disposition; it cannot make cleanup optional.

**Recommendation:** Do not add speculative backend-plug-in factories. One enum dispatch in `sea-forge-sandbox` is enough until a third implemented backend demands a different boundary.

## 8. Failure behavior

| Failure | Required behavior |
|---|---|
| Deny/escalate or missing exact grant | No package resolution, staging, session/profile creation, or process launch. |
| Contract hash or backend differs from grant | Reject before preparation; never substitute another backend. |
| Mutable/unresolved package identity | Reject. |
| WSLC preview not explicitly enabled for evaluation | Reject WSLC execution. |
| Required Windows/WSL/API feature unavailable | Typed unavailable error; no local fallback. |
| Unsafe/ambiguous input path | Reject before copying or mounting. |
| Staging ACL/DACL or copy validation fails | Remove/quarantine partial state; do not launch. |
| Requested network policy is not exactly enforceable | Reject; do not broaden to bridged, Internet, LAN, loopback exemption, or WFP. |
| WSLC image/effective user/security profile mismatches declaration | Delete container/session where possible, quarantine storage, reject. |
| AppContainer token/Job/handle/mitigation verification fails | Terminate suspended Job and reject. |
| Timeout/cancel cannot prove tree/session termination | Containment failure; quarantine state and reject settlement. |
| Writable-volume or stream quota cannot be enforced before launch | Reject; monitoring or post-run counting is not a quota. |
| Output validation or quota fails | Stop the run when possible, publish nothing, and reject settlement. |
| Cleanup fails or is uncertain | Persist failure, quarantine identity/storage, retry using the fixed service identity, and always reject settlement unless the exact grant permits that verified quarantine disposition. Cleanup cannot be omitted. |

## 9. Implementation phases

### Phase 0: Contracts and proof harness

1. Define the worker package manifest and canonical `SandboxContract`.
2. Bind backend, package, entrypoint, argv, cwd, environment, mounts, network semantics, limits, ACP limits/permissions, evidence, settlement, output, and cleanup disposition into authority and ACP grants.
3. Resolve the current stringly network-boundary contradiction.
4. Make prepared sandbox state non-cloneable and cleanup errors observable.
5. Build platform-neutral denial, exact-binding, lifecycle, evidence, and settlement conformance tests.

### Phase 1: WSLC evaluation spike

1. Use the public C SDK through one narrow reviewed Windows FFI boundary.
2. Require WSL 2.9.3+ only for evaluation; create one unique non-persistent session/storage root per run.
3. Pull an approved digest, create a non-root image container with network `none`, a grant-bounded session storage VHD, read-only staged inputs, a separate bounded writable VHD, auto-remove, and no GPU.
4. Run the exact grant-bound argv/cwd/environment; enforce supervisor wall time and bounded stdout/stderr writers.
5. Explicitly delete/terminate, wait, collect evidence, and exercise startup recovery.
6. Do not ship production WSLC support until GA and the native gate in section 10 passes.

### Phase 2: Native AppContainer backend

1. Add a fixed non-interactive service identity with bounded profile/temp, owner-SID-bound AppContainer profile lifecycle and restart recovery, explicit bundle/staging DACLs and ACE cleanup, a non-writable profile with redirected bounded writable volume, Job, exact `CreateProcessAsUserW` invocation, `STARTUPINFOEX`, exact handles, and process verification.
2. Support only registered native bundles that pass the compatibility matrix.
3. Keep network denied; add the common broker only for a concrete worker requirement.
4. Integrate the same lifecycle guard with ACP.

### Phase 3: Windows product completion

1. Replace Unix-only server transport and peer identity with authenticated Windows equivalents.
2. Create transcript keys and sensitive state with private DACLs.
3. Add Windows build, package, update, recovery, and native CI lanes.
4. Correct documentation that currently implies unavailable cross-platform backends.

### Phase 4: MicroVM escalation

Do not claim MicroVM availability or strength yet. When a real hostile or incompatible worker requires it, first select and threat-model the hypervisor, guest image, virtual devices, host sharing, identity, lifecycle, patching, attestation/evidence, and cleanup. Then reuse the package, staging, network, evidence, cleanup, and settlement contracts rather than inventing a second governance path.

## 10. Native validation plan

No skipped native test counts as passed.

### 10.1 Common governance tests

| Test | Required proof |
|---|---|
| Denied authority | No staging, profile/session, image pull, process, mount, or external side effect. |
| Exact grant binding | Mutating backend, package digest, entrypoint, argv, cwd, environment, input, output, network, child, limit, compatibility profile, ACP/protocol limit or permission, evidence requirement, settlement criterion, or cleanup disposition rejects before launch. |
| No fallback | Forced backend unavailability never launches under another backend or `local`. |
| Input race | Repeated rename/replacement cannot change bytes copied from the validated open handle. |
| Output escape | Every output is validated and copied through one retained file handle; reparse points, streams, hard links, namespace aliases, and parent replacement publish nothing outside staging. |
| Output exhaustion | Grant-bounded WSLC session/rootfs storage, a fixed-capacity per-run writable volume, a bounded service profile/temp/crash-dump volume, a non-writable AppContainer profile, and bounded stdout/stderr prevent host-disk growth beyond admitted capacity; post-run checks separately govern publication. |
| False success | Exit zero with missing/invalid evidence or outputs is rejected by settlement. |
| Recovery | Kill supervisor/service/host at each lifecycle step; restart identifies, terminates, deletes, or quarantines every incomplete run. |

### 10.2 WSLC tests

1. Confirm each SEA Forge run receives a distinct HCS VM/kernel, session process, storage path, Docker daemon, network namespace, and identifiers.
2. Capture the generated OCI `config.json` and effective `/proc` state: UID/GID, capabilities, seccomp, `NoNewPrivs`, namespaces, mounts, masked/read-only paths, cgroups, and devices.
3. Verify public `Privileged=true` does not widen privileges; SEA Forge must never set it regardless.
4. Verify network `none` blocks TCP/UDP external connectivity, IPv4/IPv6 DNS, host/LAN/metadata/link-local access, and published ports with zero external observations. Record whether loopback bind/listen remains possible; do not accept a `NoTcpEndpoints` grant if it does.
5. Verify read-only VirtioFS mounts against write, rename, unlink, link, metadata, xattr, and relevant Windows stream/reparse cases.
6. Verify worker cannot see source parents, repository root, user profile, another run, WSLC storage, governance records, or host secrets.
7. Verify digest requested, image ID/repository digest used, architecture selected, and evidence recorded are consistent.
8. Verify root direct-child exit does not complete the run while descendants/container processes survive.
9. Hard-kill client, supervisor, session process, WSL service, VM, and host; measure bounded teardown and enumerate residual HCS systems, jobs, sockets, endpoints, shares, VHD handles, containers, volumes, dumps, and files. Crash processes repeatedly and prove `%TEMP%\wslc-crashes` cannot exceed admitted service-volume capacity.
10. Run supported Windows/WSL/architecture/enterprise firewall/VPN/proxy matrices.

### 10.3 AppContainer tests

1. Verify exact AppContainer SID/capabilities, Low IL, Job membership, mitigations, and inherited handle set before resume.
2. Inventory the ambient standard-AppContainer file, registry, COM, object, and network surface; prove unrelated user/profile/repository resources fail and the registered worker accepts every remaining ambient system resource explicitly.
3. Prove child and grandchild processes cannot escape the Job or survive normal root exit, timeout, cancellation, or supervisor loss.
4. Exercise process-count, memory, CPU, wall-time, bounded writable-volume, and stdout/stderr limits independently.
5. Run every advertised native bundle under its declared DLL/registry/COM/font/JIT/plugin/child/mitigation profile.
6. Fault-inject profile, DACL, Job, attribute, pipe, token query, resume, output, and cleanup operations; no path launches unsandboxed.

## 11. Risk register

| Risk | Severity | Mitigation | Release blocker |
|---|---|---|---|
| WSLC preview API breaks or changes behavior | Critical | Evaluation only; pin version; re-review at GA | Yes |
| Effective OCI defaults are weaker than assumed | Critical | Capture generated OCI/effective state; immutable image; reject mismatch | Yes |
| Public SDK cannot set required user/security/resource control | Critical | Encode safe image defaults; verify; use MicroVM if insufficient | Yes |
| Consomme/bridge widens network authority | Critical | Initial `none`; broker only; no broad direct networking | Yes |
| VirtioFS or Windows path race exposes host files | Critical | Open-once copy to trusted staging; mount only staging; hostile tests | Yes |
| WSLC crash cleanup leaves VM/storage/network state | Critical | Unique identities, durable cleanup intent, recovery sweeper, quarantine | Yes |
| Image tag/provenance race | Critical | Approved immutable digest and post-pull identity verification | Yes |
| AppContainer native runtime incompatibility | High | Registered bundles and real compatibility matrix | Yes per worker |
| AppContainer child survives or escapes | Critical | Creation-time Job assignment, no breakaway, wait for empty Job | Yes |
| Output/rootfs/profile/crash dump fills host disk | Critical | Use a dedicated identity with bounded profile/temp; reserve bounded WSLC session/data VHDs; make AppContainer profile non-writable and redirect writes; bound stdout/stderr; do not rely on monitoring, post-run limits, or Job | Yes |
| Broker becomes confused deputy/SSRF path | Critical | Exact grant binding, narrow protocol, DNS/address checks, bounded I/O | Yes when broker ships |
| Backend choice is changed after authority | Critical | Canonical contract hash and exact runtime comparison | Yes |
| Cleanup errors or wrong identity leave profile/state | High | Fixed service owner, reacquirable primary token, owned lifecycle state, recorded owner SID, retry/quarantine, and propagated cleanup result | Yes |
| Full Windows product retains Unix assumptions | High | Complete transport/identity/DACL phase before support claim | Yes |
| Windows Server/ARM64/older builds differ | High | Keep unsupported until native matrix passes | Yes for each claim |

## 12. Versions and readiness

| Item | Current conclusion |
|---|---|
| WSLC source reviewed | WSL 2.9.3, public pre-release, commit `9c8821651512f4d25516935d792a4ac91b5cc19f` |
| Public SDK feature gate | Source recognizes WSLC from WSL 2.8.0; Microsoft tutorial requires 2.9.3+ for users (`src/windows/WslcSDK/wslcsdk.cpp:277-289`).[^wslc-tutorial] |
| SDK/service compatibility | WSL 2.9.3 service accepts SDK 2.9.0+ with a pre-2.9 exception (`src/windows/service/exe/WSLCSessionManager.cpp:564-580`). |
| Windows package floor | Projections declare Windows 10 19041, but this is not proof of feature equivalence. |
| Native test evidence in source | Windows 10 22H2 and Windows 11 23H2 cloud lanes; WSLC excluded on Server due teardown hang; no ARM64 cloud lane (`cloudtest/CMakeLists.txt:4-24,54-59`). |
| SEA Forge evaluation floor | Windows 11 x64 with WSL 2.9.3 pre-release, on an isolated test host only. |
| SEA Forge production floor | Undecided until WSLC GA and native qualification. Do not retain the previous unsupported Windows 11 24H2 assertion as a decision. |

Production readiness requires all of the following:

1. Microsoft marks the public WSLC API GA with a production support contract.
2. SEA Forge pins and feature-probes a supported Windows/WSL matrix.
3. Every advertised worker has an immutable declaration and passes its backend compatibility matrix.
4. Exact grant/backend/package binding tests pass.
5. Denied network and filesystem side-effect tests pass with external observation.
6. Generated OCI security state is captured and accepted for every supported WSLC version.
7. Process-tree, timeout, cancellation, service-crash, host-crash, cleanup, and recovery tests pass.
8. Output quotas and settlement reject false success.
9. Full Windows transport, identity, key DACL, packaging, update, and evidence paths pass.
10. Independent security review accepts the FFI boundaries, broker if present, and residual risk.

## 13. Final recommendation

**Recommendation:** Adopt Option C, the declared hybrid. Implement the contract and proof harness first, evaluate WSLC second, and add AppContainer when a native Windows worker is in scope. Refuse hostile or incompatible workloads until a concrete MicroVM backend is designed and proved.

The smallest complete first slice is one immutable, non-root OCI worker on one non-persistent WSLC session, network `none`, staging-only VirtioFS mounts, explicit limits/evidence, and durable cleanup recovery. It is an **evaluation slice**, not a production backend, until GA and native gates pass.

This preserves SEA Forge's invariant: authority selects the exact backend and resources before any side effect; isolation never widens that grant; execution is not success until evidence, cleanup, and settlement accept the declared outcome.

## Sources

### Microsoft documentation

[^wslc-announcement]: Microsoft, "WSL container is now available for public preview," 2026-06-29: https://devblogs.microsoft.com/commandline/wsl-container-is-now-available-for-public-preview/
[^wslc-api]: Microsoft, "WSL container API developer reference," including the preview/GA warning: https://wsl.dev/api-reference/
[^wslc-tutorial]: Microsoft Learn, "Get started with containers on WSL," including WSL 2.9.3 pre-release prerequisites: https://learn.microsoft.com/en-us/windows/wsl/tutorials/wsl-containers
[^wslc-release]: Microsoft WSL 2.9.3 pre-release notes: https://github.com/microsoft/WSL/releases/tag/2.9.3
[^consomme]: OpenVMM, "Consomme backend": https://openvmm.dev/guide/reference/backends/consomme.html
[^appcontainer]: Microsoft Learn, "Launch an AppContainer": https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer
[^process-attributes]: Microsoft Learn, `UpdateProcThreadAttribute`: https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute
[^job-objects]: Microsoft Learn, "Job Objects": https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects
[^app-capabilities]: Microsoft Learn, "App capability declarations": https://learn.microsoft.com/en-us/windows/uwp/packaging/app-capability-declarations

### Microsoft source snapshot

All `src/`, `test/`, `doc/`, `intune/`, and `cloudtest/` line citations refer to `microsoft/WSL` tag `2.9.3`, commit `9c8821651512f4d25516935d792a4ac91b5cc19f`: https://github.com/microsoft/WSL/tree/2.9.3
