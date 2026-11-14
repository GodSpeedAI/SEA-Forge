#!/usr/bin/env sh
# NT-05 (WP-18): a denied, unavailable (unknown, pending, none named), expired or digest-mismatched decision
# cannot promote; one code each (PROMOTION_DENIED, AUTHORITY_UNAVAILABLE, DECISION_EXPIRED,
# DECISION_DIGEST_MISMATCH), and the pin and the transition list do not change. Exit 0 is PASS.
exec "$(cd "$(dirname "$0")" && pwd)/_cargo-test.sh" sea-forge-authority world_pin 1 nt05
