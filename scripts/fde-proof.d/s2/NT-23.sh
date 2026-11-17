#!/usr/bin/env sh
# NT-23 (S2, WP-21b): no close without a close disposition: advancing a Field Case to Completed needs a case-level close-* disposition; a close disposition closes.
# Exit 0 is PASS.
set -eu
h="$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh"
"$h" sea-forge-server conformance_fde_scenarios 1 nt_23_scenarios
