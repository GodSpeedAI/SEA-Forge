#!/usr/bin/env bash
# Round-4 pathname-socket containment gates (SEC-06, ADR-014).
#
# Run from a process tree WITHOUT an inherited seccomp user-notify listener:
#   grep -E '^Seccomp' /proc/self/status   # must show Seccomp: 0 (or 1)
# Under an inherited listener (Seccomp: 2 with a listener in the chain) the
# launcher correctly fails closed and every isolation test reports
# unavailable; those runs prove nothing about the mechanism.
set -u
cd "$(dirname "$0")/../.."
EV=.agents/evidence/security-v2/round4-20261018/gates
mkdir -p "$EV"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0
FAIL=0

# Isolation availability decides PASS/FAIL vs NOT-YET-EXECUTABLE: with the
# require flag set, an unavailable pool panics the tests (a FAIL), which
# would conflate "host cannot exercise the mechanism" with "mechanism
# broken". Probe first; when the host refuses mapped user namespaces, run
# without the flag (tests self-report unavailable) and record NYE.
if unshare --user --map-users=1000:100000:1 --map-groups=1000:100000:1 \
     --setuid=1000 --setgid=1000 -- true 2>/dev/null; then
  export SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION=1
  ISOLATION=AVAILABLE
else
  ISOLATION=UNAVAILABLE
fi

log() { tee -a "$EV/summary.txt"; }

echo "== environment ==" | log
uname -a | log
grep -E "^Seccomp|^NoNewPrivs" /proc/self/status | log
echo "workload isolation: $ISOLATION" | log
cat > /tmp/llabi-gate.c <<'EOF'
#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
int main(void){ printf("landlock ABI = %ld\n", syscall(444, 0, 0, 1)); }
EOF
gcc -o /tmp/llabi-gate /tmp/llabi-gate.c && /tmp/llabi-gate | log

run() {
  local name="$1"; shift
  echo "== $name ==" | log
  if "$@" > "$EV/$name.log" 2>&1; then
    echo "PASS $name" | log
  else
    echo "FAIL $name (see $EV/$name.log)" | log
    FAIL=1
  fi
}

if [ "$ISOLATION" = AVAILABLE ]; then
  run sandbox-suite cargo test -p sea-forge-sandbox -j 1
  run postgres-broker cargo test -p sea-forge-server --test postgres_broker -j 1
  if [ -d /usr/lib/postgresql ] || command -v initdb >/dev/null; then
    SEA_FORGE_REQUIRE_POSTGRES=1 run postgres-lifecycle \
      cargo test -p sea-forge-server --test postgres_lifecycle -j 1
  else
    echo "SKIP postgres-lifecycle (no local PostgreSQL fixture)" | log
  fi
else
  # NOT-YET-EXECUTABLE, honestly recorded: the host refuses mapped user
  # namespaces, the suites self-report unavailable, and nothing here proves
  # or disproves the mechanism. Still run the suites so the unavailable
  # path itself is exercised and logged.
  run sandbox-suite-no-require cargo test -p sea-forge-sandbox -j 1
  run postgres-broker-no-require cargo test -p sea-forge-server --test postgres_broker -j 1
  echo "NOT-EXECUTABLE socket gates (workload isolation unavailable on this host)" | log
fi

# Mutation controls: break the fix, its test must fail, restore, re-verify.
mutation() {
  local name="$1" file="$2" from="$3" to="$4" test="$5"
  echo "== mutation $name ==" | log
  python3 - "$file" "$from" "$to" <<'EOF'
import sys
path, frm, to = sys.argv[1:4]
text = open(path).read()
assert frm in text, "mutation anchor not found"
open(path, "w").write(text.replace(frm, to, 1))
EOF
  if cargo test -p sea-forge-sandbox --test unix_socket_reach -j 1 "$test" \
      > "$EV/mutation-$name.log" 2>&1; then
    echo "FAIL mutation $name: test still passed" | log
    FAIL=1
  else
    echo "KILLED mutation $name" | log
  fi
  git -C . checkout -- "$file"
  touch "$file"
  cargo test -p sea-forge-sandbox --test unix_socket_reach -j 1 "$test" \
    > "$EV/mutation-$name-restored.log" 2>&1 \
    && echo "restored PASS $name" | log \
    || { echo "FAIL restored $name" | log; FAIL=1; }
}

# M-S2: supervisor allows every pathname target under any tree.
# M-S4: helper stops passing socket grants (grant plumbing broken).
# Mutations only mean something when the targeted test actually executes:
# a skipped test "fails" under any mutant, so without isolation they are
# recorded NOT-EXECUTABLE rather than run.
if [ "$ISOLATION" = AVAILABLE ]; then
  mutation M-S2-supervisor-allows-all \
    "crates/sea-forge-sandbox/src/socket_policy.rs" \
    'let under_grant = granted.iter().any(|tree| path.starts_with(tree));' \
    'let under_grant = true; // MUTANT' \
    pathname_sockets_outside_declared_trees_are_unreachable

  mutation M-S4-grants-dropped \
    "crates/sea-forge-sandbox/src/bin/sea-forge-workload-exec.rs" \
    'let mut socket_reach = socket_grants;' \
    'let mut socket_reach = Vec::new(); // MUTANT' \
    pathname_sockets_outside_declared_trees_are_unreachable
else
  echo "NOT-EXECUTABLE mutations (workload isolation unavailable; a skipped test cannot kill a mutant)" | log
fi

echo "== summary ==" | log
grep -E "^(PASS|FAIL|KILLED|SKIP|restored|NOT-EXECUTABLE)" "$EV/summary.txt" | log
exit $FAIL
