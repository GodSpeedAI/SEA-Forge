#!/usr/bin/env sh
# NT-08 (WP-18): a candidate that breaks a prior confirmed element not in the intended changes fails the
# regression matrix (RealityTrace) and a failing, forged or absent matrix blocks promotion with
# REGRESSION_FAILED even with a valid approval (SEA-Forge). Exit 0 is PASS; exit 10 (PARTIAL) when the
# RealityTrace half cannot run.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
"$here/_cargo-test.sh" sea-forge-authority world_pin 1 nt08
exec "$here/_rt-test.sh" 1 nt08
