#!/usr/bin/env sh
# NT-02: no verified conclusion without identity (INGEST_REJECTED_IDENTITY) or evidence (INGEST_REJECTED_EVIDENCE).
. "$(dirname "$0")/_lib.sh"
rt_test 2 -p sxr-observed --test wp16 nt02; rc=$?; [ $rc -eq 0 ] || exit $rc
rt_test 1 -p realitytrace --test observed; rc=$?; [ $rc -eq 0 ] || exit $rc
st=$(cell_status NT-02)
[ "$st" = ok ] && exit 0
[ "$st" = absent ] && partial "producer tests pass; the refusal on the real cell events was not run"
exit 1
