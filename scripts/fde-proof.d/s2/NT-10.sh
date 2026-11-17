#!/usr/bin/env sh
# NT-10 (S2, WP-21b): no self-settlement (I-2): the executor, and an agent in the executor's delegation chain, are refused; the assessor of prove_reliability is not the proposer or behind it.
# Exit 0 is PASS.
set -eu
h="$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh"
"$h" sea-forge-server conformance_fde_scenarios 1 nt_10_scenarios i2_gate_assessment_on_the_delegation_chain
