#!/usr/bin/env sh
# NT-16 (WP-14): an absent anchor or one not before the first run fails the order check (tamper check). Anchoring row: PARTIAL without OD-6.
. "$(dirname "$0")/_lib.sh"
"$(cd "$(dirname "$0")" && pwd)/_cargo-test.sh" sea-forge-server conformance_field_case 1 nt_16 || exit 1
partial "the tamper/order check refuses as specified; the server push time is not read from the remote (AnchorTimeLocalOnly) and the anchor is a shared-OS-user tag without a protected namespace (RD-26, OD-6)"
