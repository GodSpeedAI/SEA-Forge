# Workload isolation — 2026-10-17

Supersedes the blocker in `../isolation-20261016/`. The operator installed
`uidmap` 1:4.17.4-2ubuntu3 (`/usr/bin/newuidmap`, `/usr/bin/newgidmap`,
setuid root). Delegation: `sprime01:100000:65536` in `/etc/subuid` and
`/etc/subgid`. Kernel 6.18.54.1-microsoft-standard-WSL2, Yama ptrace_scope 1.

## Selected boundary

One subordinate host UID/GID per workload, single-ID map, via util-linux
`unshare` (`--map-users=1000:<sub>:1 --setuid/--setgid`, new user/pid/mount/
ipc namespaces, net namespace when network is denied). The trusted
`sea-forge-workload-exec` helper applies the existing jail Landlock posture
inside the namespace before exec. Workspace/artifact trees get a POSIX ACL
for that identity only (group/other cleared; default ACL keeps the launcher
able to observe outputs). Slots are exclusive `flock`s in a 0700 launcher
state directory. No daemon, no new privileged component.

Rejected: shared-UID rootless bwrap (demonstrated fd theft, 2026-10-16);
`--map-auto` ranges (one workload could hold many IDs, including another
workload's); OpenShell (not configured on this host).

## Observed (host-probe.sh and the captured Rust tests)

Attacker at a different subordinate UID, with and without pid/mount
namespaces, against a victim holding a secret on fd 9:
`/proc/<pid>/fd/9`, fd listing, environ, mem, `kill -0`, `nsenter -U`,
`unshare --map-users=0:<victim>:1`, `unshare --map-auto`, the victim's
workspace file and a launcher 0600 file: all denied. Positive controls:
victim reads its own fd; attacker writes its own workspace; launcher reads
workload output through the default ACL.

Mutation controls (run manually, restored, not committed):

- `shared_identity_mutation_is_detected` (committed test): attacker reusing
  the victim's identity gets fd listing, signal and file theft — VULNERABLE.
- `conformance_m16::t16_4b` with the policy class changed from `isolated` to
  `jail`: the ACP fixture detects it runs as the launcher UID, disconnects,
  and settlement is Rejected (test fails as required).

## Limits stated explicitly

- The namespace owner (launcher UID) can enter workload namespaces; the
  launcher is the trusted control plane. Untrusted agents must never run as
  the launcher UID (classes `local`/`jail` still do; require `isolated`).
- World-readable launcher files remain readable (Landlock grants read on
  `/`); secrets must be 0600/0700. Narrowing the read allowlist is open work.
- Ancestors of the server root must be traversable (0711) by workload IDs;
  otherwise the launch fails closed.
- TCP-only Landlock network control when network is granted; denied network
  uses a network namespace (also blocks UDP and abstract sockets).
- The runtime `execute_command` path (`SandboxClass`) does not yet offer the
  isolated class; only the ACP agent launcher does.
