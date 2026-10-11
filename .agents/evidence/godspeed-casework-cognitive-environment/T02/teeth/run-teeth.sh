#!/usr/bin/env bash
# T02 teeth — donor mechanics and dependency/license boundaries.
#
#   Tooth 1 (application-contract independence): search this plan's application-contract surfaces
#     for donor product names, namespaces or types. Expected: none. If those surfaces do not exist
#     yet (T01/T04 not implemented), say so explicitly and record the tooth as re-runnable rather
#     than silently passing.
#   Tooth 2 (no donor available / no copied code): confirm neither donor checkout is reachable and
#     that no donor-derived file exists in the repository. Expected: T02 still settles with a native
#     implementation decision and no copied code, and the license/provenance obligation is vacuous.
#
# Usage: run-teeth.sh   (exit 0 = both teeth behaved as specified)
set -u
WT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
test -f "$WT/.agents/plans/godspeed-casework-cognitive-environment.plan.yaml" || {
  echo "run-teeth: cannot locate the plan from $WT - refusing to report vacuous teeth" >&2
  exit 2
}
cd "$WT" || exit 99
fails=0
say() { printf '%s\n' "$*"; }

say "== Tooth 1: no donor-product ontology in application-contract surfaces =="
# The tooth must distinguish a decision that NAMES a donor from a contract that DEPENDS on one.
# A mention in decision prose is expected and fine; what must not appear is a donor marker inside a
# contract definition - a code declaration line, an import path, or a namespaced type reference.
MARKERS='open[-_ ]?mct|openmct|open[-_ ]?montage|openmontage|\bmct\b|montage'
DEFN_RE='(interface|type|struct|func|package|import|namespace|module|from[[:space:]]+["'"'"'])'
CONTRACT_FILES=""
for cand in \
  .agents/plans/godspeed-casework-cognitive-environment.implementation.md \
  apps/godspeed-casework-go/go.mod \
  apps/godspeed-cognitive-ui/package.json ; do
  [ -e "$cand" ] && CONTRACT_FILES="$CONTRACT_FILES $cand"
done
# Also inspect any Go/TS contract source once it exists.
SRC=$(find apps -type f \( -name '*.go' -o -name '*.ts' -o -name '*.tsx' \) 2>/dev/null | head -200)
if [ -z "$CONTRACT_FILES" ] && [ -z "$SRC" ]; then
  say "   NOT-APPLICABLE-YET: no application-contract surface exists to inspect (T01/T04 not built)."
  say "   This tooth MUST be re-run by T04 and T07 once the Go ports and the React contract package exist."
else
  say "   inspecting definitions in:${CONTRACT_FILES}${SRC:+ <apps sources>}"
  bad=0
  for f in $CONTRACT_FILES $SRC; do
    [ -f "$f" ] || continue
    # (a) declaration/import lines that carry a donor marker
    if grep -niE "$MARKERS" "$f" 2>/dev/null | grep -niE "$DEFN_RE" > /tmp/t02-tooth1a.out 2>&1; then
      say "   donor marker on a declaration/import line in $f (TOOTH FAILED):"
      sed 's/^/     /' /tmp/t02-tooth1a.out | head -5
      bad=1
    fi
    # (b) backticked namespaced references such as `mct.core.Foo` or `@openmct/bar`
    if grep -oE '`[^`]*\.(mct|montage)[^`]*`|`@openmct[^`]*`|`[^`]*Montage[A-Z][^`]*`' "$f" > /tmp/t02-tooth1b.out 2>&1; then
      say "   donor-namespaced reference in $f (TOOTH FAILED):"
      sed 's/^/     /' /tmp/t02-tooth1b.out | head -5
      bad=1
    fi
  done
  if [ "$bad" -eq 0 ]; then
    say "   ok: no donor marker in any contract definition, import path, or namespaced reference"
    say "   (mentions in decision prose are expected and are not counted - the tooth separates"
    say "    'the decision names the donor' from 'the contract depends on the donor')"
  else
    fails=1
  fi
fi
# Independently: donor markers anywhere in this plan's implementation paths (code that does not
# exist yet is reported, never assumed clean).
CODE_HITS=$(grep -rniE "$MARKERS" apps/ crates/ workbench/apps 2>/dev/null | grep -viE 'montage|mct' | head -5 || true)
if [ -n "$CODE_HITS" ]; then
  say "   note: donor-like markers elsewhere in the tree (review, not automatically a failure):"
  printf '%s\n' "$CODE_HITS" | sed 's/^/     /'
fi

say ""
say "== Tooth 2: no donor checkout, no copied donor code =="
say "   searched roots: \$HOME/projects, \$HOME, /opt (depth-limited); donor roots below"
found=0
for root in "$HOME/projects" "$HOME" /opt; do
  [ -d "$root" ] || continue
  hits=$(find "$root" -maxdepth 3 -iname '*open*mct*' -o -maxdepth 3 -iname '*openmontage*' 2>/dev/null | head -5)
  if [ -n "$hits" ]; then
    say "   donor-like path under $root:"; printf '%s\n' "$hits" | sed 's/^/     /'
    found=1
  fi
done
if [ "$found" -eq 0 ]; then
  say "   ok: neither Open MCT nor OpenMontage checkout is reachable from the searched roots"
else
  say "   NOTE: a donor-like path exists; T02's omit decision would then rest on licensing/review, not absence"
fi

say "   -- copied-code / provenance check over this plan's implementation paths --"
# No donor file should have been copied in. Nothing under apps/ exists yet; assert that explicitly.
if [ -d apps ]; then
  if grep -rniE 'Copyright.*(NASA|Open MCT|OpenMontage)|SPDX-License-Identifier' apps/ > /tmp/t02-tooth2.out 2>&1; then
    say "   license/notice headers found under apps/ (review required):"
    sed 's/^/     /' /tmp/t02-tooth2.out | head -10
    fails=1
  else
    say "   ok: no third-party licence or notice header under apps/ - no copied file to attribute"
  fi
else
  say "   ok: apps/ does not exist, so no donor file can have been copied into an application"
fi

say "   -- dependency boundary (T02 admits no dependency) --"
if git -C "$WT" diff --stat HEAD -- . | grep -qE 'go\.mod|go\.sum|package\.json|Cargo\.toml'; then
  say "   TOOTH FAILED: this task changed a dependency manifest"
  fails=1
else
  say "   ok: no dependency manifest changed by this task's uncommitted work"
fi

say ""
if [ "$fails" -eq 0 ]; then
  say "TEETH RESULT: both teeth behaved as specified"
else
  say "TEETH RESULT: FAILED"
fi
exit $fails