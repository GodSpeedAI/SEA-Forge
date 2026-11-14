#!/usr/bin/env sh
# NT-16 (WP-14): an absent anchor or one not before the first run fails the order check. Exit 0 is PASS.
exec "$(cd "$(dirname "$0")" && pwd)/_cargo-test.sh" sea-forge-server conformance_field_case 1 nt_16
