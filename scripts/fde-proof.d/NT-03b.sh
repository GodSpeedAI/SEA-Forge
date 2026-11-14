#!/usr/bin/env sh
# NT-03b: a deleted or tampered record is CHAIN_VERIFY_FAILED.
. "$(dirname "$0")/_lib.sh"
rt_test 2 -p sxr-observed --test wp16 nt03b; rc=$?; [ $rc -eq 0 ] || exit $rc
rt_test 1 -p realitytrace --test observed; rc=$?; [ $rc -eq 0 ] || exit $rc
st=$(cell_status NT-03b)
[ "$st" = ok ] && exit 0
[ "$st" = absent ] && partial "producer tests pass; deletion on the real episode ledger copy was not run"
exit 1
