#!/usr/bin/env sh
# AT-01: the GS-FDE-SOP-001 declared model validates, its envelope is
# reproducible, and the two settlement concepts are distinct.
#
#   1. domainforge validate --application exits 0 (graph and Application Contract)
#   2. domainforge envelope --emit cep exits 0 repeatedly and yields the same
#      world_ref both times
#   3. the evidence and outcome "Settlement" concepts have different ConceptIds
#      and namespaces, and merging them into one namespace is refused
#   4. negative controls: a broken model is rejected for the reason claimed
#
# Does not use `domainforge test`. Capture the evidence with:
#   scripts/check-fde-sop-model.sh | tee .sea/interaction/fde-sop/validation/domainforge-output.txt
#
# Environment: DOMAINFORGE overrides the binary (default: the mise shim).
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
model_dir="$root/.sea/interaction/fde-sop"
model="$model_dir/fde-sop.sea"
df=${DOMAINFORGE:-$HOME/.local/share/mise/shims/domainforge}
world_name=godspeed-fde-sop

[ -x "$df" ] || { echo "fail: domainforge not executable at $df" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo 'fail: python3 is required' >&2; exit 2; }

work=$(mktemp -d "${TMPDIR:-/tmp}/fde-sop-model.XXXXXX")
trap 'rm -rf "$work"' EXIT INT TERM

failures=0
fail() { printf 'FAIL: %s\n' "$1"; failures=$((failures + 1)); }
pass() { printf 'ok:   %s\n' "$1"; }

echo "FDE SOP declared model: DomainForge validation evidence (AT-01)"
echo "Captured: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo
echo "== toolchain =="
echo "\$ domainforge --version"
"$df" --version
sop_sha=$(sha256sum "$root/docs/sop/GS-FDE-SOP-001-v0.4.1.md" | cut -d' ' -f1)
echo "source (docs/sop/GS-FDE-SOP-001-v0.4.1.md) sha256: $sop_sha"
for f in fde-sop.sea evidence.sea outcome.sea; do
  echo "model $f sha256: $(sha256sum "$model_dir/$f" | cut -d' ' -f1)"
done
echo

echo "== 1. validate --application =="
echo "\$ domainforge validate --application .sea/interaction/fde-sop/fde-sop.sea"
if out=$(cd "$model_dir" && "$df" validate --application fde-sop.sea 2>&1); then
  printf '%s\n' "$out"
  pass "validate --application exits 0"
else
  printf '%s\n' "$out"
  fail "validate --application exits non-zero"
fi
echo

echo "== 2. envelope --emit cep, repeated =="
echo "\$ domainforge envelope --emit cep --world-name $world_name .sea/interaction/fde-sop/fde-sop.sea"
for run in 1 2; do
  if (cd "$model_dir" && "$df" envelope --emit cep --world-name "$world_name" fde-sop.sea >"$work/env$run.json" 2>"$work/env$run.err"); then
    pass "envelope run $run exits 0"
  else
    cat "$work/env$run.err"
    fail "envelope run $run exits non-zero"
  fi
done
field() { python3 - "$1" "$2" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
def find(o, key):
    if isinstance(o, dict):
        if key in o and isinstance(o[key], str):
            return o[key]
        for v in o.values():
            r = find(v, key)
            if r: return r
    elif isinstance(o, list):
        for v in o:
            r = find(v, key)
            if r: return r
    return ""
print(find(d, sys.argv[2]))
PY
}
w1=$(field "$work/env1.json" world_ref)
w2=$(field "$work/env2.json" world_ref)
# Third run with the representation inlined, so section 3 can read the closure's
# declarations (the default threshold stores a model this size by reference).
(cd "$model_dir" && "$df" envelope --emit cep --world-name "$world_name" --inline-threshold-bytes 100000000 fde-sop.sea >"$work/env3.json") || fail "inlined envelope run exits non-zero"
w3=$(field "$work/env3.json" world_ref)
echo "run 1 world_ref: $w1"
echo "run 2 world_ref: $w2"
echo "run 3 world_ref (representation inlined): $w3"
echo "source_set_hash: $(field "$work/env1.json" source_set_hash)"
echo "semantic_closure_hash: $(field "$work/env1.json" semantic_closure_hash)"
echo "created_by: $(field "$work/env1.json" created_by)"
echo "envelope_id differs between runs (expected, not part of identity): $(field "$work/env1.json" envelope_id) / $(field "$work/env2.json" envelope_id)"
case "$w1" in
  world:$world_name@sha256:????????????????????????????????????????????????????????????????) pass "world_ref has the form world:$world_name@sha256:<64 hex>" ;;
  *) fail "world_ref is malformed: $w1" ;;
esac
[ -n "$w1" ] && [ "$w1" = "$w2" ] && [ "$w1" = "$w3" ] && pass "world_ref identical across three runs" || fail "world_ref differs across runs"
echo

echo "== 3. the two settlement concepts are distinct =="
echo "\$ domainforge parse --format json fde-sop.sea  (entities named Settlement)"
(cd "$model_dir" && "$df" parse --format json fde-sop.sea) >"$work/graph.json"
if python3 - "$work/graph.json" "$work/env3.json" <<'PY'
import json, sys
g = json.load(open(sys.argv[1]))
env = json.load(open(sys.argv[2]))
s = [(e["namespace"], e["id"]) for e in g["entities"].values() if e["name"] == "Settlement"]
for ns, i in sorted(s):
    print(f"  Settlement  namespace={ns}  ConceptId=concept:{i}")
ok = True
if len(s) != 2:
    print(f"  expected exactly 2 Settlement entities, found {len(s)}"); ok = False
else:
    if s[0][1] == s[1][1]: print("  ConceptIds are equal"); ok = False
    if s[0][0] == s[1][0]: print("  namespaces are equal"); ok = False
    want = {"godspeed-fde-sop.evidence", "godspeed-fde-sop.outcome"}
    if {x[0] for x in s} != want: print(f"  namespaces are not {sorted(want)}"); ok = False
# the same two ids must appear as distinct declarations in the envelope closure
inline = None
def walk(o):
    global inline
    if isinstance(o, dict):
        if isinstance(o.get("inline_content"), str) and "semantic_declarations" in o["inline_content"]:
            inline = o["inline_content"]
        for v in o.values(): walk(v)
    elif isinstance(o, list):
        for v in o: walk(v)
walk(env)
if inline is None:
    print("  envelope has no inline semantic_declarations"); ok = False
else:
    decls = json.loads(inline)["envelope"]["semantic_declarations"]
    ids = [d["id"]["data"]["id"] for d in decls if d["declaration"]["kind"] == "entity" and d["declaration"]["data"]["name"] == "Settlement"]
    print(f"  envelope declarations named Settlement: {len(ids)}, distinct ids: {len(set(ids))}")
    if len(ids) != 2 or len(set(ids)) != 2: ok = False
sys.exit(0 if ok else 1)
PY
then pass "two Settlement concepts, different namespaces, different ConceptIds"; else fail "settlement concepts are not distinct"; fi

# Negative control: put both in one namespace; DomainForge must refuse to unify them.
neg="$work/merge"; mkdir -p "$neg"; cp "$model_dir"/*.sea "$neg/"
sed -i 's/^@namespace "godspeed-fde-sop.outcome"/@namespace "godspeed-fde-sop.evidence"/' "$neg/outcome.sea"
echo "\$ domainforge validate (outcome.sea moved into the evidence namespace)"
if out=$(cd "$neg" && "$df" validate --application fde-sop.sea 2>&1); then
  printf '%s\n' "$out"; fail "merging the settlement namespaces was accepted"
else
  printf '%s\n' "$out"
  case "$out" in *"distinct declarations share qualified ID"*) pass "T4 merging the two namespaces is refused (symbol collision)" ;;
    *) fail "T4 merge refused for an unexpected reason" ;; esac
fi
echo

echo "== 4. negative controls and capability probes =="
reset() { rm -rf "$neg"; mkdir -p "$neg"; cp "$model_dir"/*.sea "$neg/"; }
expect_reject() { # name, expected-substring, command...
  name=$1; want=$2; shift 2
  if out=$("$@" 2>&1); then printf '%s\n' "$out"; fail "$name: accepted a broken model"
  else
    case "$out" in *"$want"*) pass "$name: rejected ($want)" ;;
      *) printf '%s\n' "$out"; fail "$name: rejected for a different reason" ;; esac
  fi
}
reset
sed -i 's/as: settler_ref != executor_ref/as: settler_ref != executer_ref/' "$neg/fde-sop.sea"
echo "\$ T1 bound policy referencing a field that is not on the input record"
expect_reject "T1" "does not resolve to a declared input field" sh -c "cd '$neg' && '$df' validate --application fde-sop.sea"
reset
sed -i 's/at precondition fails with nav_claim_policy_denied/at postcondition fails with nav_claim_policy_denied/' "$neg/fde-sop.sea"
echo "\$ T2 policy bound at postcondition"
expect_reject "T2" "is reserved; only precondition is executable" sh -c "cd '$neg' && '$df' validate --application fde-sop.sea"
reset
sed -i 's/^policy nine_evidence_gates as: \(.*\)) = 9$/policy nine_evidence_gates as: \1) = 8/' "$neg/fde-sop.sea"
echo "\$ T3 graph policy count broken (nine gates declared as eight)"
expect_reject "T3" "nine_evidence_gates" sh -c "cd '$neg' && '$df' validate fde-sop.sea"
reset
sed -i 's/^role_binding "IndependentVerifier" for "Episode"/role_binding "IndependentVerifier" for "EvidenceSettlement"/' "$neg/fde-sop.sea"
echo "\$ T5 role_binding to an entity imported from another namespace"
expect_reject "T5" "role_binding references undefined entity" sh -c "cd '$neg' && '$df' validate fde-sop.sea"
echo "\$ T6 role_binding on the real model populates entity_roles"
if python3 - "$work/graph.json" <<'PY'
import json, sys
g = json.load(open(sys.argv[1]))
n = sum(len(v) for v in g["entity_roles"].values())
print(f"  entity_roles: {len(g['entity_roles'])} entities, {n} role bindings")
sys.exit(0 if n >= 7 else 1)
PY
then pass "T6 role bindings are authored and recorded (7 declared)"; else fail "T6 entity_roles is empty or short"; fi
echo

if [ "$failures" -eq 0 ]; then
  echo "VERDICT: PASS"
else
  echo "VERDICT: FAIL ($failures failing check(s))"
  exit 1
fi
