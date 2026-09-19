#!/usr/bin/env bash
# T00 teeth. Both attacks are executed, not described.
#
#   Tooth 1: modify one byte of a copied spec and run the frozen-hash check.
#            Expected: the binding check detects hash divergence and blocks
#            (non-zero exit) until authority is explicitly re-frozen.
#   Tooth 2: claim an absent Go / GitHub / donor / SFWP capability already exists.
#            Expected: the recorded inventory contradicts the claim, names the
#            missing seam, and leaves the dependent task blocked.
#
# Usage: run-teeth.sh   (exit 0 = both teeth behaved as specified)
set -u
WT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
test -f "$WT/.agents/plans/godspeed-casework-cognitive-environment.plan.yaml" || {
  echo "run-teeth: cannot locate the plan from $WT — refusing to report vacuous teeth" >&2
  exit 2
}
SPEC="$WT/.agents/specs/godspeed.casework-cognitive-environment-spec.yaml"
PLAN="$WT/.agents/plans/godspeed-casework-cognitive-environment.plan.yaml"
VALIDATOR="$WT/.agents/plans/validate-godspeed-casework-cognitive-environment.py"
LAB="$(mktemp -d)"
trap 'rm -rf "$LAB"' EXIT
fail=0
say() { printf '%s\n' "$*"; }

say "== Tooth 1: one-byte spec mutation must be detected and block =="
mkdir -p "$LAB/.agents/plans" "$LAB/.agents/specs"
cp "$VALIDATOR" "$LAB/.agents/plans/validate-godspeed-casework-cognitive-environment.py"
cp "$PLAN"      "$LAB/.agents/plans/godspeed-casework-cognitive-environment.plan.yaml"
cp "$SPEC"      "$LAB/.agents/specs/godspeed.casework-cognitive-environment-spec.yaml"
VALIDATE="$LAB/.agents/plans/validate-godspeed-casework-cognitive-environment.py"
MUTANT="$LAB/.agents/specs/godspeed.casework-cognitive-environment-spec.yaml"

say "-- control: unmutated copy must PASS --"
if python3 "$VALIDATE" > "$LAB/control.out" 2>&1; then
  say "   control exit=0 (expected) :: $(cat "$LAB/control.out")"
else
  say "   control exit non-zero (UNEXPECTED) :: $(cat "$LAB/control.out")"
  fail=1
fi

say "-- attack: flip exactly one byte of the copied spec --"
python3 - "$MUTANT" <<'PY'
import sys
p = sys.argv[1]
b = bytearray(open(p, 'rb').read())
i = b.index(b'\n') + 1                     # first byte of the second line
before = b[i]
b[i] = (before + 1) % 256                  # single-byte mutation
open(p, 'wb').write(bytes(b))
print(f"   mutated byte at offset {i}: {before:#04x} -> {b[i]:#04x}")
PY
if python3 "$VALIDATE" > "$LAB/attack.out" 2>&1; then
  say "   attack exit=0 (TOOTH FAILED: mutation accepted) :: $(cat "$LAB/attack.out")"
  fail=1
else
  say "   attack exit non-zero (expected: blocked) :: $(cat "$LAB/attack.out")"
  grep -qi 'SHA-256 differs' "$LAB/attack.out" || { say "   (blocked for an unexpected reason)"; fail=1; }
fi
grep -q 'frozen spec SHA-256 differs from current bytes' "$LAB/attack.out" \
  && say "   divergence named explicitly: frozen spec SHA-256 differs from current bytes"

say ""
say "== Tooth 2: a claim that an absent capability already exists must be contradicted =="
claim_fail=0
check_present() { # <label> <shell test> — a corrected-to-PRESENT claim must hold
  local label="$1"; shift
  if eval "$@"; then
    say "   INVENTORY CONFIRMS (corrected) PRESENT: $label"
  else
    say "   recorded PRESENT but no anchor found (TOOTH FAILED): $label"; claim_fail=1
  fi
}

check_absent() { # <label> <shell test>
  local label="$1"; shift
  if eval "$@"; then
    say "   INVENTORY CONTRADICTS CLAIM: $label"
  else
    say "   claim survived inventory (TOOTH FAILED): $label"; claim_fail=1
  fi
}

check_absent "Go toolchain (claim: available)"   "! command -v go >/dev/null 2>&1"
check_absent "Go module apps/godspeed-casework-go (claim: exists)" "! test -e '$WT/apps/godspeed-casework-go'"
check_absent "React app apps/godspeed-cognitive-ui (claim: exists)" "! test -e '$WT/apps/godspeed-cognitive-ui'"
check_absent "Casework Go/UI just recipes (claim: exist)" "! grep -qE '^casework-(go-check|ui-check|integrated|removal-check):' '$WT/justfile'"
check_absent "SFWP repository-fact method (claim: exists)" \
  "! grep -qE 'method: \"(repo|github|pr)\\.[a-z_]+\"' '$WT/crates/sea-forge-server/src/sfwp/mod.rs'"
check_absent "GitHub REST/webhook client inside the server crate (claim: exists)" \
  "! grep -rqiE 'api\\.github\\.com|hooks\\.github' '$WT/crates/sea-forge-server/src/'"
check_absent "Open MCT / OpenMontage donor checkout (claim: present)" "! test -e /home/sprime01/projects/OpenMCT && ! test -e /home/sprime01/projects/OpenMontage"
check_absent "SFWP lease/claim/reserve method (claim: exists)" \
  "! grep -qE 'method: \"(lease|work)\\.[a-z_]+\"' '$WT/crates/sea-forge-server/src/sfwp/mod.rs'"
check_absent "SFWP artifact-persistence method (claim: exists)" \
  "! grep -qE 'method: \"artifact\\.[a-z_]+\"' '$WT/crates/sea-forge-server/src/sfwp/mod.rs'"

# Inventory correction found by this tooth run: a "GitHub adapter" claim is HALF true.
# The authority/approval surface for a GitHub PR does exist; only repository-fact
# execution and webhook ingestion are absent. The tooth must record both halves.
check_present "GitHub PR authority action (AuthorityAction::GithubPr)" \
  "grep -q 'GithubPr' '$WT/crates/sea-forge-core/src/types.rs'"
check_present "GitHub PR policy surface (github_pr, default escalate)" \
  "grep -q 'github_pr' '$WT/crates/sea-forge-authority/src/lib.rs' && grep -q 'github_pr' '$WT/crates/sea-forge-server/src/sfwp/approvals.rs'"

say "-- dependent tasks stay blocked (plan DAG at initial_state) --"
python3 - "$WT/.agents/plans/godspeed-casework-cognitive-environment.plan.yaml" <<'PY'
import sys, yaml
plan = yaml.safe_load(open(sys.argv[1]))
blocked = plan['initial_state']['blocked']
ready = plan['initial_state']['ready']
print(f"   ready at plan start: {ready}")
for t in ('T03', 'T04'):
    print(f"   {t} blocked_on={blocked[t]} (Go-gated; T01 owns GATE_GO)")
ok = ready == ['T00'] and blocked['T03'] == ['T01'] and blocked['T04'] == ['T01']
sys.exit(0 if ok else 1)
PY
[ $? -eq 0 ] || { say "   DAG expectation changed (TOOTH FAILED)"; claim_fail=1; }

if [ $claim_fail -eq 0 ]; then say "   all absent seams remain recorded as absent"; else fail=1; fi

say ""
if [ $fail -eq 0 ]; then say "TEETH RESULT: both teeth behaved as specified"; else say "TEETH RESULT: FAILED"; fi
exit $fail