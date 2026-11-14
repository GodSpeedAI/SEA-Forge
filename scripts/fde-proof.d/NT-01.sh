#!/usr/bin/env sh
# NT-01 (WP-14): a second freeze for the same (case, item, n) is refused whatever its digest. Exit 0 is PASS.
exec "$(cd "$(dirname "$0")" && pwd)/_cargo-test.sh" sea-forge-server conformance_field_case 1 nt_01
