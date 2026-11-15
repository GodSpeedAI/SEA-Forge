#!/usr/bin/env bash
# FDE proof runner, skeleton (WP-09). WP-19 completes it (S1 form); WP-53 extends it.
#
#   scripts/fde-proof.sh --slice s1 [--only <ID>] [--remote]
#   scripts/fde-proof.sh --replay <run-id>      not implemented in WP-09 (records NOT_RUN, exits 4)
#   scripts/fde-proof.sh --mutate <NT-id>       not implemented in WP-09 (records NOT_RUN, exits 4)
#
# Order: (1) verify the anchored pre-registration (scripts/fde-anchor.sh verify); refuse to run
# if it fails. (2) pre-flight scripts/check-fde-contract-digests.sh. (3) for each selected test
# in .agents/preregistrations/fde-<slice>/tests.yaml run scripts/fde-proof.d/<ID>.sh when it
# exists, else record NOT_RUN. NOT_RUN is never PASS.
#
# Dispatcher exit codes: 0 PASS, 10 PARTIAL, 11 NOT_TESTED, 12 NOT_RUN, anything else FAIL.
# A script may print one line "REASON: <text>" (last one wins); it is recorded with the status.
# Each result records command, exit code, status, duration, reason, the commit of every repo involved
# and tool versions (top level `repos`, `tools`; see scripts/fde-proof.d/_lib.sh).
# Output: .tmp/fde-proof/<slice>/results.json (id, status, command, exit_code, evidence) and
# one log per test. Exit 0 only when every selected test is PASS.
# Exit: 0 all PASS, 1 not all PASS, 2 usage, 3 pre-registration or pre-flight refused, 4 stub.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
slice=""; only=""; replay=""; mutate=""; remote=""; run_eps=""
while [ $# -gt 0 ]; do
  case "$1" in
    --slice) slice=${2:?--slice needs a value}; shift 2;;
    --only) only=${2:?--only needs an id}; shift 2;;
    --replay) replay=${2:?--replay needs a run id}; shift 2;;
    --mutate) mutate=${2:?--mutate needs an NT id}; shift 2;;
    --remote) remote=--remote; shift;;
    --run-episodes) run_eps=1; shift;;
    -h|--help) sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) echo "fde-proof: unknown argument: $1" >&2; exit 2;;
  esac
done
[ -n "$slice" ] || slice=s1
case "$slice" in s1|s2) ;; *) echo "fde-proof: only slices s1 and s2 are registered" >&2; exit 2;; esac
prereg_id=fde-$slice
pdir=$root/.agents/preregistrations/$prereg_id
tests_file=$pdir/tests.yaml
wrapper_dir=scripts/fde-proof.d
if [ "$slice" = s2 ]; then
  # S2-gate tests of WP-21a: a list under scripts/fde-proof.d, wrappers under scripts/fde-proof.d/s2.
  # There is no anchored S2 pre-registration yet; the S1 one is not touched.
  tests_file=$root/scripts/fde-proof.d/s2-tests.yaml
  wrapper_dir=scripts/fde-proof.d/s2
fi
out=$root/.tmp/fde-proof/$slice
mkdir -p "$out"

write_results() { # $1 = JSON-lines file, $2 = overall note
  python3 - "$1" "$out/results.json" "$slice" "$prereg_id" "$2" <<'PY'
import sys, json, datetime
lines = [json.loads(l) for l in open(sys.argv[1]) if l.strip()]
import subprocess, os
ctx = json.loads(subprocess.run(["sh", os.path.join(os.path.dirname(os.path.abspath(sys.argv[2])), "../../../scripts/fde-proof.d/_context.sh")], capture_output=True, text=True).stdout or "{}")
doc = {"slice": sys.argv[3], "preregistration": sys.argv[4], "note": sys.argv[5], "repos": ctx.get("repos", {}), "tools": ctx.get("tools", {}),
       "generated_at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
       "results": lines}
json.dump(doc, open(sys.argv[2], "w"), indent=2, sort_keys=True); open(sys.argv[2], "a").write("\n")
PY
}
rec() { # id status command exit evidence [duration_s] [reason]
  python3 -c 'import json,sys
a=sys.argv
print(json.dumps({"id":a[1],"status":a[2],"command":a[3],"exit_code":int(a[4]),"evidence":a[5],"duration_s":float(a[6]) if len(a)>6 else 0.0,"reason":a[7] if len(a)>7 else ""}))' "$@" >>"$lines"
}
lines=$(mktemp "${TMPDIR:-/tmp}/fde-proof.XXXXXX"); trap 'rm -f "$lines"' EXIT INT TERM

if [ -n "$replay" ] || [ -n "$mutate" ]; then
  what=${replay:+--replay $replay}${mutate:+--mutate $mutate}
  echo "fde-proof: $what is not implemented in WP-09; recorded NOT_RUN" >&2
  rec "${mutate:-replay:$replay}" NOT_RUN "scripts/fde-proof.sh $what" 4 ""
  write_results "$lines" "not implemented in WP-09"
  exit 4
fi

echo "== pre-registration: scripts/fde-anchor.sh verify prereg/$prereg_id ${remote}"
if [ "$slice" = s2 ]; then
  echo "(slice s2 has no anchored pre-registration; skipped)"
elif ! scripts/fde-anchor.sh verify "prereg/$prereg_id" $remote; then
  rec preflight:prereg FAIL "scripts/fde-anchor.sh verify prereg/$prereg_id" 1 ""
  write_results "$lines" "pre-registration verification failed; no test was run"
  echo "fde-proof: refusing to run: the pre-registration does not verify" >&2
  exit 3
fi
echo "== pre-flight: scripts/check-fde-contract-digests.sh"
if ! scripts/check-fde-contract-digests.sh; then
  rec preflight:contract-digests FAIL "scripts/check-fde-contract-digests.sh" 1 ""
  write_results "$lines" "contract digest pre-flight failed; no test was run"
  echo "fde-proof: refusing to run: contract digests differ" >&2
  exit 3
fi
rec preflight:contract-digests PASS "scripts/check-fde-contract-digests.sh" 0 ""

ids=$(python3 - "$tests_file" <<'PY'
import sys, yaml
for t in yaml.safe_load(open(sys.argv[1]))["tests"]: print(t["id"])
PY
)
if [ -n "$only" ]; then
  printf '%s\n' "$ids" | grep -Fxq "$only" || { echo "fde-proof: $only is not in $tests_file" >&2; exit 2; }
  ids=$only
fi

# Episode cell: --run-episodes performs the whole real run first (scripts/fde-s1-run.sh); otherwise the
# latest cell (FDE_S1_CELL, else ~/projects/_wt/tmp/fde-s1/latest) is used. No cell: dependent tests are PARTIAL/NOT_RUN.
if [ -n "$run_eps" ]; then
  echo "== episodes: scripts/fde-s1-run.sh"
  FDE_S1_CELL=$(scripts/fde-s1-run.sh | tail -1) || echo "fde-proof: episode run did not complete (see its log)" >&2
fi
FDE_S1_CELL=${FDE_S1_CELL:-$(readlink -f "$HOME/projects/_wt/tmp/fde-s1/latest" 2>/dev/null || true)}
export FDE_S1_CELL
all_pass=1
for id in $ids; do
  d=$wrapper_dir/$id.sh
  cmd="scripts/fde-proof.sh --slice $slice --only $id"
  if [ -x "$d" ]; then
    log=$out/$id.log
    t0=$(date +%s.%N)
    set +e; "$d" >"$log" 2>&1; code=$?; set -e
    dur=$(python3 -c "import sys;print(round(float(sys.argv[1])-float(sys.argv[2]),2))" "$(date +%s.%N)" "$t0")
    case "$code" in 0) st=PASS;; 10) st=PARTIAL;; 11) st=NOT_TESTED;; 12) st=NOT_RUN;; *) st=FAIL;; esac
    reason=$(sed -n 's/^REASON: //p' "$log" | tail -1)
    rec "$id" "$st" "$cmd" "$code" "${log#"$root"/}" "$dur" "$reason"
  else
    st=NOT_RUN; code=0
    rec "$id" NOT_RUN "$cmd" 0 "" 0 "no $wrapper_dir/$id.sh"
  fi
  [ "$st" = PASS ] || all_pass=0
  printf '%-8s %s\n' "$id" "$st"
done
write_results "$lines" "skeleton: tests without scripts/fde-proof.d/<ID>.sh are NOT_RUN"
echo "results: ${out#"$root"/}/results.json"
[ "$all_pass" -eq 1 ]
