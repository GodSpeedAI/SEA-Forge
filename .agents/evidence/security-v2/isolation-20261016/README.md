# Rootless bwrap same-UID regular-file descriptor access

This record captures a local, read-only isolation probe on 2026-10-16. It tests whether a separate process with the same host UID can read an open descriptor held by a process launched under rootless bubblewrap with user, PID, and mount/proc namespaces. The descriptor points only to a synthetic sentinel file in `/tmp`.

The probe confirmed regular-file descriptor content access: an external observer process with host UID 1000 opened `/proc/<payload-pid>/fd/9` and read `SEA_FORGE_SYNTHETIC_FD_SENTINEL_20261016`. The payload was launched inside bwrap with namespace UID/GID 0; its host `/proc/<pid>/status` showed UID/GID 1000. Afterward bwrap and its payload were terminated, the payload PID and descriptor disappeared, and the temporary sentinel file was removed. Exact observed values and outputs are in [observation.json](./observation.json).

## Reproduce

Run from a Linux host with bubblewrap and util-linux installed. The Python probe below creates a temporary sentinel, starts the sandbox process, locates its host PID through the process tree, reads its FD from a separate same-UID observer process, then terminates the sandbox and removes the sentinel. It prints JSON containing the observed results.

```sh
bwrap --version
id -u
id -g
unshare --user --map-root-user sh -c 'id -u; cat /proc/self/uid_map; cat /proc/self/gid_map'
unshare --user --map-auto sh -c 'cat /proc/self/uid_map; cat /proc/self/gid_map'
```

On the captured host, the root-user mapping command exited 0 and reported namespace UID/GID 0 mapped to host UID/GID 1000. The auto-map command exited 127 with `unshare: failed to execute newuidmap: No such file or directory`.

```sh
python3 - <<'PY'
import json, os, subprocess, sys, tempfile, time

sentinel = b'SEA_FORGE_SYNTHETIC_FD_SENTINEL_20261016'
fd, path = tempfile.mkstemp(prefix='sea-forge-isolation-sentinel-')
os.write(fd, sentinel)
os.close(fd)
launcher = None
result = {'host_uid': os.getuid(), 'host_gid': os.getgid(), 'sentinel': sentinel.decode()}
try:
    status_read, status_write = os.pipe()
    payload = f"cat /proc/self/uid_map; cat /proc/self/gid_map; id -u; exec 9<{path}; exec /bin/sleep 20"
    argv = [
        'bwrap', '--unshare-user', '--unshare-pid', '--ro-bind', '/', '/',
        '--proc', '/proc', '--dev', '/dev', '--uid', '0', '--gid', '0',
        '--die-with-parent', '--json-status-fd', str(status_write), '--',
        '/bin/sh', '-c', payload,
    ]
    launcher = subprocess.Popen(
        argv, pass_fds=(status_write,), stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, text=True, close_fds=True,
    )
    os.close(status_write)
    result['bwrap_status'] = json.loads(os.read(status_read, 4096).splitlines()[0])
    os.close(status_read)
    result['namespace_uid_map'] = launcher.stdout.readline().strip()
    result['namespace_gid_map'] = launcher.stdout.readline().strip()
    result['namespace_uid'] = launcher.stdout.readline().strip()

    payload_pid = None
    for _ in range(50):
        pending, visited = [launcher.pid], set()
        while pending:
            pid = pending.pop(0)
            if pid in visited:
                continue
            visited.add(pid)
            try:
                comm = open(f'/proc/{pid}/comm').read().strip()
                children = open(f'/proc/{pid}/task/{pid}/children').read().split()
                if comm == 'sleep' and os.path.exists(f'/proc/{pid}/fd/9'):
                    payload_pid = pid
                    break
                pending.extend(map(int, children))
            except (OSError, ValueError):
                pass
        if payload_pid is not None:
            break
        time.sleep(0.02)
    if payload_pid is None:
        raise RuntimeError('sandbox payload with FD 9 was not found')
    result['payload_host_pid'] = payload_pid
    status = open(f'/proc/{payload_pid}/status').read().splitlines()
    result['payload_host_uid_line'] = next(x for x in status if x.startswith('Uid:'))
    result['payload_host_gid_line'] = next(x for x in status if x.startswith('Gid:'))

    observer_code = (
        'import json,os,sys; p=sys.argv[1]; '
        'print(json.dumps({"observer_uid":os.getuid(),'
        '"fd_target":os.readlink(f"/proc/{p}/fd/9"),'
        '"read_bytes":open(f"/proc/{p}/fd/9","rb").read(256).decode()}))'
    )
    observer = subprocess.run(
        [sys.executable, '-c', observer_code, str(payload_pid)],
        text=True, capture_output=True, check=False,
    )
    result['observer_exit_code'] = observer.returncode
    result['observer'] = json.loads(observer.stdout.strip())
    result['observer_stderr'] = observer.stderr.strip()

    launcher.terminate()
    try:
        launcher.wait(timeout=2)
    except subprocess.TimeoutExpired:
        launcher.kill()
        launcher.wait(timeout=2)
    time.sleep(0.05)
    result['bwrap_exit_code_after_terminate'] = launcher.returncode
    result['payload_pid_remains'] = os.path.exists(f'/proc/{payload_pid}')
    result['cleanup_confirmed'] = not result['payload_pid_remains']
finally:
    if launcher is not None and launcher.poll() is None:
        launcher.kill()
        launcher.wait(timeout=2)
    os.unlink(path)
    result['sentinel_removed'] = not os.path.exists(path)
print(json.dumps(result, indent=2, sort_keys=True))
PY
```

## Limits

The opened descriptor was a regular file. This demonstrates same-host-UID access to regular-file descriptor contents through host `/proc` on this machine. It does **not** demonstrate opening or duplicating a Unix socket descriptor, stealing a Unix socket endpoint, or reading another process's memory; no socket or ptrace probe was run. The payload's private `/proc` view does not prevent host-side inspection through the host `/proc`. It does not test a different host UID, SELinux/AppArmor policy, or a production launcher. The captured `/etc/subuid` and `/etc/subgid` entries allocate `100000:65536`, but `newuidmap` and `newgidmap` are unavailable; therefore this host did not demonstrate a distinct host-UID mapping. Namespace UID 0 must not be represented as a separate host principal here.

This is local implementation evidence, not adversarial validation.
