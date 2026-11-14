#!/usr/bin/env sh
# NT-42 (WP-14): result outside S/F/U/C, or reliability merged into the result, is refused. Exit 0 is PASS.
exec "$(cd "$(dirname "$0")" && pwd)/_cargo-test.sh" sea-forge-server conformance_field_case 1 nt_42
