#!/usr/bin/env python3
"""Actions-only disposable QEMU crash/recovery controller; never touches host disks."""
import argparse
import gzip
import json
import lzma
import os
from pathlib import Path
import re
import selectors
import shutil
import signal
import subprocess
import sys
import tempfile
import time

PHASES = ('stage-created-first', 'before-first', 'after-first', 'acked-first', 'before-replace',
          'after-replace', 'acked-replace')
TEST = 'case_ops::case_guard::staged_commit_tests::guest_crash_publication_probe'
TIMEOUT = 120


def run(args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)


def copy_binary(src, root, dest):
    """Copy the binary and the shared objects/loader resolved by host ldd."""
    target = root / dest.lstrip('/')
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, target, follow_symlinks=True)
    result = subprocess.run(['ldd', str(src)], text=True, capture_output=True)
    output = result.stdout + result.stderr
    if 'not found' in output:
        raise RuntimeError(f'unresolved library for {src}: {output}')
    if result.returncode and not any(s in output for s in
                                     ('not a dynamic executable', 'statically linked')):
        raise RuntimeError(f'ldd failed for {src}: {output}')
    for lib in re.findall(r'(?<!\S)(/[^\s()]+)', output):
        path = Path(lib)
        target = root / lib.lstrip('/')
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target, follow_symlinks=True)


def assemble(root, probe, release, archive):
    for directory in ('bin', 'dev', 'proc', 'sys', 'data', 'modules', 'tmp'):
        (root / directory).mkdir(parents=True, exist_ok=True)
    busybox = shutil.which('busybox')
    if not busybox:
        raise RuntimeError('busybox-static is required')
    copy_binary(busybox, root, '/bin/busybox')
    for applet in ('mount', 'poweroff', 'sleep', 'grep', 'cat'):
        (root / 'bin' / applet).symlink_to('busybox')
    copy_binary(probe, root, '/probe')
    # An explicit, dependency-ordered list avoids requiring kmod inside the guest.
    modules = []
    for name in ('virtio_pci', 'virtio_blk', 'ext4'):
        p = run(['modprobe', '--show-depends', '--set-version', release, name],
                text=True, capture_output=True)
        for line in p.stdout.splitlines():
            if line.startswith('insmod '):
                module = line[7:].strip()
                if module not in modules:
                    modules.append(module)
            elif line and not line.startswith(('builtin ', 'install ')):
                raise RuntimeError(f'unexpected modprobe output: {line}')
            elif line.startswith('install '):
                raise RuntimeError(f'unsupported modprobe install command: {line}')
    commands = []
    for index, module in enumerate(modules):
        source = Path(module)
        data = source.read_bytes()
        if module.endswith('.zst'):
            data = run(['zstd', '-q', '-d', '-c', module], capture_output=True).stdout
        elif module.endswith('.xz'):
            data = lzma.decompress(data)
        elif module.endswith('.gz'):
            data = gzip.decompress(data)
        dest = root / 'modules' / f'{index}.ko'
        dest.write_bytes(data)
        commands.append(f'/bin/busybox insmod /modules/{index}.ko || fail module-{index}')
    init = r'''#!/bin/busybox sh
export PATH=/bin
fail() { echo "SEA_FDE_INIT_FAILURE $*"; poweroff -f; while :; do sleep 60; done; }
mount -t proc proc /proc || fail proc
mount -t sysfs sysfs /sys || fail sysfs
mount -t devtmpfs devtmpfs /dev || fail devtmpfs
exec </dev/console >/dev/console 2>&1
MODULES
tries=0
while [ ! -b /dev/vda ] && [ "$tries" -lt 10 ]; do sleep 1; tries=$((tries + 1)); done
[ -b /dev/vda ] || fail missing-vda
mount -t ext4 /dev/vda /data || fail ext4-mount
grep -q '^/dev/vda /data ext4 ' /proc/mounts || fail mount-verification
grep '^/dev/vda /data ext4 ' /proc/mounts || fail mount-report
mode= phase=
for arg in $(cat /proc/cmdline); do
  case "$arg" in sea_mode=*) mode=${arg#sea_mode=} ;; sea_phase=*) phase=${arg#sea_phase=} ;; esac
done
[ "$mode" = write ] || [ "$mode" = verify ] || fail mode
[ -n "$phase" ] || fail phase
export SEA_FDE_CRASH_ROOT=/data SEA_FDE_CRASH_MODE="$mode" SEA_FDE_CRASH_PHASE="$phase"
echo "SEA_FDE_GUEST_MOUNT_OK /dev/vda /data ext4 $mode $phase"
/probe --exact @PROBE_TEST@ --ignored --nocapture || fail probe-exit
echo "SEA_FDE_GUEST_TEST_OK $mode $phase"
poweroff -f
while :; do sleep 60; done
'''.replace('MODULES', '\n'.join(commands)).replace('@PROBE_TEST@', TEST)
    (root / 'init').write_text(init)
    (root / 'init').chmod(0o755)
    # cpio paths are relative and NUL-delimited; no shell interpolation.
    paths = [b'.'] + [os.fsencode('./' + str(p.relative_to(root)))
                     for p in sorted(root.rglob('*'))]
    with archive.open('wb') as out:
        run(['cpio', '--null', '-o', '-H', 'newc', '--quiet'], cwd=root,
            input=b'\0'.join(paths) + b'\0', stdout=out)
    return modules


def boot(qemu, kernel, initramfs, disk, phase, mode, log_path):
    command = [qemu, '-machine', 'pc,accel=tcg', '-cpu', 'max', '-m', '512',
               '-smp', '2', '-display', 'none', '-monitor', 'none', '-serial', 'stdio',
               '-nodefaults', '-no-reboot', '-nic', 'none',
               '-kernel', str(kernel), '-initrd', str(initramfs),
               '-append', f'console=ttyS0 rdinit=/init panic=-1 sea_mode={mode} sea_phase={phase}',
               '-drive', f'file={disk},if=none,format=raw,cache=none,id=data',
               '-device', 'virtio-blk-pci,drive=data']
    expected_mount = f'SEA_FDE_GUEST_MOUNT_OK /dev/vda /data ext4 {mode} {phase}'
    expected_test = f'SEA_FDE_GUEST_TEST_OK {mode} {phase}'
    marker = 'SEA_FDE_RESET_READY' if mode == 'write' else 'SEA_FDE_RECOVERY_OK'
    test_passed = False
    seen = set()
    proc = None
    pending = b''
    deadline = time.monotonic() + TIMEOUT
    with log_path.open('xb', buffering=0) as log:
        try:
            proc = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                    stderr=subprocess.STDOUT, bufsize=0)
            with selectors.DefaultSelector() as selector:
                selector.register(proc.stdout, selectors.EVENT_READ)
                while True:
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        raise TimeoutError(f'{phase} {mode}: timeout after {TIMEOUT}s')
                    events = selector.select(min(remaining, 0.5))
                    if not events:
                        if proc.poll() is not None:
                            break
                        continue
                    chunk = os.read(proc.stdout.fileno(), 65536)
                    if not chunk:
                        break
                    log.write(chunk)
                    pending += chunk
                    lines = pending.split(b'\n')
                    pending = lines.pop()
                    for raw in lines:
                        line = raw.rstrip(b'\r').decode('utf-8', 'replace')
                        if line.startswith('test result: ok. 1 passed; 0 failed;'):
                            test_passed = True
                        if line in (marker, expected_mount, expected_test):
                            seen.add(line)
                        if mode == 'write' and line == marker:
                            if expected_mount not in seen:
                                raise RuntimeError('reset marker arrived before verified mount')
                            proc.kill()  # This exact child only: the simulated power loss.
                            rc = proc.wait(timeout=10)
                            if rc != -signal.SIGKILL:
                                raise RuntimeError(f'writer did not die from SIGKILL: {rc}')
                            return {'marker': marker, 'mount': expected_mount,
                                    'qemu_returncode': rc, 'killed_and_reaped': True}
            rc = proc.wait(timeout=max(0.1, deadline - time.monotonic()))
            if mode == 'write':
                raise RuntimeError(f'writer exited before reset marker: {rc}')
            missing = {marker, expected_mount, expected_test} - seen
            if rc != 0 or missing or not test_passed:
                raise RuntimeError(
                    f'{phase} recovery failed: qemu={rc}, missing={sorted(missing)}, '
                    f'one test passed={test_passed}'
                )
            return {'marker': marker, 'mount': expected_mount,
                    'test': expected_test, 'qemu_returncode': rc}
        finally:
            if proc is not None:
                if proc.poll() is None:
                    proc.kill()
                proc.wait()
                proc.stdout.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--probe', type=Path, required=True)
    parser.add_argument('--kernel', type=Path, required=True)
    parser.add_argument('--kernel-release', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    probe, kernel = args.probe.resolve(strict=True), args.kernel.resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=True)
    output = args.output.resolve()
    # Reject an existing evidence set instead of silently overwriting it.
    metadata_path = output / 'metadata.json'
    with metadata_path.open('x') as f:
        json.dump({'status': 'initializing'}, f)
    metadata = {'status': 'running', 'kernel': str(kernel), 'kernel_release': args.kernel_release,
                'probe': str(probe), 'filesystem': 'ext4', 'phases': {}}
    def save():
        metadata_path.write_text(json.dumps(metadata, indent=2) + '\n')
    try:
        qemu = shutil.which('qemu-system-x86_64')
        if not qemu:
            raise RuntimeError('qemu-system-x86_64 is required')
        metadata['qemu_version'] = run([qemu, '--version'], text=True,
                                       capture_output=True).stdout.splitlines()[0]
        metadata['disk'] = {'format': 'raw', 'cache': 'none', 'bus': 'virtio',
                            'bytes': 256 * 1024 * 1024}
        with tempfile.TemporaryDirectory(prefix='sea-fde-qemu-') as temp:
            work = Path(temp)
            root = work / 'root'
            root.mkdir()
            initramfs = work / 'initramfs.cpio'
            metadata['modules'] = assemble(root, probe, args.kernel_release, initramfs)
            save()
            for phase in PHASES:
                disk = work / (phase + '.raw')
                with disk.open('xb') as image:
                    image.truncate(256 * 1024 * 1024)
                run(['mkfs.ext4', '-q', '-F', str(disk)])
                entry = metadata['phases'][phase] = {'filesystem': 'ext4', 'status': 'writing'}
                save()
                entry['writer'] = boot(qemu, kernel, initramfs, disk, phase, 'write',
                                       output / f'{phase}-write.serial.log')
                entry['status'] = 'recovering'
                save()
                entry['recovery'] = boot(qemu, kernel, initramfs, disk, phase, 'verify',
                                         output / f'{phase}-verify.serial.log')
                entry['status'] = 'passed'
                save()
        metadata['status'] = 'passed'
    except Exception as exc:
        metadata['status'] = 'failed'
        metadata['error'] = f'{type(exc).__name__}: {exc}'
        save()
        raise
    save()


if __name__ == '__main__':
    try:
        main()
    except Exception as error:
        print(f'controller failed: {error}', file=sys.stderr)
        sys.exit(1)
