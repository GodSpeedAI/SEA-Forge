#!/usr/bin/env sh
# AT-08: input sensitivity (derive half WP-16, compare half WP-17; episode variant via --until).
. "$(dirname "$0")/_lib.sh"
rt_test 1 -p sxr-observed --test wp16 at08; rc=$?; [ $rc -eq 0 ] || exit $rc
rt_test 1 -p sxr-observed --test wp17 at08_nt03; rc=$?; [ $rc -eq 0 ] || exit $rc
st=$(cell_status AT-08)
[ "$st" = ok ] && exit 0
[ "$st" = absent ] && partial "producer tests pass; the --until variant over the real episode ledger was not run"
exit 1
