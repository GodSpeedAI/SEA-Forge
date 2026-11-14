#!/usr/bin/env sh
# NT-07 (WP-18; re-pin variant WP-22): settled evidence stays bound to its world_ref after promotion and after
# re-pin. SEA-Forge: authority decisions (read by their entry) and case records (frozen and settled under the
# old world). RealityTrace: a classification is stamped with the world it was computed under. Exit 0 is PASS;
# exit 10 (PARTIAL) when the RealityTrace half cannot run.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
"$here/_cargo-test.sh" sea-forge-authority world_pin 1 nt07
"$here/_cargo-test.sh" sea-forge-server conformance_world_pin 1 wp18_
exec "$here/_rt-test.sh" 1 at13_the_same_records
