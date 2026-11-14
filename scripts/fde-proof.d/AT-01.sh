#!/usr/bin/env sh
# AT-01 (WP-13): the declared SOP model validates, its world_ref is reproducible, the two
# Settlement concepts stay distinct. Exit 0 is PASS.
exec "$(cd "$(dirname "$0")/.." && pwd)/check-fde-sop-model.sh"
