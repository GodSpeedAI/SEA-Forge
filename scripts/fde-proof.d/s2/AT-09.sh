#!/usr/bin/env sh
# AT-09 (S2, WP-21b): independent settlement: an agent verifier outside the executor's delegation chain settles (I-2); assessor/proposer I-2 on the chain. The human-settlement A3 clause is WP-31 and keeps the row PARTIAL until S3 (G AT-09).
# Exit 0 is PASS.
set -eu
h="$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh"
"$h" sea-forge-server conformance_fde_scenarios 1 at_09_scenarios nt_10_scenarios i2_gate_assessment_on_the_delegation_chain
echo "REASON: the programmatic I-2 and derived-verifier-context checks pass; the human settlement's A3 proof is owned by WP-31 (G AT-09: PARTIAL pre-S3)"
exit 10
