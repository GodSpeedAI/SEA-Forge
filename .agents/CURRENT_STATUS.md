# Current status

**Status revision:** 181

**Stage:** Cross-platform native execution: integration ready for merge after review pass 3

**Summary:** Linux OpenShell vm live proof 9/9 (KVM, Docker stopped); Windows Local job and named-pipe control plane; Linux/macOS Jail; native installers (.deb, .exe, .pkg, .dmg). Three independent review passes done; pass-3 F27, F31, F18, F24 fixed and F32 mitigated; F05, F08, F15, F17, F33 open.

**Verified:**

- Live run 40310712495 on d28e00dc: cell_episode_live 9 passed
- Package run 40278790759 green on 3 OSes; clean ubuntu:22.04 and debian:12 .deb installs
- Portability 40308234437 green (Windows, macOS jail)
- sea-forge-sandbox, sea-forge-evidence, sea-forge-runtime tests pass locally after pass-3 fixes

**Limits:**

- F08: setsid descendants escape Local/Jail group kill; not for adversarial lifecycle containment
- F05: Windows pinned directory reparse mutation after validation
- F17: OpenShell network denial not distinguished from an unreachable target (scenario 4 BLK)
- F33: tokio unifies into sea-forge-os on Windows server builds
- F15: prerequisite skips count as libtest passes outside required CI lanes
- macOS OpenShell vm, WSLc and SEA-Forge on MXC are Blocked
- No backend meets the 12-scenario bar

**Next move:** Merge PR #67 once required checks are green; release via release-please; decide F33 crate split and F08 supervisor.

**Evidence:**

- `.agents/plans/2026-10-25-cross-platform-native-execution.md`
- `~/projects/_wt/containers/review/pass3/pass3-findings.md`
- `~/projects/_wt/containers/evidence/oslive-linux-40310712495`
