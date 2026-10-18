# Outside-sandbox Linux/WSL preflight

Captured 2026-10-16 through the approved escalated read-only probe, outside the tool sandbox. The environment is WSL2 with Linux 6.18.54.1. Unlike the earlier in-sandbox record, the process has `NoNewPrivs=0`, PID 1 is `systemd`, and the current UID/GID maps cover the full range. The earlier sandbox-only restrictions must not be reported as host restrictions.

Unprivileged user namespaces work. `unshare --map-root-user` maps namespace UID/GID 0 to host UID/GID 1000. Bubblewrap 0.11.1 successfully ran a read-only mounted `/usr/bin/gnutrue` in transient user+PID namespaces, both with and without a transient network namespace. These results establish that those namespaces are available for this probe; they do not establish distinct host UIDs for workloads, production launcher integration, filesystem policy correctness, or network mediation.

The host has configured subordinate UID/GID ranges, but `newuidmap` and `newgidmap` are absent, so util-linux `--map-auto` failed with “No such file or directory.” No distinct host UID was created or verified. The process has no effective capabilities. A trusted operator-provided mapping mechanism is still needed before SEA Forge can claim independently authenticated workloads by distinct host UID. It must establish and verify unique host-visible IDs for workload PIDs. No daemon is inherently required solely for UID separation. Cgroup v2 is mounted read/write in the namespace view, but the root and current cgroup path are owned by UID/GID 0 with mode 0755, and `os.access(..., W_OK)` is false for the current UID; no delegated writable workload cgroup was observed.

The Landlock ABI query returned 7; it only queried the version and did not install or test a policy. The bwrap probe ran only `/usr/bin/gnutrue`, with `/usr`, `/lib`, and `/lib64` mounted read-only, plus transient `/proc` and `/dev`. No workload secrets, application data, or credentials were inspected. The earlier regular-file descriptor probe remains separate and only demonstrated regular-file contents readable through host `/proc`; it did not test socket FD duplication or socket theft.

The exact commands, arguments, outputs, and safety limits are preserved in [observations.json](./observations.json). The prior sandbox-scoped raw record remains untouched at [host-preflight](../host-preflight/README.md), with a scope caveat in [scope-caveat.json](../host-preflight/scope-caveat.json).

No production implementation, adversarial validation, or security-contract completion is claimed by this preflight.
