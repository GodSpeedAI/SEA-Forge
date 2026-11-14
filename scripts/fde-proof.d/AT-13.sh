#!/usr/bin/env sh
# AT-13 (WP-18): capability delta. The same subsequent-episode records classified under the old and the new
# world (re-classification) plus a control variant without the newly declared step, against a frozen
# falsifier. The result is GAIN or NO_GAIN and the test passes either way; the GAIN/NO_GAIN value is in the
# RealityTrace test output (`capability-delta.v1`). SEA-Forge half: a new episode freezes under the new
# world_ref and the earlier episode keeps its own. Exit 10 (PARTIAL): the new world exists only through a
# promotion approved under pre-S3 identity (predictions.yaml), and ordering of the declared step is `unknown`
# in the model; also exit 10 when the RealityTrace half cannot run. R-58 is PROVEN only on GAIN, never by this
# script's exit code.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
"$here/_cargo-test.sh" sea-forge-server conformance_world_pin 1 wp18_
code=0
"$here/_rt-test.sh" 3 at13 || code=$?
[ "$code" -eq 0 ] || [ "$code" -eq 10 ] || exit 1
code2=0
"$here/_rt-test.sh" 1 nt38 || code2=$?
[ "$code2" -eq 0 ] || [ "$code2" -eq 10 ] || exit 1
echo "AT-13 PARTIAL: see the header; the delta result (GAIN or NO_GAIN) is reported by the RealityTrace test, not by this exit code" >&2
exit 10
