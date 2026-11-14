#!/usr/bin/env sh
# AT-12 (WP-18): governed revision. RealityTrace half: held-out rule (no overlap with the diagnosis input),
# the E section 7 regression matrix, over real compare results. SEA-Forge half: promotion over the existing
# approval-gated semantic_change transition, proposer is not the approver, both worlds registered, refused
# attempts and a non-promotion retained. Exit 10 (PARTIAL) when both halves pass; exit 0 is not offered:
# the promotion approval is a human-reserved decision and before S3 it is an operator identity at assurance
# `session` (predictions.yaml, RD-21). Exit 10 also when the RealityTrace half cannot run.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
"$here/_cargo-test.sh" sea-forge-authority world_pin 5 at12
"$here/_cargo-test.sh" sea-forge-server conformance_world_pin 1 wp18_
code=0
"$here/_rt-test.sh" 5 at12 || code=$?
[ "$code" -eq 0 ] || [ "$code" -eq 10 ] || exit 1
code2=0
"$here/_rt-test.sh" 2 nt55 || code2=$?
[ "$code2" -eq 0 ] || [ "$code2" -eq 10 ] || exit 1
cat >&2 <<'MSG'
AT-12 PARTIAL: both halves' tests passed (or the RealityTrace half is reported NOT_RUN above).
  - the promotion approval is an operator identity at assurance `session`, not a human-reserved A3
    decision (S3, OD-5): "proposer is not the approver" is enforced on actor ids, not on a person;
  - the regression matrix covers what `observed compare` classifies (structural and precondition-evaluable
    elements); postcondition-dependent elements are `insufficient`;
  - a matrix that lists a weakening (escalated approval class) is refused, not promoted.
MSG
exit 10
