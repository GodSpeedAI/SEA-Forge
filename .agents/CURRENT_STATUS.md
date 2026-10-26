# Current status

**Status revision:** 180

**Stage:** cross-platform native execution: review pass 2 fixes merged on feat/cross-platform-native (PR #67)

**Summary:** Windows control plane on named pipes with SID identity (Windows FFI unsafe in sea-forge-os; the two pre-existing Linux sandbox unsafe sites are unchanged), OpenShell driver verification from gateway info, projection snapshot bound to the approved identity, macOS Jail via App Sandbox, Linux microVM live proof (OpenShell 0.1.3 vm driver, 9/9), and .deb/.pkg/.dmg/.exe packages that run without Nix.

**Verified:**

- PR #67 CI green on Linux, macOS, Windows incl. packaging
- OpenShell live proof 9/9 (runs 40286201426, 40306124770)
- upstream MXC e2e 4/4 on Windows 11 26300.9807
- clean ubuntu:22.04 and debian:12 .deb install

**Limits:**

- Local is trusted host execution on every OS
- Linux Jail network rules are TCP-only
- setsid descendants outlive Local/Jail runs (review F08 open)
- macOS OpenShell vm blocked on hosted runners (no HVF)
- SEA-Forge on MXC and WSLc blocked
- no positive control for OpenShell egress denial

**Next move:** independent review pass 3, then merge PR #67 and release

**Evidence:**

- `.agents/plans/2026-10-25-cross-platform-native-execution.md`
- `~/projects/_wt/containers/review/`
