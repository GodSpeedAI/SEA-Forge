#!/usr/bin/env sh
# NT-34 (S2, WP-21a, D39): program execution needs a CEP decision of the authority path whose resource_id is
# execution:sha256:<digest of the nine-member execution object>; missing, denied, exhausted or another
# execution is AUTHORITY_DENIED. Exit 0 is PASS.
exec "$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh" sea-forge-server conformance_fde_scenarios 1 nt_34_scenarios
