#!/usr/bin/env sh
# NT-17 (WP-14): a reference body carrying result content is refused. Exit 0 is PASS.
exec "$(cd "$(dirname "$0")" && pwd)/_cargo-test.sh" sea-forge-server conformance_field_case 1 nt_17
