#!/usr/bin/env sh
# AT-05 (WP-14 SEA-Forge half, WP-15 Cognate half): semantic exhaust complete and authentic.
# Exit 0 would be PASS; this script exits 10 (PARTIAL) when the SEA-Forge half passes, because the
# runtime half (WP-15: Cognate run/invocation events) is absent and no emitter signs yet (WP-33).
set -eu
here=$(cd "$(dirname "$0")" && pwd)
t=conformance_interaction_events
# SEA-Forge half: every ledger record and authority decision yields one schema-valid event,
# source.record_id and authority.decision_id resolve at their owner, correlation is structured,
# a replayed duplicate changes no count, an emitter failure leaves the write committed.
"$here/_cargo-test.sh" sea-forge-server "$t" 5 at_05
"$here/_cargo-test.sh" sea-forge-server "$t" 1 replay_and_redelivery injected_emitter_failure \
  an_outbox_write_failure an_episode_in_another_world vendored_extension_schema
cat >&2 <<'MSG'
AT-05 PARTIAL: the SEA-Forge half passed (ledger and I-2 decision events). Not covered:
  - Cognate run and invocation events (WP-15, absent): 100 % of consequential events cannot be claimed;
  - signature check: nothing is signed before WP-33, so authenticity rests on source-record
    resolution and the independent-emitter rule (E 5), not on a verified signature.
MSG
exit 10
