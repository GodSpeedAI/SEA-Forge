#!/usr/bin/env sh
# NT-13 (WP-14): S or F on an exploratory episode is refused. Exit 0 is PASS.
exec "$(cd "$(dirname "$0")" && pwd)/_cargo-test.sh" sea-forge-server conformance_field_case 1 nt_13
