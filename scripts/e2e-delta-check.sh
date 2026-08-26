#!/usr/bin/env sh
# Mechanically verify the GodSpeed convergence status surface (plan task T00):
#
#   1. the frozen preregistration is intact (`check-e2e-preregistration.sh`);
#   2. .agents/status/e2e-current-status.yml carries exactly one verdict for
#      every explicit frozen requirement (12 edges + 15 invariants + 8
#      envelope rules), using only the frozen verdict vocabulary;
#   3. every requirement cites at least one evidence path that resolves under
#      $REPOS_ROOT (default ~/projects), and CONFIRMED verdicts cite
#      production-path evidence (gen-1/synthetic/mock evidence can never
#      CONFIRM a production edge);
#   4. the committed Delta-0 report is byte-identical to a fresh regeneration
#      from the status file (mechanical reproducibility / drift detection).
#
# A failure here blocks the convergence/Gauntlet run. This script never
# rewrites the status file or the committed report.
#
# Overrides for isolated testing:
#   STATUS_FILE     (default <repo>/.agents/status/e2e-current-status.yml)
#   REPORT_FILE     (default <repo>/.agents/evidence/e2e/T00/delta0.md)
#   PREREG_FILE / PLAN_FILE / REPOS_ROOT (passed through)
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
status=${STATUS_FILE:-"$root/.agents/status/e2e-current-status.yml"}
report=${REPORT_FILE:-"$root/.agents/evidence/e2e/T00/delta0.md"}
repos_root=${REPOS_ROOT:-"$HOME/projects"}

fail() {
  printf 'fail: %s\n' "$1" >&2
  printf 'VERDICT: FAIL\n' >&2
  exit 1
}

[ -f "$status" ] || fail "status file not found: $status"

# --- 1. frozen target intact -------------------------------------------------
"$root/scripts/check-e2e-preregistration.sh" >/dev/null ||
  fail "frozen preregistration gate failed; see its output above"

# --- 2. expected requirement IDs straight from the frozen preregistration ----
expected_ids=$("$root/scripts/e2e-prereg-ids.sh") || fail "cannot derive requirement IDs from the frozen preregistration"
[ -n "$expected_ids" ] || fail "derived empty requirement ID set from the preregistration"
expected_count=$(printf '%s\n' "$expected_ids" | grep -c .)

# --- 3. parse the status file -------------------------------------------------
parsed=$(awk '
  /^requirements:/ { inr = 1; next }
  inr && /^[^ #]/ && !/^  / && !/^$/ { inr = 0 }
  inr {
    if (/^  - id:/) {
      if (nid != "") emit()
      sub(/^  - id:[[:space:]]*/, "")
      nid = $1; verdict = ""; ekind = ""; nev = 0; ev = ""
    } else if (nid != "") {
      if (/^    verdict:/)       { sub(/^    verdict:[[:space:]]*/, ""); verdict = $0 }
      else if (/^    evidence_kind:/) { sub(/^    evidence_kind:[[:space:]]*/, ""); ekind = $0 }
      else if (/^      - /)      { sub(/^      - /, ""); nev++; ev = ev (nev > 1 ? "," : "") $0 }
    }
  }
  function emit() {
    printf "%s\t%s\t%s\t%s\t%d\t%s\n", nid, verdict, ekind, (nev > 0 ? "has_evidence" : "no_evidence"), nev, ev
  }
  END { if (nid != "") emit() }
' "$status")

[ -n "$parsed" ] || fail "no requirements parsed from $status (format contract violated)"

actual_count=$(printf '%s\n' "$parsed" | grep -c .)
[ "$actual_count" = "$expected_count" ] ||
  fail "requirement count mismatch: status has $actual_count, frozen preregistration has $expected_count"

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT
printf '%s\n' "$expected_ids" | sort > "$tmpdir/expected"
printf '%s\n' "$parsed" | cut -f1 | sort > "$tmpdir/actual"

dups=$(cut -f1 "$tmpdir/actual" | uniq -d)
[ -z "$dups" ] || fail "duplicate requirement entries: $(printf '%s' "$dups" | tr '\n' ' ')"

unmapped=$(comm -23 "$tmpdir/expected" "$tmpdir/actual")
[ -z "$unmapped" ] || fail "frozen requirements missing from status: $(printf '%s' "$unmapped" | tr '\n' ' ')"

extra=$(comm -13 "$tmpdir/expected" "$tmpdir/actual")
[ -z "$extra" ] || fail "status carries non-frozen requirement IDs: $(printf '%s' "$extra" | tr '\n' ' ')"

while IFS="$(printf '\t')" read -r id verdict ekind has_ev nev evs; do
  case $verdict in
    CONFIRMED|PARTIAL|ABSENT|CONTRADICTED|UNKNOWN) ;;
    *) fail "$id has illegal verdict '$verdict' (frozen vocabulary only)" ;;
  esac
  case $ekind in
    production_path|production_partial|synthetic_only|documentation) ;;
    *) fail "$id has illegal evidence_kind '$ekind'" ;;
  esac
  if [ "$verdict" = "CONFIRMED" ] && [ "$ekind" != "production_path" ]; then
    fail "$id is CONFIRMED on evidence_kind='$ekind'; production edges can only be CONFIRMED by production_path evidence"
  fi
  if [ "$has_ev" != "has_evidence" ] || [ "$nev" -lt 1 ]; then
    fail "$id cites no evidence path"
  fi
  for p in $(printf '%s' "$evs" | tr ',' '\n'); do
    repo=${p%%/*}
    rest=${p#*/}
    if [ -d "$repos_root/$repo" ]; then
      [ -e "$repos_root/$p" ] || fail "$id cites unresolvable evidence path: $p (under $repos_root)"
    elif [ -e "$root/$p" ]; then
      :
    else
      fail "$id cites unresolvable evidence path: $p"
    fi
  done
done <<EOF
$parsed
EOF

confirmed=$(printf '%s\n' "$parsed" | awk -F'\t' '$2 == "CONFIRMED"' | wc -l)

# --- 4. committed Delta-0 report must equal a fresh regeneration --------------
if [ ! -f "$report" ]; then
  fail "committed Delta-0 report missing: $report (generate it with scripts/e2e-delta-report.sh)"
fi
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT
STATUS_FILE="$status" "$root/scripts/e2e-delta-report.sh" > "$tmp" ||
  fail "could not regenerate the Delta-0 report"
if ! diff -q "$report" "$tmp" >/dev/null 2>&1; then
  diff "$report" "$tmp" >&2 || true
  fail "committed Delta-0 report drifted from .agents/status/e2e-current-status.yml; regenerate it (scripts/e2e-delta-report.sh) after reviewing the drift"
fi

printf 'Delta-0 check passed:\n'
printf '  requirements: %s (one verdict each, all evidence resolvable)\n' "$actual_count"
printf '  confirmed: %s\n' "$confirmed"
printf '  open delta: %s\n' "$((actual_count - confirmed))"
printf 'VERDICT: PASS\n'
