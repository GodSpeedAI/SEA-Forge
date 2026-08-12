#!/usr/bin/env bash
# Semantic-teeth harness for the SEA Forge canonical interaction model.
#
# Every case below mutates a copy of the real canonical model and asserts that
# DomainForge rejects the mutation. A case that passes proves the corresponding
# invariant is enforced by the toolchain rather than by author discipline.
#
# Usage:
#   DOMAINFORGE=/path/to/domainforge .sea/interaction/validation/semantic-teeth.sh
#
# Default DOMAINFORGE is the local DomainForge repository release build, which
# is the authority for this evidence (see validation/domainforge-change-review.md).

set -uo pipefail

DF="${DOMAINFORGE:-/home/sprime01/projects/domainforge/target/release/domainforge}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
MODEL="$ROOT/.sea/interaction/interaction-model.sea"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

PASS=0
FAIL=0

# expect_reject <id> <expected-substring> <sed-program...>
# Applies the sed program to the canonical model and requires DomainForge to
# fail with a message containing the expected substring.
expect_reject() {
  local id="$1" expect="$2"; shift 2
  local file="$WORK/$id.sea"
  cp "$MODEL" "$file"
  local prog
  for prog in "$@"; do sed -i "$prog" "$file"; done
  local out
  out="$("$DF" validate --format human --no-color "$file" 2>&1)"
  if [ $? -eq 0 ]; then
    printf 'FAIL %-8s no rejection: mutation validated clean\n' "$id"; FAIL=$((FAIL+1)); return
  fi
  if printf '%s' "$out" | grep -qF "$expect"; then
    printf 'PASS %-8s %s\n' "$id" "$expect"; PASS=$((PASS+1))
  else
    printf 'FAIL %-8s expected %q, got: %s\n' "$id" "$expect" "$(printf '%s' "$out" | head -c 300)"
    FAIL=$((FAIL+1))
  fi
}

expect_accept() {
  local id="$1" file="$2"
  if "$DF" validate --format human --no-color "$file" >/dev/null 2>&1; then
    printf 'PASS %-8s accepted as expected\n' "$id"; PASS=$((PASS+1))
  else
    printf 'FAIL %-8s expected acceptance: %s\n' "$id" "$("$DF" validate --format human --no-color "$file" 2>&1 | head -c 300)"
    FAIL=$((FAIL+1))
  fi
}

echo "domainforge: $("$DF" --version)"
echo "model:       $MODEL"
echo

echo "-- baseline"
expect_accept T00 "$MODEL"

echo
echo "-- typed instance validation"

expect_reject T01 "is not a member of enum 'ImplementationMaturity'" \
  '0,/implementation_maturity: "exercised"/s//implementation_maturity: "shipped"/'

expect_reject T02 "is not a member of enum 'InterfaceKind'" \
  '0,/interface_kind: "web_ui"/s//interface_kind: "tui"/'

expect_reject T03 "is not a member of enum 'CanonicalizationClass'" \
  '0,/class_value: "canonical"/s//class_value: "primary"/'

expect_reject T04 "contains dangling typed reference" \
  '0,/    journey: "CJ01"/s//    journey: "CJ13"/'

expect_reject T05 "contains dangling typed reference" \
  '0,/    surface: "workbench_readiness_routes"/s//    surface: "workbench_missing_route"/'

expect_reject T06 "must be unique for entity 'CanonicalJourney'" \
  's/    journey_id: "CJ02",/    journey_id: "CJ01",/'

expect_reject T07 "has unknown field" \
  '0,/    slug: "establish_trusted_cell_context",/s//    slug: "establish_trusted_cell_context",\n    owner_team: "platform",/'

expect_reject T08 "is missing required field 'slug'" \
  '/^    slug: "establish_trusted_cell_context",$/d'

expect_reject T09 "violates its pattern constraint" \
  's/    journey_id: "CJ12",/    journey_id: "CJ99",/'

expect_reject T10 "violates its pattern constraint" \
  's/    binding_id: "IB090",/    binding_id: "B90",/'

expect_reject T11 "does not satisfy its int type" \
  's/    observed_story_count: 15,/    observed_story_count: "fifteen",/'

expect_reject T12 "violates its max constraint" \
  's/    ordinal: 8,/    ordinal: 9,/'

expect_reject T13 "violates its min_length constraint" \
  's/    slug: "execute_governed_work",/    slug: "exec",/'

echo
echo "-- graph invariants (instance-aware policies)"

# Deleting a journey trips referential integrity before the count policy: every
# canonical journey is referenced by at least one interface binding, so removal
# is caught as a dangling reference. That is the stronger of the two guards.
expect_reject T14 "contains dangling typed reference" \
  '/^instance transfer_and_adopt_governed_assets of "CanonicalJourney" {$/,/^}$/d'

# A thirteenth canonical identity is structurally impossible: the JourneyId
# pattern admits CJ01-CJ12 only, so the identifier is rejected before any policy
# runs. Journey count is therefore guarded twice over.
expect_reject T14b "value 'CJ13' does not match pattern 'JourneyId'" \
  's/    journey_id: "CJ12",/    journey_id: "CJ13",/'

# Removing a journey after re-pointing its bindings leaves no dangling reference,
# so the count policy itself is what rejects the eleven-journey catalog.
expect_reject T14c "'twelve_canonical_journeys' was violated" \
  's/    journey: "CJ12"$/    journey: "CJ11"/' \
  '/^instance transfer_and_adopt_governed_assets of "CanonicalJourney" {$/,/^}$/d'

expect_reject T15 "'all_observed_stories_accounted' was violated" \
  's/    observed_story_count: 15,/    observed_story_count: 14,/'

expect_reject T16 "'exercised_stories_reconcile' was violated" \
  's/    exercised_story_count: 5,/    exercised_story_count: 4,/'

expect_reject T17 "'interface_bindings_reconcile' was violated" \
  's/    interface_binding_count: 5$/    interface_binding_count: 6/'

expect_reject T18 "'eight_interaction_steps' was violated" \
  '/^instance step_preflight of "JourneyStep" {$/,/^}$/d'

expect_reject T19 "'interaction_step_ordinals_complete' was violated" \
  's/^    ordinal: 5,$/    ordinal: 4,/'

# As with T14, deleting a bound surface trips referential integrity first.
expect_reject T20 "contains dangling typed reference" \
  '/^instance cli_ledger_family of "InterfaceSurface" {$/,/^}$/d'

expect_reject T20b "'thirty_eight_interface_surfaces' was violated" \
  '$a\
instance undeclared_surface of "InterfaceSurface" {\
    surface_id: "undeclared_surface",\
    interface_kind: "web_ui",\
    entry_or_method: "/undeclared",\
    binding_role: "A surface added without a reviewed binding, present only so the harness can prove the surface-count invariant has teeth.",\
    implementation_maturity: "declared",\
    source_ref: "validation/semantic-teeth.md case T20b; no repository source backs this surface.",\
    limitations: "This surface does not exist in the SEA Forge repository. It appears only inside a temporary mutated copy of the canonical model."\
}'

expect_reject T21 "'eleven_variation_dimensions' was violated" \
  '/^instance dimension_boundary_and_scope of "VariationDimension" {$/,/^}$/d'

expect_reject T22 "'classification_rows_account_for_all_stories' was violated" \
  's/    observed_rows: 69,/    observed_rows: 68,/'

expect_reject T23 "'eight_canonicalization_classifications' was violated" \
  '/^instance classification_unsupported of "CanonicalizationClassification" {$/,/^}$/d'

expect_reject T23b "'exactly_one_product_purpose' was violated" \
  '$a\
instance competing_purpose of "ProductPurpose" {\
    purpose_id: "competing",\
    statement: "A second statement of what SEA Forge is for, added by the semantic-teeth harness so that the single-purpose invariant can be shown to reject a competing definition of the product.",\
    governing_constraint: "None. This instance exists only inside a temporary mutated copy of the canonical model and states no governing constraint that any journey, surface, or binding may rely upon.",\
    out_of_scope: "Everything. This instance is a negative fixture for validation/semantic-teeth.md case T23b and never appears in the canonical model under .sea/interaction."\
}'

echo
echo "-- module closure (DomainForge 0.16.0 transitive filesystem closure)"

# Positive: an imported entity type backs a same-namespace instance in another
# module. This is the capability DomainForge 0.15.0 lacked.
mkdir -p "$WORK/closure"
cat > "$WORK/closure/types.sea" <<'SEA'
@namespace "sea_forge.interaction.probe"
export enum Maturity { specified = "specified", exercised = "exercised" }
export entity "ProbeJourney" {
    key journey_id: string (min_length 4)
    maturity: Maturity
}
SEA
cat > "$WORK/closure/instances.sea" <<'SEA'
@namespace "sea_forge.interaction.probe"
import { ProbeJourney, Maturity } from "./types.sea"
instance probe01 of "ProbeJourney" { journey_id: "CJ01", maturity: "exercised" }
SEA
expect_accept T24 "$WORK/closure/instances.sea"

# Negative: the closure still rejects an invalid instance across the import.
cat > "$WORK/closure/bad.sea" <<'SEA'
@namespace "sea_forge.interaction.probe"
import { ProbeJourney, Maturity } from "./types.sea"
instance probe02 of "ProbeJourney" { journey_id: "CJ02", maturity: "shipped" }
SEA
closure_out="$("$DF" validate --format human --no-color "$WORK/closure/bad.sea" 2>&1)"
if printf '%s' "$closure_out" | grep -qF "is not a member of enum 'Maturity'"; then
  printf 'PASS %-8s imported enum still enforced across the closure\n' T25; PASS=$((PASS+1))
else
  printf 'FAIL %-8s imported enum not enforced across the closure: %s\n' T25 "$closure_out"; FAIL=$((FAIL+1))
fi

# Negative: an unresolvable import specifier fails closure resolution.
cat > "$WORK/closure/missing.sea" <<'SEA'
@namespace "sea_forge.interaction.probe"
import { ProbeJourney } from "./absent.sea"
instance probe03 of "ProbeJourney" { journey_id: "CJ03" }
SEA
if ! "$DF" validate --format human --no-color "$WORK/closure/missing.sea" >/dev/null 2>&1; then
  printf 'PASS %-8s unresolved import specifier rejected\n' T26; PASS=$((PASS+1))
else
  printf 'FAIL %-8s unresolved import specifier accepted\n' T26; FAIL=$((FAIL+1))
fi

echo
echo "==== $PASS passed, $FAIL failed ===="
[ "$FAIL" -eq 0 ]
