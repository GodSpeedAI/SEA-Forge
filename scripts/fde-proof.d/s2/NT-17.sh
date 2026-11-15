#!/usr/bin/env sh
# NT-17 (S2, WP-21a): the scenarios of the S2 contracts that carry NT-17, driven through the real server.
# Exit 0 is PASS.
exec "$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh" sea-forge-server conformance_fde_scenarios 1 nt_17_scenarios
