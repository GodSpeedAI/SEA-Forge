# Current status

**Status revision:** 140

**Stage:** security-round4-broker-caller

**Summary:** Root cause of the 24.04 postgres-lifecycle refusal: the supervisor copied the trapped socket descriptor (pidfd_getfd) and aimed race-class kills at the helper's direct child, not at the process that trapped. The lifecycle workloads are /bin/sh scripts whose psql children connect, so descriptor N was taken from the shell: the copy failed or named an unrelated shell descriptor, and the granted connect was refused. That is also a containment defect, since the broker could act on a descriptor the caller never named. Each notification now resolves its trapped thread to its thread group, holds a pidfd, and checks SECCOMP_IOCTL_NOTIF_ID_VALID after opening it so a recycled pid cannot stand in for the caller; descriptor copies and SIGKILLs (via pidfd_send_signal) go to that process, and an unresolvable caller is refused. A new unix_socket_reach test covers a granted connect from a child process and from a non-main thread.

**Verified:**

- cargo clippy -p sea-forge-sandbox --all-targets -D warnings clean; sandbox unit tests 11 pass
- unix_socket_reach passes locally only vacuously (isolation unavailable on this host); the 24.04 kernel-security run is the real check

**Limits:**

- Unverified until the 24.04 kernel-security run for this commit passes; the datagram test still passes on file permissions alone. See revision 113 for the other round-4 open limits.

**Next move:** Kernel-security 24.04 run for this commit; then tighten the datagram test.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
