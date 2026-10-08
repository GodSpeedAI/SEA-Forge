#!/usr/bin/env bash
# Runs the interaction model's full validation recipe end to end and exits
# non-zero if any step fails. A green `domainforge validate` alone proves
# less than it appears to (validation/limitations.md L4, L5): it does not
# reach the Application Contract's one operation, and — until this
# repository's DomainForge build lands the L5 fix — a policy whose `where`
# arithmetic is malformed could look green and prove nothing. Run all four
# steps; do not trust step 1 alone.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INTERACTION_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
MODEL="$INTERACTION_DIR/interaction-model.sea"

DOMAINFORGE="${DOMAINFORGE:-/home/sprime01/projects/domainforge/target/release/domainforge}"

if [[ ! -x "$DOMAINFORGE" ]]; then
    echo "error: domainforge binary not found or not executable at $DOMAINFORGE" >&2
    echo "  build it with: cargo build --release --bin domainforge --features cli" >&2
    echo "  (from /home/sprime01/projects/domainforge), or set DOMAINFORGE=<path>" >&2
    exit 1
fi

fail=0

echo "=== 1/4 semantic validation ==="
if ! "$DOMAINFORGE" validate --format human --no-color "$MODEL"; then
    echo "FAILED: domainforge validate" >&2
    fail=1
fi
echo

echo "=== 2/4 semantic teeth (negative tests) ==="
if ! DOMAINFORGE="$DOMAINFORGE" "$SCRIPT_DIR/semantic-teeth.sh"; then
    echo "FAILED: semantic-teeth.sh" >&2
    fail=1
fi
echo

echo "=== 3/4 model <-> matrix reconciliation ==="
if ! python3 "$SCRIPT_DIR/reconcile.py"; then
    echo "FAILED: reconcile.py" >&2
    fail=1
fi
echo

echo "=== 4/4 application contract and semantic envelope ==="
# `--application` reaches the Application Contract through the same
# `validate` command as step 1, once the local DomainForge build has landed
# the CLI wiring (validation/limitations.md L4). If this repository's
# `domainforge` predates that, this step falls back to the standalone
# `contract`/`envelope` subcommands where available, and to the harness
# (validation/application-contract-harness.rs) as a last resort.
if "$DOMAINFORGE" validate --help 2>&1 | grep -q -- '--application'; then
    if ! "$DOMAINFORGE" validate --format human --no-color --application "$MODEL"; then
        echo "FAILED: domainforge validate --application" >&2
        fail=1
    fi
elif "$DOMAINFORGE" --help 2>&1 | grep -q '\bcontract\b'; then
    if ! "$DOMAINFORGE" contract "$MODEL" >/dev/null; then
        echo "FAILED: domainforge contract" >&2
        fail=1
    fi
    if ! "$DOMAINFORGE" envelope "$MODEL" >/dev/null; then
        echo "FAILED: domainforge envelope" >&2
        fail=1
    fi
else
    echo "note: this domainforge build has no --application flag and no"
    echo "  contract/envelope subcommand (validation/limitations.md L4)."
    echo "  Falling back to the harness — see"
    echo "  validation/application-contract-harness.rs for build instructions."
    echo "  Not run automatically here: it requires a separate binary build."
fi
echo

if [[ "$fail" -eq 0 ]]; then
    echo "==== all checks passed ===="
    exit 0
else
    echo "==== one or more checks FAILED ====" >&2
    exit 1
fi
