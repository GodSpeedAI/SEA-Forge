# SEA Forge canonical command surface (Shell-SPEC §10.2).
# Run from the repository root. CI invokes `devbox run -- just <recipe>`.

set := "set -euo pipefail"
evidence_dir := "target/bootstrap-evidence"

# Default: print grouped recipes.
[group('meta')]
default:
    @just --list --list-heading "SEA Forge commands:" --justfile justfile

# --- setup --------------------------------------------------------------

# Converge on the pinned toolchain: Devbox packages + Rust components + deps.
[group('setup')]
setup:
    #!/usr/bin/env bash
    {{set}}
    echo "[setup] installing devbox packages"
    devbox install
    echo "[setup] ensuring rust components"
    rustup toolchain install "$(sed -n 's/^channel *= *"\(.*\)".*/\1/p' rust-toolchain.toml)" --profile minimal --component rustfmt,clippy --no-self-update
    echo "[setup] fetching workspace dependencies"
    cargo fetch --locked
    echo "[setup] done — run 'just doctor' to verify"

# --- quality ------------------------------------------------------------

# Machine-readable environment check (Shell-SPEC §7.1). Writes doctor.jsonl.
[group('quality')]
doctor:
    #!/usr/bin/env bash
    {{set}}
    bash scripts/doctor.sh

# Build all crates.
[group('quality')]
build:
    cargo build --workspace --all-targets --locked

# Apply rustfmt to every crate.
[group('quality')]
fmt:
    cargo fmt --all

# Verify rustfmt without modifying files.
[group('quality')]
fmt-check:
    cargo fmt --all -- --check

# Run clippy with `-D warnings` across all targets and features.
[group('quality')]
lint:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

# Type-check every crate (`cargo check --locked`).
[group('quality')]
typecheck:
    cargo check --workspace --all-targets --locked

# Supply-chain + secret scan: cargo-deny then gitleaks.
# `cargo deny check advisories` fetches the RustSec database; the other
# categories are offline. gitleaks scans staged + committed history.
[group('quality')]
security:
    #!/usr/bin/env bash
    {{set}}
    cargo deny check
    gitleaks detect --no-banner --redact

# Apply only safe automatic fixes (rustfmt). Clippy fixes are intentionally
# excluded: `-A clippy::all --fix` mutates behavior and should be reviewed.
[group('quality')]
fix:
    cargo fmt --all

# Quick high-signal local verification (context + fmt + typecheck).
# Target runtime well under 10s — this is what pre-commit runs.
[group('quality')]
check-fast:
    #!/usr/bin/env bash
    {{set}}
    scripts/check-agent-context.sh
    cargo fmt --all -- --check
    cargo check --workspace --all-targets --locked

# Developer quality sweep (Shell-SPEC §10.3): context + fmt + clippy + check
# + deny + gitleaks. Delegates to granular recipes so local and CI share one
# implementation per category.
[group('quality')]
check:
    #!/usr/bin/env bash
    {{set}}
    just context-check
    just fmt-check
    just lint
    just typecheck
    just security
    echo "[check] all gates green"

# Validate agent handoff structure and freshness without vendor-specific tooling.
[group('quality')]
context-check:
    scripts/check-agent-context.sh

# Run the test suite (Shell-SPEC §10.3).
[group('quality')]
test:
    cargo test --workspace --all-features --locked

# Canonical clean, deterministic, noninteractive CI verification.
# GitHub Actions invokes this (or its documented constituent recipes when
# parallelized). Local `just ci` is equivalent to the union of required jobs.
[group('quality')]
ci:
    #!/usr/bin/env bash
    {{set}}
    just context-check
    just fmt-check
    just lint
    just typecheck
    just security
    just test
    just build
    echo "[ci] all gates green"

# Re-converge after a pull that touched Cargo.toml/Cargo.lock/rust-toolchain.
[group('setup')]
sync:
    #!/usr/bin/env bash
    {{set}}
    rustup toolchain install "$(sed -n 's/^channel *= *"\(.*\)".*/\1/p' rust-toolchain.toml)" --profile minimal --component rustfmt,clippy --no-self-update
    cargo fetch --locked
    echo "[sync] toolchain + deps current"

# Verify every published manifest reports the same version, and that the tag
# (if passed as $1 or read from GITHUB_REF_NAME) matches. Phase 10.3 contract.
# Fails loudly with all values printed on any mismatch.
[group('quality')]
release-check tag='':
    #!/usr/bin/env bash
    {{set}}
    tag="{{tag}}"
    if [ -z "$tag" ] && [ -n "${GITHUB_REF_NAME:-}" ]; then
        tag="${GITHUB_REF_NAME#v}"
    fi
    # Strip a leading "v" from whatever source supplied the tag (CLI arg,
    # GITHUB_REF_NAME, or release-please's tag_name output).
    tag="${tag#v}"
    ws_version=$(awk '/^\[workspace\.package\]/{f=1} f&&/^version/{gsub(/[ "=]/,"",$0); print substr($0,8); exit}' Cargo.toml)
    meta=$(cargo metadata --no-deps --format-version 1)
    core_version=$(printf '%s' "$meta" | jq -r '.packages[] | select(.name=="sea-forge-core") | .version')
    cli_version=$(printf '%s' "$meta" | jq -r '.packages[] | select(.name=="sea-forge-cli") | .version')
    echo "workspace.package.version = $ws_version"
    echo "sea-forge-core            = $core_version"
    echo "sea-forge-cli             = $cli_version"
    [ -n "$tag" ] && echo "tag (stripped)            = $tag"
    if [ "$ws_version" != "$core_version" ] || [ "$ws_version" != "$cli_version" ]; then
        echo "fail: workspace, core, and cli versions disagree" >&2
        exit 1
    fi
    if [ -n "$tag" ] && [ "$ws_version" != "$tag" ]; then
        echo "fail: workspace version $ws_version != tag $tag" >&2
        exit 1
    fi
    echo "ok: versions synchronized"

# Minimum-spec proof commands (spec-minimum §12.2). Requires the slice.
[group('quality')]
proof:
    #!/usr/bin/env bash
    {{set}}
    echo "[proof] running spec-minimum §12.2 P1-P4b"
    cargo build -q -p sea-forge-cli
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    policy="$tmp/permissive.yaml"
    deny="$tmp/deny.yaml"
    printf '%s\n' 'version: "0.1"' 'rules:' \
      '  - name: allow-model-write' '    verdict: allow' \
      '    actor_role: operator' '    operation_kind: write_file' \
      '    path_prefix: ""' '  - name: allow-self-validate' \
      '    verdict: allow' '    actor_role: operator' \
      '    operation_kind: execute_command' '    argv0: sea-forge' >"$policy"
    printf '%s\n' 'version: "0.1"' 'rules: []' >"$deny"
    target/debug/sea-forge run --root "$tmp/state" --policy "$policy" \
      --intent "Generate and validate a simple DomainForge .sea model" >"$tmp/run.out"
    run_id="$(sed -n 's/^run_id=//p' "$tmp/run.out")"
    run="$tmp/state/runs/$run_id"
    test -f "$run/plan.json" && test -f "$run/authority.json"
    test -f "$run/trace.jsonl" && test -f "$run/evidence.jsonl"
    test -f "$run/settlement.json" && test -f "$run/semantic-envelope.json"
    jq -e '.status == "accepted" and (.basis | length > 0)' "$run/settlement.json" >/dev/null
    settlement_id="$(jq -r '.settlement_id' "$run/settlement.json")"
    jq -e --arg settlement_id "$settlement_id" '.settlement_ref == $settlement_id and (.artifact_refs | length == 1) and (.extension_refs|type=="array") and (.projection_refs|type=="array")' "$run/semantic-envelope.json" >/dev/null
    while read -r decision_id; do jq -e --arg id "$decision_id" 'any(.[]; .decision_id == $id)' "$run/authority.json" >/dev/null; done < <(jq -r '.authority_decisions[]' "$run/semantic-envelope.json")
    while read -r evidence_id; do jq -e --arg id "$evidence_id" 'select(.evidence_id == $id)' "$run/evidence.jsonl" >/dev/null; done < <(jq -r '.evidence_refs[]' "$run/semantic-envelope.json")
    case_id="$(jq -r '.case_ref' "$run/semantic-envelope.json")"
    jq -e --arg run_id "$run_id" '.state == "completed" and (.run_ids | index($run_id))' "$tmp/state/cases/$case_id.json" >/dev/null
    jq -e '.artifact_refs[0].pre_mint_identity | test("^ifl:hash:[a-f0-9]{64}$")' "$run/semantic-envelope.json" >/dev/null
    jq -e 'select(.kind=="artifact" and .uri=="artifacts/model.sea") | .metadata.artifact.pre_mint_identity | test("^ifl:hash:[a-f0-9]{64}$")' "$run/evidence.jsonl" >/dev/null
    jq -e 'all(.[]; (.determinism.policy_bundle_hash|test("^sha256:[a-f0-9]{64}$")) and .audit_record.engine and .audit_record.disposition and .audit_record.subject)' "$run/authority.json" >/dev/null
    while IFS=$'\t' read -r uri hash; do
      if command -v sha256sum >/dev/null 2>&1; then
        printf '%s  %s\n' "$hash" "$run/$uri" | sha256sum -c - >/dev/null
      else
        printf '%s  %s\n' "$hash" "$run/$uri" | shasum -a 256 -c - >/dev/null
      fi
    done < <(jq -r 'select(.kind=="artifact") | [.uri,.sha256] | @tsv' "$run/evidence.jsonl")
    set +e
    target/debug/sea-forge run --root "$tmp/denied" --policy "$deny" \
      --intent "Generate and validate a simple DomainForge .sea model" >"$tmp/denied.out"
    denied_exit=$?
    target/debug/sea-forge run --root "$tmp/boundary" --policy "$policy" \
      --intent "TEST_ONLY: write generated zone" >"$tmp/boundary.out"
    boundary_exit=$?
    set -e
    test "$denied_exit" -eq 3
    test "$boundary_exit" -eq 3
    denied_run="$tmp/denied/runs/$(sed -n 's/^run_id=//p' "$tmp/denied.out")"
    jq -e 'all(.[]; .verdict == "deny" and .matched_rule == null)' "$denied_run/authority.json" >/dev/null
    ! grep -q '"kind":"command_started"' "$denied_run/trace.jsonl"
    test -d "$denied_run/workspace"
    test -z "$(ls -A "$denied_run/workspace")"
    test "$(jq -r 'select(.kind=="authority_decision") | .evidence_id' "$denied_run/evidence.jsonl" | wc -l)" -eq "$(jq 'length' "$denied_run/authority.json")"
    boundary_run="$tmp/boundary/runs/$(sed -n 's/^run_id=//p' "$tmp/boundary.out")"
    jq -e 'any(.[]; .reason_codes | index("generated_zone_denied"))' "$boundary_run/authority.json" >/dev/null
    test ! -e "$tmp/boundary/runs"/*/workspace/src/gen/model.sea
    echo "[proof] P1-P4b passed"

# --- clean --------------------------------------------------------------

# Remove build output and bootstrap evidence. Does not touch tracked files.
[group('quality')]
clean:
    cargo clean
    rm -rf {{evidence_dir}}

# --- secrets (Shell-SPEC §8.4) -----------------------------------------

# Create a local age key if absent and print its public key.
[group('secrets')]
secrets-init:
    #!/usr/bin/env bash
    {{set}}
    : "${SOPS_AGE_KEY_FILE:=$HOME/.config/sops/key.txt}"
    mkdir -p "$(dirname "$SOPS_AGE_KEY_FILE")"
    if [ -f "$SOPS_AGE_KEY_FILE" ]; then
        echo "age key already exists at $SOPS_AGE_KEY_FILE"
    else
        umask 077
        age-keygen -o "$SOPS_AGE_KEY_FILE"
        chmod 600 "$SOPS_AGE_KEY_FILE"
        echo "created age key at $SOPS_AGE_KEY_FILE"
    fi
    echo "public recipient (add to .sops.yaml):"
    grep -o 'age1[a-z0-9]*' "$SOPS_AGE_KEY_FILE" | head -1

# Edit an encrypted profile via SOPS without leaving plaintext behind.
[group('secrets')]
secrets-edit profile='dev':
    #!/usr/bin/env bash
    {{set}}
    f="secrets/{{profile}}.enc.env"
    [ -f "$f" ] || { echo "missing $f — creating from template"; \
        printf '# %s profile\nSEA_EXAMPLE_API_KEY=replace-me\n' "{{profile}}" > "$f"; }
    : "${SOPS_AGE_KEY_FILE:=$HOME/.config/sops/key.txt}"
    export SOPS_AGE_KEY_FILE
    sops "$f"

# Decrypt to a pipe, validate names, redact values, report missing requirements.
[group('secrets')]
secrets-check profile='dev':
    #!/usr/bin/env bash
    {{set}}
    f="secrets/{{profile}}.enc.env"
    [ -f "$f" ] || { echo "fail: $f not found"; echo "next move: just secrets-edit {{profile}}"; exit 1; }
    : "${SOPS_AGE_KEY_FILE:=$HOME/.config/sops/key.txt}"
    if [ ! -f "$SOPS_AGE_KEY_FILE" ]; then
        echo "fail: missing_credential_error — no age key at $SOPS_AGE_KEY_FILE"
        echo "next move: just secrets-init"
        exit 1
    fi
    export SOPS_AGE_KEY_FILE
    if ! sops -d "$f" >/dev/null 2>/dev/null; then
        echo "fail: secret_decrypt_error — could not decrypt $f (redacted)"
        echo "next move: verify SOPS_AGE_KEY_FILE matches .sops.yaml recipient"
        exit 1
    fi
    echo "ok: {{profile}} profile decrypts"
    echo "declared variables (values redacted):"
    sops -d "$f" | sed -n 's/^\(SEA_[A-Za-z0-9_]*\)=.*/  \1=<redacted>/p'

# Update encrypted files after recipient changes.
[group('secrets')]
secrets-rekey:
    #!/usr/bin/env bash
    {{set}}
    : "${SOPS_AGE_KEY_FILE:=$HOME/.config/sops/key.txt}"
    export SOPS_AGE_KEY_FILE
    for f in secrets/*.enc.env; do
        [ -f "$f" ] || continue
        echo "[rekey] $f"
        sops updatekeys "$f"
    done

# --- integration (Shell-SPEC §7.2, §10.2) ------------------------------

# Validate and run a declared API/MCP integration by name.
[group('integration')]
integration name:
    #!/usr/bin/env bash
    {{set}}
    echo "fail: no integrations declared yet (Shell-SPEC §2.2)"
    echo "next move: declare a SecretRequirement in .agents/specs/Shell-SPEC.md §7.2, then implement its validation"
    exit 1

# --- hooks (Phase 6) ----------------------------------------------------

# Install the checked-in hooks at .githooks via `core.hooksPath`.
# Idempotent: safe to rerun after pulling hook changes.
[group('hooks')]
hooks-install:
    git config core.hooksPath .githooks
    chmod +x .githooks/pre-commit .githooks/pre-push .githooks/post-checkout
    echo "[hooks] core.hooksPath = .githooks (run 'git config --unset core.hooksPath' to disable)"

# Fast staged check invoked by .githooks/pre-commit. Kept to checks that are
# reliably under ~10s and require no network: context, fmt, typecheck.
[group('hooks')]
pre-commit:
    #!/usr/bin/env bash
    {{set}}
    just check-fast

# Broad local verification invoked by .githooks/pre-push. Adds clippy,
# supply-chain, secret scan, tests, and a debug build on top of pre-commit.
[group('hooks')]
pre-push:
    #!/usr/bin/env bash
    {{set}}
    just ci

# Same recipe the PR-title workflow + gate rely on; alias of `just ci` so a
# maintainer can ask "what does CI see?" with one command.
[group('hooks')]
pr-check:
    just ci

# --- pull-request helper (Phase 9) -------------------------------------

# Verify, push the current branch, and open a PR via gh. Refuses to operate
# from main, requires a clean tree, and never merges automatically.
[group('hooks')]
pr:
    #!/usr/bin/env bash
    {{set}}
    branch=$(git rev-parse --abbrev-ref HEAD)
    if [ "$branch" = "main" ]; then
        echo "fail: refusing to open a PR from main — switch to a feature branch" >&2
        echo "next move: git switch -c feat/short-description" >&2
        exit 1
    fi
    if ! git diff --quiet || ! git diff --cached --quiet; then
        echo "fail: working tree has uncommitted changes — commit or stash first" >&2
        exit 1
    fi
    git fetch origin main --quiet
    if ! git merge-base --is-ancestor origin/main HEAD; then
        echo "fail: branch is behind main — rebase first" >&2
        echo "next move: git fetch origin && git rebase origin/main" >&2
        exit 1
    fi
    just pr-check
    git push -u origin HEAD
    gh pr create --base main --fill
    echo "[pr] opened; review at the URL above. Merging is a manual step."

# --- publication (Phase 10) --------------------------------------------

# One-time manual crates.io bootstrap publish. OIDC trusted publishing for
# crates.io requires (a) at least one classic-token publish of each crate and
# (b) linking the repository on https://crates.io/settings before the
# automated publish job can use trusted publishing. Run this once per crate
# with a short-lived classic token in CARGO_REGISTRY_TOKEN, then revoke the
# token. After this, release-please.yml handles subsequent publishes via OIDC.
[group('release')]
publish-bootstrap crate:
    #!/usr/bin/env bash
    {{set}}
    if [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
        echo "fail: CARGO_REGISTRY_TOKEN is not set" >&2
        echo "next move: create a one-time classic token at https://crates.io/settings/tokens (scope: publish-new), export CARGO_REGISTRY_TOKEN=<token>, then rerun" >&2
        exit 1
    fi
    case "{{crate}}" in
        sea-forge-core|sea-forge-cli) ;;
        *) echo "fail: unknown crate '{{crate}}' (expected sea-forge-core or sea-forge-cli)" >&2; exit 1 ;;
    esac
    just release-check
    cargo publish --locked -p {{crate}}
    echo "[publish-bootstrap] {{crate}} published; now link the repo on https://crates.io/crates/{{crate}}/settings"
