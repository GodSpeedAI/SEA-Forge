#!/usr/bin/env bash
# T03 teeth — the interaction grammar is independent of renderer, agent, and provider.
#
#   Tooth 1: replace the optional scene and agent adapters with no-op/test adapters. Expected: the
#     UI-core tests still execute against the fixture adapter, with narration degrading honestly.
#   Tooth 2: swap the fixture provider for a second, structurally different provider implementing the
#     same world port. Expected: the SAME shared interaction scenario passes unchanged — no fixture
#     structure may have leaked into the core grammar.
#   Tooth 3 (non-vacuity of the purity gate): inject `react` into the core; the purity scan that
#     GATE_UI runs must FAIL while the injection is present, and pass again once removed.
#
# Usage: run-teeth.sh   (exit 0 = all teeth behaved as specified)
set -u
WT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
test -f "$WT/.agents/plans/godspeed-casework-cognitive-environment.plan.yaml" || {
  echo "run-teeth: cannot locate the plan from $WT - refusing to report vacuous teeth" >&2
  exit 2
}
APP="$WT/apps/godspeed-cognitive-ui"
cd "$APP" || exit 99
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT
fails=0
say() { printf '%s\n' "$*"; }

say "== Tooth 1: scene and agent adapters replaced by no-op/test adapters =="
if bun test src/core/scenario.test.ts src/core/narration.test.ts -t 'tooth 1' >"$OUT/tooth1.log" 2>&1; then
  passes=$(grep -c '(pass)' "$OUT/tooth1.log")
  if [ "$passes" -ge 2 ]; then
    say "   ok: $passes test(s) pass with the recording scene adapter and the unavailable agent"
  else
    say "   TOOTH FAILED: filter matched fewer tests than expected ($passes) — possibly vacuous"
    fails=1
  fi
else
  say "   TOOTH FAILED: core tests did not pass with test adapters:"
  tail -12 "$OUT/tooth1.log" | sed 's/^/     /'
  fails=1
fi

say ""
say "== Tooth 2: fixture provider swapped for a structurally different provider =="
if bun test src/core/scenario.test.ts -t 'tooth 2' >"$OUT/tooth2.log" 2>&1; then
  fixture=$(grep -c '(pass).*fixture world provider' "$OUT/tooth2.log")
  alt=$(grep -c '(pass).*alt (adjacency-map) provider' "$OUT/tooth2.log")
  if [ "$fixture" -eq 1 ] && [ "$alt" -eq 1 ]; then
    say "   ok: the ONE shared interaction scenario passes under both providers"
    say "      (fixture world provider and adjacency-map alt provider: 1 pass each; zero scenario copies)"
  else
    say "   TOOTH FAILED: expected exactly one pass per provider (fixture=$fixture alt=$alt)"
    fails=1
  fi
else
  say "   TOOTH FAILED: the shared scenario failed under at least one provider:"
  tail -12 "$OUT/tooth2.log" | sed 's/^/     /'
  fails=1
fi

say ""
say "== Tooth 3: the purity gate fails when a renderer import is really injected =="
LEAK="src/core/zz_tooth_injected_react.ts"
printf "import { useState } from 'react'\nexport const leak = useState\n" > "$LEAK"
if bun test src/core/purity.test.ts >"$OUT/purity-leaked.log" 2>&1; then
  say "   TOOTH FAILED: the purity gate passed WITH react injected into the core"
  fails=1
else
  say "   ok: with react present the purity scan FAILS (renderer authority rejected):"
  grep -m2 'names react\|imports react' "$OUT/purity-leaked.log" | sed 's/^/     /'
fi
rm -f "$LEAK"
if bun test src/core/purity.test.ts >"$OUT/purity-clean.log" 2>&1; then
  say "   ok: with the injection removed the purity gate passes again"
else
  say "   TOOTH FAILED: the purity gate stayed red after removing the injection"
  fails=1
fi

say ""
if [ "$fails" -eq 0 ]; then
  say "TEETH RESULT: all three teeth behaved as specified"
else
  say "TEETH RESULT: FAILED"
fi
exit "$fails"
