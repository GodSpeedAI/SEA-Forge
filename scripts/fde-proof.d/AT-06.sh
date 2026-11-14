#!/usr/bin/env sh
# AT-06: observed representation computed (WP-16): byte-identical derive, listed occurrences, coverage, profile.
. "$(dirname "$0")/_lib.sh"
rt_test 2 -p sxr-observed --test wp16 at06; rc=$?; [ $rc -eq 0 ] || exit $rc
rt_test 1 -p realitytrace --test observed; rc=$?; [ $rc -eq 0 ] || exit $rc
st=$(cell_status AT-06)
[ "$st" = ok ] && exit 0
[ "$st" = absent ] && partial "producer tests pass; no episode cell, so derive twice over the real episode ledger was not checked"
exit 1
