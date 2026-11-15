#!/usr/bin/env sh
# AT-04 (WP-14): freeze (case record, RealityTrace records, anchor) is complete before any move and precedes the first run record.
# The producer tests pass, but the oracle also asks for an externally anchored time (anchor push time earlier than the first run, verifier
# receipt): every real receipt is AnchorTimeLocalOnly (OD-6 not granted, RD-26), so the row is PARTIAL by the frozen rule.
. "$(dirname "$0")/_lib.sh"
"$(cd "$(dirname "$0")" && pwd)/_cargo-test.sh" sea-forge-server conformance_field_case 1 at_04 || exit 1
partial "order check and digest equality pass in the producer tests; anchor time is the local clock only (AnchorTimeLocalOnly, OD-6), execution-side I-1 gate not wired (WP-20), so freeze-before-execution is verified after the fact, not enforced"
