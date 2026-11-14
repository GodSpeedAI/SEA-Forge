#!/usr/bin/env sh
# AT-05: semantic exhaust complete and authentic. SEA-Forge half (WP-14) and Cognate half (WP-15).
# Always PARTIAL when both pass: no emitter signs before WP-33, so the "check signature" step cannot run.
. "$(dirname "$0")/_lib.sh"
here=$(cd "$(dirname "$0")" && pwd)
t=conformance_interaction_events
"$here/_cargo-test.sh" sea-forge-server "$t" 5 at_05 || exit 1
"$here/_cargo-test.sh" sea-forge-server "$t" 1 replay_and_redelivery injected_emitter_failure \
  an_outbox_write_failure an_episode_in_another_world vendored_extension_schema || exit 1
need_sibling "$COG_DIR" "$COG_EXPECT" Cognate
command -v bun >/dev/null 2>&1 || not_run "bun missing for the Cognate half"
out=$(mktemp "$TMPDIR/fde-cog.XXXXXX")
( cd "$COG_DIR" && bun test ./packages/realitytrace/test/interaction-event.test.ts ) >"$out" 2>&1; code=$?
tail -n 8 "$out" >&2
[ "$code" -eq 0 ] || { rm -f "$out"; echo "FAIL: Cognate interaction-event tests failed" >&2; exit 1; }
n=$(sed -n 's/^ *\([0-9][0-9]*\) pass.*/\1/p' "$out" | tail -1); rm -f "$out"
[ "${n:-0}" -ge 10 ] || { echo "FAIL: Cognate half ran ${n:-0} tests" >&2; exit 1; }
partial "SEA-Forge half and Cognate half pass; emitter signatures arrive with WP-33 so the signature check cannot run; authenticity rests on source-record resolution and the independent-emitter rule"
