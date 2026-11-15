#!/usr/bin/env sh
# NT-13 (S2, WP-21a): the scenarios of the S2 contracts that carry NT-13, driven through the real server.
# Exit 0 is PASS.
exec "$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh" sea-forge-server conformance_fde_scenarios 1 nt_13_scenarios
