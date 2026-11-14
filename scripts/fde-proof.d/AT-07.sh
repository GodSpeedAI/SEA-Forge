#!/usr/bin/env sh
# AT-07: five classes (S1: confirmed plus one of contradicted / observed_undeclared must arise on the episode set).
. "$(dirname "$0")/_lib.sh"
rt_test 3 -p sxr-observed --test wp17 at07; rc=$?; [ $rc -eq 0 ] || exit $rc
rt_test 1 -p realitytrace --test compare; rc=$?; [ $rc -eq 0 ] || exit $rc
st=$(cell_status AT-07)
[ "$st" = fail ] && exit 1
[ "$st" = absent ] && partial "producer tests pass; the S1 episode set was not run, so arising classes are unchecked"
partial "classes arose on the real episode set (see cell results); declared_unobserved and any unarising class NOT TESTED; postcondition expectations insufficient (DomainForge accepts only at precondition)"
