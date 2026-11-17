#!/usr/bin/env sh
# AT-15 (S2, WP-21b): resumption, programmatic part: case.continuation states the next move, the authorizer and what would settle it, validates against continuation.v1, and marks unread sections unknown (the operator part is S4).
# Exit 0 is PASS.
set -eu
h="$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh"
"$h" sea-forge-server conformance_fde_scenarios 1 at_15_continuation_states_the_next_move read_verbs_answer_unknown_and_non_field_cases
