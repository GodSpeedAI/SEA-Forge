#!/usr/bin/env sh
# NT-21 (WP-18; S2 gate): a `confirmed` element that loses its evidence is LOST_CAPABILITY and an unlisted
# weakening is UNLISTED_WEAKENING, each refusing promotion. Not in the frozen S1 test set, so the S1 runner
# does not select it. Exit 0 is PASS; exit 10 (PARTIAL) when the RealityTrace half cannot run.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
"$here/_cargo-test.sh" sea-forge-authority world_pin 1 nt21
exec "$here/_rt-test.sh" 2 nt21
