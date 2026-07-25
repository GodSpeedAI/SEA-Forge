#!/usr/bin/env bash
# Shell-SPEC §7.1 DoctorResult emitter.
# Writes machine-readable JSONL to target/bootstrap-evidence/doctor.jsonl and
# stdout. Any "fail" row exits nonzero. "skipped" never counts as passed.
set -uo pipefail

EVIDENCE_DIR="target/bootstrap-evidence"
mkdir -p "$EVIDENCE_DIR"
OUT="$EVIDENCE_DIR/doctor.jsonl"
: >"$OUT"

# emit <layer> <check> <status> <detail> [next_move]
# Detail/next_move must not contain " or \ (kept simple by construction).
emit() {
  local layer="$1" check="$2" status="$3" detail="$4" next="${5:-}"
  local line
  if [ -n "$next" ]; then
    line=$(printf '{"layer":"%s","check":"%s","status":"%s","detail":"%s","next_move":"%s"}' \
      "$layer" "$check" "$status" "$detail" "$next")
  else
    line=$(printf '{"layer":"%s","check":"%s","status":"%s","detail":"%s"}' \
      "$layer" "$check" "$status" "$detail")
  fi
  printf '%s\n' "$line" | tee -a "$OUT"
}

FAIL=0
mark_fail() { FAIL=1; }

# tool_present <name>: prints version string or empty.
tool_present() {
  command -v "$1" >/dev/null 2>&1
}

# --- system layer ---
for t in git just direnv sops age gitleaks cargo-deny; do
  if tool_present "$t"; then
    emit system "$t" ok "available on PATH"
  else
    emit system "$t" fail "not found on PATH" "devbox install"
    mark_fail
  fi
done

# rustup is provided by devbox; rustc/clippy/rustfmt come from rust-toolchain.toml
if tool_present rustup; then
  emit system rustup ok "available on PATH"
else
  emit system rustup fail "not found on PATH" "devbox install"
  mark_fail
fi

# --- rust layer ---
CHANNEL=$(sed -n 's/^channel *= *"\(.*\)".*/\1/p' rust-toolchain.toml 2>/dev/null || true)
if [ -z "$CHANNEL" ]; then
  emit rust toolchain fail "could not read channel from rust-toolchain.toml" \
    "fix rust-toolchain.toml"
  mark_fail
elif tool_present rustc; then
  RUSTC_VER=$(rustc --version | awk '{print $2}')
  if [ "$RUSTC_VER" = "$CHANNEL" ]; then
    emit rust toolchain ok "rustc $RUSTC_VER matches pin $CHANNEL"
  else
    emit rust toolchain fail "rustc $RUSTC_VER != pin $CHANNEL" \
      "rustup toolchain install $CHANNEL"
    mark_fail
  fi
else
  emit rust toolchain fail "rustc not found" "rustup toolchain install $CHANNEL"
  mark_fail
fi

if tool_present rustfmt && tool_present cargo-clippy; then
  emit rust components ok "rustfmt and clippy available"
else
  emit rust components fail "rustfmt or clippy missing" \
    "rustup component add rustfmt clippy"
  mark_fail
fi

# --- workspace layer ---
if [ -f Cargo.toml ] && grep -q '\[workspace\]' Cargo.toml; then
  emit workspace manifest ok "root Cargo.toml is a workspace"
else
  emit workspace manifest fail "root Cargo.toml is not a workspace" \
    "fix Cargo.toml"
  mark_fail
fi

if [ -f Cargo.lock ]; then
  emit workspace lockfile ok "Cargo.lock present"
else
  emit workspace lockfile fail "Cargo.lock missing" "cargo generate-lockfile"
  mark_fail
fi

for c in crates/sea-forge-core/src/lib.rs crates/sea-forge-cli/src/main.rs; do
  if [ -f "$c" ]; then
    emit workspace "$c" ok "exists"
  else
    emit workspace "$c" fail "missing" "restore $c"
    mark_fail
  fi
done

# --- secrets layer ---
if [ -n "${SOPS_AGE_KEY_FILE:-}" ] && [ -f "${SOPS_AGE_KEY_FILE}" ]; then
  emit secrets age_key ok "SOPS_AGE_KEY_FILE set and file exists"
else
  emit secrets age_key warn "no age key (offline mode; secret-dependent recipes blocked)" \
    "just secrets-init"
fi

if [ -f secrets/dev.enc.env ]; then
  emit secrets dev_profile ok "secrets/dev.enc.env present"
else
  emit secrets dev_profile warn "no dev encrypted profile" "just secrets-init"
fi

# --- quality layer ---
if tool_present cargo && tool_present cargo-deny && tool_present gitleaks; then
  emit quality gates ok "fmt, clippy, deny, gitleaks available"
else
  emit quality gates fail "a quality tool is missing" "devbox install"
  mark_fail
fi

# --- integration layer ---
# Optional API/MCP integrations are selected later (Shell-SPEC §2.2, §7.2).
emit integration mcp_api skipped "no integrations declared yet"

exit $FAIL
