#!/usr/bin/env bash
# End-to-end world loop across the GodSpeed stack, in ONE pinned semantic world.
#
#   GodSpeed-Agent E1 -> Context Kernel E3 -> SWE_SEED E4 -> SEA-Forge E5/E6
#     -> SWE_SEED E7 -> RealityTrace E8 -> GodSpeed-Agent
#
# Every hop is a real production surface reading the PREVIOUS hop's real output
# from one exchange directory. The world is whatever the DomainForge CLI computes
# for the demo source; SEA-Forge must independently recompute the same one.
# Then the failure paths: another world, an unregistered world, a denied request
# and evidence from a different world must each be refused.
#
# Execution is simulated at E5B (see crates/sea-forge-server/tests/e2e_world_loop.rs);
# real execution under a CEP decision is Cognate's `just sea-forge-live`.
#
# Repos (override with the *_ROOT variables): sea-rs (this repo), godspeed_agent,
# Context_Kernel, SWE_SEED, sxr. Needs: domainforge, uv, cargo. Optional: CEP_REPO
# to validate the Context Kernel's context_bundle against the cep reference validator.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
projects="${PROJECTS:-$HOME/projects}"
SEA_RS="${SEA_RS_ROOT:-$here}"
GSA="${GODSPEED_AGENT_ROOT:-$projects/godspeed_agent}"
CK="${CONTEXT_KERNEL_ROOT:-$projects/Context_Kernel}"
SWE="${SWE_SEED_ROOT:-$projects/SWE_SEED}"
SXR="${SXR_ROOT:-$projects/sxr}"

fail() { echo "e2e-world-loop: $1" >&2; exit 1; }
for tool in domainforge uv cargo python3; do command -v "$tool" >/dev/null || fail "$tool is required"; done
for d in "$GSA" "$CK" "$SWE" "$SXR" "$SEA_RS"; do [ -d "$d" ] || fail "repo not found: $d"; done

step() { printf '\n== %s\n' "$1"; }
quiet_cargo() { cargo test --quiet "$@" 2>&1 | grep -E "^test result|^error|panicked|FAILED" || true; return "${PIPESTATUS[0]}"; }

dir="$(mktemp -d)"
trap 'rm -rf "$dir" "$dir"-*' EXIT

cat > "$dir/demo.sea" <<'SEA'
@namespace "e2e"
entity "Service" { key id: uuid }
SEA
printf '{"model":"godspeed-e2e","version":"1"}' > "$dir/canonical_model.sea.json"

export E2E_DIR="$dir"
export E2E_WORK_REQUEST_ID="wr-e2e-001"
E2E_DOMAIN_HASH="$(sha256sum "$dir/canonical_model.sea.json" | cut -d' ' -f1)"
E2E_WORLD_REF="$(cd "$dir" && domainforge envelope --emit cep --world-name demo demo.sea \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["scope"]["world_ref"])')"
export E2E_DOMAIN_HASH E2E_WORLD_REF
echo "world  $E2E_WORLD_REF"
echo "model  $E2E_DOMAIN_HASH"

step "hop 1  GodSpeed-Agent emits E1 WorkRequested"
(cd "$GSA" && uv run --quiet pytest -q tests/test_e2e_world_loop.py -k hop_1)

step "hop 0  Context Kernel retrieves and emits E3 (+ CEP context_bundle)"
(cd "$CK" && quiet_cargo -p ck-mcp --test e2e_world_loop)

step "hop 2  SWE_SEED builds E4 GovernedWorkRequest"
(cd "$SWE" && quiet_cargo -p swe-seed-core --test e2e_world_loop hop_2)

step "hop 3  SEA-Forge: verified intake, decision, E5A/E5B, E6 settlement"
(cd "$SEA_RS" && quiet_cargo -p sea-forge-server --test e2e_world_loop)

step "hop 4  SWE_SEED adjudicates E6 and emits E7 ProofCompleted"
(cd "$SWE" && quiet_cargo -p swe-seed-core --test e2e_world_loop hop_4)

step "hop 5  RealityTrace ingests E7 and emits E8 EvidenceRecorded"
(cd "$SXR" && quiet_cargo -p sxr-core --test e2e_world_loop)

step "hop 6  GodSpeed-Agent ingests E8, bound to the same world"
(cd "$GSA" && uv run --quiet pytest -q tests/test_e2e_world_loop.py -k hop_6)

if [ -n "${CEP_REPO:-}" ]; then
  step "cep  the context bundle satisfies godspeed.context_bundle"
  (cd "$CEP_REPO" && uv run --quiet python -c '
import json, sys
from cep.godspeed import validate_profile, verify_semantics
env = json.load(open(sys.argv[1]))
errs = validate_profile(env, "context_bundle") + verify_semantics(env, "context_bundle")
print("valid" if not errs else errs); sys.exit(1 if errs else 0)' "$dir/e3_context_bundle.cep.json")
fi

# ---- failure paths: each must be refused, and the refusal is the pass condition ----
expect_refused() { # <label> <must-mention> <command...>
  local label="$1" must="$2"; shift 2
  local out
  if out="$("$@" 2>&1)"; then fail "NOT REFUSED: $label"; fi
  # The refusal must be the intended one, not an unrelated failure.
  echo "$out" | grep -qiF "$must" || { echo "$out" | tail -15 >&2; fail "refused for the wrong reason: $label (expected '$must')"; }
  echo "refused: $label  [$(echo "$out" | grep -iF "$must" | head -1 | sed 's/^ *//' | cut -c1-150)]"
}

step "refusals"
other="world:demo@sha256:$(printf 'd%.0s' $(seq 64))"

# a request in another world, with a context packet in the pinned one
cp -r "$dir" "$dir-wrongworld"
python3 - "$dir-wrongworld/e4_governed_work_request.json" "$E2E_WORLD_REF" "$other" <<'PY'
import sys
p, w, o = sys.argv[1:]
text = open(p).read()
open(p, "w").write(text.replace(w, o))
PY
expect_refused "E4 in another world than its context packet" "World { reason" \
  bash -c "cd '$SEA_RS' && E2E_DIR='$dir-wrongworld' cargo test --quiet -p sea-forge-server --test e2e_world_loop"

# a request and packet both in a world SEA-Forge never registered
cp -r "$dir" "$dir-unknown"
python3 - "$dir-unknown" "$E2E_WORLD_REF" "$other" <<'PY'
import sys
d, w, o = sys.argv[1:]
for n in ("e4_governed_work_request.json", "e3_context_packet.json"):
    p = f"{d}/{n}"
    text = open(p).read()
    open(p, "w").write(text.replace(w, o))
PY
expect_refused "a world SEA-Forge never registered" "World { reason" \
  bash -c "cd '$SEA_RS' && E2E_DIR='$dir-unknown' cargo test --quiet -p sea-forge-server --test e2e_world_loop"

# a denied request authorises nothing: the hop passes only by finding no invocation
rm -rf "$dir-deny" && cp -r "$dir" "$dir-deny" && rm -f "$dir-deny"/e5a_* "$dir-deny"/e5b_* "$dir-deny"/e6_*
(cd "$SEA_RS" && E2E_DIR="$dir-deny" E2E_POLICY_VERDICT=deny cargo test --quiet -p sea-forge-server --test e2e_world_loop >/dev/null 2>&1) \
  || fail "the denied path did not hold"
[ ! -e "$dir-deny/e6_operational_settlement.json" ] || fail "a denied request produced a settlement"
echo "refused: a denied request authorises no invocation and settles nothing"

echo
echo "e2e-world-loop: OK. One world ($E2E_WORLD_REF) from E1 to the evidence the agent records."
