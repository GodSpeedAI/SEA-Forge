#!/usr/bin/env sh
# AT-04 (WP-14): freeze (case record, RealityTrace records, anchor) is complete before any move and precedes the first run record. Exit 0 is PASS.
exec "$(cd "$(dirname "$0")" && pwd)/_cargo-test.sh" sea-forge-server conformance_field_case 1 at_04
