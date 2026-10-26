---
name: governed-runtime-probing
description: Produce runtime enforcement evidence for a SEA-Forge execution backend (OpenShell VM/MXC/WSLc, Linux jail/isolated, macOS App Sandbox) by attempting forbidden side effects and observing them from outside the sandbox. Trigger when claiming a backend enforces filesystem, network, credential, process or lifecycle restrictions on a real host or CI runner.
---

# Governed runtime probing

Policy proof (`openshell-prover`, schema checks) says what a policy allows. This
procedure establishes what the running mechanism actually prevents. Keep the two
in separate result columns.

## Steps

1. Record the host first: OS, build/kernel, arch, backend binary versions and
   sha256, commit SHA under test, and whether the runner is a VM.
2. Probe prerequisites explicitly (e.g. `/dev/kvm`, `wxc-exec --probe`,
   `wslc version`, `codesign -d --entitlements -`). A missing prerequisite
   yields status `BLOCKED <exact prerequisite>`; stop that backend there.
3. Create a fresh fixture per run: random workspace dir, random canary
   outside it, synthetic credential `SFTEST-<random>` (never a real secret),
   and a local listener the sandbox must not reach.
4. For each scenario run the attempt inside the backend and check the effect
   from the host side:
   - write outside workspace → canary file unchanged/absent (hash before/after)
   - symlink/junction/hard-link escape → target unchanged
   - unauthorized network (TCP, UDP, DNS, ICMP where possible, host loopback,
     cloud metadata IP) → host listener logs no connection
   - credential read → synthetic secret absent from sandbox output and from
     any listener capture
   - kill/cancel → no surviving descendant (process list by job/cgroup/pgid)
   - backend failure (stop gateway, remove binary) → SEA-Forge refuses; no
     host process ran (check a host-side marker)
5. Classify each scenario: `PASS` (attempt observed blocked), `FAIL`
   (side effect observed), `BLOCKED` (prerequisite missing), `N/A` (backend
   does not claim the property, with the reason). A skip is never `PASS`.
6. Repeat the essential scenarios (fs write, network, credential, lifecycle)
   at least three times with a different workspace, principal or policy.
7. Save raw stdout/stderr and exit codes per attempt as JSON lines; publish
   them as CI artifacts or under `~/projects/_wt/containers/evidence/`.

## Failure modes

- Asserting only the sandbox's exit code: an escape can exit non-zero.
- Testing TCP only and claiming "network denied".
- Running the probe on the host by accident when the backend is missing;
  every attempt must prove it ran inside (print a backend-specific marker).
- Reusing one workspace across repetitions, hiding state leaks.
- Counting mock-driver runs as enforcement evidence.

## Verify

Every claimed-supported cell in the capability matrix links to a raw log with
a host-side observation and the commit SHA under test; every other cell reads
`BLOCKED`, `FAIL` or `N/A` with a reason.
