#!/usr/bin/env sh
# NT-22 (S2, WP-21b): a suspended case refuses freeze, settlement, delegation, episode-level and non-lifting dispositions; it still accepts revisions, strategies and gates; the lifecycle function is golden-tested.
# Exit 0 is PASS.
set -eu
h="$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh"
"$h" sea-forge-server conformance_fde_scenarios 1 nt_22_scenarios nt_22_suspended_case_end_to_end
"$h" sea-forge-server lib 1 lifecycle_golden
