#!/usr/bin/env bash
set -u
G=/home/sprime01/projects/gauntlet
T15=/tmp/t15-corpus
declare -a CASES=(
  "affordable-A1 e2e-happy default"
  "affordable-A2 e2e-happy default"
  "affordable-A3 e2e-happy default"
  "lying-B1 e2e-lying default"
  "lying-B2 e2e-lying default"
  "lying-B3 e2e-lying default"
  "budget-C1 e2e-happy r1"
  "budget-C2 e2e-happy r1"
  "budget-C3 e2e-happy r2"
)
for entry in "${CASES[@]}"; do
  set -- $entry
  CID=$1; SCN=$2; BUD=$3
  T="$T15/targets/$CID"
  rm -rf "$T"; cp -r "$G/examples/demo-calculator" "$T"; chmod +x "$T/scripts/"*.sh
  sed -i 's/default: prime-agent/default: mock-agent/' "$T/GAUNTLET.md"
  if [ "$BUD" != default ]; then
    sed -i "s/max_rounds: 8/max_rounds: ${BUD#r}/" "$T/GAUNTLET.md"
  fi
  export GAUNTLET_STATE_DIR="$T15/runs/$CID/state" GAUNTLET_ARTIFACT_DIR="$T15/runs/$CID/evidence"
  export GAUNTLET_TOOL_TEST="$T/scripts/check.sh" GAUNTLET_MAX_ATTEMPTS_PER_UNIT=2
  export GAUNTLET_MOCK_SCENARIO="$SCN"
  cd "$T"
  echo "=== $CID scenario=$SCN budget=$BUD ==="
  timeout 240 /home/sprime01/projects/gauntlet/target/debug/gauntlet run "Build and verify the calculator page" 2>&1 | tail -4
done
