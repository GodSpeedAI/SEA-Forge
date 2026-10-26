# Current status

**Status revision:** 179

**Stage:** Cross-platform native execution: Windows control plane, OpenShell capability checks, macOS App Sandbox jail, native packages (integration)

**Summary:** feat/cross-platform-native integrates the Windows port (named pipes with kernel-backed SID identity, Job Objects, owner-only DACLs; all unsafe confined to the Windows-only sea-forge-os crate), OpenShell enforcement-capability and typed OCI/native workload identity checks driven by the locked projection's compute_driver, the jail class on macOS via App Sandbox, and .deb/.pkg/.dmg/.exe release packages.

**Verified:**

- Portability windows lane: workspace check, sea-forge-os/core/changeset/server lib tests, release build, identity E2E 10/10 PASS (run 40260946741)
- macOS App Sandbox jail escape tests 7/7 on macos-latest (run 40258556091)
- package.yml linux/macos/windows install tests green (run 40257119484)
- just no-async-kernel ok; only new lockfile package is sea-forge-os

**Limits:**

- Integration merge not yet CI-verified as a whole
- WSLc blocked: no container-to-host route, no netns, no cap-drop flags
- MXC native path needs a DomainForge mxc projection; OpenShell Windows build in progress
- macOS MicroVM blocked on hosted runners (kern.hv_support=0)
- Packages are unsigned

**Next move:** Run the full CI on the integration PR, then the Linux live OpenShell proof with the regenerated projection, conformance matrix, independent review.

**Evidence:**

- `.agents/plans/2026-10-25-cross-platform-native-execution.md`
