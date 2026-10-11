#!/usr/bin/env sh
# Print the explicit frozen requirement IDs from the frozen preregistration,
# one per line, in canonical order: edges (E*), global invariants (I1–I15),
# envelope identity rules (ENV-I1–I8). Prefers PyYAML; falls back to a
# narrowly scoped extraction of those three sections only.
#
# Overrides for isolated testing:
#   PREREG_FILE (default <repo>/.agents/specs/e2e-preregistration.yml)
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
prereg=${PREREG_FILE:-"$root/.agents/specs/e2e-preregistration.yml"}

[ -f "$prereg" ] || {
  printf 'fail: preregistration not found: %s\n' "$prereg" >&2
  exit 1
}

if command -v python3 >/dev/null 2>&1 &&
  python3 - "$prereg" <<'PY' 2>/dev/null
import sys

try:
    import yaml
except ImportError:
    sys.exit(3)
pre = yaml.safe_load(open(sys.argv[1]))["preregistration"]
ids = list(pre["edges"].keys())
ids += list(pre["global_invariants"].keys())
ids += [rule["id"] for rule in pre["common_semantic_envelope"]["identity_rules"]]
print("\n".join(ids))
PY
then
  exit 0
fi

# Fallback: indentation-scoped sweep of exactly the three frozen sections.
awk '
  /^edges:/ { sec = "edges"; next }
  /^global_invariants:/ { sec = "invariants"; next }
  /^[A-Za-z_]+:/ { if ($0 !~ /^(edges|global_invariants):/) sec = "" }
  sec == "edges" && /^  [E][0-9AB]+:$/ { sub(/:$/, ""); print }
  sec == "invariants" && /^  I[0-9]+:$/ { sub(/:$/, ""); print }
  /id: ENV-I[0-9]+/ {
    line = $0
    sub(/^.*id:[[:space:]]*/, "", line)
    sub(/[",].*$/, "", line)
    print line
  }
' "$prereg"
