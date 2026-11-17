#!/usr/bin/env sh
# NT-11 (S2, WP-21b): hard gate not waivable: allocate_authority and integrate_govern cannot be not_applicable without an approval; a denied approval is AUTHORITY_DENIED.
# Exit 0 is PASS.
set -eu
h="$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh"
"$h" sea-forge-server conformance_fde_scenarios 1 nt_11_scenarios
