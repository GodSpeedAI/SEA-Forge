#!/usr/bin/env sh
# AT-11 (S2, WP-21b): gates: all four states reachable; new evidence marks the gate to reassess; nothing advances automatically.
# Exit 0 is PASS.
set -eu
h="$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh"
"$h" sea-forge-server conformance_fde_scenarios 1 at_11_gates
