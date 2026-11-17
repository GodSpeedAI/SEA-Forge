#!/usr/bin/env sh
# AT-10 (S2, WP-21b): status axes and dispositions: each case-level disposition yields its mapped lifecycle; episode-level outcomes never close; a run without settlement shows result absent; a blocked move names its dimension and authorizer; approvals are single use.
# Exit 0 is PASS.
set -eu
h="$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh"
"$h" sea-forge-server conformance_fde_scenarios 1 at_10_scenarios at_10_dispositions_axes_and_blocked_move an_approval_authorises_one_record
