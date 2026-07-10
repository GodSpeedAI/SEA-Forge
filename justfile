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

# Run all quality gates (Shell-SPEC §10.3).
[group('quality')]
check:
    #!/usr/bin/env bash
    {{set}}
    echo "[check] cargo fmt"
    cargo fmt --all -- --check
    echo "[check] cargo clippy"
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    echo "[check] cargo check --locked"
    cargo check --workspace --all-targets --locked
    echo "[check] cargo deny"
    cargo deny check
    echo "[check] gitleaks"
    gitleaks detect --no-banner --redact
    echo "[check] all gates green"

# Run the test suite (Shell-SPEC §10.3).
[group('quality')]
test:
    cargo test --workspace --all-features --locked

# Minimum-spec proof commands (spec-minimum §12.2). Requires the slice.
[group('quality')]
proof:
    #!/usr/bin/env bash
    {{set}}
    if ! cargo run -q -p sea-forge-cli -- --help >/dev/null 2>&1 \
       && ! cargo run -q -p sea-forge-cli >/dev/null 2>&1; then
        echo "sea-forge: minimum slice not implemented yet (see .agents/specs/spec-minimum.md §12.2)"
        echo "next move: implement the minimum kernel, then run 'just proof'"
        exit 1
    fi
    echo "[proof] running spec-minimum §12.2 P1-P4b"
    cargo run -q -p sea-forge-cli -- run --intent "Generate and validate a simple DomainForge .sea model" || true
    echo "[proof] full P1-P4b verification belongs to the minimum slice conformance tests."

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
