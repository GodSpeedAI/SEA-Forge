#!/bin/sh
# Adversarial probe: per-workload user namespace, single subordinate UID mapping.
set -u
W=$(mktemp -d /tmp/sfiso.XXXXXX); chmod 1777 "$W"
launch() { # $1=hostuid $2=cmd
  unshare --user --map-users 0:$1:1 --map-groups 0:$1:1 --setuid 0 --setgid 0 --pid --fork --mount --mount-proc --net -- sh -c "$2"
}
# Workload B: holds secret fd open, secret in 0700 dir it owns
launch 100002 "mkdir -p $W/b && chmod 700 $W/b && echo SECRET-B > $W/b/s && exec 9<$W/b/s; echo \$\$ >$W/b.pid.tmp; sleep 20" &
sleep 1.5
BPID=$(pgrep -u 100002 -n sleep); echo "B host pid=$BPID uid=$(awk '/^Uid/{print $2}' /proc/$BPID/status)"
# uid-1000 secret (trusted plane) and socket
echo SECRET-HOST > $W/host.secret; chmod 600 $W/host.secret
python3 -c "import socket,os;s=socket.socket(socket.AF_UNIX);s.bind('$W/cap.sock');os.chmod('$W/cap.sock',0o600);s.listen();import time;time.sleep(20)" &
sleep 0.5
cat > $W/a.sh <<EOS
r(){ printf '%-34s ' "\$1"; shift; if "\$@" >/dev/null 2>&1; then echo VULNERABLE; else echo denied; fi; }
echo "A: id=\$(id -u) caps=\$(grep CapEff /proc/self/status)"
r "read B fd via proc" cat /proc/$BPID/fd/9
r "list B fds" ls /proc/$BPID/fd
r "read B environ" cat /proc/$BPID/environ
r "read B secret file" cat $W/b/s
r "signal B" kill -0 $BPID
r "nsenter B userns" nsenter -U -t $BPID true
r "remap to B uid (nested)" unshare --user --map-users 0:100002:1 true
r "remap to host 1000 (nested)" unshare --user --map-users 0:1000:1 true
r "map-auto nested" unshare --user --map-auto true
r "read host 0600 secret" cat $W/host.secret
r "connect host 0600 socket" python3 -c "import socket;socket.socket(socket.AF_UNIX).connect('$W/cap.sock')"
r "ptrace host uid1000 proc" cat /proc/$$/environ
r "read ~/.ssh" ls /home/sprime01/.ssh
r "read home dir listing" ls /home/sprime01
r "read repo file" cat /home/sprime01/projects/sea-rs/Cargo.toml
EOS
echo "--- host pid namespace hidden? (A sees B only if global /proc)"
launch 100001 "sh $W/a.sh"
echo "--- A with global /proc view (no pid ns) for completeness"
unshare --user --map-users 0:100001:1 --map-groups 0:100001:1 --setuid 0 --setgid 0 -- sh $W/a.sh
echo "--- uid1000 (trusted plane) nsenter into B userns:"; nsenter -U -t $BPID -- true && echo "owner can enter (expected: launcher is trusted)"
kill %1 %2 2>/dev/null; wait 2>/dev/null; rm -rf "$W" 2>/dev/null; true
