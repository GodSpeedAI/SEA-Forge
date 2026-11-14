#!/usr/bin/env sh
# NT-06: a silent frozen evidence source stays insufficient / evidence_source_silent, result U.
. "$(dirname "$0")/_lib.sh"
rt_test 1 -p sxr-observed --test wp17 nt06; rc=$?; [ $rc -eq 0 ] || exit $rc
st=$(cell_status NT-06)
[ "$st" = ok ] && exit 0
[ "$st" = absent ] && partial "producer test passes; a silent source on the real cell was not run"
exit 1
