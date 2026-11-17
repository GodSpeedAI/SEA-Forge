#!/usr/bin/env sh
# NT-11 (S2, WP-21b): hard gate not waivable: allocate_authority and integrate_govern cannot be not_applicable without an approval; a denied approval is AUTHORITY_DENIED.
# Exit 0 is PASS.
set -eu
h="$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh"
"$h" sea-forge-server conformance_fde_scenarios 1 nt_11_scenarios nt_11_both_hard_gates_and_the_non_hard_waiver review1_gate_item_moves_only_on_a_passing_assessment
