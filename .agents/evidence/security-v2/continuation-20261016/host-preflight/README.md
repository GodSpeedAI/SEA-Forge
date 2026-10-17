# Linux/WSL host preflight observations

Captured 2026-10-16 as read-only host evidence for the requested Linux-first launcher assessment. The machine is WSL2 running a Linux 6.18 kernel, but the SEA Forge process is itself constrained: it has no capabilities, `NoNewPrivs=1`, seccomp mode 2, one-entry UID/GID mappings, and a read-only cgroup v2 mount. User namespaces and the existing Landlock ABI are present. These facts do **not** provide distinct host UIDs.

The concrete result is that this process cannot currently create a workload with a distinct host-visible UID using unprivileged `unshare` or rootless bwrap. `unshare --map-root-user` maps namespace root back to host-visible UID 1000. Although subordinate UID/GID ranges are configured, `newuidmap` and `newgidmap` are absent and `--map-auto` fails. No privilege escalation, `sudo`, install, service launch, or system security change was attempted.

The evidence in [observations.json](./observations.json) records exact outputs and tool paths. The earlier regular-file descriptor probe is separately recorded at [the isolation evidence directory](../../isolation-20261016/README.md); it is only evidence that a same-UID process read a regular-file descriptor through host `/proc`. It did not test Unix socket descriptor duplication or socket theft.

## Reproduce the read-only probes

These commands inspect identity, namespace mappings, kernel features, tools, and cgroup mount state. `unshare` commands create only transient namespaces. The last command attempts an auto-map and is expected to fail on the captured host because `newuidmap` is not installed.

```sh
systemd-detect-virt
uname -a
id
grep -E '^(Cap(Inh|Prm|Eff|Bnd)|NoNewPrivs|Seccomp):' /proc/self/status
cat /proc/self/uid_map
cat /proc/self/gid_map
cat /proc/self/cgroup
cat /sys/fs/cgroup/cgroup.controllers
cat /sys/fs/cgroup/cgroup.subtree_control
grep ' - cgroup2 ' /proc/self/mountinfo
stat -c '%a %u %g %n' /sys/fs/cgroup /sys/fs/cgroup/cgroup.procs
for tool in bwrap unshare setpriv prlimit newuidmap newgidmap podman docker dockerd dockerd-rootless.sh rootlesskit slirp4netns fuse-overlayfs runc crun containerd firecracker qemu-system-x86_64 openshell; do command -v "$tool" || printf '%s unavailable\n' "$tool"; done
for path in /dev/fuse /dev/kvm /run/user/1000/podman/podman.sock /run/docker.sock /run/containerd/containerd.sock /run/user/1000/docker.sock; do test -e "$path" && stat -c '%a %u %g %n' "$path" || printf '%s absent\n' "$path"; done
unshare --user --map-root-user sh -c 'id -u; cat /proc/self/uid_map; cat /proc/self/gid_map'
unshare --user --map-auto sh -c 'cat /proc/self/uid_map; cat /proc/self/gid_map'
unshare --user --map-root-user --net true
bwrap --unshare-user --unshare-net --ro-bind / / --proc /proc --dev /dev -- /usr/bin/true
```

The Landlock probe was a version query only, using x86_64 syscall 444 with `LANDLOCK_CREATE_RULESET_VERSION` (flags 1). It does not install a ruleset or change process state:

```sh
python3 - <<'PY'
import ctypes, os
libc = ctypes.CDLL(None, use_errno=True)
ctypes.set_errno(0)
result = libc.syscall(ctypes.c_long(444), ctypes.c_void_p(0), ctypes.c_size_t(0), ctypes.c_uint(1))
error = ctypes.get_errno()
print('result', result, 'errno', error, os.strerror(error) if error else '')
PY
```

The captured result was ABI 7 (`result 7 errno 0`). The probe did not call `landlock_restrict_self`, verify an installed policy, or observe a denied resource operation. The repository's existing Jail code still owns the Landlock enforcement path.

## Prerequisite and implementation boundary

Distinct host IDs require an operator-provided trusted launcher outside this process's `NoNewPrivs`/seccomp boundary that can establish the UID/GID maps using validated subordinate-ID helpers (`newuidmap`/`newgidmap`) or another explicit, controlled privilege path. It must assign a unique host-visible UID/GID to each untrusted workload and verify the resulting host `/proc/<pid>/status` IDs; seeing UID 0 inside a user namespace is not enough. This probe did not implement such a launcher or production preflight.

An OS-level background daemon or cgroup delegation is not inherently required just to assign distinct host UIDs. If hard aggregate memory/process limits are separately required, a writable delegated cgroup controller is needed; this environment's cgroup2 mount is read-only, so that additional guarantee is unavailable here. Do not silently fall back to a shared UID when the distinct-host-ID prerequisite is absent. Retain the existing Landlock boundary and OpenShell backend; the observations here do not show that OpenShell supplies distinct host IDs or private-FD transport.

No production implementation, adversarial validation, or security-contract completion is claimed by this preflight record.
