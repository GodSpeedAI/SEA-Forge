#!/usr/bin/env sh
# NT-04: an undeclared occurrence is not an authorized change. Before half and after half (RealityTrace WP-17/18)
# and the SEA-Forge promotion records the transition (WP-18). The promotion approval is at session assurance.
. "$(dirname "$0")/_lib.sh"
rt_test 1 -p sxr-observed --test wp17 nt04_undeclared; rc=$?; [ $rc -eq 0 ] || exit $rc
rt_test 1 -p sxr-observed --test wp18 nt04; rc=$?; [ $rc -eq 0 ] || exit $rc
"$(dirname "$0")/_cargo-test.sh" sea-forge-authority world_pin 1 nt04 || exit 1
partial "both halves pass; the authorizing approval is a session-assurance approval (A3 and a human-reserved decision need S3)"
