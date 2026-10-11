#!/usr/bin/env bash
# T01 teeth — application-owned ports and preflight blast radius.
#
#   Tooth 1: inject an unknown provider payload type into a core application path. Expected: the
#     mechanical boundary check rejects the leakage, so translation at an adapter is required.
#   Tooth 2: start with one missing required adapter credential and one optional artifact adapter
#     unavailable. Expected: required consequential capability blocks with a typed error, the
#     optional one degrades, and an unrelated healthy capability still reports ready.
#
# Usage: run-teeth.sh   (exit 0 = both teeth behaved as specified)
set -u
WT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
test -f "$WT/.agents/plans/godspeed-casework-cognitive-environment.plan.yaml" || {
  echo "run-teeth: cannot locate the plan from $WT - refusing to report vacuous teeth" >&2
  exit 2
}
MOD="$WT/apps/godspeed-casework-go"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"; rm -f "$MOD/internal/ports/zz_tooth_injected_leak.go"' EXIT
fails=0
say() { printf '%s\n' "$*" ; }
cd "$MOD" || exit 99

say "== Tooth 1: a provider payload type must not survive in the core =="
# (a) non-vacuity first: the scanner must catch each forbidden shape in a synthetic source.
if go test ./internal/boundary/ -run TestScanDetectsInjectedViolations -v > "$TMP/inject.out" 2>&1; then
  say "   ok: scanner detected all $(grep -c -- '--- PASS' "$TMP/inject.out") injected violation(s)"
else
  say "   TOOTH FAILED: the scanner missed an injected violation:"
  tail -12 "$TMP/inject.out" | sed 's/^/     /'
  fails=1
fi
# (b) the real injection: an unknown provider payload type placed in a core package.
LEAK="$MOD/internal/ports/zz_tooth_injected_leak.go"
printf 'package ports\n\n// injected by the T01 tooth: a provider payload type in the core\ntype sfwpEnvelope struct{ WireCaseID string }\n' > "$LEAK"
if go test ./internal/boundary/ -run TestCorePackagesCarryNoProviderVocabulary > "$TMP/with_leak.out" 2>&1; then
  say "   TOOTH FAILED: the boundary gate passed with a provider type in the core"
  fails=1
else
  say "   ok: with the provider type present the gate FAILS (leakage rejected):"
  grep -m2 'boundary violation' "$TMP/with_leak.out" | sed 's/^/     /'
fi
rm -f "$LEAK"
if go test ./internal/boundary/ -run TestCorePackagesCarryNoProviderVocabulary > "$TMP/clean.out" 2>&1; then
  say "   ok: with the injection removed the gate passes again"
else
  say "   TOOTH FAILED: the gate stayed red after removing the injection"
  fails=1
fi

say ""
say "== Tooth 2: missing required credential + unavailable optional adapter =="
# (a) contract level: an unrelated healthy capability must still report ready while a required one blocks.
if go test ./internal/preflight/ -run 'TestPreflightBlastRadius|TestUnwiredCapabilityIsUnavailableNotReady|TestFailingProbeKeepsCauseAndClassification' -v > "$TMP/preflight.out" 2>&1; then
  say "   ok: preflight blast-radius tests pass ($(grep -c -- '--- PASS' "$TMP/preflight.out") test(s))"
else
  say "   TOOTH FAILED: preflight blast-radius tests failed:"
  tail -14 "$TMP/preflight.out" | sed 's/^/     /'
  fails=1
fi
# (b) end-to-end: the real binary against the fixture config, with the repository credential unset.
if ! go build -o "$TMP/godspeed-casework" ./cmd/godspeed-casework; then
  say "   TOOTH FAILED: the binary did not build"
  fails=1
else
  FIXTURE="$WT/.agents/evidence/godspeed-casework-cognitive-environment/T01/fixtures/blast-radius.json"
  set +e
  GODSPEED_FIXTURE_AUTHORITY_TOKEN=present "$TMP/godspeed-casework" -config "$FIXTURE" > "$TMP/cli.out" 2>&1
  rc=$?
  set -e
  say "   end-to-end exit code: $rc (expect 2 = refuse to proceed)"
  sed 's/^/     /' "$TMP/cli.out"
  [ "$rc" -eq 2 ] || { say "   TOOTH FAILED: expected exit 2"; fails=1; }
  grep -q 'blocking=\[[^]]*repository' "$TMP/cli.out" || { say "   TOOTH FAILED: repository (missing credential) must be blocking"; fails=1; }
  grep -q 'degraded=\[[^]]*artifact' "$TMP/cli.out" || { say "   TOOTH FAILED: the optional artifact capability must be degraded"; fails=1; }
  # The blocking fault must be TYPED and ATTRIBUTED: a config fault for the repository capability,
  # distinct from the availability faults of the capabilities that have no adapter registered yet.
  grep -qE 'err=config:.*\(repository/secret\)' "$TMP/cli.out" || { say "   TOOTH FAILED: the blocking fault must be a typed config error attributed to repository"; fails=1; }
  grep -qE 'err=unavailable:' "$TMP/cli.out" || { say "   TOOTH FAILED: capabilities without an adapter must report a typed unavailable error"; fails=1; }
  say "   note: the CLI cannot show a READY unrelated capability until real adapters are wired (T04/T05);"
  say "         that half of the tooth is proven at contract level by TestPreflightBlastRadius above."
fi

say ""
say "== React contract boundary (the contract shape T01 owes; GATE_UI is T03's) =="
CONTRACTS="$WT/apps/godspeed-cognitive-ui/contracts"
if [ -d "$CONTRACTS" ]; then
  if grep -rniE '@react-three|copilotkit|@copilot|fetch\(|EventSource|XMLHttpRequest|godspeed-casework-go|sfwp|ndjson' "$CONTRACTS"/*.ts > "$TMP/contracts.out" 2>&1; then
    say "   TOOTH FAILED: the contract package references a renderer, an agent framework or a transport:"
    sed 's/^/     /' "$TMP/contracts.out"
    fails=1
  else
    say "   ok: no renderer, agent-framework, transport or provider reference in the contract package"
  fi
  for need in WorldAdapter InteractionAdapter TemporalAdapter ArtifactAdapter AgentAdapter; do
    grep -rq "$need" "$CONTRACTS"/*.ts || { say "   TOOTH FAILED: contract $need is missing"; fails=1; }
  done
else
  say "   TOOTH FAILED: no contract package at $CONTRACTS"
  fails=1
fi

say ""
if [ "$fails" -eq 0 ]; then
  say "TEETH RESULT: both teeth behaved as specified"
else
  say "TEETH RESULT: FAILED"
fi
exit $fails
