#!/usr/bin/env sh
set -eu

INPUT=.agents/reports/workbench-completion-eval-inputs.md
FIXTURE=.agents/reports/workbench-completion-eval-exclusions.json

fail() {
  printf 'workbench_completion_eval_inputs_error: %s\n' "$1" >&2
  exit 1
}

[ -f "$INPUT" ] || fail "missing $INPUT"
[ -f "$FIXTURE" ] || fail "missing $FIXTURE"

if rg -n '<[A-Z_]+>' "$INPUT" >/dev/null; then
  fail "evaluator input still contains a protocol placeholder"
fi

jq -e '
  .eval_protocol == "sea-forge-governed-workbench-frontend-v0.1"
  and .claim_platform == "linux"
  and (.declared_exclusions | type == "array")
  and (.declared_exclusions | length == 11)
  and all(.declared_exclusions[]; (.story_ids | type == "array" and length > 0)
      and (.reason | type == "string" and length > 0)
      and (.effect_on_claim | type == "string" and length > 0))
  and (.non_story_exclusions | type == "array" and length > 0)
' "$FIXTURE" >/dev/null || fail "fixture has an invalid completion-evaluation shape"

expected_story_ids='["10.1","10.2","10.3","10.4","10.5","10.6","11.2","11.3","11.7","14.10","14.2","14.6","14.7","14.8","14.9","15.1","15.2","15.3","15.4","15.5","15.6","15.7","16.3","2.3","4.2","4.5","4.6","6.1","6.3","6.4","6.5","6.6","7.6","8.6","9.4","9.5","9.6","9.7"]'
actual_story_ids="$(jq -c '[.declared_exclusions[].story_ids[]] | sort' "$FIXTURE")"
[ "$actual_story_ids" = "$expected_story_ids" ] || fail "fixture story IDs differ from the owner-approved exclusion set"

grep -Fq "$FIXTURE" "$INPUT" || fail "evaluator input does not reference the machine-checkable exclusion fixture"
grep -Fq 'Execution mode required: `INTEGRATED`' "$INPUT" || fail "evaluator input must prohibit mocked completion claims"
grep -Fq 'just workbench-package' "$INPUT" || fail "evaluator input lacks the packaged Linux build command"
grep -Fq 'just cell-seed /tmp/sea-forge-workbench-eval-cell' "$INPUT" || fail "evaluator input lacks the real temporary-cell setup"
grep -Fq 'just workbench-demo /tmp/sea-forge-workbench-eval-cell' "$INPUT" || fail "evaluator input lacks the packaged application start command"

printf 'workbench completion evaluator inputs check passed\n'
