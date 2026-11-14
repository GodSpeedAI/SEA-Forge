#!/usr/bin/env sh
# NT-03: a changed occurrence changes the result (derive half WP-16, compare half WP-17).
. "$(dirname "$0")/_lib.sh"
rt_test 1 -p sxr-observed --test wp16 nt03_; rc=$?; [ $rc -eq 0 ] || exit $rc
rt_test 1 -p sxr-observed --test wp17 nt03; rc=$?; [ $rc -eq 0 ] || exit $rc
st=$(cell_status NT-03)
[ "$st" = ok ] && exit 0
[ "$st" = absent ] && partial "producer tests pass; a real re-run with a changed outcome was not run"
exit 1
