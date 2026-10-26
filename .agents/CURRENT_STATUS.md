# Current status

**Status revision:** 188

**Stage:** capability-harvest: round-3 fixes committed — cgroup.kill pidfd degrade, absolute-target anchored rename, C cage helper, proxy-CONNECT egress control; pushing for CI round 3

**Summary:** Round 2 named every residual. Round 3: (1) cgroup.kill write failures degrade to the membership-scoped pidfd sweep (delegation containment applies to the kill file beyond procs delegation); (2) the Windows rename keeps the handle-identified source and switches to an absolute \??\ target with a revalidate() immediately before the rename — the relative-target form is recorded as a platform-verdict spike (both class-10-Win32 and class-65-NT rejected ERROR_INVALID_PARAMETER on Server 2025); (3) the macOS cage helper is now a C program compiled at test time against libSystem only (the nix-linked Rust helper was correctly blocked by the cage from loading /nix/store dylibs); (4) the F17 allow/deny pair discovers the guest egress proxy and tunnels HTTP CONNECT through it — a granting proxy must yield exactly one token arrival, a refusing proxy must yield a refusal marker with an unchanged arrival count, and no-proxy produces the honest BLOCKED evidence pack. All payloads were functionally simulated off-lane against a real CONNECT proxy.

**Verified:**

- Full local sweep green: sandbox lib+all tests, case-runner non-ignored suites, openshell_stub 35, clippy -D warnings, windows-msvc and apple-darwin cross-targets, fmt
- F17 payloads simulated end-to-end with a fake CONNECT proxy: allow grant path (200+arrival), deny refuse path (403, no arrival), deny leak path (asserts fire), no-proxy paths

**Limits:**

- All four fixes await their CI lanes: cgroup kill drain, windows rename verdict, macOS helper execution, KVM proxy discovery

**Next move:** Push, re-dispatch openshell-live, collect round 3, then freeze for the codex adversarial review.

**Evidence:**

- .tmp/harvest/
- .agents/OBSERVED_DEBT.md
- gh runs 40369591334 / 40384807768 / 40384823244
