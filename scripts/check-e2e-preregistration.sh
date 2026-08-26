#!/usr/bin/env sh
# Verify the frozen E2E preregistration has not drifted from the hash bound in
# the convergence plan (.agents/plans/e2e-plan.yml -> source.spec.sha256).
#
# The preregistration is a frozen target. A mismatch STOPS the convergence /
# Gauntlet run until the change is explicitly reviewed and re-frozen; this
# script never updates the stored hash.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
prereg=${PREREG_FILE:-"$root/.agents/specs/e2e-preregistration.yml"}
plan=${PLAN_FILE:-"$root/.agents/plans/e2e-plan.yml"}

fail() {
  printf 'fail: %s\n' "$1" >&2
  printf 'VERDICT: FAIL\n' >&2
  exit 1
}

# Read exactly source.spec.sha256 via a real YAML parser when PyYAML exists.
extract_with_yaml() {
  command -v python3 >/dev/null 2>&1 || return 3
  python3 - "$1" <<'PY' || return 4
import sys

try:
    import yaml
except ImportError:
    sys.exit(3)
try:
    with open(sys.argv[1]) as fh:
        node = yaml.safe_load(fh)["source"]["spec"]["sha256"]
except Exception:
    sys.exit(4)
print(str(node).strip())
PY
}

# Fallback without PyYAML: indentation-aware extraction of that one key only
# (top-level `source:` -> `spec:` -> `sha256:`), never any other sha256 field.
extract_scoped_fallback() {
  awk '
    /^source:/ { in_source = 1; in_spec = 0; next }
    /^[^ #]/ && NF { in_source = 0; in_spec = 0 }
    in_source && /^  spec:/ { in_spec = 1; next }
    in_spec && /^    sha256:/ {
      sub(/^    sha256:[[:space:]]*/, "")
      gsub(/"/, "")
      gsub(/'\''/, "")
      print
      exit
    }
  ' "$1"
}

[ -f "$prereg" ] || fail "frozen preregistration not found: $prereg"
[ -f "$plan" ] || fail "convergence plan not found: $plan"

observed=$(sha256sum "$prereg" | awk '{print $1}')
expected=$(extract_with_yaml "$plan") || expected=$(extract_scoped_fallback "$plan")

[ -n "$expected" ] || fail "source.spec.sha256 is missing or unreadable in $plan"

expected_upper=$(printf '%s' "$expected" | tr '[:lower:]' '[:upper:]')
case $expected_upper in
  *BIND_BEFORE_EXECUTION*)
    fail "source.spec.sha256 still holds an unbound placeholder; the frozen hash must be bound before execution" ;;
esac
printf '%s' "$expected" | grep -Eq '^[0-9a-fA-F]{64}$' ||
  fail "source.spec.sha256 is malformed (expected 64 hex characters): $expected"

expected_lc=$(printf '%s' "$expected" | tr '[:upper:]' '[:lower:]')

printf 'Frozen preregistration:\n'
printf '  plan:     %s\n' "$plan"
printf '  expected: %s\n' "$expected_lc"
printf '  observed: %s\n' "$observed"

if [ "$expected_lc" != "$observed" ]; then
  printf '\nfail: the frozen preregistration changed after it was bound.\n' >&2
  printf 'The convergence/Gauntlet run must NOT continue until the change is\n' >&2
  printf 'explicitly reviewed and re-frozen (owner-approved rebind of\n' >&2
  printf 'source.spec.sha256). This check never rewrites the stored hash.\n' >&2
  printf 'VERDICT: FAIL\n' >&2
  exit 1
fi

printf '\nVERDICT: PASS\n'
printf 'Preregistration hash is intact.\n'
